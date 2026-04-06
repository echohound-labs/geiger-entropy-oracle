const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [operatorStr, sequenceStr] = process.argv.slice(2);

  const keypairPath = path.join(process.env.HOME, ".config/solana/id.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );
  const connection = new Connection("https://rpc.testnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "../idl/testnet/geiger_entropy.json"), "utf8"
  ));
  const programId = new PublicKey("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const operatorPubkey = operatorStr ? new PublicKey(operatorStr) : wallet.publicKey;
  const sequence = parseInt(sequenceStr || "1");

  const [pendingFinalizePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("finalize"), operatorPubkey.toBuffer(),
     Buffer.from(new anchor.BN(sequence).toArray('le', 8))],
    programId
  );
  const [entropyPoolPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_pool")], programId
  );
  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), operatorPubkey.toBuffer()], programId
  );
  const SLOT_HASHES_PUBKEY = new PublicKey("SysvarS1otHashes111111111111111111111111111");

  // Check current state
  const pf = await program.account.pendingFinalize.fetch(pendingFinalizePDA);
  const currentSlot = await connection.getSlot();

  console.log(`Sequence: ${pf.sequence}`);
  console.log(`Binding slot: ${pf.bindingSlot}`);
  console.log(`Current slot: ${currentSlot}`);
  console.log(`Finalized: ${pf.finalized}`);

  if (pf.finalized) {
    console.log("✅ Already finalized!");
    return;
  }

  if (currentSlot < pf.bindingSlot) {
    console.log(`⏳ Not ready — wait ${pf.bindingSlot - currentSlot} more slots`);
    return;
  }

  console.log("Finalizing...");
  const tx = await program.methods
    .finalizeEntropy()
    .accounts({
      entropyPool: entropyPoolPDA,
      pendingFinalize: pendingFinalizePDA,
      entropyNode: entropyNodePDA,
      slotHashes: SLOT_HASHES_PUBKEY,
      caller: wallet.publicKey,
    })
    .rpc({ commitment: "confirmed" });

  console.log(`✅ Finalized! TX: ${tx}`);
  console.log(`Sequence ${sequence} mixed into pool with SlotHash[${pf.bindingSlot}]`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
