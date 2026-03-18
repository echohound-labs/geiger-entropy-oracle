# ☢️ Geiger Entropy Oracle — Deployment Reference

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
| `main` | Mainnet | Fast VDF + direct submit | 🟢 Production |
| `testnet-vdf-verification` | Testnet | Full secure stack — commit-reveal + device fingerprint + slash | 🟡 Testing |
| `mainnet-commit-reveal` | Mainnet | Future upgrade — commit-reveal on mainnet | 🔵 Pending |
| `chain-spammer` | Mainnet | Ultra fast, no VDF — stress testing X1 | ⚪ Archived |

---

## IDL File Locations
```
entropy-daemon/idl/
├── mainnet/               ← main branch IDL
│   └── geiger_entropy.json
├── testnet/               ← testnet-vdf-verification IDL
│   └── geiger_entropy.json
└── mainnet-commit-reveal/ ← future mainnet upgrade IDL
    └── geiger_entropy.json
```

### IDL Update Rule
After every `anchor build` copy IDL to correct folder:
```bash
# Mainnet build (on main branch):
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet/geiger_entropy.json

# Testnet build (on testnet-vdf-verification branch):
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/testnet/geiger_entropy.json

# Mainnet-commit-reveal build:
cp entropy-contract/target/idl/geiger_entropy.json \
   entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json
```

---

## Wallet Keypairs
```
Testnet wallet:  ~/.config/solana/id.json
                 FB4jp1T1YB5ttaeCEvNisqPmVpqfQyet4WxN6HsQdqxh

Mainnet wallet:  ~/.config/solana/mainnet-deployer.json
                 HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
```

---

## Start Commands

### Mainnet (main branch)
```bash
git checkout main
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
```

### Testnet (testnet-vdf-verification branch)
```bash
git checkout testnet-vdf-verification
cd entropy-daemon
CONFIG_PATH=./config.toml \
SUBMIT_SCRIPT=./submit_entropy.js \
python3 daemon.py > logs/testnet-daemon.log 2>&1 &
```

### Chain Spammer (chain-spammer branch)
```bash
git checkout chain-spammer
cd entropy-daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &
```

---

## Health Checks
```bash
# Mainnet
curl http://localhost:8746/health

# Testnet
curl http://localhost:8745/health
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

*Last updated: March 18, 2026*
*Echo Hound Labs (@EchoHoundX) ☢️🦴*
