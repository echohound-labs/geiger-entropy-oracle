const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey, SystemProgram, ComputeBudgetProgram } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

const RPC_URL = "https://rpc.testnet.x1.xyz";
const PROGRAM_ID = "2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY";
const KEYPAIR_PATH = path.join(process.env.HOME, ".config/solana/id.json");
const IDL_PATH = path.join(__dirname, "../idl/testnet/geiger_entropy.json");

async function main() {
  const wallet = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(KEYPAIR_PATH))));
  const connection = new Connection(RPC_URL, "confirmed");
  const idl = JSON.parse(fs.readFileSync(IDL_PATH));
  const programId = new PublicKey(PROGRAM_ID);
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [oracleState] = PublicKey.findProgramAddressSync([Buffer.from("oracle_state")], programId);
  const [entropyPool] = PublicKey.findProgramAddressSync([Buffer.from("entropy_pool")], programId);

  // Get all pending requests
  const accounts = await connection.getProgramAccounts(programId, {
    filters: [{ dataSize: 122 }]
  });

  console.log(`Scanning for pending RandomnessRequest accounts...`);
  const pending = accounts.filter(({ account }) => account.data[104] === 0);
  console.log(`Found ${pending.length} pending request(s) — fulfilling...`);

  let fulfilled = 0;
  let failed = 0;

  for (const { pubkey, account } of pending) {
    const requester = new PublicKey(account.data.slice(8, 40));
    console.log(`  → ${pubkey.toBase58()} (requester: ${requester.toBase58().slice(0, 8)}...)`);

    try {
      const tx = await program.methods
        .fulfillRandomness()
        .accounts({ oracleState, entropyPool, randomnessRequest: pubkey, requester, systemProgram: SystemProgram.programId })
        .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
        .rpc();
      console.log(`  ✅ Fulfilled! TX: ${tx}`);
      fulfilled++;
    } catch (e) {
      console.log(`  ❌ Failed: ${e.message?.slice(0, 100)}`);
      failed++;
    }
  }

  console.log(`Done — ${fulfilled} fulfilled, ${failed} failed`);
  console.log(JSON.stringify({ fulfilled, failed, total: pending.length }));
}

main().catch(console.error);
