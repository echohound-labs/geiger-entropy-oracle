#!/usr/bin/env node
/**
 * fulfill_randomness.js
 * Scans for pending RandomnessRequest accounts and fulfills them.
 *
 * Usage:
 *   node fulfill_randomness.js           — scan and fulfill all pending
 *   node fulfill_randomness.js <pubkey>  — fulfill one specific request account
 *
 * Called by daemon after each finalize cycle.
 */

const { Connection, Keypair, PublicKey, SystemProgram, ComputeBudgetProgram, TransactionMessage, VersionedTransaction } = require("@solana/web3.js");
const anchor = require("@coral-xyz/anchor");
const fs = require("fs");
const path = require("path");

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const RPC_URL       = "https://rpc.testnet.x1.xyz";
const PROGRAM_ID    = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
const KEYPAIR_PATH  = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
const IDL_PATH      = path.join(__dirname, "../idl/testnet/geiger_entropy.json");

// RandomnessRequest account layout
const STATUS_OFFSET  = 104;  // byte offset of status field
const STATUS_PENDING = 0;
const STATUS_FULFILLED = 1;

// How many accounts to scan per batch (getProgramAccounts can be heavy)
const BATCH_SIZE = 20;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

const connection = new Connection(RPC_URL, "confirmed");
const walletKeypair = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync(KEYPAIR_PATH)))
);
const wallet = new anchor.Wallet(walletKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
anchor.setProvider(provider);

const idl = JSON.parse(fs.readFileSync(IDL_PATH));
const program = new anchor.Program(idl, provider);

// ---------------------------------------------------------------------------
// Derive oracle_state and entropy_pool PDAs
// ---------------------------------------------------------------------------

function getOracleStatePDA() {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("oracle_state")],
    PROGRAM_ID
  );
  return pda;
}

function getEntropyPoolPDA() {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")],
    PROGRAM_ID
  );
  return pda;
}

// ---------------------------------------------------------------------------
// Fetch all pending RandomnessRequest accounts
// ---------------------------------------------------------------------------

async function fetchPendingRequests() {
  // Filter by:
  // 1. Owner = our program
  // 2. Status byte = 0 (Pending)
  //
  // Use dataSize filter to only get RandomnessRequest accounts (122 bytes from test output)
  const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
    commitment: "confirmed",
    filters: [
      { dataSize: 122 },                                   // RandomnessRequest size
      { memcmp: { offset: STATUS_OFFSET, bytes: "1" } },   // Status = 0 (Pending) — bs58("00") = "1"
    ],
  });
  return accounts;
}

// ---------------------------------------------------------------------------
// Fulfill a single request
// ---------------------------------------------------------------------------

async function fulfillRequest(requestPubkey, requesterPubkey) {
  const oracleState = getOracleStatePDA();
  const entropyPool = getEntropyPoolPDA();

  try {
    const tx = await program.methods
      .fulfillRandomness()
      .accounts({
        oracleState:       oracleState,
        entropyPool:       entropyPool,
        randomnessRequest: requestPubkey,
        requester:         requesterPubkey,
        systemProgram:     SystemProgram.programId,
      })
      .preInstructions([
        // Bump CU limit — fulfill does SHA256 mixing so needs headroom
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1000 }),
      ])
      .rpc({ commitment: "confirmed", skipPreflight: false });

    return { success: true, tx };
  } catch (err) {
    return { success: false, error: err.message || String(err) };
  }
}

// ---------------------------------------------------------------------------
// Parse requester pubkey from account data
// The RandomnessRequest struct starts with 8-byte Anchor discriminator,
// then requester pubkey (32 bytes) at offset 8.
// ---------------------------------------------------------------------------

function parseRequester(data) {
  // Anchor discriminator = 8 bytes, then requester pubkey
  const requesterBytes = data.slice(8, 40);
  return new PublicKey(requesterBytes);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const specificAccount = process.argv[2];

  if (specificAccount) {
    // Fulfill one specific account
    const requestPubkey = new PublicKey(specificAccount);
    const accountInfo = await connection.getAccountInfo(requestPubkey);
    if (!accountInfo) {
      console.error(`Account not found: ${specificAccount}`);
      process.exit(1);
    }
    const status = accountInfo.data[STATUS_OFFSET];
    if (status !== STATUS_PENDING) {
      console.log(`Account ${specificAccount} is not Pending (status=${status})`);
      process.exit(0);
    }
    const requester = parseRequester(accountInfo.data);
    console.log(`Fulfilling single request: ${specificAccount}`);
    console.log(`Requester: ${requester.toBase58()}`);
    const result = await fulfillRequest(requestPubkey, requester);
    if (result.success) {
      console.log(`✅ Fulfilled TX: ${result.tx}`);
    } else {
      console.error(`❌ Failed: ${result.error}`);
      process.exit(1);
    }
    return;
  }

  // Scan for all pending requests
  console.log("Scanning for pending RandomnessRequest accounts...");
  let pending;
  try {
    pending = await fetchPendingRequests();
  } catch (err) {
    console.error(`Scan failed: ${err.message}`);
    process.exit(1);
  }

  if (pending.length === 0) {
    console.log("No pending requests found.");
    process.exit(0);
  }

  console.log(`Found ${pending.length} pending request(s) — fulfilling...`);

  let fulfilled = 0;
  let failed = 0;

  for (const { pubkey, account } of pending) {
    const requester = parseRequester(account.data);
    console.log(`  → ${pubkey.toBase58()} (requester: ${requester.toBase58().slice(0, 8)}...)`);

    const result = await fulfillRequest(pubkey, requester);
    if (result.success) {
      console.log(`  ✅ Fulfilled TX: ${result.tx}`);
      fulfilled++;
    } else {
      // Already fulfilled race condition is not a real error
      if (result.error.includes("already fulfilled") || result.error.includes("AlreadyFulfilled")) {
        console.log(`  ℹ️  Already fulfilled (race condition) — skipping`);
      } else {
        console.error(`  ❌ Failed: ${result.error.slice(0, 120)}`);
        failed++;
      }
    }

    // Small delay between TXs to avoid rate limiting
    if (pending.length > 1) await new Promise(r => setTimeout(r, 500));
  }

  console.log(`Done — ${fulfilled} fulfilled, ${failed} failed`);

  const result = { fulfilled, failed, total: pending.length };
  console.log(JSON.stringify(result));
  process.exit(failed > 0 ? 1 : 0);
}

main().catch(err => {
  console.error("Fatal:", err.message || err);
  process.exit(1);
});
