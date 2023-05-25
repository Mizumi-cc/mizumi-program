mod constants;

use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount}
};
declare_id!("6pm1yXLY9AHUSwQmsK481YJaKgfChgjCkzvXQoZsRUg");

#[program]
pub mod mizumi_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // initialize all the accounts for new usdc and usdt pools
        Ok(())
    }

    pub fn new_user(ctx: Context<NewUser>) -> Result<()> {
        if ctx.accounts.admin.key() != constants::ADMIN_PUBKEY {
            return Err(ProgramError::InvalidInstructionData.into());
        }
        // inititalize a user account for new users
        ctx.accounts.user_account.authority = *ctx.accounts.authority.key;

        Ok(())
    }

    pub fn first_swap(ctx: Context<FirstSwap>, _swap_id: String) -> Result<()> {
        if ctx.accounts.user_account.swaps_count > 0 {
            return Err(ProgramError::InvalidInstructionData.into());
        }

        // initialize the first swap account for a new user
        ctx.accounts.swap_account.authority = *ctx.accounts.authority.key;
        ctx.accounts.user_account.swaps_count = 1;

        Ok(())
    }

    pub fn new_swap(ctx: Context<NewSwap>, _swap_id: String) -> Result<()> {
        if ctx.accounts.admin.key() != constants::ADMIN_PUBKEY {
            return Err(ProgramError::InvalidInstructionData.into());
        }

        // initialize a swap account
        ctx.accounts.new_swap_account.authority = *ctx.accounts.authority.key;
        ctx.accounts.user_account.swaps_count += 1;

        Ok(())
    }

    pub fn initiate_swap(
        ctx: Context<Swap>, 
        token: MizumiStable, 
        amount: u64, 
        fiat: MizumiFiat, 
        tx_kind: TransactionKind,
        _swap_id: String,
    ) -> Result<()> {
        if ctx.accounts.admin.key() != constants::ADMIN_PUBKEY {
            return Err(ProgramError::InvalidInstructionData.into());
        }
        ctx.accounts.swap_account.authority = *ctx.accounts.authority.key;
        ctx.accounts.swap_account.token = token;
        ctx.accounts.swap_account.settled = false;
        ctx.accounts.swap_account.amount_in = amount;
        ctx.accounts.swap_account.fiat = fiat;
        ctx.accounts.swap_account.tx_kind = tx_kind;
        ctx.accounts.swap_account.created_ts = ctx.accounts.clock.unix_timestamp;
        ctx.accounts.swap_account.bump = *ctx.bumps.get("swap_account").unwrap();

        let usdc_key = &ctx.accounts.usdc.key();
        let usdt_key = &ctx.accounts.usdt.key();

        let usdc_seeds = &[
            b"usdc-vault".as_ref(),
            usdc_key.as_ref(),
            &[*ctx.bumps.get("usdc_vault").unwrap()],
        ];
        let usdc_signer = [&usdc_seeds[..]];

        let usdt_seeds = &[
            b"usdt-vault".as_ref(),
            usdt_key.as_ref(),
            &[*ctx.bumps.get("usdt_vault").unwrap()]
        ];
        let usdt_signer = [&usdt_seeds[..]];

        // transfer token from sender -> PDA vault
        match token {
            MizumiStable::USDC => {
                match tx_kind {
                    TransactionKind::Onramp => {
                        
                        let transfer_ctx = CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.usdc_vault.to_account_info(),
                                to: ctx.accounts.authority_usdc.to_account_info(),
                                authority: ctx.accounts.usdc_vault.to_account_info(),
                            },
                            &usdc_signer,
                        );
                        token::transfer(transfer_ctx, amount)?;
                    },
                    TransactionKind::Offramp => {
                        let transfer_ctx = CpiContext::new(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.authority_usdc.to_account_info(),
                                to: ctx.accounts.usdc_vault.to_account_info(),
                                authority: ctx.accounts.authority.to_account_info()
                            }
                        );
                        token::transfer(transfer_ctx, amount)?;
                    }
                }
                
            },
            MizumiStable::USDT => {
                match tx_kind {
                    TransactionKind::Onramp => {
                        let transfer_ctx = CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.usdt_vault.to_account_info(),
                                to: ctx.accounts.authority_usdt.to_account_info(),
                                authority: ctx.accounts.usdt_vault.to_account_info(),
                            },
                            &usdt_signer
                        );
                        token::transfer(transfer_ctx, amount)?;
                    },
                    TransactionKind::Offramp => {
                        let transfer_ctx = CpiContext::new(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.authority_usdt.to_account_info(),
                                to: ctx.accounts.usdt_vault.to_account_info(),
                                authority: ctx.accounts.authority.to_account_info()
                            }
                        );
                        token::transfer(transfer_ctx, amount)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn complete_swap(
        ctx: Context<CompleteSwap>,
        settled: bool,
        settled_amount: u64,
        _swap_id: String
    ) -> Result<()> {
        if ctx.accounts.admin.key() != constants::ADMIN_PUBKEY {
            return Err(ProgramError::InvalidInstructionData.into());
        }
        ctx.accounts.swap_account.settled = settled;
        ctx.accounts.swap_account.settled_amount = settled_amount;
        ctx.accounts.swap_account.settled_ts = ctx.accounts.clock.unix_timestamp;
        ctx.accounts.user_account.total_swaps_value += settled_amount;

        Ok(())
    }
}

