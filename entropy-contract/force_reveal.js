const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");

async function main() {
  const keypairPath = (process.env.NETWORK || "mainnet") === "mainnet"
    ? process.env.HOME + "/.config/solana/mainnet-deployer.json"
    : process.env.HOME + "/.config/solana/id.json";
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync("target/idl/geiger_entropy.json"));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {commitment: "confirmed"});
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

  // Reveal with zeros — just to clear the stuck state
  const zeroOutput = Array(32).fill(0);
  const zeroNonce = Array(32).fill(0);
  const zeroSig = Array(64).fill(0);

  try {
    const tx = await program.methods
      .revealEntropy(zeroOutput, zeroNonce, 5, new anchor.BN(0), zeroSig)
      .accounts({
        oracleState: oracleStatePDA,
        entropyPool: entropyPoolPDA,
        pendingCommitment: pendingCommitmentPDA,
        entropyNode: entropyNodePDA,
        operator: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Force revealed! TX:", tx);
  } catch(e) {
    console.log("Error:", e.message);
  }
}

main().catch(console.error);
