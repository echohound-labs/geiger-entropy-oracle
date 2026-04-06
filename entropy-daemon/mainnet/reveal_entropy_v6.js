const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

async function main() {
  const [vdfOutputHex, operatorNonceHex, cpmStr] = process.argv.slice(2);
  if (!vdfOutputHex || !operatorNonceHex) {
    console.error("Usage: node reveal_entropy_v6.js <vdf_hex> <nonce_hex> [cpm]");
    process.exit(1);
  }

  const keypairPath = path.join(process.env.HOME, ".config/solana/mainnet-deployer.json");
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(keypairPath, "utf8")))
  );
  const connection = new Connection("https://rpc.mainnet.x1.xyz", "confirmed");
  const idl = JSON.parse(fs.readFileSync(
    path.join(__dirname, "../idl/mainnet-commit-reveal/geiger_entropy.json"), "utf8"
  ));
  const programId = new PublicKey("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");
  const provider = new anchor.AnchorProvider(
    connection, new anchor.Wallet(wallet), { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const [pendingCommitmentPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("commitment"), wallet.publicKey.toBuffer()], programId
  );

  const pc = await program.account.pendingCommitment.fetch(pendingCommitmentPDA);
  const sequence = pc.sequence;

  const [pendingFinalizePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("finalize"), wallet.publicKey.toBuffer(),
     Buffer.from(new anchor.BN(sequence).toArray('le', 8))],
    programId
  );

  const [entropyNodePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("entropy_node"), wallet.publicKey.toBuffer()], programId
  );

  const vdfOutput = Array.from(Buffer.from(vdfOutputHex, "hex").slice(0, 32));
  const nonce = Array.from(Buffer.from(operatorNonceHex, "hex").slice(0, 32));
  const zeroSig = Array(64).fill(0);
  const cpm = parseInt(cpmStr || "20");

  console.log("Calling revealEntropyV6...");
  console.log("Sequence:", sequence.toString());
  console.log("PendingFinalize PDA:", pendingFinalizePDA.toBase58());

  const tx = await program.methods
    .revealEntropyV6(
      vdfOutput,
      nonce,
      cpm,
      new anchor.BN(Math.floor(Date.now() / 1000)),
      zeroSig,
      new anchor.BN(0),
      0,
      100000
    )
    .accounts({
      pendingCommitment: pendingCommitmentPDA,
      pendingFinalize: pendingFinalizePDA,
      entropyNode: entropyNodePDA,
      operator: wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc({ commitment: "confirmed" });

  await new Promise(r => setTimeout(r, 1000));

  const pf = await program.account.pendingFinalize.fetch(pendingFinalizePDA);
  const currentSlot = await connection.getSlot();

  console.log(`✅ Revealed v6! TX: ${tx}`);
  console.log(`Binding slot: ${pf.bindingSlot} (current: ${currentSlot})`);
  console.log(`Wait ${pf.bindingSlot - currentSlot} more slots then call finalize`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
