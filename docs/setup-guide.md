# 🔧 Setup Guide — Geiger Entropy Oracle Node v5
Complete guide to running a Geiger Entropy Oracle node on X1 Mainnet.

> ⚠️ **Important:** Node operators must maintain a minimum balance of **25 XNT** in their wallet at all times. The slash mechanism deducts 20 XNT for missed reveals. Keep your wallet funded.

---

## Hardware Required

| Item | Model | Cost | Link |
|------|-------|------|------|
| Geiger Counter | GMC-500+ | ~$100 | GQ Electronics |
| Computer | Any Linux/WSL2/RPi | varies | — |
| Radioactive source | Background radiation is sufficient | $0 | — |

> **Note:** No special radioactive source is required. Background radiation exists everywhere on Earth — produced by cosmic rays, soil, building materials, and natural radioactive decay. The Genesis Node uses Cenozoic fossils to enhance the signal, but any GMC-500 anywhere on Earth can run a node.

---

## Software Prerequisites

### 1. Install Node.js 20+
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
node --version  # should show v20+
```

### 2. Install Python 3.9+
```bash
sudo apt-get install -y python3 python3-pip
python3 --version  # should show 3.9+
```

### 3. Install chiavdf build dependencies
chiavdf (VDF library) requires these system packages:
```bash
sudo apt-get install -y \
    build-essential \
    cmake \
    libgmp-dev \
    python3-dev \
    pkg-config
```

### 4. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### 5. Install Solana CLI
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```

### 6. Install Anchor CLI (developers only — skip if just running a node)
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.32.1
avm use 0.32.1
anchor --version
```

### 7. Install Python dependencies
```bash
cd geiger-entropy-oracle/entropy-daemon
pip3 install -r requirements.txt --break-system-packages
```

---

## Windows WSL2 Setup

### Install usbipd (Windows PowerShell as Administrator)
```powershell
winget install usbipd
```

### Attach Geiger Counter to WSL2
Run in Windows PowerShell as Administrator every time you plug in:
```powershell
# List USB devices
usbipd list
# Find: USB-SERIAL CH340 (COM4) — note the busid (e.g. 2-2)

# Attach to WSL
usbipd attach --busid 2-2 --wsl
```

Verify in WSL:
```bash
dmesg | grep tty
# Should show: ch341-uart converter now attached to ttyUSB0
ls /dev/ttyUSB0
```

> **Note:** Repeat `usbipd attach` after every reboot or replug.

---

## Linux / Raspberry Pi Setup

No usbipd needed — just plug in the GMC-500:
```bash
ls /dev/ttyUSB*
# Should show /dev/ttyUSB0

# Add user to dialout group if needed
sudo usermod -a -G dialout $USER
# Log out and back in
```

---

## Wallet Setup

### Create a new wallet
```bash
solana-keygen new --outfile ~/.config/solana/id.json
# Save your seed phrase securely!

# View your address
solana address -k ~/.config/solana/id.json
```

### Fund your wallet
You need XNT for transaction fees and slash protection:

- **Minimum recommended: 25 XNT**
- Each commit-reveal cycle costs ~0.00005 XNT
- Slash mechanism deducts **20 XNT** for missed reveals
- Keep at least 25 XNT at all times as a safety buffer
- X1 Faucet: https://faucet.x1.xyz
```bash
# Check balance
solana balance --url https://rpc.mainnet.x1.xyz
```

---

## Installation

### Clone the repository
```bash
git clone https://github.com/echohound-labs/geiger-entropy-oracle
cd geiger-entropy-oracle
```

### Install Node dependencies
```bash
cd entropy-contract
npm install
cd ..
```

### Install Python dependencies
```bash
cd entropy-daemon
pip3 install -r requirements.txt --break-system-packages
cd ..
```

### Create logs directory
```bash
mkdir -p ~/geiger-entropy-oracle/entropy-daemon/logs
```

### Verify Geiger Counter
Before configuring, verify your GMC-500 is working:
```bash
# Check device is detected
ls /dev/ttyUSB0

# Test serial connection
python3 -c "
import serial
s = serial.Serial('/dev/ttyUSB0', 115200, timeout=5)
s.write(b'<GETVER>>')
print('Response:', s.read(14))
s.close()
print('GMC-500 OK!')
"
```

Expected output:
```
Response: b'GMC-500Re 2.50'
GMC-500 OK!
```

If no response — check USB connection and usbipd attachment.

---

## Configuration

### Copy and edit config
```bash
cd entropy-daemon
cp config.toml config-mainnet.toml
```

Edit `config-mainnet.toml`:
```toml
[daemon]
port = 8746
log_level = "INFO"

