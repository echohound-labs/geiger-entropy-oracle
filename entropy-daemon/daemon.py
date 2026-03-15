#!/usr/bin/env python3
"""
Geiger Entropy Oracle Daemon
Reads GMC-500 CSV data, extracts physical entropy, serves via REST API.

Author: Skywalker (@skywalker12345678)
License: MIT
"""

import asyncio
import csv
import hashlib
import json
import logging
import os
import struct
import time
from datetime import datetime
from pathlib import Path
from typing import Optional

import toml
import uvicorn
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

CONFIG_PATH = Path(__file__).parent / "config.toml"


def load_config() -> dict:
    if not CONFIG_PATH.exists():
        raise FileNotFoundError(f"Config not found: {CONFIG_PATH}")
    return toml.load(CONFIG_PATH)


# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

def setup_logging(level: str = "INFO") -> logging.Logger:
    log_dir = Path(__file__).parent / "logs"
    log_dir.mkdir(exist_ok=True)
    logging.basicConfig(
        level=getattr(logging, level.upper(), logging.INFO),
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        handlers=[
            logging.StreamHandler(),
            logging.FileHandler(log_dir / "daemon.log"),
        ],
    )
    return logging.getLogger("geiger-entropy")


# ---------------------------------------------------------------------------
# Entropy Extraction
# ---------------------------------------------------------------------------

def extract_entropy_from_cps(cps_values: list[int]) -> bytes:
    """
    Extract a 256-bit entropy seed from 60 per-second CPS readings.

    Each CPS value represents the number of radioactive decay events in one
    second — a Poisson process governed by quantum mechanics. The exact timing
    and magnitude are physically unpredictable.

    Method: pack all CPS values as little-endian uint16 array, SHA-256 hash.
    """
    if not cps_values:
        raise ValueError("Empty CPS values")
    raw = struct.pack(f"<{len(cps_values)}H", *[max(0, v) for v in cps_values])
    return hashlib.sha256(raw).digest()


def compute_quality_score(cpm: int, cps_values: list[int]) -> float:
    """
    Estimate entropy quality 0.0–1.0 based on CPM and variance of CPS values.
    Higher CPM = more decay events = more entropy bits.
    """
    if cpm <= 0:
        return 0.0
    # Normalize CPM: background ~20, high activity ~200+
    cpm_score = min(cpm / 100.0, 1.0)
    # Variance of CPS values indicates true randomness (not stuck sensor)
    if len(cps_values) > 1:
        mean = sum(cps_values) / len(cps_values)
        variance = sum((x - mean) ** 2 for x in cps_values) / len(cps_values)
        variance_score = min(variance / 10.0, 1.0)
    else:
        variance_score = 0.0
    return round(0.7 * cpm_score + 0.3 * variance_score, 4)


def xor_seeds(seeds: list[bytes]) -> bytes:
    """XOR a list of 32-byte seeds together for the rolling entropy pool."""
    if not seeds:
        return b"\x00" * 32
    result = bytearray(seeds[0])
    for seed in seeds[1:]:
        for i in range(min(32, len(seed))):
            result[i] ^= seed[i]
    return bytes(result)


# ---------------------------------------------------------------------------
# CSV Parser
# ---------------------------------------------------------------------------

def parse_gmc_csv(filepath: Path) -> list[dict]:
    """
    Parse a GMC Data Viewer CSV file.
    Returns list of dicts with keys: datetime, type, usv_h, cpm, cps_values
    Skips header rows and malformed lines.
    """
    results = []
    try:
        with open(filepath, newline="", encoding="utf-8", errors="replace") as f:
            reader = csv.reader(f)
            for row in reader:
                # Skip header rows
                if not row or row[0].startswith("GQ Electronics") or row[0] == "Date Time":
                    continue
                if len(row) < 5:
                    continue
                try:
                    dt_str = row[0].strip()
                    rec_type = row[1].strip()
                    usv_h = float(row[2].strip())
                    cpm = int(row[3].strip())
                    # CPS values: columns 4 onward, up to 60 values, skip trailing empty
                    cps_raw = [c.strip() for c in row[4:] if c.strip() != ""]
                    cps_values = [int(x) for x in cps_raw if x.lstrip("-").isdigit()]
                    if not cps_values:
                        continue
                    results.append({
                        "datetime": dt_str,
                        "type": rec_type,
                        "usv_h": usv_h,
                        "cpm": cpm,
                        "cps_values": cps_values,
                    })
                except (ValueError, IndexError):
                    continue
    except Exception as e:
        logging.getLogger("geiger-entropy").warning(f"Failed to parse {filepath}: {e}")
    return results


