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
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {commitment: "confirmed"});
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()], programId
  );

  // Slash ourselves to clear the stuck commitment
  const tx = await program.methods
    .slashMissedReveal()
    .accounts({
      pendingCommitment: pendingCommitmentPDA,
      operator: wallet.publicKey,
      reporter: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("Cleared! TX:", tx);
}

main().catch(e => console.error(e.message));
