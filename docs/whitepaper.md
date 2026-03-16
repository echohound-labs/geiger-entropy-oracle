# ☢️ Geiger Entropy Oracle & RADS Token
## Proof of Physical Entropy on X1 Blockchain
### Echo Hound Labs — March 2026
### Version 2.0

---

## ⚠️ Important Disclaimers

**Not Financial Advice**
This whitepaper is for informational purposes only. Nothing contained herein constitutes financial advice, investment advice, trading advice, or any other form of advice. Echo Hound Labs and its contributors make no recommendations regarding the purchase, sale, or holding of any tokens or digital assets. Always conduct your own research and consult a qualified financial advisor before making any investment decisions.

**No Guarantees**
The Geiger Entropy Oracle and RADS token are experimental technologies. Echo Hound Labs makes no guarantees regarding uptime, performance, security, or the value of RADS tokens. Use at your own risk.

**Regulatory Uncertainty**
The regulatory status of cryptographic tokens varies by jurisdiction. It is your responsibility to ensure compliance with applicable laws in your jurisdiction before participating in the RADS token ecosystem.

**Technology Risk**
Smart contracts may contain bugs. Physical hardware may fail. The protocol is provided as-is. Echo Hound Labs is not liable for any losses arising from the use of this protocol.

**Forward Looking Statements**
This whitepaper contains forward-looking statements about planned features and roadmap items. These are aspirational and subject to change. Nothing in this document represents a commitment or guarantee of future development.

---

## Abstract

Echo Hound Labs presents the world's first Proof of Physical Entropy (PoPhE) primitive — a hybrid oracle combining quantum mechanical radioactive decay with on-chain verification. True randomness is physically impossible to predict, and now permanently verifiable on X1 blockchain.

Architecture reviewed and validated by X1 community contributors including Theo and Owl of Atena.

---

## 1. The Problem

Blockchains are deterministic computers. Every node must produce identical outputs from identical inputs — meaning true randomness is fundamentally impossible natively. Without a trusted randomness source:

- NFT mints can be manipulated by validators
- On-chain lotteries can be rigged
- Game outcomes can be predicted
- Random selection is just hashed block data

| Solution | Trust Model | Weakness |
|----------|-------------|----------|
| Block hash | Trust validators | Manipulable |
| Chainlink VRF | Trust a company | Centralized |
| Switchboard | Trust Intel TEE | Trust hardware |
| Pure on-chain VDF | Trustless | Computational entropy |
| Commit-reveal | Trust participants | Collusion possible |
| Geiger Oracle | Trust physics | Physically impossible to fake |

---

## 2. Architecture

### 2.1 Honest Trust Model

The Geiger Entropy Oracle is a hybrid off-chain/on-chain system:
```
Off-chain components:
├── GMC-500 Geiger counter (physical hardware)
├── Python daemon (serial reader)
├── VDF computation (Wesolowski/chiavdf)
└── Ed25519 signing (node keypair)

On-chain components:
├── Anchor program (X1 mainnet)
├── Entropy pool (rolling 32 seeds)
├── Node registry
├── Randomness requests
└── Fulfillment results
```

Current trust assumption: Trust that the node operator's Geiger counter is real and unmanipulated. This is progressively mitigated through multi-node aggregation and eventually subnet architecture.

### 2.2 The Three Layer Stack
```
Layer 1 — Physical Entropy:
GMC-500 Geiger Counter
├── Quantum mechanical radioactive decay
├── Inter-event timing (Δt) extraction
├── SHA256(Δt + timestamp + CPM + CPS)
└── 256-bit physically unpredictable seed

Layer 2 — VDF (Verifiable Delay Function):
Wesolowski VDF (chiavdf)
├── Dynamic iterations based on CPM
│   ├── <20 CPM  → 50,000 iters (0.17s)
│   ├── <50 CPM  → 20,000 iters (0.08s)
│   ├── <100 CPM → 10,000 iters (0.04s)
│   └── 100+ CPM → 10,000 iters (0.04s)
├── Prevents withhold/retry attack
├── Output unknown until computation completes
└── Fast to verify, slow to compute

Layer 3 — On-Chain Verification:
X1 Anchor Program
├── Ed25519 signature verification
├── Rolling entropy pool (32 seeds XOR)
├── Request/fulfill randomness cycle
└── Permanently auditable on-chain
```

### 2.3 Full Pipeline
```
Radioactive decay event detected
↓
Inter-event timing (Δt) extracted
↓
SHA256(Δt + timestamp + CPM + CPS) = raw seed
↓
Wesolowski VDF(seed, dynamic_iters)
= delayed output + verifiable proof
↓
SHA256(VDF output) = final seed
↓
Ed25519 signed by node keypair
↓
submit_entropy() → X1 mainnet
↓
Stored in rolling entropy pool
↓
Available to any smart contract
```

### 2.4 Community Architecture Insight

