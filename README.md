# ☢️ Geiger Entropy Oracle
### The World's First Physical Entropy Oracle with Five Independent Security Layers on X1 Blockchain
**True randomness sourced from quantum mechanical radioactive decay — secured by physics, cryptography, and blockchain consensus**

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
Submissions:   58,000+ quantum decay events on-chain
Version:       v5 — SlotHash Binding + SHA256 Chained Pool + Domain Separation
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

Radioactive decay is governed by quantum mechanics. It is fundamentally unpredictable — not just computationally hard, but **physically impossible to predict**. No amount of compute power can tell you when the next atom will decay. Not now. Not ever.

The Geiger Entropy Oracle captures this physical randomness using a GMC-500 Geiger counter placed beside Cenozoic fossils — 2 to 23 million years old. The inter-event timing (Δt) between decay events follows a true Poisson process, the same quantum randomness that has governed matter since the Big Bang.

But physical entropy alone is not enough. A dishonest operator could still cherry-pick favorable readings. That's why every decay event is immediately locked by a Wesolowski VDF, blind-committed on-chain, bound to an X1 SlotHash outside the operator's control, and mixed into a domain-separated SHA256 chained pool. Withholding is made economically irrational by a 20 XNT slash mechanism.

The result: an entropy source that no single party — not even the operator — can predict, manipulate, or selectively withhold.

> "The chain of proof becomes: Physical decay (uncontrollable) → seed committed → VDF locks it in time → SlotHash binds it to consensus → verifiable output. No one — including the operator — could have manipulated the result."
> — Theo, X1 Community Architect

This is not "trust me bro" randomness. This is trust physics. ☢️

---

## Five Independent Security Layers

The Geiger Entropy Oracle employs five independent security layers. An attacker must simultaneously defeat all five — each from a fundamentally different domain of physics, cryptography, blockchain consensus, and game theory.

### Layer 1 — Physical Quantum Entropy
Radioactive decay from Cenozoic fossils is quantum mechanical. It is not computationally hard to predict — it is **physically impossible to predict**. The inter-event timing (Δt) follows a true Poisson process governed by the same quantum randomness that has existed since the Big Bang. No computer, no algorithm, no adversary can predict when the next atom decays.

### Layer 2 — Wesolowski VDF Time Lock
After capturing the decay event, a Verifiable Delay Function (VDF) is computed. This creates a cryptographic time lock — the final seed cannot be known until the VDF computation completes. Dynamic iterations ensure the VDF always takes longer than one X1 slot (~400ms), making post-capture manipulation impossible.
```
CPM < 20  → 50,000 iterations (~0.17s)
CPM < 50  → 30,000 iterations (~0.10s)
CPM < 100 → 20,000 iterations (~0.08s)
CPM 100+  → 15,000 iterations (~0.05s)
```

### Layer 3 — X1 SlotHash Binding ✨ NEW in v5
At reveal time, the current X1 SlotHash is mixed into the final seed:
```
bound_seed = SHA256(vdf_output || slot_hash || sequence)
```

The slot hash is determined by X1 blockchain consensus — completely outside the operator's control. Even if an adversary could somehow predict the physical decay and the VDF output, they cannot predict a future slot hash. Two fundamentally independent and unpredictable entropy sources are combined on every single reveal.

### Layer 4 — Domain-Separated SHA256 Chained Pool ✨ NEW in v5
The entropy pool uses cryptographic hashing with domain separation — not linear XOR mixing:
```
state = SHA256("GEIGER_POOL_V1" || state || seed)  × 32 seeds
```

SHA256 is the same cryptographic primitive securing Bitcoin, Solana PDAs, and X1 transaction signing. The GEIGER_POOL_V1 domain separator prevents cross-protocol collisions and makes the design uniquely attributable to this protocol forever. Each new entropy contribution is irreversibly folded into the pool — no attacker can isolate, reverse, or cancel any individual seed's contribution.

> "Even if one input is weak, it is cryptographically mixed into a non-linear pool." — Theo, X1 Community Architect

### Layer 5 — Economic Slash Mechanism
If the operator commits entropy but fails to reveal within 128 slots, anyone can call slash_missed_reveal() and claim 20 XNT from the operator. Selective withholding — choosing not to reveal an unfavorable seed — is economically irrational. The bounty goes directly to the reporter — creating a self-policing incentive where anyone watching the chain is rewarded for catching lazy or malicious operators. The blind commit-reveal scheme ensures the operator cannot see the final output before being locked in.

