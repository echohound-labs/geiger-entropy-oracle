const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

async function main() {
  const keypairPath = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath)))
  );
  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "../entropy-daemon/idl/mainnet-commit-reveal/geiger_entropy.json")
  ));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), {commitment: "confirmed"}
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [oracleStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")], programId
  );
  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")], programId
  );

  // Generate random user seed
  const userSeed = Array.from(crypto.randomBytes(32));
  console.log("User seed:", Buffer.from(userSeed).toString('hex').slice(0,16) + "...");

  // Get current total_requests from oracle state
  const oracleState = await program.account.oracleState.fetch(oracleStatePDA);
  const totalRequests = oracleState.totalRequests;
  console.log("Total requests so far:", totalRequests.toString());

  // Derive request PDA using total_requests as seed
  const requestIndex = Buffer.alloc(8);
  requestIndex.writeBigUInt64LE(BigInt(totalRequests.toString()));
  const [requestPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("rand_request"), wallet.publicKey.toBuffer(), requestIndex],
    programId
  );

  console.log("Requesting randomness...");
  const requestTx = await program.methods
    .requestRandomness(userSeed)
    .accounts({
      oracleState: oracleStatePDA,
      entropyPool: entropyPoolPDA,
      randomnessRequest: requestPDA,
      requester: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log("✓ Request TX:", requestTx);

  // Fulfill randomness
  console.log("Fulfilling randomness...");
  const fulfillTx = await program.methods
    .fulfillRandomness()
    .accounts({
      oracleState: oracleStatePDA,
      entropyPool: entropyPoolPDA,
      randomnessRequest: requestPDA,
      requester: wallet.publicKey,
      entropyNode: PublicKey.findProgramAddressSync(
        [Buffer.from("entropy_node"), wallet.publicKey.toBuffer()],
        programId
      )[0],
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log("✓ Fulfill TX:", fulfillTx);

  // Read result
  const request = await program.account.randomnessRequest.fetch(requestPDA);
  console.log("✓ Random result:", Buffer.from(request.result).toString('hex'));
  console.log("✓ Fulfilled:", request.fulfilled);
}

main().catch(console.error);
