const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");

async function main() {
  // Support mainnet and testnet via NETWORK env var
  const network = process.env.NETWORK || "mainnet";
  const isMainnet = network === "mainnet";
  const keypairPath = isMainnet
    ? process.env.HOME + "/.config/solana/mainnet-deployer.json"
    : process.env.HOME + "/.config/solana/id.json";
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const rpc = isMainnet
    ? "https://rpc.mainnet.x1.xyz"
    : "https://rpc.testnet.x1.xyz";
  const programIdStr = isMainnet
    ? "BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU"
    : "2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY";
  const idlPath = isMainnet
    ? "../entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json"
    : "target/idl/geiger_entropy.json";
  const connection = new Connection(rpc, "confirmed");
  const idl = JSON.parse(fs.readFileSync(idlPath));
  const programId = new PublicKey(programIdStr);
  console.log(`Network: ${network.toUpperCase()} | Program: ${programIdStr}`);
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
