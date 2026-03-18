const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync("target/idl/geiger_entropy.json", "utf8"));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()],
    programId
  );

  try {
    const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);
    console.log("Pending commitment found:");
    console.log("  Sequence:", pc.sequence.toString());
    console.log("  Committed slot:", pc.committedSlot.toString());
    console.log("  Revealed:", pc.revealed);
  } catch(e) {
    console.log("No pending commitment found — clean state!");
  }
}

main().catch(console.error);
