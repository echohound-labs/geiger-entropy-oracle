const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "./idl-testnet/geiger_entropy.json")
  ));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
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

  try {
    const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);
    
    if (pc.revealed) {
      console.log("CLEAN: commitment already revealed");
      console.log(JSON.stringify({status: "clean", sequence: pc.sequence.toString()}));
      return;
    }

    // Get current slot
    const slot = await connection.getSlot();
    const committedSlot = pc.committedSlot.toNumber();
    const deadline = committedSlot + 128;

    if (slot > deadline) {
      // Slash to clear
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
      // Still within reveal window — report sequence
      console.log("PENDING: commitment exists, within reveal window");
      console.log(JSON.stringify({
        status: "pending",
        sequence: pc.sequence.toString(),
        committedSlot: committedSlot,
        currentSlot: slot,
        slotsRemaining: deadline - slot
      }));
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
