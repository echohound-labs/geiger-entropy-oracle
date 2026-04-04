# ⚠️ Critical Operations Guide
## Echo Hound Labs — Geiger Entropy Oracle v5
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
# After building on main branch (production):
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# Fix IDL address to mainnet:
sed -i 's/"address": "2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY"/"address": "BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU"/' \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# After building on testnet branch:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/testnet/geiger_entropy.json
```

### Rule 3: Always verify branch before starting mainnet
```bash
git branch  # must show * main
curl http://localhost:8746/health  # verify after start
```

### Rule 4: All mainnet scripts must use mainnet-deployer.json
```
commit_entropy.js     → mainnet-deployer.json ✅
reveal_entropy.js     → mainnet-deployer.json ✅
recover_commitment.js → mainnet-deployer.json ✅
```
Using `id.json` for mainnet = wrong PDA = stuck commitments forever.

### Rule 5: Keep minimum 10 XNT in mainnet wallet
```
Slash amount: 5 XNT
Safety buffer: 5 XNT
Minimum: 10 XNT at all times
Wallet: HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
```

---

## Start Commands

### Mainnet v5 PRODUCTION (main branch)
```bash
pkill -f daemon.py
sleep 2
cd ~/geiger-entropy-oracle/entropy-daemon
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
cd ~/geiger-entropy-oracle
git checkout testnet-vdf-verification
cd entropy-daemon
CONFIG_PATH=./config.toml \
SUBMIT_SCRIPT=./submit_entropy.js \
python3 daemon.py > logs/testnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8745/health
```

### v4 Fallback (mainnet-commit-reveal branch)
```bash
pkill -f daemon.py
sleep 2
cd ~/geiger-entropy-oracle
git checkout mainnet-commit-reveal
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8746/health
```

### Chain Spammer (stress testing only)
```bash
pkill -f daemon.py
sleep 2
cd ~/geiger-entropy-oracle
git checkout chain-spammer
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
sleep 5
curl http://localhost:8746/health
```

---

## Watch Logs
```bash
# Mainnet
tail -f ~/geiger-entropy-oracle/entropy-daemon/logs/mainnet-daemon.log

# Testnet
tail -f ~/geiger-entropy-oracle/entropy-daemon/logs/testnet-daemon.log
```

---

## After Every anchor build
```
Branch                    → Copy IDL to
─────────────────────────────────────────
main                      → idl/mainnet-commit-reveal/
testnet-vdf-verification  → idl/testnet/
```

Always fix IDL address after copying for mainnet:
```bash
sed -i 's/2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU/' \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json
```

---

## Stuck Commitment Recovery

If you see `UnrevealedCommitmentPending` spam:
```bash
# Step 1 — Kill daemon
pkill -f daemon.py
sleep 2

# Step 2 — Run recovery
cd ~/geiger-entropy-oracle/entropy-daemon
node mainnet/recover_commitment.js

# Step 3 — Restart daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
sleep 5
tail -f logs/mainnet-daemon.log
```

Recovery script outputs:
```
{"status":"clean"}   → no stuck commitment, safe to start
{"status":"slashed"} → cleared via slash, safe to start
{"status":"pending"} → within reveal window, auto-reveal attempted
```

> ⚠️ If recovery keeps finding STALE commitments, check wallet balance.
> Slash requires 5 XNT in operator wallet.

---

## Program Upgrade
```bash
cd entropy-contract
anchor build

# Update IDL
cp target/idl/geiger_entropy.json \
   ../entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json
sed -i 's/2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU/' \
   ../entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# Deploy upgrade (note: uses id.json for upgrade authority)
anchor upgrade \
  target/deploy/geiger_entropy.so \
  --program-id BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU \
  --provider.wallet ~/.config/solana/id.json \
  --provider.cluster https://rpc.mainnet.x1.xyz
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

## Branch → Scripts Mapping
```
main (PRODUCTION):
→ entropy-daemon/mainnet/commit_entropy.js
→ entropy-daemon/mainnet/reveal_entropy.js
→ entropy-daemon/mainnet/recover_commitment.js
→ entropy-daemon/idl/mainnet-commit-reveal/

testnet-vdf-verification:
→ entropy-daemon/testnet/commit_entropy.js
→ entropy-daemon/testnet/reveal_entropy.js
→ entropy-daemon/testnet/recover_commitment.js
→ entropy-daemon/idl/testnet/

mainnet-commit-reveal (v4 fallback):
→ entropy-daemon/mainnet/commit_entropy.js
→ entropy-daemon/mainnet/reveal_entropy.js
→ entropy-daemon/mainnet/recover_commitment.js
→ entropy-daemon/idl/mainnet-commit-reveal/

chain-spammer:
→ entropy-daemon/submit_entropy_mainnet.js
→ entropy-daemon/idl/mainnet/
```

---

## Safe Doc Updates (without stopping mainnet)

Use git worktree to update docs on other branches without affecting running daemon:
```bash
git worktree add /tmp/docs-update main
cp docs/setup-guide.md /tmp/docs-update/docs/setup-guide.md
cd /tmp/docs-update
git add -A
git commit -m "📝 Update docs"
git push
cd ~/geiger-entropy-oracle
git worktree remove /tmp/docs-update
```

---

## Cycle Timing (v5)
```
Decay event detected
→ VDF computed (50k-15k iters based on CPM)
→ Blind commit on-chain
→ 3 slot delay (~1.2s)
→ Reveal with SlotHash binding
→ 15s sleep
→ Next cycle

Total cycle: ~20-25 seconds
Daily cycles: ~3,000-4,000
Daily TXs: ~6,000-8,000
```

---

## Slash Mechanism
```
Slash amount:    5 XNT
Reveal deadline: 128 slots (~51 seconds)
Reporter bounty: 5 XNT (goes to whoever calls slash)
Auto-recovery:   3 layers — prevents accidental slash
Minimum wallet:  10 XNT at all times
```

---

*Last updated: March 29, 2026*
*Echo Hound Labs (@EchoHoundX) ☢️🦴*
