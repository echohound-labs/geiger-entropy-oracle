const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  // Load wallet
  const keypairPath = (process.env.NETWORK || "mainnet") === "mainnet"
    ? process.env.HOME + "/.config/solana/mainnet-deployer.json"
    : process.env.HOME + "/.config/solana/id.json";
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf8"));
  const wallet = Keypair.fromSecretKey(new Uint8Array(keypairData));
  console.log("Wallet:", wallet.publicKey.toBase58());

  // Connect to X1 testnet
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const balance = await connection.getBalance(wallet.publicKey);
  console.log("Balance:", balance / 1e9, "SOL");

  // Load IDL
  const idlPath = path.join(__dirname, "target/idl/geiger_entropy.json");
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));

  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);

  const program = new anchor.Program(idl, provider);

  // Derive PDAs
  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")],
    programId
  );
  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")],
    programId
  );

  console.log("Oracle State PDA:", oracleStatePDA.toBase58());
  console.log("Entropy Pool PDA:", entropyPoolPDA.toBase58());

  // Check if already initialized
  try {
    const existing = await program.account.oracleState.fetch(oracleStatePDA);
    console.log("✓ Oracle already initialized!");
    console.log("  Authority:", existing.authority.toBase58());
    console.log("  Total nodes:", existing.totalNodes.toString());
    console.log("  Total requests:", existing.totalRequests.toString());
    return;
  } catch (e) {
    console.log("Not yet initialized — initializing now...");
  }

  // Initialize
  const tx = await program.methods
    .initialize()
    .accounts({
      oracleState: oracleStatePDA,
      entropyPool: entropyPoolPDA,
      authority: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([wallet])
    .rpc();

  console.log("✓ Oracle initialized!");
  console.log("  Transaction:", tx);
  console.log("  Oracle State:", oracleStatePDA.toBase58());
  console.log("  Entropy Pool:", entropyPoolPDA.toBase58());
}

main().catch(console.error);
