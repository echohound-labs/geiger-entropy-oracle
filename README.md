# ☢️ Geiger Entropy Oracle
### The World's First Physical Entropy Oracle with VDF-secured Randomness on X1 Blockchain
**True randomness sourced from quantum mechanical radioactive decay**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Python 3.9+](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![X1 Mainnet](https://img.shields.io/badge/X1-Mainnet-green.svg)](https://x1.xyz)
[![Echo Hound Labs](https://img.shields.io/badge/Echo%20Hound-Labs-purple.svg)](https://github.com/echohound-labs)

---

## 🔴 Live on X1 Mainnet
```
Program ID:    BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
Oracle State:  BygMTZ1oLBD9tDmssnt9LkNT7BEd2PCJBCzurwtMuTqm
Entropy Pool:  GDECYXCXietabJs9Y1baKzD3t4VFBw4eZWPnvYenyi77
Node PDA:      z4Psp8qVfP4t3jiWHE29rrisTPMC78tu8LmDhRSEL3s
Submissions:   7,000+ quantum decay events on-chain
Version:       v3 — VDF-secured Physical Entropy
```

[📄 Read the Whitepaper](docs/whitepaper.md) | [🔍 Explorer](https://explorer.mainnet.x1.xyz/address/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU) | [💬 Telegram](https://t.me/+axtvX9GbsnJkMGRh)

---

## The Problem with Blockchain Randomness

Blockchains are deterministic computers. Every node must produce identical outputs from identical inputs — meaning true randomness is impossible natively. Without a trusted randomness source:

- NFT mints can be manipulated by validators
- On-chain lotteries can be rigged
- Game outcomes can be predicted and exploited
- "Random" selection is just hashed block data

## The Solution: Trust the Universe ☢️

Radioactive decay is governed by quantum mechanics. It is fundamentally unpredictable — not just computationally hard, but physically impossible to predict. No amount of compute power can tell you when the next atom will decay.

Entropy is derived from the time between decay events (Δt), which follows a Poisson process and is fundamentally unpredictable. The Geiger Entropy Oracle captures this physical randomness using a GMC-500 Geiger counter, processes it through a Wesolowski VDF for cryptographic tamper-resistance, and posts it on-chain to X1 — where any smart contract can request a provably fair random number.

> "The chain of proof becomes: Physical decay (uncontrollable) → seed committed → VDF locks it in time → verifiable output. No one — including the operator — could have manipulated the result once the decay event was recorded."
> — Theo, X1 Community Architect

---

## Architecture — v3 (VDF-secured Physical Entropy)
```
┌─────────────────────────────────────────────────────────────┐
│                    GMC-500+ Geiger Counter                   │
│              (Quantum mechanical radioactive decay)          │
└──────────────────────────┬──────────────────────────────────┘
                           │ USB Serial (/dev/ttyUSB0)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              geiger_stream + daemon.py                       │
│  • polls CPS every 250ms via GETCPS command                 │
│  • detects rising edge decay events                         │
│  • extracts Δt between events                               │
│  • SHA256(Δt + timestamp + CPM + CPS) = raw seed            │
│  • Wesolowski VDF(seed, dynamic_iters) = tamper-proof       │
│  • Ed25519 signs final seed                                 │
│  • pushes to entropy queue                                  │
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
                           ▓
            Your dApp: NFT mints, lotteries, games...
```

### Why VDF-secured Physical Entropy?
```
Signature alone: "Trust me I did not cheat"
VDF-secured Physical Entropy:  "Here's a cryptographic proof cheating was impossible"

Physical decay  = quantum unpredictable
VDF enforces minimum delay, preventing manipulation after entropy capture
Ed25519 signed  = verifiable on-chain
```

---

## Comparison

| Feature | Switchboard VRF | Chainlink VRF | Geiger Oracle |
|---------|----------------|---------------|---------------|
| Chain | Solana/SVM | Multi-chain | X1 Native |
| Entropy Source | TEE hardware | Cryptographic | Physical decay |
| Trust Model | Trust Intel | Trust Chainlink | Trust physics |
| VDF Layer | No | No | Yes ✓ |
| Physical Entropy | No | No | Yes ✓ |
| Deployed on X1 | ❌ | ❌ | ✅ Live |
| Node Cost | High | High | ~$155 |

---

## Use It Today

Any dApp on X1 can integrate right now:
```javascript
// Request randomness
const userSeed = crypto.randomBytes(32);
await program.methods
    .requestRandomness(Array.from(userSeed))
    .accounts({
        oracleState: ORACLE_STATE,
        entropyPool: ENTROPY_POOL,
        randomnessRequest: requestPDA,
        requester: wallet.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .rpc();

// Fulfill and read result
await program.methods
    .fulfillRandomness()
    .accounts({...})
    .rpc();

const request = await program.account
    .randomnessRequest.fetch(requestPDA);
// result = 256-bit quantum random number
console.log(Buffer.from(request.result).toString('hex'));
```

**Use cases:**
```
🎰 Lotteries and raffles  → provably fair draws
🎮 On-chain games         → unbiasable outcomes
🖼️ NFT trait generation   → fair attribute assignment
🗳️ DAO selection          → random committee members
💰 DeFi                   → random liquidation ordering
```

---

## RADS Token — Coming Q2 2026

RADS is a novel token where supply is controlled entirely by radioactive decay.
```
Max Supply:  1,000,000 RADS — ever
Emission:    4 years
Mint:        Oracle program only — automatic

Year 1: 400,000 RADS (40%) — highest rewards
Year 2: 300,000 RADS (30%)
Year 3: 200,000 RADS (20%)
Year 4: 100,000 RADS (10%)
```

**How to earn RADS:**
1. Buy a GMC-500 Geiger counter (~$100)
2. Run the entropy daemon
3. Register your node on X1
4. Earn RADS automatically from every decay event

> "The universe controls the supply. No team can mint extra RADS. Ever." ☢️

---

## Run a Node

**Device Fingerprinting:**
On first run the daemon automatically registers your GMC-500 hardware fingerprint. If someone swaps your device the daemon refuses to start. To reset: `rm entropy-daemon/.geiger_device_fingerprint`


📖 **[Full Setup Guide](docs/setup-guide.md)** — Windows WSL2, Raspberry Pi, troubleshooting


**Hardware:**
```
GMC-500+ Geiger Counter: ~$100
Raspberry Pi 4:          ~$35
Total:                   ~$135
```

**Works on:**
```
✓ USB direct to PC/laptop
✓ Raspberry Pi
✓ Any Linux machine
✓ WSL2 on Windows
```

**Quick Start:**
```bash
git clone https://github.com/echohound-labs/geiger-entropy-oracle
cd geiger-entropy-oracle/entropy-daemon
pip install -r requirements.txt
cp config.toml config-mainnet.toml
# Edit config-mainnet.toml with your settings
CONFIG_PATH=./config-mainnet.toml python3 daemon.py
```

**Verify:**
```bash
curl http://localhost:8745/health
curl http://localhost:8745/entropy
```

---

## REST API

**GET /health**
```json
{
  "status": "ok",
  "uptime_seconds": 3600,
  "total_submissions": 7000,
  "latest_cpm": 20,
  "vdf_iters": 50000
}
```

**GET /entropy**
```json
{
  "seed": "4e3f...",
  "pool_seed": "9ab1...",
  "cpm": 20,
  "usv_h": 0.13,
  "timestamp": 1773637219,
  "signature": "ed25519...",
  "vdf_iters": 50000,
  "vdf_time_ms": 170.3,
  "total_submissions": 7000
}
```

---

## X1 Network Endpoints

| Network | RPC |
|---------|-----|
| Mainnet | https://rpc.mainnet.x1.xyz |
| Testnet | https://rpc.testnet.x1.xyz |

---

## The Genesis Node

Running beside fossils from the Cenozoic Era — Miocene to Pleistocene epoch, roughly 2–23 million years ago. The same quantum randomness that has governed matter since the Big Bang now secures X1 smart contracts. 🦴
```
Operator: Skywalker (@skywalker12345678)
Org:      Echo Hound Labs (@EchoHoundX)
Location: Florida, USA
Hardware: GMC-500+ Geiger Counter
Wallet:   HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
Live:     March 16, 2026
```

---

## Repository Structure
```
geiger-entropy-oracle/
├── entropy-contract/     Anchor smart contract (Rust)
├── entropy-daemon/       Python entropy daemon (VDF-secured)
└── docs/
    └── whitepaper.md     Full technical whitepaper
```

---

## Links

- 📄 [Whitepaper](docs/whitepaper.md)
- 🐦 [Twitter](https://twitter.com/EchoHoundX)
- 💬 [Telegram](https://t.me/+axtvX9GbsnJkMGRh)
- 🔍 [Explorer](https://explorer.mainnet.x1.xyz/address/BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU)
- 🌐 [X1 Network](https://x1.xyz)

---

## License

MIT © Echo Hound Labs (@EchoHoundX)

*Building X1 Infrastructure from the ground up* 🦴☢️
