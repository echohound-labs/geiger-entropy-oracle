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
        let mut pool_seed = [0u8; 32];
        for seed in &pool.seeds {
            for i in 0..32 {
                pool_seed[i] ^= seed[i];
            }
        }

        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = request.user_seed[i] ^ pool_seed[i];
        }

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

        // Store in entropy pool
        let pool = &mut ctx.accounts.entropy_pool;
        let idx = (pool.head as usize) % POOL_CAPACITY;
        pool.seeds[idx] = vdf_output;
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
        });

        msg!("☢️ Entropy revealed | seq={} CPM={} verified✓", pc.sequence, cpm);
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
pub const COMMIT_REVEAL_DELAY_SLOTS: u64 = 3;
pub const REVEAL_DEADLINE_SLOTS: u64 = 128;
pub const SLASH_AMOUNT_LAMPORTS: u64 = 100_000_000;

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
