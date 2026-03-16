# ☢️ Geiger Entropy Oracle & RADS Token
## Proof of Physical Entropy on X1 Blockchain
### Echo Hound Labs — March 2026
### Version 2.2

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

The protocol is live on X1 mainnet today. Any dApp can call get_randomness() right now and receive a verified, unbiasable random value backed by quantum physics.

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

### 2.4 Why This Works Today — Single Node + VDF

The VDF layer is what makes a single honest node cryptographically sufficient for dApp use right now.

As explained by Theo during public technical review:

"Without VDF, a single-node oracle has a fundamental trust problem: how do we know the operator did not just pick a favorable number?

VDF flips that. It cryptographically proves:

1. The input was committed first — you cannot choose the seed after seeing what output you want
2. The computation took real sequential time — no amount of parallel hardware lets you skip ahead
3. The result was inevitable — given that seed and those iterations, there was only one possible output

The chain of proof becomes:

Physical decay (uncontrollable)
→ seed committed
→ VDF locks it in time
→ verifiable output

No one — including the operator — could have manipulated the result once the decay event was recorded. The physics happened, the seed was set, and the VDF made it provably tamper-proof.

It is the difference between:
'Trust me, I did not cheat'
vs
'Here is a cryptographic proof that cheating was physically impossible'

Assuming the hardware is honest, VDF makes a single node as trustworthy as a multi-node committee for randomness generation. That is actually a pretty rare property. Most single-node oracles cannot claim it."

— Theo, X1 Community Architect

### 2.5 Why More Nodes Make It Unstoppable

"The trust floor is already met at 1 node. Additional nodes add:

- Redundancy — oracle stays live if one node goes down
- Decentralization — harder to pressure any single operator
- Entropy diversity — multiple independent physical sources
- Economic credibility — harder to say it is just one guy
- Perception — ecosystem looks more robust to dapp builders

You do not need more nodes to be usable.
You want more nodes to be unstoppable."

— Theo, X1 Community Architect
```
Version 1 (now):
Single node, VDF-proven, open for business

Version 2 (RADS token):
Multi-node network, same interface, stronger guarantees

Same API forever:
get_randomness() — never changes for dApp builders
The underlying network gets stronger over time
while dApp code never needs to change
```

### 2.6 Community Architecture Insight

"The node operators ARE the subnet in a sense — they form their own p2p layer, aggregate entropy, and post to X1. That is functionally similar to an optimistic rollup model without formally being one."

— Theo, X1 Community Architect

---

## 3. Use It Today

Any dApp on X1 can integrate right now:
```javascript
// Request randomness — that is literally it
const userSeed = crypto.randomBytes(32);
await program.methods
    .requestRandomness(Array.from(userSeed))
    .accounts({...})
    .rpc();

// Fulfill and read result
await program.methods
    .fulfillRandomness()
    .accounts({...})
    .rpc();

const request = await program.account
    .randomnessRequest.fetch(requestPDA);
console.log('Random result:',
    Buffer.from(request.result).toString('hex'));
```

**What dApp builders get out of the box:**
- Call get_randomness() and receive a verified unbiasable random value
- VDF proof attached — verify it yourself or trust on-chain verification
- No need to understand Geiger counters or physics — just use the output
- Same interface regardless of how many nodes are in the network

**Use cases live right now:**
```
Lotteries and raffles  → provably fair draws
On-chain games         → unbiasable dice, cards, loot drops
NFT trait generation   → random attribute assignment at mint
DAO selection          → picking winners, committee members
DeFi                   → random liquidation ordering
```

---

## 4. Current Deployment
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

## 5. RADS Token

### 5.1 Overview

RADS is a novel token primitive where supply is controlled entirely by radioactive decay. No team can mint extra tokens. No inflation schedule exists. The universe controls supply.

"Only 1,000,000 RADS. Mined by radioactive decay over exactly 4 years. Then fixed forever."

The purpose of RADS is simple: incentivize more node operators to join the network. More nodes means more decentralization, more redundancy, and a stronger protocol.

### 5.2 Token Properties
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

### 5.3 Emission Schedule
```
Total: 1,000,000 RADS over 4 years

Year 1: 400,000 RADS (40%) — highest rewards for early nodes
Year 2: 300,000 RADS (30%)
Year 3: 200,000 RADS (20%)
Year 4: 100,000 RADS (10%)

= ~685 RADS/day per node (background radiation)
= 1 RADS per ~42 decay events

After 4 years:
→ Hard cap reached
→ No new RADS ever minted
→ Oracle continues running forever
→ Fixed supply = deflationary as dApps burn
```

