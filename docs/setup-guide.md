# 🔧 Setup Guide — Geiger Entropy Oracle Node

Complete guide to running a Geiger Entropy Oracle node on X1 Mainnet.

---

## Hardware Required

| Item | Model | Cost | Link |
|------|-------|------|------|
| Geiger Counter | GMC-500+ | ~$100 | GQ Electronics |
| Computer | Any Linux/WSL2/RPi | varies | — |
| Radioactive source | Natural fossils or smoke detector | $0-5 | — |

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

### 3. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### 4. Install Solana CLI
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```

### 5. Install Anchor CLI
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.32.1
avm use 0.32.1
anchor --version
```

### 6. Install Python dependencies
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
Run in **Windows PowerShell as Administrator** every time you plug in:
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

**Note:** Repeat usbipd attach after every reboot or replug.

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

### Create a new Solana wallet
```bash
solana-keygen new --outfile ~/.config/solana/id.json
# Save your seed phrase securely!

# View your address
solana address -k ~/.config/solana/id.json
```

### Fund your wallet
You need XNT for transaction fees:
- Get XNT from X1 faucet or exchange
- Minimum ~1 XNT recommended

### Check balance
```bash
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
```

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
Balance: 10.5 XNT
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

On first run the daemon automatically registers your GMC-500 hardware fingerprint:
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

Expected output:
```
✓ Device fingerprint verified: 83ff336d...
☢️ DECAY EVENT | Δt=3.2s | CPM=20 | seed=abc123... | VDF=50000iters/0.162s
✓ On-chain submission OK | CPM=20 | VDF=50000iters
```

---

## Auto-start on Boot (systemd)
```bash
cat > ~/.config/systemd/user/geiger-entropy.service << 'SEOF'
[Unit]
Description=Geiger Entropy Oracle
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

# View on explorer
# https://explorer.mainnet.x1.xyz/address/YOUR_WALLET
```

---

## ENTROPY Token

Every decay event submission earns ENTROPY tokens (coming Q2 2026):
```
Max Supply:  1,000,000 ENTROPY
Emission:    4 years equal distribution
Year 1-4:    250,000 ENTROPY each (25%)
Mint:        Oracle program only
```

The more decay events your node captures, the more ENTROPY you earn.
Higher CPM = more events = more ENTROPY.

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
# Top up if below 1 XNT
```

**Submissions failing:**
```bash
tail -f logs/mainnet-daemon.log
# Look for "On-chain submission failed"
```

**Permission denied on /dev/ttyUSB0:**
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

---

## Explorer Links

- Mainnet: https://explorer.mainnet.x1.xyz
- Program: https://explorer.mainnet.x1.xyz/address/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
- Operator: https://explorer.mainnet.x1.xyz/address/HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo

---

*Echo Hound Labs (@EchoHoundX) ☢️🦴*
*Building X1 Infrastructure from the ground up*