---

## Architecture — v5
```
┌─────────────────────────────────────────────────────────────┐
│                    GMC-500+ Geiger Counter                   │
│         Cenozoic fossils — 2–23 million years old            │
│              (Quantum mechanical radioactive decay)          │
└──────────────────────────┬──────────────────────────────────┘
                           │ USB Serial (/dev/ttyUSB0)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      daemon.py                               │
│                                                              │
│  • GMC-500 hardware fingerprint verification (device lock)  │
│  • polls CPS every 250ms via <GETCPS>> command              │
│  • detects rising edge decay events                         │
│  • extracts Δt between events (Poisson process)             │
│  • SHA256(Δt + timestamp + CPM + CPS) = raw seed            │
│  • Wesolowski VDF(seed, dynamic_iters) = time-locked        │
│  • Ed25519 signs final seed                                 │
│  • BLIND commit hash on-chain (commit-reveal)               │
│  • reveal after 3 slots — SlotHash mixed on-chain           │
│  • auto-recovery on RPC timeout or restart                  │
│  • slash mechanism for missed reveals                       │
│  • 15s cycle sleep — cost efficient, pool always fresh      │
└──────────────────────────┬──────────────────────────────────┘
                           │ Solana TX
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Geiger Entropy Oracle (Anchor / X1)             │
│                                                              │
│  commit_entropy()     → blind hash on-chain                 │
│  reveal_entropy()     → SlotHash bind + verify + pool       │
│  slash_missed_reveal()→ slash operator for missed reveal    │
│  request_randomness() → commit user seed                    │
│  fulfill_randomness() → SHA256 chain pool + user seed       │
└──────────────────────────┬──────────────────────────────────┘
                           ▓
            Your dApp: NFT mints, lotteries, games...
```

### What gets logged on-chain forever
Every reveal is permanently recorded on X1:
```
☢️ Entropy revealed | seq=58077 CPM=26 uSv/h=0.169 dt=2.637s
VDF=30000iters seed=[104,88,184,213] slot_hash=[194,253,135,141]
binding_slot=39512140 sources=0x07 verified✓
```

sources=0x07 means all three entropy layers active:
- 0x01 = Physical Geiger decay
- 0x02 = Wesolowski VDF
- 0x04 = X1 SlotHash binding

This is a permanent scientific record. Every decay event timestamped to the nanosecond, with CPM and µSv/h radiation readings — immutable and publicly auditable forever.

---

## Security Model
```
What an attacker must simultaneously defeat:

Layer 1 — Predict quantum radioactive decay     (physically impossible)
Layer 2 — Predict VDF output before completion  (computationally impossible)
Layer 3 — Predict future X1 slot hash          (consensus impossible)
Layer 4 — Reverse SHA256 chained pool          (cryptographically impossible)
Layer 5 — Accept 0.1 SOL slash for withholding (economically irrational)
```

No other oracle on any SVM chain requires an attacker to defeat five independent layers from five different domains simultaneously.

---

## Comparison

| Feature | Switchboard VRF | Chainlink VRF | PHOTON | Geiger Oracle |
|---------|----------------|---------------|--------|---------------|
| Chain | Solana/SVM | Multi-chain | X1 | X1 Native |
| Entropy Source | TEE hardware | Cryptographic | 5 APIs | Physical decay |
| Trust Model | Trust Intel | Trust Chainlink | Trust APIs | Trust physics |
| VDF Layer | No | No | No | Yes ✓ |
| Physical Entropy | No | No | No | Yes ✓ |
| SlotHash Binding | No | No | No | Yes ✓ |
| SHA256 Chained Pool | No | No | No | Yes ✓ |
| Domain Separation | No | No | No | Yes ✓ |
| Commit-Reveal | No | No | No | Yes ✓ |
| Device Fingerprint | No | No | No | Yes ✓ |
| Works Offline | No | No | No | Yes ✓ |
| Deployed on X1 | ❌ | ❌ | ✅ | ✅ Live |
| Node Cost | High | High | Low | ~$135 |
| Security Layers | 1 | 1 | 1 | 5 ✓ |

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

