# ☢️ Geiger Entropy Oracle — Deployment Reference v5

## Network Addresses

### Mainnet (X1 Mainnet)
```
Program ID:    BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
Oracle State:  BygMTZ1oLBD9tDmssnt9LkNT7BEd2PCJBCzurwtMuTqm
Entropy Pool:  GDECYXCXietabJs9Y1baKzD3t4VFBw4eZWPnvYenyi77
Node PDA:      z4Psp8qVfP4t3jiWHE29rrisTPMC78tu8LmDhRSEL3s
IDL Account:   vriwJsd8QLAzCF3cpv8uavBUirBMAMgyjZT2aRzzoV8
Operator:      HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
RPC:           https://rpc.mainnet.x1.xyz
Explorer:      https://explorer.mainnet.x1.xyz
Version:       v5 — SlotHash Binding + SHA256 Chained Pool + Domain Separation
Slash Amount:  5 XNT
Status:        🟢 LIVE
```

### Testnet (X1 Testnet)
```
Program ID:    2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY
Oracle State:  CrrLuXpoCuK8szmtXxBEDPc5FTkbUGzEWfMyjeSL83bS
Entropy Pool:  KMgwwzxYxrXufHySyMNchwyhupsNNsc4wPN71xtqoGG
Node PDA:      3KA1UPPZf1N36RgmLwDKqAdJB6WPnX44aBs9rgP3TvdV
Operator:      FB4jp1T1YB5ttaeCEvNisqPmVpqfQyet4WxN6HsQdqxh
RPC:           https://rpc.testnet.x1.xyz
Explorer:      https://explorer.x1.xyz/?cluster=testnet
Status:        🟡 TESTING
```

---

## Branch → Network Mapping

| Branch | Network | Description | Status |
|--------|---------|-------------|--------|
| `main` | Mainnet | v5 PRODUCTION — SlotHash binding + SHA256 chained pool + 5 XNT slash + 15s cycle | 🟢 Production |
| `mainnet-commit-reveal` | Mainnet | v4 legacy — commit-reveal + VDF + device fingerprint | 🔵 Fallback |
| `testnet-vdf-verification` | Testnet | Testing sandbox — same as main pointing to testnet | 🟡 Testing |
| `chain-spammer` | Mainnet | Ultra fast, no VDF — stress testing X1 | ⚪ Stress Test |

---

## Version History

| Version | Branch | Date | Changes |
|---------|--------|------|---------|
| v5 | `main` | March 29, 2026 | SlotHash binding, SHA256 chained pool, GEIGER_POOL_V1 domain separator, 5 XNT slash, 15s cycle sleep, auto-reveal recovery |
| v4 | `mainnet-commit-reveal` | March 16, 2026 | Commit-reveal, VDF, device fingerprint, slash mechanism |
| v3 | `main` (old) | March 2026 | Fast VDF direct submit |

---

## IDL File Locations
```
entropy-daemon/idl/
├── mainnet-commit-reveal/ ← PRODUCTION IDL (main branch uses this)
│   └── geiger_entropy.json
├── mainnet/               ← legacy main branch IDL
│   └── geiger_entropy.json
└── testnet/               ← testnet-vdf-verification IDL
    └── geiger_entropy.json
```

### IDL Update Rule
After every `anchor build` copy IDL to correct folder and fix address:
```bash
# Main branch (production):
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# Fix address to mainnet (if built on testnet):
sed -i 's/"address": "2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY"/"address": "BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU"/' \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json

# Testnet branch:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/testnet/geiger_entropy.json
```

---

## Wallet Keypairs
```
Testnet wallet:  ~/.config/solana/id.json
                 FB4jp1T1YB5ttaeCEvNisqPmVpqfQyet4WxN6HsQdqxh

Mainnet wallet:  ~/.config/solana/mainnet-deployer.json
                 HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
```

> ⚠️ **Critical:** All mainnet scripts (commit_entropy.js, reveal_entropy.js, recover_commitment.js) must use `mainnet-deployer.json`. Using `id.json` for mainnet will result in wrong PDA derivation and stuck commitments.

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

> ⚠️ **CRITICAL:** Always `pkill -f daemon.py` before switching branches. Scripts disappear on branch switch and will break the running daemon immediately.

---

## Health Checks
```bash
# Mainnet (port 8746)
curl http://localhost:8746/health
curl http://localhost:8746/entropy

# Testnet (port 8745)
curl http://localhost:8745/health
curl http://localhost:8745/entropy
```

---

## Recovery Commands

### Check for stuck commitments
```bash
cd ~/geiger-entropy-oracle/entropy-daemon
node mainnet/recover_commitment.js
```

### Manual slash to clear stuck commitment
```bash
cd ~/geiger-entropy-oracle/entropy-daemon
node mainnet/recover_commitment.js
# If status is STALE it will auto-slash and clear
```

### Watch live logs
```bash
tail -f ~/geiger-entropy-oracle/entropy-daemon/logs/mainnet-daemon.log
```

---

## Device Fingerprint
```
Hardware:    GMC-500+Re 2.5
Serial:      33090041333133
USB VID:PID: 6790:29987
Fingerprint: 83ff336d752b6b12...
File:        entropy-daemon/.geiger_device_fingerprint
```

To reset fingerprint (if replacing hardware):
```bash
rm entropy-daemon/.geiger_device_fingerprint
# Daemon re-registers on next start
```

---

## Program Upgrade Authority
```
Upgrade Authority: HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
                   (mainnet-deployer.json)
Status:            Retained — active development
Planned:           Revoke after Phase 5 audit
```

To upgrade program:
```bash
cd entropy-contract
anchor build
anchor upgrade \
  target/deploy/geiger_entropy.so \
  --program-id BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU \
  --provider.wallet ~/.config/solana/id.json \
  --provider.cluster https://rpc.mainnet.x1.xyz
```

---

*Last updated: March 29, 2026*
*Echo Hound Labs (@EchoHoundX) ☢️🦴*
