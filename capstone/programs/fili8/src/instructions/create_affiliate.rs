use anchor_lang::prelude::*;

use crate::errors::Error;
use crate::state::Affiliate;

#[derive(Accounts)]
pub struct CreateAffiliate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer=signer,
        seeds=[b"affiliate", signer.key.as_ref()],
        bump,
        space=Affiliate::INIT_SPACE
    )]
    pub affiliate: Account<'info, Affiliate>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateAffiliate<'info> {
    pub fn create_affiliate(
        &mut self,
        name: String,
        description: String,
        bumps: &CreateAffiliateBumps,
    ) -> Result<()> {
        require!(name.len() <= 50, Error::NameTooLong);
        require!(name.len() >= 10, Error::NameTooShort);
        require!(description.len() <= 100, Error::DescriptionTooLong);

        self.affiliate.set_inner(Affiliate {
            name,
            description,
            total_campaigns: 0,
            total_earned: 0,
            bump: bumps.affiliate,
        });
        Ok(())
    }
}
