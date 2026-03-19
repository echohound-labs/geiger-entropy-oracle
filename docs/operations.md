# ⚠️ Critical Operations Guide
## Echo Hound Labs — Geiger Entropy Oracle

---

## 🚨 GOLDEN RULES — NEVER BREAK THESE

### Rule 1: Always stop daemon before switching branches
```bash
# CORRECT:
pkill -f daemon.py
sleep 2
git checkout [branch]
# restart daemon

# WRONG:
git checkout [branch]  # while daemon running = BREAKS EVERYTHING
```

### Rule 2: Always copy IDL after anchor build
```bash
# After building on mainnet-commit-reveal branch:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# Fix IDL address to mainnet:
sed -i 's/"address": "2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY"/"address": "BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU"/' \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# After building on testnet branch:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/testnet/geiger_entropy.json

# After building on main branch:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet/geiger_entropy.json
```

### Rule 3: Always verify branch before starting mainnet
```bash
git branch  # must show * mainnet-commit-reveal
curl http://localhost:8746/health  # verify after start
```

---

## Start Commands

### Mainnet (mainnet-commit-reveal branch) — PRODUCTION
```bash
pkill -f daemon.py
sleep 2
git checkout mainnet-commit-reveal
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8746/health
```

### Testnet (testnet-vdf-verification branch)
```bash
pkill -f daemon.py
sleep 2
git checkout testnet-vdf-verification
cd entropy-daemon
CONFIG_PATH=./config.toml \
SUBMIT_SCRIPT=./submit_entropy.js \
python3 daemon.py > logs/testnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8745/health
```

### Fast mode (main branch)
```bash
pkill -f daemon.py
sleep 2
git checkout main
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8746/health
```

---

## After Every anchor build
```
Branch                    → Copy IDL to
─────────────────────────────────────────
main                      → idl/mainnet/
testnet-vdf-verification  → idl/testnet/
mainnet-commit-reveal     → idl/mainnet-commit-reveal/
```

Always fix IDL address after copying:
```bash
# If built on testnet branch but need mainnet address:
sed -i 's/2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU/' \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json
```

---

## USB / Geiger Counter

### WSL2 — reconnect after unplug
```powershell
# Windows PowerShell as Administrator:
usbipd list
# Find CH340 device and note busid

usbipd bind --busid 2-1
usbipd attach --busid 2-1 --wsl
```
```bash
# Verify in WSL:
ls /dev/ttyUSB0
```

### Device fingerprint reset
```bash
rm entropy-daemon/.geiger_device_fingerprint
# Daemon re-registers on next start
```

---

## Branch → Scripts mapping
```
mainnet-commit-reveal:
→ entropy-daemon/mainnet/commit_entropy.js
→ entropy-daemon/mainnet/reveal_entropy.js
→ entropy-daemon/mainnet/recover_commitment.js
→ entropy-daemon/idl/mainnet-commit-reveal/

testnet-vdf-verification:
→ entropy-daemon/testnet/commit_entropy.js
→ entropy-daemon/testnet/reveal_entropy.js
→ entropy-daemon/testnet/recover_commitment.js
→ entropy-daemon/idl/testnet/

main:
→ entropy-daemon/submit_entropy_mainnet.js
→ entropy-daemon/idl/mainnet/
```

---

*Echo Hound Labs (@EchoHoundX) ☢️🦴*