def get_latest_row(filepath: Path) -> Optional[dict]:
    """Return the most recent valid row from a CSV file."""
    rows = parse_gmc_csv(filepath)
    return rows[-1] if rows else None


# ---------------------------------------------------------------------------
# Node Keypair (ed25519)
# ---------------------------------------------------------------------------

def load_keypair(path: str) -> Ed25519PrivateKey:
    """Load a Solana-format keypair JSON (64-byte array) as ed25519 key."""
    expanded = os.path.expanduser(path)
    with open(expanded) as f:
        key_bytes = json.load(f)
    # Solana keypair: first 32 bytes = private key seed, last 32 = pubkey
    private_seed = bytes(key_bytes[:32])
    return Ed25519PrivateKey.from_private_bytes(private_seed)


def sign_entropy(private_key: Ed25519PrivateKey, seed: bytes, timestamp: int) -> bytes:
    """Sign the entropy seed + timestamp with the node keypair."""
    message = seed + struct.pack("<Q", timestamp)
    return private_key.sign(message)


# ---------------------------------------------------------------------------
# Entropy State
# ---------------------------------------------------------------------------

class EntropyState:
    def __init__(self, pool_size: int = 10):
        self.pool_size = pool_size
        self.pool: list[bytes] = []
        self.latest_seed: Optional[bytes] = None
        self.latest_cpm: int = 0
        self.latest_usv_h: float = 0.0
        self.latest_timestamp: int = 0
        self.latest_signature: Optional[bytes] = None
        self.latest_quality: float = 0.0
        self.total_submissions: int = 0
        self.source_file: str = ""

    def update(self, seed: bytes, cpm: int, usv_h: float,
               signature: bytes, quality: float, source: str):
        self.pool.append(seed)
        if len(self.pool) > self.pool_size:
            self.pool.pop(0)
        self.latest_seed = seed
        self.latest_cpm = cpm
        self.latest_usv_h = usv_h
        self.latest_timestamp = int(time.time())
        self.latest_signature = signature
        self.latest_quality = quality
        self.total_submissions += 1
        self.source_file = source

    @property
    def pool_seed(self) -> bytes:
        return xor_seeds(self.pool)

    def to_dict(self) -> dict:
        return {
            "seed": self.latest_seed.hex() if self.latest_seed else None,
            "pool_seed": self.pool_seed.hex(),
            "cpm": self.latest_cpm,
            "usv_h": self.latest_usv_h,
            "timestamp": self.latest_timestamp,
            "quality_score": self.latest_quality,
            "signature": self.latest_signature.hex() if self.latest_signature else None,
            "total_submissions": self.total_submissions,
            "source_file": self.source_file,
        }


# ---------------------------------------------------------------------------
# File Watcher
# ---------------------------------------------------------------------------

