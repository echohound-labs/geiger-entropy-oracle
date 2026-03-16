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

  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );

  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idlPath = path.join(__dirname, "../entropy-contract/target/idl/geiger_entropy.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const oracleState = new PublicKey("CrrLuXpoCuK8szmtXxBEDPc5FTkbUGzEWfMyjeSL83bS");
  const entropyPool = new PublicKey("KMgwwzxYxrXufHySyMNchwyhupsNNsc4wPN71xtqoGG");
  const entropyNode = new PublicKey("3KA1UPPZf1N36RgmLwDKqAdJB6WPnX44aBs9rgP3TvdV");

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