[node]
keypair_path = "~/.config/solana/id.json"
node_name = "Your Node Name Here"

[entropy]
rolling_pool_size = 10
min_cpm = 5

[serial]
port = "/dev/ttyUSB0"
baud = 115200
poll_interval_ms = 250

[x1]
rpc_url = "https://rpc.mainnet.x1.xyz"
program_id = "BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU"
oracle_state = "BygMTZ1oLBD9tDmssnt9LkNT7BEd2PCJBCzurwtMuTqm"
entropy_pool = "GDECYXCXietabJs9Y1baKzD3t4VFBw4eZWPnvYenyi77"
entropy_node = "YOUR_NODE_PDA_HERE"

[tuning]
cycle_sleep_seconds = 15
vdf_iters_low_cpm = 50000
```

> **Tuning:** `cycle_sleep_seconds` controls how long the daemon waits between commit-reveal cycles. 15 seconds is the recommended default — fresh enough for all dApp use cases while remaining cost efficient for operators.

---

## VDF Iterations

The daemon dynamically adjusts VDF iterations based on CPM (counts per minute):
```
CPM < 20  → 50,000 iterations (~0.17s) — background radiation
CPM < 50  → 30,000 iterations (~0.10s) — mild source
CPM < 100 → 20,000 iterations (~0.08s) — hot source
CPM 100+  → 15,000 iterations (~0.05s) — very hot source
```

All iteration counts exceed one X1 slot (~400ms), ensuring the VDF time-lock is always cryptographically meaningful. The VDF ensures that after a decay event is captured, the operator cannot predict the final seed before it is committed on-chain.

---

## Register Your Node
```bash
cd entropy-contract
node register_node.js "My Node Name"
```

Expected output:
```
Network: MAINNET
RPC: https://rpc.mainnet.x1.xyz
Operator: YourWalletAddress...
Balance: 25+ XNT
Node PDA: YourNodePDA...
Node registered!
  Transaction: txSignature...
  Node PDA: YourNodePDA...
Add to config-mainnet.toml:
  entropy_node = "YourNodePDA..."
```

Copy the Node PDA into your `config-mainnet.toml`.

---

## Device Fingerprinting

On first run the daemon automatically registers your GMC-500 hardware fingerprint using the internal serial number, USB VID:PID, and firmware version:
```
✓ Device fingerprint registered: 83ff336d752b6b12...
  Model: GMC-500 | USB: /dev/ttyUSB0
```

Every subsequent run verifies it:
```
✓ Device fingerprint verified: 83ff336d752b6b12...
```

If someone swaps your hardware:
```
🚨 DEVICE FINGERPRINT MISMATCH!
Hardware verification failed — daemon refusing to start
```

To reset (if you replace your Geiger counter):
```bash
rm entropy-daemon/.geiger_device_fingerprint
# Restart daemon — re-registers automatically
```

---

## Start the Oracle
```bash
cd entropy-daemon
chmod +x start.sh
./start.sh
```

Or manually:
```bash
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
```

### Verify it's running
```bash
curl http://localhost:8746/health
```

Expected:
```json
{
  "status": "ok",
  "uptime_seconds": 10,
  "total_submissions": 1,
  "latest_cpm": 20,
  "vdf_iters": 50000
}
```

### Watch live logs
```bash
tail -f logs/mainnet-daemon.log
```

Expected output (v5 commit-reveal mode):
```
☢️  Geiger Entropy Oracle v3 — VRF+VDF starting up
✓ Keypair loaded
Verifying Geiger counter hardware fingerprint...
✓ Device fingerprint verified: 83ff336d752b6b12...
Connecting to Geiger counter on /dev/ttyUSB0 at 115200 baud...
✓ Serial port /dev/ttyUSB0 opened
✓ Clean state — starting at sequence 514
☢️  DECAY EVENT | Δt=3.2s | CPM=20 | µSv/h=0.130 | seed=abc123... | VDF=50000iters/0.162s
Committed | seq=514 CPM=20
Revealed  | seq=514 CPM=20 VDF=50000iters
Cycle complete — sleeping 15s before next commit
```

---

## What Gets Logged On-Chain Forever

Every reveal is permanently recorded on X1:
```
☢️ Entropy revealed | seq=514 CPM=20 uSv/h=0.130 dt=3.200s
VDF=50000iters seed=[104,88,184,213] slot_hash=[194,253,135,141]
binding_slot=39512140 sources=0x07 verified✓
```

`sources=0x07` confirms all three entropy layers active:
- `0x01` = Physical Geiger decay
- `0x02` = Wesolowski VDF
- `0x04` = X1 SlotHash binding

This is a permanent scientific record. Every decay event timestamped to the nanosecond, with CPM and µSv/h radiation readings — immutable and publicly auditable forever.

---

## Slash Mechanism

The oracle uses a **20 XNT slash** to ensure honest operation:

- Operator commits entropy → blind hash locked on-chain
- Operator must reveal within **128 slots (~51 seconds)**
- If operator fails to reveal → anyone can call `slash_missed_reveal()`
- Operator loses **20 XNT** → reporter earns **20 XNT** as bounty

**Three layers of automatic recovery prevent accidental slashing:**

1. **Commit timeout handler** — detects RPC timeout, waits 10s, runs recovery
2. **Reveal retry handler** — 3 attempts with 10s delay for timeouts
3. **Startup recovery** — auto-reveals stuck commitments using saved pending data

> **Keep at least 25 XNT in your operator wallet at all times.**

---

## Auto-start on Boot (systemd)
```bash
cat > ~/.config/systemd/user/geiger-entropy.service << 'SEOF'
[Unit]
Description=Geiger Entropy Oracle v5
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/YOUR_USERNAME/geiger-entropy-oracle/entropy-daemon
Environment="CONFIG_PATH=./config-mainnet.toml"
Environment="SUBMIT_SCRIPT=./submit_entropy_mainnet.js"
ExecStart=/usr/bin/python3 daemon.py
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
SEOF

