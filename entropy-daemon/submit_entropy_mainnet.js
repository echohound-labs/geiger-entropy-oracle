const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [seedHex, sigHex, cpmStr, timestampStr] = process.argv.slice(2);
  if (!seedHex || !sigHex || !cpmStr || !timestampStr) {
    console.error("Usage: node submit_entropy.js <seed_hex> <sig_hex> <cpm> <timestamp>");
    process.exit(1);
  }

  const keypairPath = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );

  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idlPath = path.join(__dirname, "./idl/mainnet/geiger_entropy.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const oracleState = new PublicKey("BygMTZ1oLBD9tDmssnt9LkNT7BEd2PCJBCzurwtMuTqm");
  const entropyPool = new PublicKey("GDECYXCXietabJs9Y1baKzD3t4VFBw4eZWPnvYenyi77");
  const entropyNode = new PublicKey("z4Psp8qVfP4t3jiWHE29rrisTPMC78tu8LmDhRSEL3s");

  const seed = Array.from(Buffer.from(seedHex, "hex"));
  const signature = Array.from(Buffer.from(sigHex, "hex"));
  const cpm = parseInt(cpmStr);
  const timestamp = parseInt(timestampStr);

  const tx = await program.methods
    .submitEntropy(seed, cpm, new anchor.BN(timestamp), signature)
    .accounts({
      oracleState,
      entropyPool,
      entropyNode,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log(`TX: ${tx}`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
