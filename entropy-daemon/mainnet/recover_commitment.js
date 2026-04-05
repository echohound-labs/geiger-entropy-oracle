const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const keypairPath = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "../idl/mainnet-commit-reveal/geiger_entropy.json")
  ));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), {commitment: "confirmed"}
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()], programId
  );
  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")], programId
  );
  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")], programId
  );
  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), wallet.publicKey.toBuffer()], programId
  );

  const SLOT_HASHES_PUBKEY = new PublicKey("SysvarS1otHashes111111111111111111111111111");

  try {
    const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);

    if (pc.revealed) {
      console.log("CLEAN: commitment already revealed");
      console.log(JSON.stringify({status: "clean", sequence: pc.sequence.toString()}));
      return;
    }

    const slot = await connection.getSlot();
    const committedSlot = pc.committedSlot.toNumber();
    const deadline = committedSlot + 128;
    const slotsRemaining = deadline - slot;

    console.log(`Slots remaining: ${slotsRemaining}`);

    if (slot > deadline) {
      // Past deadline — slash to clear
      console.log("STALE: deadline missed, slashing to clear...");
      const tx = await program.methods
        .slashMissedReveal()
        .accounts({
          pendingCommitment: pendingCommitmentPDA,
          operator: wallet.publicKey,
          reporter: wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      console.log("SLASHED: cleared stuck commitment TX:", tx);
      console.log(JSON.stringify({status: "slashed", sequence: pc.sequence.toString()}));
    } else {
      // Still within reveal window — try to reveal using saved data
      const pendingPath = path.join(__dirname, "../.pending_commit.json");
      if (fs.existsSync(pendingPath)) {
        const saved = JSON.parse(fs.readFileSync(pendingPath, "utf8"));
        console.log(`PENDING: attempting auto-reveal for seq=${pc.sequence.toString()} (${slotsRemaining} slots remaining)`);

        try {
          // Use Anchor directly — same as Theo suggested
          const vdfOutput = Array.from(Buffer.from(saved.vdfOutputHex, "hex").slice(0, 32));
          const nonce = Array.from(Buffer.from(saved.operatorNonceHex, "hex"));
          const zeroSig = Array(64).fill(0);

          const revealTx = await program.methods
            .revealEntropy(
              vdfOutput,
              nonce,
              20,
              new anchor.BN(Math.floor(Date.now() / 1000)),
              zeroSig,
              new anchor.BN(0),
              0,
              0
            )
            .accounts({
              oracleState: oracleStatePDA,
              entropyPool: entropyPoolPDA,
              pendingCommitment: pendingCommitmentPDA,
              entropyNode: entropyNodePDA,
              operator: wallet.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              slotHashes: SLOT_HASHES_PUBKEY,
            })
            .rpc();

          console.log("AUTO-REVEAL SUCCESS:", revealTx);
          console.log(JSON.stringify({
            status: "clean",
            sequence: (parseInt(pc.sequence.toString()) + 1).toString()
          }));

          // Clean up saved file
          fs.unlinkSync(pendingPath);

        } catch(revealErr) {
          console.error("AUTO-REVEAL FAILED:", revealErr.message.slice(0, 150));
          // Slash to clear since reveal failed
          console.log("Slashing to clear...");
          try {
            const tx = await program.methods
              .slashMissedReveal()
              .accounts({
                pendingCommitment: pendingCommitmentPDA,
                operator: wallet.publicKey,
                reporter: wallet.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
              })
              .rpc();
            console.log("SLASHED:", tx);
            console.log(JSON.stringify({status: "slashed", sequence: pc.sequence.toString()}));
          } catch(slashErr) {
            console.error("SLASH FAILED:", slashErr.message.slice(0, 100));
            console.log(JSON.stringify({status: "pending", sequence: pc.sequence.toString()}));
          }
        }
      } else {
        // No saved data — slash to clear
        console.log("PENDING: no saved data — slashing to clear");
        try {
          const tx = await program.methods
            .slashMissedReveal()
            .accounts({
              pendingCommitment: pendingCommitmentPDA,
              operator: wallet.publicKey,
              reporter: wallet.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
            })
            .rpc();
          console.log("SLASHED:", tx);
          console.log(JSON.stringify({status: "slashed", sequence: pc.sequence.toString()}));
        } catch(slashErr) {
          console.error("SLASH FAILED:", slashErr.message.slice(0, 100));
          console.log(JSON.stringify({status: "pending", sequence: pc.sequence.toString()}));
        }
      }
    }
  } catch(e) {
    // No account = clean state
    console.log(JSON.stringify({status: "clean", sequence: "0"}));
  }
}

main().catch(e => {
  console.error(e.message);
  process.exit(1);
});
