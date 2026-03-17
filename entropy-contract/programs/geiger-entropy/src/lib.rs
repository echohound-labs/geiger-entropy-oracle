//! Geiger Entropy Oracle — X1 Anchor Program
use anchor_lang::prelude::*;

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
}

// ---------------------------------------------------------------------------
// Commit-Reveal Constants
// ---------------------------------------------------------------------------

pub const COMMITMENT_SEED: &[u8] = b"commitment";
pub const COMMIT_REVEAL_DELAY_SLOTS: u64 = 8;
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
