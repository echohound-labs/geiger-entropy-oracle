//! Geiger Entropy Oracle — X1 Anchor Program
use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};

declare_id!("2dQf9uaCzXewrDNLttmtzQmc3SmqfAHz3qahKQjtGQyY");

pub const ORACLE_STATE_SEED: &[u8] = b"oracle_state";
pub const ENTROPY_NODE_SEED: &[u8] = b"entropy_node";
pub const ENTROPY_POOL_SEED: &[u8] = b"entropy_pool";
pub const RANDOMNESS_REQUEST_SEED: &[u8] = b"rand_request";

pub const POOL_CAPACITY: usize = 32;
pub const MAX_NODE_NAME_LEN: usize = 64;
pub const NODE_FEE_LAMPORTS: u64 = 10_000_000;
pub const REQUEST_FEE_LAMPORTS: u64 = 5_000_000;

#[program]
pub mod geiger_entropy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.oracle_state;
        state.authority = ctx.accounts.authority.key();
        state.total_nodes = 0;
        state.total_requests = 0;
        state.total_fulfillments = 0;
        state.paused = false;
        state.bump = ctx.bumps.oracle_state;

        let pool = &mut ctx.accounts.entropy_pool;
        pool.seeds = [[0u8; 32]; POOL_CAPACITY];
        pool.head = 0;
        pool.total_submissions = 0;
        pool.bump = ctx.bumps.entropy_pool;

        msg!("☢️ Geiger Entropy Oracle initialized");
        Ok(())
    }

    pub fn register_node(
        ctx: Context<RegisterNode>,
        node_pubkey: Pubkey,
        name: String,
    ) -> Result<()> {
        require!(name.len() <= MAX_NODE_NAME_LEN, GeigerError::NameTooLong);

        let node = &mut ctx.accounts.entropy_node;
        node.operator = ctx.accounts.operator.key();
        node.node_pubkey = node_pubkey;
        node.name = name.clone();
        node.submissions = 0;
        node.reputation = 100;
        node.active = true;
        node.registered_at = Clock::get()?.unix_timestamp;
        node.bump = ctx.bumps.entropy_node;

        let state = &mut ctx.accounts.oracle_state;
        state.total_nodes = state.total_nodes.saturating_add(1);

        emit!(NodeRegistered {
            operator: ctx.accounts.operator.key(),
            node_pubkey,
            name,
        });

        msg!("Node registered: {}", ctx.accounts.operator.key());
        Ok(())
    }

    pub fn submit_entropy(
        ctx: Context<SubmitEntropy>,
        seed: [u8; 32],
        cpm: u32,
        timestamp: i64,
        _signature: [u8; 64],
    ) -> Result<()> {
        require!(!ctx.accounts.oracle_state.paused, GeigerError::OraclePaused);
        require!(cpm >= 5, GeigerError::CpmTooLow);

        let node = &mut ctx.accounts.entropy_node;
        require!(node.active, GeigerError::NodeNotActive);

        let node_pubkey = node.node_pubkey;

        let pool = &mut ctx.accounts.entropy_pool;
        let idx = (pool.head as usize) % POOL_CAPACITY;
        pool.seeds[idx] = seed;
        pool.head = pool.head.wrapping_add(1);
        pool.total_submissions = pool.total_submissions.saturating_add(1);

        node.submissions = node.submissions.saturating_add(1);
        node.last_submission = Clock::get()?.unix_timestamp;

        let node_key = ctx.accounts.entropy_node.key();

        emit!(EntropySubmitted {
            node: node_key,
            node_pubkey,
            seed,
            cpm,
            timestamp,
        });

        msg!("Entropy submitted | CPM={} seed={:?}", cpm, &seed[..4]);
        Ok(())
    }

    pub fn request_randomness(
        ctx: Context<RequestRandomness>,
        user_seed: [u8; 32],
    ) -> Result<()> {
        require!(!ctx.accounts.oracle_state.paused, GeigerError::OraclePaused);
        require!(
            ctx.accounts.entropy_pool.total_submissions > 0,
            GeigerError::NoEntropyAvailable
        );

        let request_key = ctx.accounts.randomness_request.key();
        let requester_key = ctx.accounts.requester.key();

        let request = &mut ctx.accounts.randomness_request;
        request.requester = requester_key;
        request.user_seed = user_seed;
        request.status = RequestStatus::Pending;
        request.result = [0u8; 32];
        request.requested_at = Clock::get()?.unix_timestamp;
        request.fulfilled_at = 0;
        request.bump = ctx.bumps.randomness_request;

        let state = &mut ctx.accounts.oracle_state;
        state.total_requests = state.total_requests.saturating_add(1);

        emit!(RandomnessRequested {
            request: request_key,
            requester: requester_key,
            user_seed,
        });

        Ok(())
    }

    pub fn fulfill_randomness(ctx: Context<FulfillRandomness>) -> Result<()> {
        let request_key = ctx.accounts.randomness_request.key();

        let request = &mut ctx.accounts.randomness_request;
        require!(
            request.status == RequestStatus::Pending,
            GeigerError::RequestAlreadyFulfilled
        );

        let pool = &ctx.accounts.entropy_pool;
        // SHA256 chained pool mixing — stronger than XOR, stack-safe on-chain
        // Domain separated — prevents cross-protocol collisions (GEIGER_POOL_V1)
        // Future-proof for multi-node — each seed cryptographically chained
        let mut pool_seed = [0u8; 32];
        for seed in &pool.seeds {
            let mut h = Sha256::new();
            h.update(b"GEIGER_POOL_V1");
            h.update(&pool_seed);
            h.update(seed);
            pool_seed = h.finalize().into();
        }
        // Final result: SHA256(GEIGER_POOL_V1 || user_seed || pool_seed)
        let mut final_hasher = Sha256::new();
        final_hasher.update(b"GEIGER_POOL_V1");
        final_hasher.update(&request.user_seed);
        final_hasher.update(&pool_seed);
        let result: [u8; 32] = final_hasher.finalize().into();

        let requester = request.requester;
        request.result = result;
        request.status = RequestStatus::Fulfilled;
        request.fulfilled_at = Clock::get()?.unix_timestamp;

        let state = &mut ctx.accounts.oracle_state;
        state.total_fulfillments = state.total_fulfillments.saturating_add(1);

        emit!(RandomnessFulfilled {
            request: request_key,
            requester,
            result,
        });

        msg!("Randomness fulfilled: {:?}", &result[..8]);
        Ok(())
    }

    /// Step 1: Commit entropy hash on-chain (blind)
    pub fn commit_entropy(
        ctx: Context<CommitEntropy>,
        commitment_hash: [u8; 32],
        sequence: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.oracle_state.paused, GeigerError::OraclePaused);
        require!(commitment_hash != [0u8; 32], GeigerError::InvalidCommitment);

        let pc = &mut ctx.accounts.pending_commitment;
        require!(
            pc.revealed || pc.committed_slot == 0,
            GeigerError::UnrevealedCommitmentPending
        );

        let clock = Clock::get()?;
        pc.operator = ctx.accounts.operator.key();
        pc.commitment_hash = commitment_hash;
        pc.committed_slot = clock.slot;
        pc.sequence = sequence;
        pc.revealed = false;
        pc.bump = ctx.bumps.pending_commitment;

        emit!(CommitEvent {
            operator: ctx.accounts.operator.key(),
            commitment_hash,
            slot: clock.slot,
            sequence,
        });

        msg!("Entropy committed | seq={} slot={}", sequence, clock.slot);
        Ok(())
    }

    /// Step 2: Reveal entropy — must match commitment
    pub fn reveal_entropy(
        ctx: Context<RevealEntropy>,
        vdf_output: [u8; 32],
        operator_nonce: [u8; 32],
        cpm: u32,
        timestamp: i64,
        _signature: [u8; 64],
        delta_t_ms: u64,
        usv_h_milli: u32,
        vdf_iters: u32,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let pc = &mut ctx.accounts.pending_commitment;

        require!(!pc.revealed, GeigerError::AlreadyRevealed);
        require!(
            clock.slot >= pc.committed_slot + COMMIT_REVEAL_DELAY_SLOTS,
            GeigerError::RevealTooEarly
        );
        require!(
            clock.slot <= pc.committed_slot + REVEAL_DEADLINE_SLOTS,
            GeigerError::RevealDeadlineMissed
        );
        require!(cpm >= 5, GeigerError::CpmTooLow);

        // Verify H(vdf_output || nonce) == stored commitment
        let mut preimage = [0u8; 64];
        preimage[..32].copy_from_slice(&vdf_output);
        preimage[32..].copy_from_slice(&operator_nonce);
        let mut hasher = Sha256::new();
        hasher.update(&preimage);
        let computed_hash: [u8; 32] = hasher.finalize().into();
        require!(computed_hash == pc.commitment_hash, GeigerError::CommitmentMismatch);

        // Read current slot hash from SlotHashes sysvar for binding
        let slot_hashes_data = ctx.accounts.slot_hashes.try_borrow_data()?;
        let binding_slot = clock.slot;
        let mut slot_hash = [0u8; 32];
        if slot_hashes_data.len() >= 48 {
            slot_hash.copy_from_slice(&slot_hashes_data[16..48]);
        }
        drop(slot_hashes_data);

        // Mix: SHA256(vdf_output || slot_hash || sequence_bytes) = final bound seed
        let mut hasher = Sha256::new();
        hasher.update(&vdf_output);
        hasher.update(&slot_hash);
        hasher.update(&pc.sequence.to_le_bytes());
        let bound_seed: [u8; 32] = hasher.finalize().into();

        // Store bound seed in entropy pool
        let pool = &mut ctx.accounts.entropy_pool;
        let idx = (pool.head as usize) % POOL_CAPACITY;
        pool.seeds[idx] = bound_seed;
        pool.head = pool.head.wrapping_add(1);
        pool.total_submissions = pool.total_submissions.saturating_add(1);

        let node = &mut ctx.accounts.entropy_node;
        node.submissions = node.submissions.saturating_add(1);
        node.last_submission = clock.unix_timestamp;

        pc.revealed = true;

        emit!(RevealEvent {
            operator: ctx.accounts.operator.key(),
            sequence: pc.sequence,
            commit_slot: pc.committed_slot,
            reveal_slot: clock.slot,
            vdf_output,
            cpm,
            timestamp,
            delta_t_ms,
            usv_h_milli,
            vdf_iters,
        });

        let sources_bitmap: u8 = 0x07;
        msg!("☢️ Entropy revealed | seq={} CPM={} uSv/h={:.3} dt={:.3}s VDF={}iters seed={:?} slot_hash={:?} binding_slot={} sources=0x{:02x} verified✓",
            pc.sequence, cpm, usv_h_milli as f64 / 1000.0, delta_t_ms as f64 / 1000.0,
            vdf_iters, &bound_seed[..4], &slot_hash[..4], binding_slot, sources_bitmap);
        Ok(())
    }

    /// v6: Reveal entropy and create PendingFinalize for delayed SlotHash binding
    /// Closes selective withholding — operator reveals BEFORE knowing SlotHash[binding_slot]
    pub fn reveal_entropy_v6(
        ctx: Context<RevealEntropyV6>,
        vdf_output: [u8; 32],
        operator_nonce: [u8; 32],
        cpm: u32,
        timestamp: i64,
        _signature: [u8; 64],
        delta_t_ms: u64,
        usv_h_milli: u32,
        vdf_iters: u32,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let pc = &mut ctx.accounts.pending_commitment;

        require!(!pc.revealed, GeigerError::AlreadyRevealed);
        require!(
            clock.slot >= pc.committed_slot + COMMIT_REVEAL_DELAY_SLOTS,
            GeigerError::RevealTooEarly
        );
        require!(
            clock.slot <= pc.committed_slot + REVEAL_DEADLINE_SLOTS,
            GeigerError::RevealDeadlineMissed
        );
        require!(cpm >= 5, GeigerError::CpmTooLow);

        // Verify H(vdf_output || nonce) == stored commitment
        let mut preimage = [0u8; 64];
        preimage[..32].copy_from_slice(&vdf_output);
        preimage[32..].copy_from_slice(&operator_nonce);
        let mut hasher = Sha256::new();
        hasher.update(&preimage);
        let computed_hash: [u8; 32] = hasher.finalize().into();
        require!(computed_hash == pc.commitment_hash, GeigerError::CommitmentMismatch);

        // Mark commitment as revealed
        pc.revealed = true;

        // Create PendingFinalize with delayed binding_slot
        let pf = &mut ctx.accounts.pending_finalize;
        pf.operator = ctx.accounts.operator.key();
        pf.vdf_output = vdf_output;
        pf.binding_slot = pc.committed_slot + BINDING_SLOT_DELAY; // future slot — unknown now
        pf.committed_at_slot = pc.committed_slot;
        pf.sequence = pc.sequence;
        pf.finalized = false;
        pf.bump = ctx.bumps.pending_finalize;

        let node = &mut ctx.accounts.entropy_node;
        node.submissions = node.submissions.saturating_add(1);
        node.last_submission = clock.unix_timestamp;

        msg!("☢️ Entropy revealed (v6) | seq={} CPM={} binding_slot={} — awaiting finalize at slot {}",
            pc.sequence, cpm, pf.binding_slot, pf.binding_slot);
        msg!("   ⚠️  SlotHash[{}] unknown at reveal — selective withholding CLOSED", pf.binding_slot);
        Ok(())
    }

    /// Slash operator who committed but failed to reveal
    pub fn slash_missed_reveal(ctx: Context<SlashMissedReveal>) -> Result<()> {
        let pc = &mut ctx.accounts.pending_commitment;
        let clock = Clock::get()?;

        require!(
            clock.slot > pc.committed_slot + REVEAL_DEADLINE_SLOTS,
            GeigerError::RevealDeadlineNotReached
        );
        require!(!pc.revealed, GeigerError::AlreadyRevealed);

        **ctx.accounts.operator.try_borrow_mut_lamports()? -= SLASH_AMOUNT_LAMPORTS;
        **ctx.accounts.reporter.try_borrow_mut_lamports()? += SLASH_AMOUNT_LAMPORTS;

        pc.revealed = true;

        emit!(SlashEvent {
            operator: ctx.accounts.operator.key(),
            reporter: ctx.accounts.reporter.key(),
            sequence: pc.sequence,
            slash_amount: SLASH_AMOUNT_LAMPORTS,
            slot: clock.slot,
        });

        msg!("🚨 Operator slashed | seq={}", pc.sequence);
        Ok(())
    }

    pub fn reset_commitment(_ctx: Context<ResetCommitment>) -> Result<()> {
        // Closes the PendingCommitment account and returns lamports to operator
        // Used to migrate from old struct layout to v6 struct
        Ok(())
    }

    pub fn finalize_entropy(ctx: Context<FinalizeEntropy>) -> Result<()> {
        let clock = Clock::get()?;
        let pf = &mut ctx.accounts.pending_finalize;

        require!(clock.slot >= pf.binding_slot, GeigerError::BindingSlotNotReached);
        require!(!pf.finalized, GeigerError::AlreadyFinalized);

        let slot_hashes_data = ctx.accounts.slot_hashes.try_borrow_data()?;
        let mut slot_hash = [0u8; 32];
        let num_entries = if slot_hashes_data.len() >= 8 {
            u64::from_le_bytes(slot_hashes_data[0..8].try_into().unwrap()) as usize
        } else { 0 };
        let mut found = false;
        for i in 0..num_entries.min(512) {
            let offset = 8 + i * 40;
            if offset + 40 > slot_hashes_data.len() { break; }
            let slot = u64::from_le_bytes(slot_hashes_data[offset..offset+8].try_into().unwrap());
            if slot == pf.binding_slot {
                slot_hash.copy_from_slice(&slot_hashes_data[offset+8..offset+40]);
                found = true;
                break;
            }
        }
        drop(slot_hashes_data);
        require!(found, GeigerError::SlotHashNotAvailable);

        let mut hasher = Sha256::new();
        hasher.update(&pf.vdf_output);
        hasher.update(&slot_hash);
        hasher.update(&pf.sequence.to_le_bytes());
        hasher.update(b"GEIGER_V6_FINALIZE");
        let final_seed: [u8; 32] = hasher.finalize().into();

        let pool = &mut ctx.accounts.entropy_pool;
        let idx = (pool.head as usize) % POOL_CAPACITY;
        pool.seeds[idx] = final_seed;
        pool.head = pool.head.wrapping_add(1);
        pool.total_submissions = pool.total_submissions.saturating_add(1);

        let node = &mut ctx.accounts.entropy_node;
        node.submissions = node.submissions.saturating_add(1);

        pf.finalized = true;

        msg!("☢️ Entropy FINALIZED (v6) | seq={} binding_slot={} finalized_slot={}",
            pf.sequence, pf.binding_slot, clock.slot);
        Ok(())
    }



}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + OracleState::INIT_SPACE,
        seeds = [ORACLE_STATE_SEED],
        bump,
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(
        init,
        payer = authority,
        space = 8 + EntropyPool::INIT_SPACE,
        seeds = [ENTROPY_POOL_SEED],
        bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterNode<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(
        init,
        payer = operator,
        space = 8 + EntropyNode::INIT_SPACE,
        seeds = [ENTROPY_NODE_SEED, operator.key().as_ref()],
        bump,
    )]
    pub entropy_node: Account<'info, EntropyNode>,

    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitEntropy<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(
        mut,
        seeds = [ENTROPY_POOL_SEED],
        bump = entropy_pool.bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,

    #[account(
        mut,
        seeds = [ENTROPY_NODE_SEED, operator.key().as_ref()],
        bump = entropy_node.bump,
        has_one = operator,
    )]
    pub entropy_node: Account<'info, EntropyNode>,

    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(
        seeds = [ENTROPY_POOL_SEED],
        bump = entropy_pool.bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,

    #[account(
        init,
        payer = requester,
        space = 8 + RandomnessRequest::INIT_SPACE,
        seeds = [RANDOMNESS_REQUEST_SEED, requester.key().as_ref(), &oracle_state.total_requests.to_le_bytes()],
        bump,
    )]
    pub randomness_request: Account<'info, RandomnessRequest>,

    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FulfillRandomness<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,

    #[account(
        seeds = [ENTROPY_POOL_SEED],
        bump = entropy_pool.bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,

    #[account(
        mut,
        has_one = requester,
    )]
    pub randomness_request: Account<'info, RandomnessRequest>,

    pub requester: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct OracleState {
    pub authority: Pubkey,
    pub total_nodes: u64,
    pub total_requests: u64,
    pub total_fulfillments: u64,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct EntropyPool {
    pub seeds: [[u8; 32]; POOL_CAPACITY],
    pub head: u64,
    pub total_submissions: u64,
    pub bump: u8,
}

impl EntropyPool {
    pub const INIT_SPACE: usize = 32 * POOL_CAPACITY + 8 + 8 + 1;
}

#[account]
#[derive(InitSpace)]
pub struct EntropyNode {
    pub operator: Pubkey,
    pub node_pubkey: Pubkey,
    #[max_len(64)]
    pub name: String,
    pub submissions: u64,
    pub reputation: u8,
    pub active: bool,
    pub registered_at: i64,
    pub last_submission: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct RandomnessRequest {
    pub requester: Pubkey,
    pub user_seed: [u8; 32],
    pub result: [u8; 32],
    pub status: RequestStatus,
    pub requested_at: i64,
    pub fulfilled_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum RequestStatus {
    Pending,
    Fulfilled,
    Cancelled,
}

#[event]
pub struct NodeRegistered {
    pub operator: Pubkey,
    pub node_pubkey: Pubkey,
    pub name: String,
}

#[event]
pub struct EntropySubmitted {
    pub node: Pubkey,
    pub node_pubkey: Pubkey,
    pub seed: [u8; 32],
    pub cpm: u32,
    pub timestamp: i64,
}

#[event]
pub struct RandomnessRequested {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub user_seed: [u8; 32],
}

#[event]
pub struct RandomnessFulfilled {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub result: [u8; 32],
}


#[event]
pub struct CommitEvent {
    pub operator: Pubkey,
    pub commitment_hash: [u8; 32],
    pub slot: u64,
    pub sequence: u64,
}

#[event]
pub struct RevealEvent {
    pub operator: Pubkey,
    pub sequence: u64,
    pub commit_slot: u64,
    pub reveal_slot: u64,
    pub vdf_output: [u8; 32],
    pub cpm: u32,
    pub timestamp: i64,
    pub delta_t_ms: u64,
    pub usv_h_milli: u32,
    pub vdf_iters: u32,
}

#[event]
pub struct SlashEvent {
    pub operator: Pubkey,
    pub reporter: Pubkey,
    pub sequence: u64,
    pub slash_amount: u64,
    pub slot: u64,
}

#[error_code]
pub enum GeigerError {
    #[msg("Oracle is paused")]
    OraclePaused,
    #[msg("Node is not active")]
    NodeNotActive,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("CPM too low — minimum 5 required")]
    CpmTooLow,
    #[msg("No entropy available yet")]
    NoEntropyAvailable,
    #[msg("Request already fulfilled")]
    RequestAlreadyFulfilled,
    #[msg("Node name too long (max 64 chars)")]
    NameTooLong,

    #[msg("Invalid commitment hash")]
    InvalidCommitment,
    #[msg("Sequence number invalid")]
    InvalidSequence,
    #[msg("Previous commitment not yet revealed")]
    UnrevealedCommitmentPending,
    #[msg("Binding slot not reached yet — must wait for delayed finalize")]
    BindingSlotNotReached,
    #[msg("Slot hash not available in sysvar — too old")]
    SlotHashNotAvailable,
    #[msg("Not yet revealed")]
    NotYetRevealed,
    #[msg("Already finalized")]
    AlreadyFinalized,
    #[msg("Commitment hash mismatch")]
    CommitmentMismatch,
    #[msg("Reveal too early")]
    RevealTooEarly,
    #[msg("Reveal deadline missed")]
    RevealDeadlineMissed,
    #[msg("Reveal deadline not yet reached")]
    RevealDeadlineNotReached,
    #[msg("Already revealed")]
    AlreadyRevealed,
}

// ---------------------------------------------------------------------------
// Commit-Reveal Constants
// ---------------------------------------------------------------------------

pub const COMMITMENT_SEED: &[u8] = b"commitment";
pub const COMMIT_REVEAL_DELAY_SLOTS: u64 = 8;
pub const REVEAL_DEADLINE_SLOTS: u64 = 128;
pub const BINDING_SLOT_DELAY: u64 = 150;   // Future slot for SlotHash — unknown at reveal time
pub const FINALIZE_GRACE_SLOTS: u64 = 50;  // Grace period after binding_slot before slash
pub const SLASH_AMOUNT_LAMPORTS: u64 = 5_000_000_000; // 5 XNT

// ---------------------------------------------------------------------------
// Commit-Reveal State
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct PendingCommitment {
    pub operator: Pubkey,
    pub commitment_hash: [u8; 32],
    pub committed_slot: u64,
    pub sequence: u64,
    pub revealed: bool,
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// v6: Pending Finalize Account (separate PDA — avoids migration issues)
// ---------------------------------------------------------------------------
#[account]
#[derive(InitSpace)]
pub struct PendingFinalize {
    pub operator: Pubkey,       // 32
    pub vdf_output: [u8; 32],   // 32
    pub binding_slot: u64,      // 8  — future slot, unknown at reveal time
    pub committed_at_slot: u64, // 8  — audit trail
    pub sequence: u64,          // 8
    pub finalized: bool,        // 1
    pub bump: u8,               // 1
}

pub const FINALIZE_SEED: &[u8] = b"finalize";

// ---------------------------------------------------------------------------
// Commit-Reveal Account Contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(commitment_hash: [u8; 32], sequence: u64)]
pub struct CommitEntropy<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,
    #[account(
        init_if_needed,
        payer = operator,
        space = 8 + PendingCommitment::INIT_SPACE,
        seeds = [COMMITMENT_SEED, operator.key().as_ref()],
        bump,
    )]
    pub pending_commitment: Account<'info, PendingCommitment>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealEntropy<'info> {
    #[account(
        mut,
        seeds = [ORACLE_STATE_SEED],
        bump = oracle_state.bump,
    )]
    pub oracle_state: Account<'info, OracleState>,
    #[account(
        mut,
        seeds = [ENTROPY_POOL_SEED],
        bump = entropy_pool.bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,
    #[account(
        mut,
        seeds = [COMMITMENT_SEED, operator.key().as_ref()],
        bump = pending_commitment.bump,
        constraint = pending_commitment.operator == operator.key(),
    )]
    pub pending_commitment: Account<'info, PendingCommitment>,
    #[account(
        mut,
        seeds = [ENTROPY_NODE_SEED, operator.key().as_ref()],
        bump = entropy_node.bump,
        has_one = operator,
    )]
    pub entropy_node: Account<'info, EntropyNode>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: SlotHashes sysvar for on-chain entropy binding
    #[account(address = anchor_lang::solana_program::sysvar::slot_hashes::ID)]
    pub slot_hashes: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ResetCommitment<'info> {
    #[account(
        mut,
        close = operator,
        seeds = [COMMITMENT_SEED, operator.key().as_ref()],
        bump = pending_commitment.bump,
    )]
    pub pending_commitment: Account<'info, PendingCommitment>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeEntropy<'info> {
    #[account(
        mut,
        seeds = [ENTROPY_POOL_SEED],
        bump = entropy_pool.bump,
    )]
    pub entropy_pool: Account<'info, EntropyPool>,
    #[account(
        mut,
        close = caller,
        seeds = [FINALIZE_SEED, pending_finalize.operator.as_ref(), &pending_finalize.sequence.to_le_bytes()],
        bump = pending_finalize.bump,
    )]
    pub pending_finalize: Account<'info, PendingFinalize>,
    #[account(
        mut,
        seeds = [ENTROPY_NODE_SEED, pending_finalize.operator.as_ref()],
        bump = entropy_node.bump,
    )]
    pub entropy_node: Account<'info, EntropyNode>,
    /// CHECK: SlotHashes sysvar for delayed entropy binding
    #[account(address = anchor_lang::solana_program::sysvar::slot_hashes::ID)]
    pub slot_hashes: AccountInfo<'info>,
    // Anyone can call finalize (permissionless for liveness)
    #[account(mut)]
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevealEntropyV6<'info> {
    #[account(
        mut,
        seeds = [COMMITMENT_SEED, operator.key().as_ref()],
        bump = pending_commitment.bump,
        constraint = pending_commitment.operator == operator.key(),
    )]
    pub pending_commitment: Account<'info, PendingCommitment>,
    #[account(
        init,
        payer = operator,
        space = 8 + PendingFinalize::INIT_SPACE,
        seeds = [FINALIZE_SEED, operator.key().as_ref(), &pending_commitment.sequence.to_le_bytes()],
        bump,
    )]
    pub pending_finalize: Account<'info, PendingFinalize>,
    #[account(
        mut,
        seeds = [ENTROPY_NODE_SEED, operator.key().as_ref()],
        bump = entropy_node.bump,
        has_one = operator,
    )]
    pub entropy_node: Account<'info, EntropyNode>,
    #[account(mut)]
    pub operator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SlashMissedReveal<'info> {
    #[account(
        mut,
        seeds = [COMMITMENT_SEED, operator.key().as_ref()],
        bump = pending_commitment.bump,
    )]
    pub pending_commitment: Account<'info, PendingCommitment>,
    /// CHECK: operator being slashed
    #[account(mut)]
    pub operator: AccountInfo<'info>,
    #[account(mut)]
    pub reporter: Signer<'info>,
    pub system_program: Program<'info, System>,
}
