use std::cell::RefMut;

use anchor_lang::{
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::{spl_token_2022, Token2022}, token_interface::{Mint, TokenAccount, TokenInterface}
};
use spl_tlv_account_resolution::{
    state::ExtraAccountMetaList,
};
use spl_token_2022::{
    extension::transfer_hook::TransferHookAccount,
    state::{Account as PodAccount},
    extension::StateWithExtensionsMut,
};
use spl_transfer_hook_interface::instruction::{ExecuteInstruction, TransferHookInstruction};

declare_id!("Ghwc2QmqWrVjv1vtJwPjUy2MXjbFoSWJetmrNeq7pimu");
// Variável global para armazenar o ID do programa de token

#[program]
pub mod transfer_hook {
    use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed};

    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {

        // A função auxiliar `addExtraAccountsToInstruction` do JS está resolvendo incorretamente
        let account_metas = vec![
            ExtraAccountMeta::new_with_pubkey(&ctx.accounts.mint.key(), false, true)?,
            // index 6, token program
            ExtraAccountMeta::new_with_pubkey(&ctx.accounts.token_program.key(), false, false)?,
            // index 7, associated token program
            ExtraAccountMeta::new_with_pubkey(
                &ctx.accounts.associated_token_program.key(),
                false,
                false,
            )?,
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // associated token program index
                &[
                    Seed::AccountKey { index: 7 }, // owner index
                    Seed::AccountKey { index: 2 }, // token program index
                    Seed::AccountKey { index: 5 }, // wsol mint index
                ],
                false, // is_signer
                true,  // is_writable
            )?,
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // associated token program index
                &[
                    Seed::AccountKey { index: 3 }, // owner index
                    Seed::AccountKey { index: 2 }, // token program index
                    Seed::AccountKey { index: 5 }, // wsol mint index
                ],
                false, // is_signer
                true,  // is_writable
            )?
        ];

        // calcular o tamanho da conta
       // Ajuste para garantir que a conta tenha espaço suficiente
let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64 + 100; // Aumente um pouco

        // calcular o mínimo de lamports necessários
        let lamports = Rent::get()?.minimum_balance(account_size as usize);

        let mint = ctx.accounts.mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"extra-account-metas",
            &mint.as_ref(),
            &[ctx.bumps.extra_account_meta_list],
        ]];

        // criar conta ExtraAccountMetaList
        create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.extra_account_meta_list.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            lamports,
            account_size,
            ctx.program_id,
        )?;

        // inicializar conta ExtraAccountMetaList com contas extras
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &account_metas,
        )?;

        Ok(())
    }

    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint-authority", &[ctx.bumps.mint_authority]]];
       
       
        msg!("Processing Transfer Hook!");

        

        Ok(())
    }

    // manipulador de instrução de fallback como solução alternativa para a verificação do discriminador de instrução do anchor
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;

        // corresponder o discriminador de instrução à instrução de execução da interface do gancho de transferência
        // o programa token2022 CPIs esta instrução na transferência de token
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();

                // invocar instrução de gancho de transferência personalizada no nosso programa
                __private::__global::transfer_hook(program_id, accounts, &amount_bytes)
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
    }
    
}
 
#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: Conta ExtraAccountMetaList, deve usar essas seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}




#[derive(Accounts)]
pub struct TransferHook<'info> {
  #[account(token::mint = mint, token::authority = owner)]
  pub source_token: InterfaceAccount<'info, TokenAccount>,
  #[account(mut)]
  pub mint: InterfaceAccount<'info, Mint>,
  #[account(token::mint = mint)]
  pub destination_token: InterfaceAccount<'info, TokenAccount>,
  /// CHECK: source token account owner
  pub owner: UncheckedAccount<'info>,
  /// CHECK: ExtraAccountMetaList Account,
  #[account(seeds = [b"extra-account-metas", mint.key().as_ref()], bump)]
  pub extra_account_meta_list: UncheckedAccount<'info>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  
  /// CHECK: mint authority Account,
  #[account(seeds = [b"mint-authority"], bump)]
  pub mint_authority: UncheckedAccount<'info>,
  
  
}

