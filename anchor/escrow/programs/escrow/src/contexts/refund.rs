use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
			mint::token_program = token_program
		)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
			init_if_needed,
            payer = maker,
			associated_token::mint = mint_a,
			associated_token::authority = maker,
            associated_token::token_program = token_program
		)]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
			mut,
			close = maker,
			has_one = maker,
			has_one = mint_a,
			seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
			bump = escrow.bump
		)]
    pub escrow: Account<'info, Escrow>,

    #[account(
			mut,
			associated_token::mint = mint_a,
			associated_token::authority = escrow,
			associated_token::token_program = token_program
		)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn withdraw_and_close(&mut self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let seed = self.escrow.seed.to_le_bytes();
        let bump = [self.escrow.bump];

        let signer_seeds = &[&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            seed.as_ref(),
            &bump,
        ][..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        close_account(cpi_ctx)
    }
}
