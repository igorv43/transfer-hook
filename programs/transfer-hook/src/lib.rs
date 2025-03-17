use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{self, Mint, TokenAccount, TokenInterface},
    token,
};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod burn_program {
    use super::*;

    pub fn process_burn_and_transfer(ctx: Context<ProcessBurnAndTransfer>, amount: u64) -> Result<()> {
        // Calcula 0.01% para queimar
        let burn_amount = amount.checked_mul(1).unwrap().checked_div(10000).unwrap();
        let transfer_amount = amount.checked_sub(burn_amount).unwrap();

        // Queima tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.source_token.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            burn_amount,
        )?;

        // Transfere o restante
        let cpi_accounts = token_interface::TransferChecked {
            from: ctx.accounts.source_token.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination_token.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token_interface::transfer_checked(
            cpi_ctx, 
            transfer_amount,
            ctx.accounts.mint.decimals
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ProcessBurnAndTransfer<'info> {
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    
    #[account(mut, token::mint = mint, token::authority = owner)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut, token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
}