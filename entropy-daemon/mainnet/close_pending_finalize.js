const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [sequenceStr] = process.argv.slice(2);
  if (!sequenceStr) {
    console.error("Usage: node close_pending_finalize.js <sequence>");
    process.exit(1);
  }

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
    connection, new anchor.Wallet(wallet), { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const sequence = parseInt(sequenceStr);
  const [pendingFinalizePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("finalize"), wallet.publicKey.toBuffer(),
     Buffer.from(new anchor.BN(sequence).toArray('le', 8))],
    programId
  );

  const info = await connection.getAccountInfo(pendingFinalizePDA);
  if (!info) {
    console.log(`No stuck account for seq=${sequence} — already clean`);
    return;
  }

  console.log(`Closing stuck PendingFinalize seq=${sequence}: ${pendingFinalizePDA.toString()}`);
  const tx = await program.methods
    .closePendingFinalize()
    .accounts({
      pendingFinalize: pendingFinalizePDA,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log(`✅ Closed seq=${sequence} TX: ${tx}`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