# Replace YOUR_USERNAME
sed -i "s/YOUR_USERNAME/$USER/g" ~/.config/systemd/user/geiger-entropy.service

systemctl --user enable geiger-entropy
systemctl --user start geiger-entropy
systemctl --user status geiger-entropy
```

---

## Monitor Your Node
```bash
# Health check
curl http://localhost:8746/health

# Latest entropy
curl http://localhost:8746/entropy

# Check balance
solana balance --url https://rpc.mainnet.x1.xyz

# Watch live logs
tail -f logs/mainnet-daemon.log

# View on explorer
# https://explorer.mainnet.x1.xyz/address/YOUR_WALLET
```

---

## ENTROPY Token

Every decay event submission earns ENTROPY tokens (coming Q2 2026):
```
Max Supply:  1,000,000 ENTROPY — ever
Emission:    4 years equal distribution
Year 1-4:    250,000 ENTROPY each (25%)
Mint:        Oracle program only — no team can mint extra
```

The more decay events your node captures, the more ENTROPY you earn. Higher CPM = more events = more ENTROPY.

Token launch prerequisites — NOT MET YET:
- Multi-node operators
- Staking contract
- Slash in ENTROPY
- Statistical audit

---

## Troubleshooting

**GMC-500 not detected:**
```bash
dmesg | grep tty
# If empty, reattach USB (WSL2):
# usbipd attach --busid 2-2 --wsl
```

**Port already in use:**
```bash
pkill -f daemon.py
fuser -k 8746/tcp
./start.sh
```

**Insufficient balance:**
```bash
solana balance --url https://rpc.mainnet.x1.xyz
# Keep minimum 25 XNT — 20 XNT needed for slash protection
```

**Stuck commitment / UnrevealedCommitmentPending spam:**
```bash
pkill -f daemon.py
sleep 2
cd ~/geiger-entropy-oracle/entropy-daemon
node mainnet/recover_commitment.js
# Then restart daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
```

**Submissions failing:**
```bash
tail -f logs/mainnet-daemon.log
# Look for "Commit failed" or "Reveal failed"
```

**Permission denied on /dev/ttyUSB0:**
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

**Device fingerprint mismatch:**
```bash
rm entropy-daemon/.geiger_device_fingerprint
# Restart daemon — re-registers automatically
```

**RPC timeout loop:**
```bash
# Daemon has auto-recovery — wait 30s for it to self-heal
# If still stuck after 60s, restart:
pkill -f daemon.py
sleep 2
node mainnet/recover_commitment.js
# Then restart daemon
```

---

## Explorer Links

- Mainnet: https://explorer.mainnet.x1.xyz
- Program: https://explorer.mainnet.x1.xyz/address/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
- Operator: https://explorer.mainnet.x1.xyz/address/HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