class CSVHandler(FileSystemEventHandler):
    def __init__(self, state: EntropyState, private_key: Ed25519PrivateKey,
                 min_cpm: int, logger: logging.Logger):
        self.state = state
        self.private_key = private_key
        self.min_cpm = min_cpm
        self.logger = logger
        self._last_processed: dict[str, str] = {}  # path → last datetime

    def process_file(self, filepath: Path):
        row = get_latest_row(filepath)
        if not row:
            return
        # Deduplicate: skip if same datetime already processed
        key = str(filepath)
        if self._last_processed.get(key) == row["datetime"]:
            return
        if row["cpm"] < self.min_cpm:
            self.logger.debug(f"CPM {row['cpm']} below minimum {self.min_cpm}, skipping")
            return
        seed = extract_entropy_from_cps(row["cps_values"])
        quality = compute_quality_score(row["cpm"], row["cps_values"])
        timestamp = int(time.time())
        signature = sign_entropy(self.private_key, seed, timestamp)
        self.state.update(seed, row["cpm"], row["usv_h"], signature, quality, filepath.name)
        self._last_processed[key] = row["datetime"]
        self.logger.info(
            f"Entropy updated | CPM={row['cpm']} µSv/h={row['usv_h']} "
            f"quality={quality} seed={seed.hex()[:16]}..."
        )

    def on_created(self, event):
        if not event.is_directory and event.src_path.endswith(".csv"):
            self.process_file(Path(event.src_path))

    def on_modified(self, event):
        if not event.is_directory and event.src_path.endswith(".csv"):
            self.process_file(Path(event.src_path))


# ---------------------------------------------------------------------------
# FastAPI REST API
# ---------------------------------------------------------------------------

app = FastAPI(
    title="Geiger Entropy Oracle",
    description="Physical randomness oracle powered by GMC-500 radioactive decay",
    version="1.0.0",
)

# Global state (set in main)
_state: Optional[EntropyState] = None
_start_time: float = time.time()


class EntropyResponse(BaseModel):
    seed: Optional[str]
    pool_seed: str
    cpm: int
    usv_h: float
    timestamp: int
    quality_score: float
    signature: Optional[str]
    total_submissions: int
    source_file: str


class HealthResponse(BaseModel):
    status: str
    uptime_seconds: float
    total_submissions: int
    latest_cpm: int
    latest_quality: float


@app.get("/entropy", response_model=EntropyResponse)
async def get_entropy():
    if _state is None or _state.latest_seed is None:
        raise HTTPException(status_code=503, detail="No entropy available yet — is the Geiger counter running?")
    return EntropyResponse(**_state.to_dict())


@app.get("/health", response_model=HealthResponse)
async def health():
    return HealthResponse(
        status="ok" if (_state and _state.latest_seed) else "waiting",
        uptime_seconds=round(time.time() - _start_time, 1),
        total_submissions=_state.total_submissions if _state else 0,
        latest_cpm=_state.latest_cpm if _state else 0,
        latest_quality=_state.latest_quality if _state else 0.0,
    )


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    global _state

    cfg = load_config()
    logger = setup_logging(cfg["daemon"].get("log_level", "INFO"))
    logger.info("☢️  Geiger Entropy Oracle starting up")

    # Load keypair
    keypair_path = cfg["node"]["keypair_path"]
    try:
        private_key = load_keypair(keypair_path)
        logger.info(f"Node keypair loaded from {keypair_path}")
    except FileNotFoundError:
        logger.error(f"Keypair not found at {keypair_path}. Generate one with: solana-keygen new")
        raise

    # Set up entropy state
    pool_size = cfg["entropy"].get("rolling_pool_size", 10)
    min_cpm = cfg["entropy"].get("min_cpm", 5)
    _state = EntropyState(pool_size=pool_size)

    # Process any existing CSVs in the watch folder on startup
    watch_folder = Path(cfg["daemon"]["watch_folder"])
    handler = CSVHandler(_state, private_key, min_cpm, logger)

    if watch_folder.exists():
        for csv_file in sorted(watch_folder.glob("*.csv")):
            handler.process_file(csv_file)
        logger.info(f"Processed existing CSVs in {watch_folder}")
    else:
        logger.warning(f"Watch folder does not exist yet: {watch_folder}")

    # Start file watcher
    observer = Observer()
    observer.schedule(handler, str(watch_folder), recursive=False)
    observer.start()
    logger.info(f"Watching {watch_folder} for new CSV files")

    # Start REST API
    port = cfg["daemon"].get("port", 8745)
    logger.info(f"REST API starting on http://localhost:{port}")
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="warning")

    observer.stop()
    observer.join()


if __name__ == "__main__":
    main()
