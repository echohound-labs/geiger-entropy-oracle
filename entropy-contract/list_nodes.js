const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "target/idl/geiger_entropy.json"), "utf8"));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");

  const dummyKeypair = Keypair.generate();
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(dummyKeypair),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  console.log("Fetching all registered nodes on X1 Mainnet...\n");

  const nodes = await program.account.entropyNode.all();

  if (nodes.length === 0) {
    console.log("No nodes registered yet.");
    return;
  }

  console.log(`Total nodes: ${nodes.length}\n`);
  console.log("─────────────────────────────────────────────────────────────────");

  nodes.forEach((node, i) => {
    const { name, operator, submissions } = node.account;
    console.log(`${i + 1}. ${name}`);
    console.log(`   Operator : ${operator.toBase58()}`);
    console.log(`   Node PDA : ${node.publicKey.toBase58()}`);
    console.log(`   Submissions: ${submissions.toString()}`);
    console.log("─────────────────────────────────────────────────────────────────");
  });
}

main().catch(console.error);