### 5.4 Per-Node Emission Cap
```
Max CPM counted: 100 CPM per node
Above 100 CPM: same reward rate
Purpose: prevent hot specimen dominance
Result: incentivizes MORE nodes not hotter nodes
```

### 5.5 Why Run a Node Early
```
Year 1 rewards: 400,000 RADS (40% of all RADS ever)
Year 4 rewards: 100,000 RADS (10% of all RADS ever)

Early node operators earn 4x more than late ones
Hardware cost is the same: ~$155
The incentive to join early is massive ☢️
```

### 5.6 Token Utility — v1 (MVP)
```
Mine:   Run a Geiger node → earn RADS automatically
        Every decay event = RADS minted to your wallet

Spend:  dApps burn RADS → request randomness
        Creates deflationary pressure on supply
```

### 5.7 Token Utility — v2 (Roadmap)
```
Stake:  Lock RADS → earn share of protocol fees
Slash:  Submit bad data → lose staked RADS
Govern: Stake to vote on protocol upgrades
Tier:   Higher stake = higher node reputation
        Higher reputation = priority fulfillment
```

Staking and governance are intentionally excluded from v1. The goal right now is to get the token live and start incentivizing node operators as quickly as possible. Complexity comes later when the network has grown.

### 5.8 The Flywheel
```
More nodes → more RADS mined
More nodes → more decentralization
More decentralization → more dApps trust it
More dApps → more RADS burned
More burned → more scarce
More scarce → higher value
Higher value → more nodes join
→ repeat ♻️
```

---

## 6. Node Economics

### 6.1 Hardware Requirements
```
GMC-500+ Geiger Counter: ~$100
Raspberry Pi 4:          ~$35
MicroSD + power:         ~$20
Total:                   ~$155 one time
```

### 6.2 Any Hardware Works
```
USB direct to PC/laptop  ✓
Raspberry Pi 4           ✓
Any Linux machine        ✓
Old laptop               ✓

The Geiger counter connects via USB
Daemon runs on your machine
Entropy posts directly to X1
No data center needed
No validator status required
Radioactive source never leaves your location
Only cryptographic hashes touch the internet
```

### 6.3 Earnings Model (Year 1)
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

### 6.4 Setup (Coming in Phase 2)
```
git clone github.com/echohound-labs/geiger-entropy-oracle
./install.sh
register_node() on X1
→ start earning RADS automatically
```

---

## 7. Roadmap

### Phase 1 — Genesis (Complete ✅)
```
✓ GMC-500 hardware integration
✓ Entropy extraction (Δt timing)
✓ VDF layer (Wesolowski dynamic iterations)
✓ Ed25519 signing
✓ REST API
✓ Anchor program deployed on X1 mainnet
✓ 7,000+ entropy transactions live
✓ Full VRF cycle tested on mainnet
✓ Whitepaper published
✓ Public technical review (Theo, Owl, X1 community)
```

### Phase 2 — RADS Token (Q2 2026)
```
□ RADS SPL token creation
□ Mint on every decay event
□ 4 year emission schedule
□ Per-node CPM cap (100 CPM max)
□ Hard cap enforcement (1,000,000 RADS)
□ Node operator onboarding guide
□ Token launch on X1
```

### Phase 3 — Multi-Node + Staking (Q3 2026)
```
□ Multiple independent node support
□ Threshold aggregation (3-of-5 nodes)
□ Node reputation system
□ Staking contract
□ Slashing mechanism
□ Merkle tree entropy batching
│  ├── Batch 100 events → 1 on-chain tx
│  ├── 100x cheaper at scale
│  └── Full auditability preserved
□ On-chain RANDAO integration
```

### Phase 4 — Subnet/Settlement Layer (Q4 2026)
```
□ Geiger node operators as validators
□ Stake-weighted participation
□ Dedicated entropy subnet
□ Bridge randomness to X1 mainnet
□ DAO governance
□ Frontend dashboard
```

---

## 8. Security Analysis

### 8.1 Attack Vectors