**Important:** For maximum security, your dApp should provide an unpredictable userSeed — for example SHA256(user_wallet || nonce) where the nonce is committed before the oracle seed is known.

**Use cases:**
```
🎰 Lotteries and raffles  → provably fair draws
🎮 On-chain games         → unbiasable outcomes
🖼️ NFT trait generation   → fair attribute assignment
🗳️ DAO selection          → random committee members
💰 DeFi                   → random liquidation ordering
```

---

## ENTROPY Token — Coming Q2 2026

ENTROPY is a novel token where supply is controlled entirely by radioactive decay.
```
Max Supply:  1,000,000 ENTROPY — ever
Emission:    4 years equal distribution
Mint:        Oracle program only — no team can mint extra

Year 1: 250,000 ENTROPY (25%)
Year 2: 250,000 ENTROPY (25%)
Year 3: 250,000 ENTROPY (25%)
Year 4: 250,000 ENTROPY (25%)
```

Token launch prerequisites — NOT MET YET:
- Multi-node operators
- Staking contract
- Slash in ENTROPY
- Statistical audit

> "The universe controls the supply. No team can mint extra ENTROPY. Ever." ☢️

---

## Run a Node

**Device Fingerprinting:**
On first run the daemon automatically registers your GMC-500 hardware fingerprint using the internal serial number, USB VID:PID, and firmware version. If someone swaps your device the daemon refuses to start.

To reset: `rm entropy-daemon/.geiger_device_fingerprint`

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
cd geiger-entropy-oracle/entropy-contract
npm install
cd ../entropy-daemon
pip3 install -r requirements.txt --break-system-packages
cp config.toml config-mainnet.toml
# Edit config-mainnet.toml with your settings
chmod +x start.sh
./start.sh
```

**Verify:**
```bash
curl http://localhost:8746/health
curl http://localhost:8746/entropy
```

---

## REST API

**GET /health**
```json
{
  "status": "ok",
  "uptime_seconds": 3600,
  "total_submissions": 58000,
  "latest_cpm": 22,
  "vdf_iters": 30000
}
```

**GET /entropy**
```json
{
  "seed": "4e3f...",
  "pool_seed": "9ab1...",
  "cpm": 22,
  "usv_h": 0.143,
  "timestamp": 1773637219,
  "signature": "ed25519...",
  "vdf_iters": 30000,
  "vdf_time_ms": 103.2,
  "total_submissions": 58000
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

Running beside fossils from the Cenozoic Era — Miocene to Pleistocene epoch, roughly 2–23 million years ago. The same quantum randomness that has governed matter since the Big Bang now secures X1 smart contracts.

Each decay event is quantum mechanical — impossible to predict, permanently recorded on X1, cryptographically verified, and auditable by anyone forever. This is not "trust me bro" randomness. This is trust physics. 🦴
```
Operator: Skywalker (@skywalker12345678)
Org:      Echo Hound Labs (@EchoHoundX)
Location: Florida, USA
Hardware: GMC-500+ Geiger Counter
Fossils:  Cenozoic Era — 2–23 million years old
Wallet:   HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
Live:     March 16, 2026
```

---

## Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 — Genesis | ✅ Complete | VDF, commit-reveal, device fingerprint, slash, 58k+ submissions |
| Phase 2 — v5 Upgrade | ✅ Complete | SlotHash binding, SHA256 chained pool, domain separation |
| Phase 3 — Token | 🔜 Q2 2026 | ENTROPY SPL token, emission tied to reveal_entropy() |
| Phase 4 — Multi-Node | 🔜 Planned | Multiple operators, staking, slash in ENTROPY |
| Phase 5 — Immutable | 🔜 Planned | Third-party audit, revoke upgrade authority |

---

## Repository Structure
```
geiger-entropy-oracle/
├── entropy-contract/     Anchor smart contract (Rust)
├── entropy-daemon/       Python entropy daemon (VDF-secured)
└── docs/
    ├── whitepaper.md     Full technical whitepaper
    ├── setup-guide.md    Node operator setup guide
    ├── deployments.md    Network addresses & branch mapping
    └── operations.md     Critical operations guide
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

*The universe controls the supply. Trust physics.* 🦴☢️
