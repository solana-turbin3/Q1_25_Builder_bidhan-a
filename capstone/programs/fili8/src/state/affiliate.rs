use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Affiliate {
    pub owner: Pubkey,
    #[max_len(50)]
    pub name: String,
    #[max_len(100)]
    pub description: String,
    pub payout_address: Pubkey,
    pub total_campaigns: u32,
    pub total_earned: u64,
    pub bump: u8,
}