| Attack | Current Mitigation | Phase 3 Mitigation |
|--------|-------------------|-------------------|
| Fake Geiger data | Node reputation | Multi-node threshold |
| Withhold/retry | VDF delay | VDF + threshold |
| Validator censorship | Multiple submissions | Multi-node redundancy |
| Hot specimen dominance | 100 CPM cap | Same |
| Single node bias | VDF proves tamper-proof | RANDAO aggregation |
| Upgrade authority abuse | Deployer wallet | Transfer to DAO |

### 8.2 Current Limitations — Honest
```
Single node = hardware trust required
VDF is off-chain computation
Ed25519 full on-chain verification = TODO
No slashing implemented yet
Upgrade authority = single wallet
```

### 8.3 Progressive Trust Model
```
Phase 1: Trust Echo Hound Labs Genesis Node
         + VDF cryptographic proof
Phase 2: Trust RADS-incentivized node operators
Phase 3: Trust 3-of-5 threshold consensus
Phase 4: Trust subnet validator set
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
| Usable today on X1 | No | No | Yes |

---

## 10. The Genesis Node
```
Location:   Florida, USA
Operator:   Skywalker (@skywalker12345678)
Org:        Echo Hound Labs (@EchoHoundX)
Hardware:   GMC-500+ Geiger Counter
Source:     Cenozoic Era fossils
            Miocene-Pleistocene, 2-23M years old
Wallet:     HGFisVbULNKqogtPuGTfcHG9y6i5nboZabYwifkiiodo
Live since: March 16, 2026
```

The same quantum randomness that has governed matter since the Big Bang now secures X1 smart contracts. 🦴

---

## 11. Conclusion

"The laws of physics are the most trustless oracle that exists."

The Geiger Entropy Oracle is live on X1 mainnet today. Any dApp can integrate right now and receive provably fair randomness backed by quantum mechanical radioactive decay — with a VDF cryptographic proof that manipulation was physically impossible.

RADS token transforms node operators into entropy miners, creating a self-sustaining economic loop where the universe itself controls token supply.

The node operators ARE the subnet.
You do not need more nodes to be usable.
You want more nodes to be unstoppable. ☢️

---

## Credits

Architecture review and technical insights:
- Theo — VDF design insight, RANDAO/L2 architecture, Merkle batching
- Owl of Atena — on-chain vs off-chain distinction
- Marat — X1 ecosystem context
- X1 Community — public technical review on March 16, 2026

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

Version 2.2 — March 2026

---

## Appendix C — Hardware Trust Assumption

The GMC-500+ is a commercial off-the-shelf Geiger counter available to anyone for ~$100 from GQ Electronics. The daemon software is fully open source and auditable by anyone on GitHub.

**The hardware trust assumption is:**
> The operator is running genuine GMC-500 hardware with unmodified open source software.

**Why this is reasonable:**
```
✓ GMC-500 is a real certified Geiger counter
✓ Open source daemon — anyone can audit it
✓ Real serial protocol — publicly documented
✓ Poisson distribution — statistically verifiable
✓ Anyone can buy same hardware and verify
✓ Not a black box — fully reproducible
```

**What VDF cannot fix:**
```
VDF proves the seed was not manipulated
after it was generated.

VDF cannot prove:
- The USB device is actually a GMC-500
- The data is not software-generated
- The Geiger counter is pointed at something radioactive
```

**How this gets fixed over time:**
```
Phase 1: Trust one operator's hardware
         → mitigated by open source code
         → mitigated by statistical verification

Phase 3: Trust that not ALL operators
         are simultaneously dishonest
         → multiple independent locations
         → multiple independent hardware units
         → collusion becomes economically irrational

Phase 4: Stake-weighted participation
         → operators have economic skin in the game
         → dishonesty = financial loss
```

The open source nature of the project means anyone can run the exact same stack and verify the results independently. This is the strongest possible mitigation for hardware trust at single-node scale.


---

## Appendix D — Complete Token Economics (v1 + v2)

### Mining Rewards (Decay Events)
```
v1 (launch):
Every decay event → mint RADS → 100% to node operator
Simple. Fast. Gets nodes online immediately.

v2 (upgrade):
Every decay event → mint RADS
├── 70% → node operator wallet (automatic)
├── 20% → protocol treasury (multisig)
└── 10% → burned forever (deflationary)
```

### Usage Fees (dApp Requests)
```
v2 (upgrade):
Every randomness request → 0.001 RADS fee
├── 50% → node operator who fulfilled request
├── 30% → burned forever (deflationary)
└── 20% → protocol treasury (multisig)
```

### Node Operator Income Streams
```
Stream 1 — Mining (passive):
Just run the hardware
→ earn RADS every decay event
→ works even with zero dApps

