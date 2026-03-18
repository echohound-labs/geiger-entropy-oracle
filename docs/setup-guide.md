# 🔧 Setup Guide
## Running Geiger Entropy Oracle on Windows (WSL2)

---

## Prerequisites

- Windows 10/11 with WSL2 (Ubuntu)
- GMC-500+ Geiger counter
- Node.js 18+
- Python 3.9+
- Solana CLI
- Anchor CLI 0.32.1

---

## Step 1 — Install usbipd (Windows)

Open **Windows PowerShell as Administrator** and run:
```powershell
winget install usbipd
```

---

## Step 2 — Attach Geiger Counter to WSL2

Every time you plug in your GMC-500, open **Windows PowerShell as Administrator**:
```powershell
# List connected USB devices
usbipd list

# You should see something like:
# 2-2    1a86:7523  USB-SERIAL CH340 (COM4)    Shared

# Attach to WSL (replace 2-2 with your busid)
usbipd attach --busid 2-2 --wsl
```

Then verify in WSL:
```bash
dmesg | grep tty
# Should show: ch341-uart converter now attached to ttyUSB0
```

**Note:** You need to do this every time you:
- Restart your PC
- Unplug and replug the Geiger counter
- Restart WSL

---

## Step 3 — Clone and Install
```bash
git clone https://github.com/echohound-labs/geiger-entropy-oracle
cd geiger-entropy-oracle/entropy-daemon
pip3 install -r requirements.txt
```

---

## Step 4 — Configure
```bash
cp config.toml config-mainnet.toml
```

Edit `config-mainnet.toml`:
```toml
[daemon]
port = 8746
log_level = "INFO"

[node]
keypair_path = "~/.config/solana/id.json"
node_name = "Your Node Name"

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

## Step 5 — Register Your Node
```bash
cd ~/geiger-entropy-oracle/entropy-contract
node register_node.js
```

Save your Node PDA address and add it to `config-mainnet.toml`.

---

## Step 6 — Start the Oracle
```bash
cd ~/geiger-entropy-oracle/entropy-daemon
chmod +x start.sh
./start.sh
```

**Verify it's running:**
```bash
curl http://localhost:8746/health
```

Expected output:
```json
{
  "status": "ok",
  "uptime_seconds": 10,
  "total_submissions": 1,
  "latest_cpm": 20,
  "vdf_iters": 50000
}
```

---

## Step 7 — Monitor

**Watch live logs:**
```bash
tail -f logs/mainnet-daemon.log
```

**Check submissions:**
```bash
curl http://localhost:8746/entropy
```

**Check balance:**
```bash
solana balance --url https://rpc.mainnet.x1.xyz YOUR_WALLET
```

---

## Running on Raspberry Pi (Linux)

No usbipd needed! Just plug in the GMC-500 and run:
```bash
ls /dev/ttyUSB*
# Should show /dev/ttyUSB0
```

Then follow steps 3-7 above.

---

## Troubleshooting

**GMC-500 not detected:**
```bash
# Check if USB is attached in WSL
dmesg | grep tty

# If not, reattach in PowerShell
usbipd attach --busid 2-2 --wsl
```

**Port already in use:**
```bash
pkill -f daemon.py
fuser -k 8746/tcp
./start.sh
```

**Low balance:**
```bash
# Check mainnet balance
solana balance --url https://rpc.mainnet.x1.xyz YOUR_WALLET

# Top up if below 5 XNT
```

**Submissions failing:**
```bash
tail -f logs/mainnet-daemon.log
# Look for "On-chain submission failed"
# Check your XNT balance
```

---

## systemd Service (Auto-start on boot)

Create service file:
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

systemctl --user enable geiger-entropy
systemctl --user start geiger-entropy
```

---

*Echo Hound Labs — Building X1 Infrastructure* 🦴☢️

## Device Fingerprinting

On first run the daemon reads and stores your GMC-500 hardware fingerprint:
- Model: GMC-500+Re 2.5
- Internal serial number
- USB VID:PID
```
✓ Device fingerprint registered: 83ff336d752b6b12...
```

Every subsequent run verifies the fingerprint:
```
✓ Device fingerprint verified: 83ff336d752b6b12...
```

If someone swaps your hardware:
```
🚨 DEVICE FINGERPRINT MISMATCH!
Hardware verification failed — daemon refusing to start
```

To reset fingerprint (if you replace your device):
```bash
rm entropy-daemon/.geiger_device_fingerprint
```
Daemon will re-register on next start.
