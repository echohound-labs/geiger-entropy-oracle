#!/usr/bin/env python3
"""
Geiger Entropy Oracle Daemon
Reads GMC-500 via direct serial, extracts physical entropy,
submits on-chain to X1, serves via REST API.

Author: Skywalker (@skywalker12345678)
License: MIT
"""

import asyncio
import hashlib
import json
import logging
import os
import queue
import struct
import threading
import time
from pathlib import Path
from typing import Optional

import toml
import uvicorn
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

import os; CONFIG_PATH = Path(os.environ.get("CONFIG_PATH", str(Path(__file__).parent / "config.toml")))

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
# Keypair
# ---------------------------------------------------------------------------

def load_keypair(path: str) -> Ed25519PrivateKey:
    expanded = os.path.expanduser(path)
    with open(expanded) as f:
        key_bytes = json.load(f)
    return Ed25519PrivateKey.from_private_bytes(bytes(key_bytes[:32]))

def sign_entropy(private_key: Ed25519PrivateKey, seed: bytes, timestamp: int) -> bytes:
    message = seed + struct.pack("<Q", timestamp)
    return private_key.sign(message)

# ---------------------------------------------------------------------------
# Entropy State
# ---------------------------------------------------------------------------

def xor_seeds(seeds: list) -> bytes:
    if not seeds:
        return b"\x00" * 32
    result = bytearray(seeds[0])
    for seed in seeds[1:]:
        for i in range(min(32, len(seed))):
            result[i] ^= seed[i]
    return bytes(result)

class EntropyState:
    def __init__(self, pool_size: int = 10):
        self.pool_size = pool_size
        self.pool: list = []
        self.latest_seed: Optional[bytes] = None
        self.latest_cpm: int = 0
        self.latest_usv_h: float = 0.0
        self.latest_timestamp: int = 0
        self.latest_signature: Optional[bytes] = None
        self.total_submissions: int = 0
        self.lock = threading.Lock()

    def update(self, seed: bytes, cpm: int, usv_h: float, signature: bytes):
        with self.lock:
            self.pool.append(seed)
            if len(self.pool) > self.pool_size:
                self.pool.pop(0)
            self.latest_seed = seed
            self.latest_cpm = cpm
            self.latest_usv_h = usv_h
            self.latest_timestamp = int(time.time())
            self.latest_signature = signature
            self.total_submissions += 1

    @property
    def pool_seed(self) -> bytes:
        return xor_seeds(self.pool)

    def to_dict(self) -> dict:
        with self.lock:
            return {
                "seed": self.latest_seed.hex() if self.latest_seed else None,
                "pool_seed": self.pool_seed.hex(),
                "cpm": self.latest_cpm,
                "usv_h": self.latest_usv_h,
                "timestamp": self.latest_timestamp,
                "signature": self.latest_signature.hex() if self.latest_signature else None,
                "total_submissions": self.total_submissions,
            }

# ---------------------------------------------------------------------------
# Serial Collector
# ---------------------------------------------------------------------------

def serial_collector(cfg: dict, state: EntropyState, private_key: Ed25519PrivateKey,
                     entropy_queue: queue.Queue, logger: logging.Logger):
    try:
        import serial as pyserial
    except ImportError:
        logger.error("pyserial not installed. Run: pip install pyserial")
        return

    port = cfg["serial"]["port"]
    baud = cfg["serial"]["baud"]
    poll_ms = cfg["serial"].get("poll_interval_ms", 250)
    min_cpm = cfg["entropy"].get("min_cpm", 5)

    logger.info(f"Connecting to Geiger counter on {port} at {baud} baud...")

    try:
        ser = pyserial.Serial(port, baud, timeout=1)
    except Exception as e:
        logger.error(f"Failed to open serial port {port}: {e}")
        return

    logger.info(f"✓ Serial port {port} opened")

    last_cps = 0
    last_event_time = None

    while True:
        try:
            # GET CPS
            ser.write(b"<GETCPS>>")
            cps_data = ser.read(4)
            if len(cps_data) != 4:
                time.sleep(poll_ms / 1000)
                continue
            cps = int.from_bytes(cps_data, "big")

            # GET CPM
            ser.write(b"<GETCPM>>")
            cpm_data = ser.read(4)
            if len(cpm_data) != 4:
                time.sleep(poll_ms / 1000)
                continue
            cpm = int.from_bytes(cpm_data, "big")
            usv_h = cpm * 0.0065

            now = time.time()
            now_ns = time.time_ns()

            # Detect rising edge decay event
            if cps >= 1 and last_cps == 0:
                if last_event_time is not None:
                    delta_ns = now_ns - last_event_time
                    # Extract entropy from inter-event timing
                    entropy_input = f"{delta_ns}-{now_ns}-{cpm}-{cps}".encode()
                    seed = hashlib.sha256(entropy_input).digest()
                    timestamp = int(now)
                    signature = sign_entropy(private_key, seed, timestamp)

                    state.update(seed, cpm, usv_h, signature)

                    logger.info(
                        f"☢️  DECAY EVENT | Δt={delta_ns/1e9:.3f}s | "
                        f"CPM={cpm} | µSv/h={usv_h:.3f} | "
                        f"seed={seed.hex()[:16]}..."
                    )

                    if cpm >= min_cpm:
                        entropy_queue.put({
                            "seed": seed,
                            "cpm": cpm,
                            "timestamp": timestamp,
                            "signature": signature,
                        })

                last_event_time = now_ns

            last_cps = cps
            time.sleep(poll_ms / 1000)

        except Exception as e:
            logger.error(f"Serial error: {e}")
            time.sleep(2)

