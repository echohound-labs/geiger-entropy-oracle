const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");

async function main() {
  const keypairPath = process.env.HOME + "/.config/solana/id.json";
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync("target/idl/geiger_entropy.json"));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {});
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  // Check pending commitment
  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()],
    programId
  );

  try {
    const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);
    console.log("Pending commitment:");
    console.log("  Sequence:", pc.sequence.toString());
    console.log("  Revealed:", pc.revealed);
    console.log("  Committed slot:", pc.committedSlot.toString());
    console.log("  Commitment hash:", Buffer.from(pc.commitmentHash).toString('hex').slice(0,16) + "...");
  } catch(e) {
    console.log("No pending commitment account found");
  }
}

main().catch(console.error);
