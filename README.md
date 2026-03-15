# ☢️ Geiger Entropy Oracle

> **The World's First Physical Entropy Oracle on X1 Blockchain**
> *True randomness sourced from quantum mechanical radioactive decay*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Python 3.9+](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/)
[![X1 Network](https://img.shields.io/badge/chain-X1-purple.svg)](https://x1.xyz)

---

## The Problem with Blockchain Randomness

Blockchains are deterministic computers. Every node must produce the exact same output
from the same inputs — meaning **true randomness is impossible natively**. Without a
trusted randomness source:

- NFT mints can be manipulated by validators
- On-chain lotteries can be rigged
- Game outcomes can be predicted and exploited
- "Random" selection is just hashed block data — which block producers can influence

## The Solution: Trust the Universe ☢️

Radioactive decay is governed by quantum mechanics. It is **fundamentally unpredictable**
— not just computationally hard, but *physically impossible* to predict. No amount of
compute power can tell you when the next atom will decay.

The **Geiger Entropy Oracle** captures this physical randomness using a GMC-500 Geiger
counter, processes it into cryptographically usable entropy, and posts it on-chain to X1
— where any smart contract can request a provably fair random number.

### The Genesis Node
The first entropy node runs beside **fossils from the Cenozoic Era** — creatures preserved
from the Miocene to Pleistocene epoch, roughly 2–23 million years ago. The same quantum
randomness that has governed matter since the Big Bang now secures your smart contracts.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    GMC-500 Geiger Counter                    │
│              (Quantum mechanical radioactive decay)          │
└──────────────────────────┬──────────────────────────────────┘
                           │ USB Serial → CSV export
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   Entropy Daemon (Python)                    │
│  • Watches CSV folder for new readings                       │
│  • Extracts 60 per-second CPS values per minute             │
│  • SHA-256 hashes the CPS array → 256-bit entropy seed      │
│  • Maintains rolling entropy pool (XOR of last N seeds)     │
│  • Signs seed with node ed25519 keypair                     │
│  • Exposes REST API on localhost:8745                        │
└──────────────────────────┬──────────────────────────────────┘
                           │ Solana TX
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Geiger Entropy Oracle (Anchor / X1)             │
│                                                              │
│  submit_entropy()     → verify sig → store in pool          │
│  request_randomness() → commit user seed                    │
│  fulfill_randomness() → XOR user seed + oracle pool         │
│                         → emit RandomnessResult event       │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
            Your dApp: NFT mints, lotteries, games...
```

---

## Comparison: Randomness Solutions on X1

| Feature                | Switchboard VRF       | Pyth Entropy      | **Geiger Entropy Oracle**       |
|------------------------|-----------------------|-------------------|---------------------------------|
| Chain                  | Solana                | EVM only          | **X1 (SVM-native)**             |
| Entropy Source         | TEE hardware enclave  | Commit-reveal     | **Physical radioactive decay**  |
| Trust Model            | Trust Intel/AMD chip  | Trust Pyth        | **Trust the laws of physics**   |
| Verifiable On-chain    | TEE attestation       | Cryptographic     | **ed25519 + on-chain**          |
| Node Hardware Cost     | High (TEE servers)    | Low               | **Low (GMC-500 ~$100)**         |
| Truly Unpredictable    | Computationally       | Computationally   | **Physically impossible**       |
| Deployed on X1         | ❌                    | ❌                | **✅ This project**             |

---

## Quick Start

### Prerequisites
- WSL2 (Ubuntu) or Linux
- Python 3.9+
- GMC-500 Geiger counter + GQ Electronics GMC Data Viewer
- X1 keypair (`~/.config/solana/id.json`)

### Install

```bash
git clone https://github.com/skywalker12345678/geiger-entropy-oracle
cd geiger-entropy-oracle/entropy-daemon
chmod +x install.sh
./install.sh
```

### Configure

Edit `entropy-daemon/config.toml`:

```toml
[daemon]
# Windows path via WSL: /mnt/c/Users/YourName/Documents/GMC
# Native WSL path:      /home/yourname/gmc-data
watch_folder = "/mnt/c/Users/YourName/Documents/GMC"
port = 8745

[node]
keypair_path = "~/.config/solana/id.json"

[entropy]
rolling_pool_size = 10
min_cpm = 10
```

### Run

```bash
# Foreground (testing)
cd entropy-daemon && source .venv/bin/activate && python daemon.py

# Background (systemd)
systemctl --user enable geiger-entropy
systemctl --user start geiger-entropy
```

### Verify

```bash
curl http://localhost:8745/health
curl http://localhost:8745/entropy
```

---

## dApp Integration

```typescript
// Request randomness
const { requestId } = await client.requestRandomness(userSeed);
// After oracle fulfills → your contract receives the result
const result = await client.awaitFulfillment(requestId);
```

---

## X1 Network Endpoints

| Network  | RPC                          |
|----------|------------------------------|
| Mainnet  | `https://rpc.x1.xyz`         |
| Testnet  | `https://rpc-testnet.x1.xyz` |

---

## Genesis Node Operator

**Skywalker** — `4jLcjZLcDcGuS1M4SHtBRPXs2h3HULUL2fDnCuaaLzzY`

Running beside fossils from the Cenozoic Era (Miocene to Pleistocene, ~2–23 million years ago).
The same quantum randomness that has governed matter since the Big Bang now secures your X1 smart contracts.

---

## Program ID

```
GeiGR4nd0mXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
*(Replace after `anchor build && anchor deploy`)*

---

## License

MIT © Skywalker (@skywalker12345678)
