#!/usr/bin/env python3
"""
Geiger Entropy Oracle Daemon v3
Reads GMC-500 via direct serial, extracts physical entropy,
applies VDF (Verifiable Delay Function) for manipulation resistance,
submits on-chain to X1, serves via REST API.

Author: Skywalker (@skywalker12345678) / Echo Hound Labs
License: MIT
"""

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
from serial.tools import list_ports
import uvicorn
from chiavdf import create_discriminant, prove, verify_wesolowski
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
# Device Fingerprinting
# ---------------------------------------------------------------------------

FINGERPRINT_FILE = Path(__file__).parent / ".geiger_device_fingerprint"

def get_device_fingerprint(port: str) -> str:
    """Get unique fingerprint using GMC-500 internal serial + USB VID:PID."""
    import serial as pyserial

    # Get USB VID:PID
    usb_info = "unknown:unknown"
    for p in list_ports.comports():
        if p.device == port:
            usb_info = f"{p.vid}:{p.pid}"
            break

    # Get GMC-500 internal model + serial number
    try:
        ser = pyserial.Serial(port, 115200, timeout=2)
        ser.write(b'<GETVER>>')
        model = ser.read(14).decode(errors='ignore').strip()
        ser.write(b'<GETSERIAL>>')
        device_serial = ser.read(7).hex()
        ser.close()
    except Exception as e:
        raise RuntimeError(f"Cannot read device identity: {e}")

    fingerprint_data = f"{usb_info}:{model}:{device_serial}"
    return hashlib.sha256(fingerprint_data.encode()).hexdigest()

def verify_device_fingerprint(port: str, logger: logging.Logger) -> bool:
    """Verify device matches stored fingerprint. Register if first run."""
    try:
        current = get_device_fingerprint(port)
    except RuntimeError as e:
        logger.error(f"Device fingerprint error: {e}")
        return False

    if FINGERPRINT_FILE.exists():
        stored = FINGERPRINT_FILE.read_text().strip()
        if current != stored:
            logger.error(
                f"🚨 DEVICE FINGERPRINT MISMATCH!\n"
                f"   Expected: {stored[:32]}...\n"
                f"   Got:      {current[:32]}...\n"
                f"   Refusing to operate with unrecognized hardware!"
            )
            return False
        logger.info(f"✓ Device fingerprint verified: {current[:16]}...")
        return True
    else:
        FINGERPRINT_FILE.write_text(current)
        logger.info(f"✓ Device fingerprint registered: {current[:16]}...")
        logger.info(f"  Model: GMC-500 | USB: {port}")
        return True

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
# VDF (Verifiable Delay Function)
# ---------------------------------------------------------------------------

def get_vdf_iterations(cpm: int) -> int:
    """Dynamic VDF iterations based on CPM.
    Hotter source = faster decay = fewer iterations needed.
    Ensures VDF completes before next decay event arrives.
    """
    if cpm < 20:
        return 100000  # 0.34s — background radiation
    elif cpm < 50:
        return 50000   # 0.17s — mild source
    elif cpm < 100:
        return 20000   # 0.08s — hot source
    else:
        return 10000   # 0.04s — very hot source

def compute_vdf(seed: bytes, cpm: int) -> tuple:
    """Compute VDF proof from entropy seed.
    Returns (vdf_output, vdf_proof, iters, discriminant).
    The VDF delay prevents manipulation — output unknown until compute completes.
    """
    challenge = seed[:10]
    initial_el = b"\x08" + b"\x00" * 99
    iters = get_vdf_iterations(cpm)
    discriminant = create_discriminant(challenge, 512)
    result = prove(challenge, initial_el, 512, iters, "")
    vdf_output = result[:100]
    vdf_proof = result[100:200]
    return vdf_output, vdf_proof, iters, discriminant

