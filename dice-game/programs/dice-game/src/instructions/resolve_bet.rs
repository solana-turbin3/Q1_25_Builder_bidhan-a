use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::load_instruction_at_checked,
    system_program::{transfer, Transfer},
};
use solana_program::{ed25519_program, hash::hash};

use crate::state::Bet;

pub const HOUSE_EDGE: u16 = 150; // 1.5% House edge

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(mut)]
    pub house: SystemAccount<'info>,

    #[account(
        mut,
        seeds=[b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds=[b"bet", vault.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        address=solana_program::sysvar::instructions::ID,
    )]
    pub instruction_sysvar: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&mut self, sig: &[u8]) -> Result<()> {
        // Get the ED25519 signature instruction.
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar)?;
        // Make sure the instruction is addressed to the ed25519 program.
        require_keys_eq!(ix.program_id, ed25519_program::ID,);
        // Make sure there are no accounts present.
        require_eq!(ix.accounts.len(), 0);

        let signatures = Ed25519InstructionSignatures::unpack(&ix.data)?.0;

        require_eq!(signatures.len(), 1);
        let signature = &signatures[0];

        // Make sure all the data is present to verify the signature
        require_eq!(signature.is_verifiable, true);

        // Ensure public keys match
        require_keys_eq!(signature.public_key.unwrap(), self.house.key(),);

        // Ensure signatures match
        require_eq!(signature.signature.unwrap().eq(sig), true);

        // Ensure messages match
        require_eq!(
            signature.message.as_ref().unwrap().eq(&self.bet.to_slice()),
            true
        );

        Ok(())
    }

    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps, sig: &[u8]) -> Result<()> {
        let hash = hash(sig).to_bytes();
        let mut hash_16: [u8; 16] = [0; 16];
        hash_16.copy_from_slice(&hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);
        hash_16.copy_from_slice(&hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower.wrapping_add(upper).wrapping_rem(100) as u8 + 1;

        if self.bet.roll > roll {
            // Payout minus house edge
            let payout = (self.bet.amount as u128)
                .checked_mul(10000 - HOUSE_EDGE as u128)
                .unwrap()
                .checked_div(self.bet.roll as u128 - 1)
                .unwrap()
                .checked_div(100)
                .unwrap() as u64;

            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            let seeds = [b"vault", &self.house.key().to_bytes()[..], &[bumps.vault]];
            let signer_seeds = &[&seeds[..]][..];

            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, payout)?;
        }
        Ok(())
    }
}