Stream 2 — Fulfillment (active):
Earn when dApps use the oracle
→ more dApps = more income
→ scales with protocol adoption
```

### Treasury — Multisig Model

The protocol treasury will be controlled by a
multisig wallet — NOT a single person.
```
v2 Launch:
2-of-3 multisig
├── Echo Hound Labs (Skywalker)
├── Trusted community member
└── Echo Hound Labs cold wallet

v3 (DAO transition):
Community governance
├── RADS holders vote on proposals
├── Treasury spending requires vote
├── Protocol upgrades require vote
└── Node standards require vote
```

Treasury funds used for:
```
├── Protocol development
├── Security audits
├── dApp integration grants
├── Node operator incentives
├── Marketing and ecosystem growth
└── Bug bounties
```

### Why Multisig Over Complex Hooks
```
Complex on-chain hooks:
✗ More attack surface
✗ Harder to audit
✗ More expensive to deploy
✗ Harder to upgrade if bugs found

Multisig treasury:
✓ Simple and battle tested
✓ Transparent on-chain
✓ Community can verify spending
✓ Easy to upgrade to DAO later
✓ Same model used by:
   - Uniswap early days
   - Many successful protocols
   - Battle tested approach
```

### Upgrade Path
```
v1 (now):
Simple mint → node operator
No splits, no fees, no complexity
Goal: get token live, get nodes online

v2 (Q2 2026):
Add 70/20/10 mining split
Add 50/30/20 request fee split
Add multisig treasury
Single oracle program upgrade
Nothing changes for token or dApps

v3 (Q3 2026):
Multisig transitions to DAO
RADS holders govern protocol
Community controls treasury
Full decentralization achieved
```

### The Two Flows Summary
```
Flow 1 — Mining:
Physics → entropy → RADS minted
└── rewards hardware operators

Flow 2 — Usage:
dApps → request randomness → RADS burned/distributed
└── rewards service providers
└── creates deflationary pressure
└── funds protocol development

Combined:
Early nodes survive on mining
Mature nodes thrive on usage fees
Protocol self-funds via treasury
Supply decreases as adoption grows
Value increases over time ☢️
```


---

## Appendix A — Why VDF Makes a Single Node Trustworthy

Contributed by Theo during public technical review, March 16, 2026:

"Without VDF, a single-node oracle has a fundamental trust problem: how do we know the operator did not just pick a favorable number?

VDF flips that. It cryptographically proves:

1. The input was committed first — you cannot choose the seed after seeing what output you want
2. The computation took real sequential time — no amount of parallel hardware lets you skip ahead
3. The result was inevitable — given that seed and those iterations, there was only one possible output

The chain of proof becomes:

Physical decay (uncontrollable)
→ seed committed
→ VDF locks it in time
→ verifiable output

No one — including the operator — could have manipulated the result once the decay event was recorded. The physics happened, the seed was set, and the VDF made it provably tamper-proof.

It is the difference between:
'Trust me, I did not cheat'
vs
'Here is a cryptographic proof that cheating was physically impossible'

The caveat: VDF legitimizes the process, not the hardware itself. The Geiger counter still needs to be real — VDF cannot compensate for a fake sensor. But assuming the hardware is honest, VDF makes a single node as trustworthy as a multi-node committee for randomness generation. That is actually a pretty rare property. Most single-node oracles cannot claim it."

— Theo, X1 Community Architect

---

## Appendix B — The Core Design Insight

Contributed by Theo, X1 Community Architect:

"Single node + VDF = already trustworthy.
The trust floor is already met at 1 node.

Additional nodes add:
- Redundancy — oracle stays live if one node goes down
- Decentralization — harder to pressure any single operator
- Entropy diversity — multiple independent physical sources
- Economic credibility — harder to say it is just one guy
- Perception — ecosystem looks more robust to dapp builders

You do not need more nodes to be usable.
You want more nodes to be unstoppable.

Version 1: Single node, VDF-proven, open for business
Version 2: Multi-node network, same interface, stronger guarantees

Same API, same get_randomness() call — the underlying
network gets stronger over time while dapp code never changes.
That is the right abstraction layer."

— Theo, X1 Community Architect