// supported Stablecoins
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, Copy)]
pub enum MizumiStable {
    USDC,
    USDT
}

// supported Fiat currencies
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, Copy)]
pub enum MizumiFiat {
    GHS,
    USD,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, Copy)]
pub enum TransactionKind {
    Onramp,
    Offramp,
}

#[account]
pub struct SwapAccount {
    pub authority: Pubkey,
    pub token: MizumiStable,
    pub settled: bool,
    pub amount_in: u64,
    pub fiat: MizumiFiat,
    pub tx_kind: TransactionKind,
    pub settled_amount: u64,
    pub created_ts: i64,
    pub settled_ts: i64,
    pub bump: u8,
}


#[account]
#[derive(Default)]
pub struct UserAccount {
    pub authority: Pubkey,
    pub swaps_count: u64,
    pub total_swaps_value: u64, 
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub usdc: Account<'info, Mint>,
    // account to hold usdc
    #[account(
        init,
        payer=payer,
        seeds=[b"usdc-vault", usdc.key().as_ref()],
        bump,
        token::mint = usdc,
        token::authority = usdc_vault
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    pub usdt: Account<'info, Mint>,
    //account to hold usdt
    #[account(
        init,
        payer=payer,
        seeds=[b"usdt-vault", usdt.key().as_ref()],
        bump,
        token::mint = usdt,
        token::authority = usdt_vault
    )]
    pub usdt_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct NewUser<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: AccountInfo<'info>,
    #[account(signer, mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8,
        seeds = [b"user-account", authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(swap_id: String)]
pub struct FirstSwap<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer, mut)]
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"user-account", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 1 + 1 + 8 + 1 + 1 + 1 + 1 + 8 + 8 + 8 + 1, 
        seeds = [b"swap-account", authority.key().as_ref(), swap_id.as_ref()], 
        bump
    )]
    pub swap_account: Account<'info, SwapAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(swap_id: String)]
pub struct NewSwap<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: AccountInfo<'info>,
    #[account(signer, mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"user-account", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 1 + 1 + 8 + 1 + 1 + 1 + 1 + 8 + 8 + 8 + 1, 
        seeds = [b"swap-account", authority.key().as_ref(), swap_id.as_ref()], 
        bump
    )]
    pub new_swap_account: Account<'info, SwapAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(token: MizumiStable, amount: u64, fiat: MizumiFiat, tx_kind: TransactionKind, swap_id: String)]
pub struct Swap<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: AccountInfo<'info>,
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub authority_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub authority_usdt: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"user-account", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(
        mut, 
        seeds = [b"swap-account", authority.key().as_ref(), swap_id.as_ref()], 
        bump,
        has_one = authority
    )]
    pub swap_account: Box<Account<'info, SwapAccount>>,
    pub usdc: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"usdc-vault", usdc.key().as_ref()],
        bump,
        token::mint = usdc,
        token::authority = usdc_vault
    )]
    pub usdc_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"usdt-vault", usdt.key().as_ref()],
        bump,
        token::mint = usdt,
        token::authority = usdt_vault
    )]
    pub usdt_vault: Box<Account<'info, TokenAccount>>,
    pub usdt: Box<Account<'info, Mint>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(settled: bool, settled_amount: u64, swap_id: String)]
pub struct CompleteSwap<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: AccountInfo<'info>,
    #[account(signer, mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"swap-account", authority.key().as_ref(), swap_id.as_ref()],
        bump,
        has_one = authority
    )]
    pub swap_account: Account<'info, SwapAccount>,
    #[account(
        mut,
        seeds = [b"user-account", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub user_account: Account<'info, UserAccount>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Debug)]
pub struct SwapData {
    pub token: MizumiStable,
    pub amount: u64,
    pub fiat: MizumiFiat,
    pub tx_kind: TransactionKind
}