# ---------------------------------------------------------------------------
# X1 On-Chain Submitter
# ---------------------------------------------------------------------------

def onchain_submitter(cfg: dict, entropy_queue: queue.Queue, logger: logging.Logger):
    """
    Reads entropy events from the queue and submits them on-chain via
    a Node.js helper script (avoids Python Solana SDK complexity).
    """
    import subprocess
    import tempfile

    rpc_url = cfg["x1"]["rpc_url"]
    program_id = cfg["x1"]["program_id"]
    oracle_state = cfg["x1"]["oracle_state"]
    entropy_pool = cfg["x1"]["entropy_pool"]
    entropy_node = cfg["x1"]["entropy_node"]
    keypair_path = os.path.expanduser(cfg["node"]["keypair_path"])

    submit_script = Path(__file__).parent / os.environ.get("SUBMIT_SCRIPT", "submit_entropy.js")

    logger.info("On-chain submitter ready — waiting for entropy events...")

    while True:
        try:
            event = entropy_queue.get(timeout=60)
            seed_hex = event["seed"].hex()
            sig_hex = event["signature"].hex()
            cpm = event["cpm"]
            timestamp = event["timestamp"]

            result = subprocess.run(
                ["node", str(submit_script),
                 seed_hex, sig_hex, str(cpm), str(timestamp)],
                capture_output=True, text=True, timeout=30
            )

            if result.returncode == 0:
                logger.info(f"✓ On-chain submission OK | CPM={cpm}")
                logger.debug(result.stdout.strip())
            else:
                logger.warning(f"On-chain submission failed: {result.stderr.strip()}")

        except queue.Empty:
            logger.debug("No entropy events in last 60s — is Geiger counter running?")
        except Exception as e:
            logger.error(f"Submitter error: {e}")
            time.sleep(5)

# ---------------------------------------------------------------------------
# FastAPI
# ---------------------------------------------------------------------------

app = FastAPI(
    title="Geiger Entropy Oracle",
    description="Physical randomness oracle powered by GMC-500 radioactive decay",
    version="2.0.0",
)

_state: Optional[EntropyState] = None
_start_time: float = time.time()

class EntropyResponse(BaseModel):
    seed: Optional[str]
    pool_seed: str
    cpm: int
    usv_h: float
    timestamp: int
    signature: Optional[str]
    total_submissions: int

class HealthResponse(BaseModel):
    status: str
    uptime_seconds: float
    total_submissions: int
    latest_cpm: int

@app.get("/entropy", response_model=EntropyResponse)
async def get_entropy():
    if _state is None or _state.latest_seed is None:
        raise HTTPException(
            status_code=503,
            detail="No entropy yet — is the Geiger counter connected?"
        )
    return EntropyResponse(**_state.to_dict())

@app.get("/health", response_model=HealthResponse)
async def health():
    return HealthResponse(
        status="ok" if (_state and _state.latest_seed) else "waiting",
        uptime_seconds=round(time.time() - _start_time, 1),
        total_submissions=_state.total_submissions if _state else 0,
        latest_cpm=_state.latest_cpm if _state else 0,
    )

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    global _state

    cfg = load_config()
    logger = setup_logging(cfg["daemon"].get("log_level", "INFO"))
    logger.info("☢️  Geiger Entropy Oracle v2 starting up")

    private_key = load_keypair(cfg["node"]["keypair_path"])
    logger.info("✓ Keypair loaded")

    pool_size = cfg["entropy"].get("rolling_pool_size", 10)
    _state = EntropyState(pool_size=pool_size)

    entropy_queue = queue.Queue(maxsize=100)

    # Serial collector thread
    serial_thread = threading.Thread(
        target=serial_collector,
        args=(cfg, _state, private_key, entropy_queue, logger),
        daemon=True
    )
    serial_thread.start()

    # On-chain submitter thread
    chain_thread = threading.Thread(
        target=onchain_submitter,
        args=(cfg, entropy_queue, logger),
        daemon=True
    )
    chain_thread.start()

    # REST API
    port = cfg["daemon"].get("port", 8745)
    logger.info(f"REST API on http://localhost:{port}")
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="warning")

if __name__ == "__main__":
    main()