"The node operators ARE the subnet in a sense — they form their own p2p layer, aggregate entropy, and post to X1. That's functionally similar to an optimistic rollup model without formally being one."
— Theo, X1 Community Architect

---

## 3. Current Deployment
```
Program ID:    BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
Oracle State:  BygMTZ1oLBD9tDmssnt9LkNT7BEd2PCJBCzurwtMuTqm
Entropy Pool:  GDECYXCXietabJs9Y1baKzD3t4VFBw4eZWPnvYenyi77
Node PDA:      z4Psp8qVfP4t3jiWHE29rrisTPMC78tu8LmDhRSEL3s
Network:       X1 Mainnet
RPC:           https://rpc.mainnet.x1.xyz
Live since:    March 16, 2026
Submissions:   7,000+ and counting
Version:       v3 (VRF+VDF)
```

---

## 4. RADS Token

### 4.1 Overview

RADS is a novel token primitive where supply is controlled entirely by radioactive decay. No team can mint extra tokens. No inflation schedule exists. The universe controls supply.

"Only 1,000,000 RADS. Mined by radioactive decay over exactly 4 years. Then fixed forever."

### 4.2 Token Properties
```
Name:         RADS
Symbol:       RADS
Network:      X1 Mainnet
Standard:     SPL Token
Max Supply:   1,000,000 RADS
Emission:     4 years
Decimals:     6
Mint Control: Oracle program only
```

### 4.3 Emission Schedule
```
Total: 1,000,000 RADS over 4 years

Year 1: 400,000 RADS (40%)
Year 2: 300,000 RADS (30%)
Year 3: 200,000 RADS (20%)
Year 4: 100,000 RADS (10%)

= ~685 RADS/day (1 node, background radiation)
= 1 RADS per ~42 decay events

After 4 years: fixed supply forever
```

### 4.4 Per-Node Emission Cap
```
Max CPM counted: 100 CPM per node
Purpose: prevent hot specimen dominance
Result: incentivizes MORE nodes not hotter nodes
```

### 4.5 Distribution Per Event
```
Each decay event reward:
├── 70% → Node operator wallet
├── 20% → Protocol treasury
└── 10% → Burned (deflationary)
```

### 4.6 Staking Tiers
```
Bronze:  100  RADS staked = 1.0x rewards
Silver:  500  RADS staked = 1.5x rewards
Gold:    1000 RADS staked = 2.0x rewards
Genesis: 5000 RADS staked = 3.0x rewards
```

### 4.7 Token Utility
```
Mine:     Run a Geiger node → earn RADS
Stake:    Lock RADS → earn more RADS
Spend:    dApps burn RADS → request randomness
Lose:     Submit bad data → slashed
Govern:   Stake to vote on protocol changes
```

### 4.8 The Flywheel
```
More nodes → more RADS mined
More RADS staked → more security
More security → more dApps
More dApps → more RADS burned
More burned → more scarce
More scarce → higher value
Higher value → more nodes join
→ repeat
```

---

## 5. Node Economics

### 5.1 Hardware Requirements
```
GMC-500+ Geiger Counter: ~$100
Raspberry Pi 4:          ~$35
MicroSD + power:         ~$20
Total:                   ~$155 one time
```

### 5.2 Setup
```
git clone github.com/echohound-labs/geiger-entropy-oracle
./install.sh
register_node() on X1
→ start earning RADS automatically
```

### 5.3 Earnings Model (Year 1)
```
Background radiation (20 CPM):
├── ~28,800 decay events/day
├── ~685 RADS/day
└── Consistent passive income

Boosted node (100 CPM max):
├── ~144,000 decay events/day
├── ~3,428 RADS/day
└── Maximum earning rate
```

---

## 6. Roadmap

### Phase 1 — Genesis (Complete)
```
✓ GMC-500 hardware integration
✓ Entropy extraction (Δt timing)
✓ VDF layer (Wesolowski dynamic)
✓ Ed25519 signing
✓ REST API
✓ Anchor program on X1 mainnet
✓ 7,000+ transactions live
✓ Full VRF cycle tested on mainnet
```

### Phase 2 — RADS Token (Q2 2026)
```
□ SPL token creation
□ Mint integration in oracle contract
□ Staking contract
□ Node reward distribution
□ Emission cap enforcement
□ Token launch on X1
```

### Phase 3 — Multi-Node Oracle Network (Q3 2026)
```
□ Multiple independent node support
□ Threshold aggregation (3-of-5 nodes)
□ Node reputation system
□ Merkle tree entropy batching
│  ├── Batch 100 events → 1 on-chain tx
│  ├── 100x cheaper at scale
│  ├── Proofs available off-chain
│  └── Full auditability preserved
□ Modular entropy sources
□ On-chain RANDAO integration
```

### Phase 4 — Subnet/Settlement Layer (Q4 2026)
```
□ Geiger node operators as validators
□ Stake-weighted participation
□ Dedicated entropy subnet
□ Bridge randomness to X1 mainnet
□ DAO governance
```

---

