const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

async function main() {
  const [vdfOutputHex, operatorNonceHex, sequenceStr] = process.argv.slice(2);

  if (!vdfOutputHex || !operatorNonceHex || !sequenceStr) {
    console.error("Usage: node commit_entropy.js <vdf_output_hex> <nonce_hex> <sequence>");
    process.exit(1);
  }

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

  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")], programId
  );
  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()], programId
  );

  // Compute commitment hash = SHA256(vdf_output || nonce)
  const vdfOutput = Buffer.from(vdfOutputHex, "hex");
  const nonce = Buffer.from(operatorNonceHex, "hex");
  const preimage = Buffer.concat([vdfOutput, nonce]);
  const commitmentHash = Array.from(crypto.createHash("sha256").update(preimage).digest());
  const sequence = parseInt(sequenceStr);

  const tx = await program.methods
    .commitEntropy(commitmentHash, new anchor.BN(sequence))
    .accounts({
      oracleState: oracleStatePDA,
      pendingCommitment: pendingCommitmentPDA,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log(`✓ Committed | seq=${sequence} tx=${tx}`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
