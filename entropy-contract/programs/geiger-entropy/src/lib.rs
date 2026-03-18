//! Geiger Entropy Oracle — Chain Spammer Edition
use anchor_lang::prelude::*;
declare_id!("BxUNg2yo5371BQMZPkfcxdCptFRDHkhvEXNM1QNPBRYU");
pub const ENTROPY_POOL_SEED: &[u8] = b"entropy_pool";
pub const POOL_CAPACITY: usize = 32;
#[program]
pub mod geiger_entropy {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pool = &mut ctx.accounts.entropy_pool;
        pool.seeds = [[0u8; 32]; POOL_CAPACITY];
        pool.head = 0;
        pool.total_submissions = 0;
        pool.bump = ctx.bumps.entropy_pool;
        Ok(())
    }
    pub fn submit_entropy(ctx: Context<SubmitEntropy>, seed: [u8; 32]) -> Result<()> {
        let pool = &mut ctx.accounts.entropy_pool;
        let idx = (pool.head as usize) % POOL_CAPACITY;
        pool.seeds[idx] = seed;
        pool.head = pool.head.wrapping_add(1);
        pool.total_submissions = pool.total_submissions.saturating_add(1);
        Ok(())
    }
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + EntropyPool::INIT_SPACE, seeds = [ENTROPY_POOL_SEED], bump)]
    pub entropy_pool: Account<'info, EntropyPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct SubmitEntropy<'info> {
    #[account(mut, seeds = [ENTROPY_POOL_SEED], bump = entropy_pool.bump)]
    pub entropy_pool: Account<'info, EntropyPool>,
    #[account(mut)]
    pub operator: Signer<'info>,
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
