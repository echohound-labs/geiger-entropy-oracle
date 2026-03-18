const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [vdfOutputHex, operatorNonceHex, sigHex, cpmStr, timestampStr] = process.argv.slice(2);

  if (!vdfOutputHex || !operatorNonceHex || !sigHex || !cpmStr || !timestampStr) {
    console.error("Usage: node reveal_entropy.js <vdf_output_hex> <nonce_hex> <sig_hex> <cpm> <timestamp>");
    process.exit(1);
  }

  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );

  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "./idl-testnet/geiger_entropy.json"), "utf8"
  ));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")], programId
  );
  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")], programId
  );
  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()], programId
  );
  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), wallet.publicKey.toBuffer()], programId
  );

  const vdfOutput = Array.from(Buffer.from(vdfOutputHex, "hex").slice(0, 32));
  const nonce = Array.from(Buffer.from(operatorNonceHex, "hex"));
  const signature = Array.from(Buffer.from(sigHex, "hex"));
  const cpm = parseInt(cpmStr);
  const timestamp = parseInt(timestampStr);

  // Wait for slot delay
  console.log("Waiting for commit-reveal delay...");
  await new Promise(resolve => setTimeout(resolve, 5000));

  const tx = await program.methods
    .revealEntropy(
      vdfOutput,
      nonce,
      cpm,
      new anchor.BN(timestamp),
      signature
    )
    .accounts({
      oracleState: oracleStatePDA,
      entropyPool: entropyPoolPDA,
      pendingCommitment: pendingCommitmentPDA,
      entropyNode: entropyNodePDA,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log(`✓ Revealed | CPM=${cpm} tx=${tx}`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