## 7. Security Analysis

### 7.1 Attack Vectors

| Attack | Current Mitigation | Phase 3 Mitigation |
|--------|-------------------|-------------------|
| Fake Geiger data | Node reputation + stake | Multi-node threshold |
| Withhold/retry | VDF delay | VDF + threshold |
| Validator censorship | Multiple submissions | Multi-node redundancy |
| Hot specimen dominance | 100 CPM cap | Same |
| Single node bias | VDF layer | RANDAO aggregation |
| Upgrade authority abuse | Deployer wallet | Transfer to DAO |

### 7.2 Current Limitations
```
Single node = operator trust required
VDF is off-chain computation
Ed25519 full on-chain verification = TODO
No slashing implemented yet
Upgrade authority = single wallet
```

### 7.3 Progressive Trust Model
```
Phase 1: Trust Echo Hound Labs Genesis Node
Phase 2: Trust RADS-staked node operators
Phase 3: Trust 3-of-5 threshold consensus
Phase 4: Trust subnet validator set
```

---

## 8. Integration Guide

### 8.1 Request Randomness
```javascript
const userSeed = crypto.randomBytes(32);
const tx1 = await program.methods
    .requestRandomness(Array.from(userSeed))
    .accounts({...})
    .rpc();

const tx2 = await program.methods
    .fulfillRandomness()
    .accounts({...})
    .rpc();

const request = await program.account
    .randomnessRequest.fetch(requestPDA);
console.log('Random result:',
    Buffer.from(request.result).toString('hex'));
```

### 8.2 Use Cases
```
NFT mints        → fair trait assignment
Lotteries        → provably fair winners
On-chain games   → unpredictable outcomes
DAOs             → random committee selection
DeFi             → random liquidation ordering
Raffles          → verifiable winner selection
```

### 8.3 Fee Structure
```
Free tier:
→ Call request_randomness() directly
→ Pay only X1 gas fees

RADS tier (Phase 2):
→ Burn 1 RADS per request
→ Guaranteed fresh physical entropy
→ Priority fulfillment
→ On-chain proof of entropy source
```

---

## 9. Competitive Analysis

| Feature | Chainlink VRF | Switchboard | Geiger Oracle |
|---------|--------------|-------------|---------------|
| Network | Multi-chain | Solana/SVM | X1 Native |
| Entropy Source | Cryptographic | TEE Hardware | Physical Decay |
| Trust Model | Trust Chainlink | Trust Intel | Trust Physics |
| VDF Layer | No | No | Yes |
| Physical Entropy | No | No | Yes |
| Deployed on X1 | No | No | Yes |
| Token Incentives | LINK | SBX | RADS |
| Node Cost | High | High | $155 |

---

## 10. The Genesis Node
```
Location:   Miami, Florida, USA
Operator:   Skywalker (@skywalker12345678)
Org:        Echo Hound Labs (@EchoHoundX)
Hardware:   GMC-500+ Geiger Counter
Source:     Cenozoic Era fossils
            Miocene-Pleistocene, 2-23M years old
Wallet:     HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
Live since: March 16, 2026
```

The same quantum randomness that has governed matter since the Big Bang now secures X1 smart contracts.

---

## 11. Conclusion

"The laws of physics are the most trustless oracle that exists."

The Geiger Entropy Oracle combines quantum unpredictability with on-chain verifiability. RADS token creates the economic incentive layer that transforms individual node operators into a decentralized entropy network — where the universe itself controls token supply.

The node operators ARE the subnet.

---

## Credits

Architecture review and technical insights:
- Theo — VDF/RANDAO/L2 architecture, Merkle batching
- Owl of Atena — on-chain vs off-chain distinction
- Marat — X1 ecosystem context
- X1 Community — public technical review

---

## References

- Wesolowski, B. (2019). Efficient Verifiable Delay Functions
- Boneh et al. (2018). Verifiable Delay Functions
- Ethereum RANDAO specification
- Chia Network chiavdf implementation
- X1 Blockchain documentation: docs.x1.xyz

---

## Legal

This document is provided for informational purposes only. RADS tokens are utility tokens intended for use within the Geiger Entropy Oracle ecosystem. This is not a securities offering. Echo Hound Labs makes no representations about the future value of RADS tokens. Participation in the RADS ecosystem may be restricted in certain jurisdictions. This whitepaper does not constitute an offer to sell or a solicitation to buy any securities or financial instruments. Echo Hound Labs is not responsible for any losses incurred through the use of the Geiger Entropy Oracle protocol or RADS tokens.

---

## Links
```
Program:   BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU
GitHub:    github.com/echohound-labs/geiger-entropy-oracle
Twitter:   @EchoHoundX
Telegram:  t.me/+axtvX9GbsnJkMGRh
Network:   X1 Mainnet
Explorer:  explorer.mainnet.x1.xyz
```

---

Echo Hound Labs — Building X1 Infrastructure from the ground up

Version 2.0 — March 2026
