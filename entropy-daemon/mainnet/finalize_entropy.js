const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [operatorStr, sequenceStr] = process.argv.slice(2);
  const autoMode = sequenceStr === "auto";

  const keypairPath = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );
  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "../idl/mainnet-commit-reveal/geiger_entropy.json"), "utf8"
  ));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const operatorPubkey = operatorStr && operatorStr !== ""
    ? new PublicKey(operatorStr)
    : wallet.publicKey;

  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")], programId
  );
  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), operatorPubkey.toBuffer()], programId
  );
  const SLOT_HASHES_PUBKEY = new PublicKey("SysvarS1otHashes111111111111111111111111111");

  const currentSlot = await connection.getSlot();

  // Auto mode — check last 20 sequences for pending finalizes
  if (autoMode) {
    // Read pending commit to get current sequence
    const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), operatorPubkey.toBuffer()], programId
    );
    let currentSeq = 0;
    try {
      const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);
      currentSeq = pc.sequence.toNumber();
    } catch(e) {
      // No pending commitment
      return;
    }

    // Check last 20 sequences for pending finalizes
    for (let seq = Math.max(0, currentSeq - 20); seq < currentSeq; seq++) {
      try {
        const [pendingFinalizePDA] = PublicKey.findProgramAddressSync(
          [Buffer.from("finalize"), operatorPubkey.toBuffer(),
           Buffer.from(new anchor.BN(seq).toArray('le', 8))],
          programId
        );
        const pf = await program.account.pendingFinalize.fetch(pendingFinalizePDA);
        if (!pf.finalized && currentSlot >= pf.bindingSlot) {
          console.log(`Finalizing seq=${seq} binding_slot=${pf.bindingSlot}...`);
          const tx = await program.methods
            .finalizeEntropy()
            .accounts({
              entropyPool: entropyPoolPDA,
              pendingFinalize: pendingFinalizePDA,
              entropyNode: entropyNodePDA,
              slotHashes: SLOT_HASHES_PUBKEY,
              caller: wallet.publicKey,
            })
            .rpc({ commitment: "confirmed" });
          console.log(`✅ Finalized seq=${seq} TX: ${tx}`);
        } else if (!pf.finalized) {
          console.log(`Not ready seq=${seq} — wait ${pf.bindingSlot - currentSlot} slots`);
        }
      } catch(e) {
        // Account doesn't exist — already finalized or not created
      }
    }
    return;
  }

  // Manual mode — finalize specific sequence
  const sequence = parseInt(sequenceStr || "0");
  const [pendingFinalizePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("finalize"), operatorPubkey.toBuffer(),
     Buffer.from(new anchor.BN(sequence).toArray('le', 8))],
    programId
  );

  const pf = await program.account.pendingFinalize.fetch(pendingFinalizePDA);
  console.log(`Sequence: ${pf.sequence} | Binding slot: ${pf.bindingSlot} | Current: ${currentSlot} | Finalized: ${pf.finalized}`);

  if (pf.finalized) { console.log("✅ Already finalized!"); return; }
  if (currentSlot < pf.bindingSlot) {
    console.log(`⏳ Not ready — wait ${pf.bindingSlot - currentSlot} more slots`);
    return;
  }

  const tx = await program.methods
    .finalizeEntropy()
    .accounts({
      entropyPool: entropyPoolPDA,
      pendingFinalize: pendingFinalizePDA,
      entropyNode: entropyNodePDA,
      slotHashes: SLOT_HASHES_PUBKEY,
      caller: wallet.publicKey,
    })
    .rpc({ commitment: "confirmed" });

  console.log(`✅ Finalized! TX: ${tx}`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
