const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf8"));
  const wallet = Keypair.fromSecretKey(new Uint8Array(keypairData));
  console.log("Operator:", wallet.publicKey.toBase58());

  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "target/idl/geiger_entropy.json"), "utf8"));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")], programId
  );
  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), wallet.publicKey.toBytes()], programId
  );

  console.log("Entropy Node PDA:", entropyNodePDA.toBase58());

  // Check if already registered
  try {
    const existing = await program.account.entropyNode.fetch(entropyNodePDA);
    console.log("✓ Node already registered!");
    console.log("  Name:", existing.name);
    console.log("  Submissions:", existing.submissions.toString());
    return;
  } catch (e) {
    console.log("Registering node...");
  }

  const tx = await program.methods
    .registerNode(wallet.publicKey, "Genesis Node — Cenozoic Fossils ☢️")
    .accounts({
      oracleState: oracleStatePDA,
      entropyNode: entropyNodePDA,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([wallet])
    .rpc();

  console.log("✓ Node registered!");
  console.log("  Transaction:", tx);
  console.log("  Node PDA:", entropyNodePDA.toBase58());
}

main().catch(console.error);