def verify_vdf(seed: bytes, vdf_output: bytes, vdf_proof: bytes, iters: int) -> bool:
    """Verify a VDF proof. Fast to verify, slow to compute."""
    try:
        challenge = seed[:10]
        initial_el = b"\x08" + b"\x00" * 99
        discriminant = create_discriminant(challenge, 512)
        is_valid = verify_wesolowski(
            str(discriminant),
            initial_el,
            vdf_output,
            vdf_proof,
            iters,
        )
        return is_valid
    except Exception:
        return False

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
        self.latest_vdf_iters: int = 0
        self.latest_vdf_time: float = 0.0
        self.total_submissions: int = 0
        self.lock = threading.Lock()

    def update(self, seed: bytes, cpm: int, usv_h: float,
               signature: bytes, vdf_iters: int = 0, vdf_time: float = 0.0):
        with self.lock:
            self.pool.append(seed)
            if len(self.pool) > self.pool_size:
                self.pool.pop(0)
            self.latest_seed = seed
            self.latest_cpm = cpm
            self.latest_usv_h = usv_h
            self.latest_timestamp = int(time.time())
            self.latest_signature = signature
            self.latest_vdf_iters = vdf_iters
            self.latest_vdf_time = vdf_time
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
                "vdf_iters": self.latest_vdf_iters,
                "vdf_time_ms": round(self.latest_vdf_time * 1000, 1),
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

    # Verify hardware fingerprint before connecting
    logger.info("Verifying Geiger counter hardware fingerprint...")
    if not verify_device_fingerprint(port, logger):
        logger.error("🚨 Hardware verification failed — daemon refusing to start")
        return

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

                    # Step 1: Extract raw entropy from inter-event timing
                    entropy_input = f"{delta_ns}-{now_ns}-{cpm}-{cps}".encode()
                    raw_seed = hashlib.sha256(entropy_input).digest()

                    # Step 2: Apply VDF for manipulation resistance
                    vdf_start = time.time()
                    vdf_output, vdf_proof, vdf_iters, discriminant = compute_vdf(raw_seed, cpm)
                    vdf_time = time.time() - vdf_start

                    # Step 3: Final seed = SHA256(VDF output)
                    final_seed = hashlib.sha256(vdf_output).digest()
                    timestamp = int(now)
                    signature = sign_entropy(private_key, final_seed, timestamp)

                    state.update(final_seed, cpm, usv_h, signature, vdf_iters, vdf_time)

                    logger.info(
                        f"☢️  DECAY EVENT | Δt={delta_ns/1e9:.3f}s | "
                        f"CPM={cpm} | µSv/h={usv_h:.3f} | "
                        f"seed={final_seed.hex()[:16]}... | "
                        f"VDF={vdf_iters}iters/{vdf_time:.3f}s"
                    )

                    if cpm >= min_cpm:
                        entropy_queue.put({
                            "seed": final_seed,
                            "vdf_output": vdf_output.hex(),
                            "vdf_proof": vdf_proof.hex(),
                            "vdf_iters": vdf_iters,
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
    import subprocess
    import secrets

    submit_script = Path(__file__).parent / os.environ.get("SUBMIT_SCRIPT", "submit_entropy.js")
    commit_script = Path(__file__).parent / "commit_entropy.js"
    reveal_script = Path(__file__).parent / "reveal_entropy.js"

    use_commit_reveal = commit_script.exists() and reveal_script.exists()

    if use_commit_reveal:
        logger.info("On-chain submitter ready -- commit-reveal mode")
    else:
        logger.info("On-chain submitter ready -- direct submit mode")

    sequence = 0

    while True:
        try:
            event = entropy_queue.get(timeout=60)
            seed_hex = event["seed"].hex()
            sig_hex = event["signature"].hex()
            cpm = event["cpm"]
            timestamp = event["timestamp"]
            vdf_output_hex = event.get("vdf_output", "00" * 32)
            vdf_iters = event.get("vdf_iters", 10000)
            vdf_out_32 = vdf_output_hex[:64]

            if use_commit_reveal:
                nonce = secrets.token_hex(32)

                commit_result = subprocess.run(
                    ["node", str(commit_script),
                     vdf_out_32, nonce, str(sequence)],
                    capture_output=True, text=True, timeout=30
                )

                if commit_result.returncode != 0:
                    logger.warning(f"Commit failed: {commit_result.stderr.strip()}")
                    continue

                logger.info(f"Committed | seq={sequence} CPM={cpm}")

                reveal_result = subprocess.run(
                    ["node", str(reveal_script),
                     vdf_out_32, nonce, sig_hex, str(cpm), str(timestamp)],
                    capture_output=True, text=True, timeout=60
                )

                if reveal_result.returncode == 0:
                    logger.info(f"Revealed | seq={sequence} CPM={cpm} VDF={vdf_iters}iters")
                    sequence += 1
                else:
                    logger.warning(f"Reveal failed: {reveal_result.stderr.strip()}")

            else:
                result = subprocess.run(
                    ["node", str(submit_script),
                     seed_hex, sig_hex, str(cpm), str(timestamp)],
                    capture_output=True, text=True, timeout=30
                )
                if result.returncode == 0:
                    logger.info(f"On-chain submission OK | CPM={cpm} | VDF={vdf_iters}iters")
                    logger.debug(result.stdout.strip())
                else:
                    logger.warning(f"On-chain submission failed: {result.stderr.strip()}")

        except queue.Empty:
            logger.debug("No entropy events in last 60s -- is Geiger counter running?")
        except Exception as e:
            logger.error(f"Submitter error: {e}")
            time.sleep(5)

# ---------------------------------------------------------------------------
# FastAPI
# ---------------------------------------------------------------------------

app = FastAPI(
    title="Geiger Entropy Oracle",
    description="Physical randomness VRF+VDF oracle powered by GMC-500 radioactive decay",
    version="3.0.0",
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
    vdf_iters: int
    vdf_time_ms: float
    total_submissions: int

class HealthResponse(BaseModel):
    status: str
    uptime_seconds: float
    total_submissions: int
    latest_cpm: int
    vdf_iters: int

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
        vdf_iters=_state.latest_vdf_iters if _state else 0,
    )

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    global _state

    cfg = load_config()
    logger = setup_logging(cfg["daemon"].get("log_level", "INFO"))
    logger.info("☢️  Geiger Entropy Oracle v3 — VRF+VDF starting up")

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
