use std::mem::size_of;
use anchor_lang::prelude::*;
use solana_program::account_info::AccountInfo;
use anchor_spl::{token, associated_token};

pub mod pyth;
use pyth::PriceFeed;
use pyth::AdminConfig;

mod error;
use error::ErrorCode;

declare_id!("3o5VrciviJWYnB39NNfmsWTNqSa4aooXKUf5AzZMdWXu");

#[program]
pub mod synthius_v0 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, config: AdminConfig) -> Result<()> {
        ctx.accounts.config.set_inner(config);
        msg!("Initialize accounts");
        Ok(())
    }

    pub fn dummy_token(ctx: Context<DummyToken>, amount: u64) -> Result<()> {
        let cpi_accounts = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.collateral_token_mint.to_account_info(),
                to: ctx.accounts.collateral_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },);
        token::mint_to(cpi_accounts, amount)?;
        msg!("Dummy token");
        Ok(())
    }

    pub fn buy_long(ctx: Context<BuyLong>, amount: u64) -> Result<()> {
        ctx.accounts.vault.amount += amount;
        ctx.accounts.vault.position = Position::Long;
        let vault = &mut (ctx.accounts.vault);
        let price_feed = &ctx.accounts.pyth_loan_account;
        let current_timestamp = Clock::get()?.unix_timestamp;
        let stock_price = price_feed
            .get_price_no_older_than(current_timestamp, 60)
            .ok_or(error!(ErrorCode::PythOffline))?;
        vault.price_entered = stock_price.price;
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.collateral_token_account.to_account_info(),
                to: ctx.accounts.vault_wallet.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::transfer(cpi_context, amount)?;
        token::mint_to(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.long_token_mint.to_account_info(),
                to: ctx.accounts.long_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
        ), 1)?;
        msg!("Buy long");
        Ok(())
    }

    pub  fn sell_long(ctx: Context<SellLong>, bump: u8, signer: Pubkey) -> Result<()> {
        let signer_pubkey = signer.key();
        let signer = signer_pubkey.as_ref();
        let seeds =  &[&[b"vault", signer , anchor_lang::__private::bytemuck::bytes_of(&bump)][..]];
        let collateral = ctx.accounts.vault.amount;
        let price_entered = ctx.accounts.vault.price_entered;
        let price_feed = &ctx.accounts.pyth_loan_account;
        let current_timestamp = Clock::get()?.unix_timestamp;
        let stock_price = price_feed
            .get_price_no_older_than(current_timestamp, 60)
            .ok_or(error!(ErrorCode::PythOffline))?;
        let remaining_collateral: i64;
        if ctx.accounts.vault.position != Position::Long {
            return Err(error!(ErrorCode::InvalidArgument));
        }
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.long_token_mint.to_account_info(),
                    from: ctx.accounts.long_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                }
            ),1)?;
        if stock_price.price > price_entered {
            let profit = stock_price.price - price_entered;
            remaining_collateral = (collateral as i64) + profit;
            msg!("You made a profit of {}!", remaining_collateral);
        } else {
            let loss = price_entered - stock_price.price;
            remaining_collateral = (collateral as i64) - loss;
            msg!("You made a loss of {}!", remaining_collateral);
        }

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_wallet.to_account_info(),
                to: ctx.accounts.collateral_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            seeds,
        );
        token::transfer(cpi_context, remaining_collateral as u64)?;
        Ok(())

    }

    pub fn buy_short(ctx: Context<BuyShort>, amount: u64) -> Result<()> {
        ctx.accounts.vault.amount += amount;
        ctx.accounts.vault.position = Position::Short;
        let vault = &mut (ctx.accounts.vault);
        let price_feed = &ctx.accounts.pyth_loan_account;
        let current_timestamp = Clock::get()?.unix_timestamp;
        let stock_price = price_feed
            .get_price_no_older_than(current_timestamp, 60)
            .ok_or(error!(ErrorCode::PythOffline))?;
        vault.price_entered = stock_price.price;
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.collateral_token_account.to_account_info(),
                to: ctx.accounts.vault_wallet.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::transfer(cpi_context, amount)?;
        token::mint_to(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.short_token_mint.to_account_info(),
                to: ctx.accounts.short_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
        ), 1)?;
        msg!("Buy short");
        Ok(())
    }

    pub fn sell_short(ctx: Context<SellShort>, bump: u8, signer: Pubkey) -> Result<()> {
        let signer_pubkey = signer.key();
        let signer = signer_pubkey.as_ref();
        let seeds =  &[&[b"vault", signer , anchor_lang::__private::bytemuck::bytes_of(&bump)][..]];
        let collateral = ctx.accounts.vault.amount;
        let price_entered = ctx.accounts.vault.price_entered;
        let price_feed = &ctx.accounts.pyth_loan_account;
        let current_timestamp = Clock::get()?.unix_timestamp;
        let stock_price = price_feed
            .get_price_no_older_than(current_timestamp, 60)
            .ok_or(error!(ErrorCode::PythOffline))?;
        let remaining_collateral: i64;
        if ctx.accounts.vault.position != Position::Short {
            return Err(error!(ErrorCode::InvalidArgument));
        }
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.short_token_mint.to_account_info(),
                    from: ctx.accounts.short_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                }
            ),1)?;
        if stock_price.price < price_entered {
            let profit = price_entered - stock_price.price;
            remaining_collateral = (collateral as i64) + profit;
            msg!("You made a profit of {}!", remaining_collateral);
        } else {
            let loss = stock_price.price - price_entered;
            remaining_collateral = (collateral as i64) - loss;
            msg!("You made a loss of {}!", remaining_collateral);
        }
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_wallet.to_account_info(),
                to: ctx.accounts.collateral_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            seeds,
        );
        token::transfer(cpi_context, remaining_collateral as u64)?;
        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount: u64) -> Result<()> {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.collateral_token_account.to_account_info(),
                to: ctx.accounts.vault_wallet.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::transfer(cpi_context, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(address = *program_id @ ErrorCode::Unauthorized)]
    pub program: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, payer = payer, space = 8 + size_of::<AdminConfig>())]
    pub config: Account<'info, AdminConfig>,
    pub system_program: Program<'info, System>,
    #[account(init_if_needed, payer = payer, space = 8 + size_of::<Vault>(), seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
}

#[derive(Accounts)]
pub struct DummyToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(init, payer = payer, mint::decimals = 9, mint::authority = mint_authority)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    pub mint_authority: SystemAccount<'info>,
    #[account(init, payer = payer, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
}

#[derive(Accounts)]
pub struct BuyLong<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: Account<'info, PriceFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(init, payer = payer, mint::decimals = 9, mint::authority = mint_authority)]
    pub long_token_mint: Account<'info, token::Mint>,
    pub mint_authority: SystemAccount<'info>,
    #[account(init_if_needed, payer = payer, associated_token::mint = long_token_mint, associated_token::authority = payer)]
    pub long_token_account: Account<'info, token::TokenAccount>,
    #[account(mut, seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    #[account(mut, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
    #[account(init_if_needed,
        payer = payer,
        token::mint = collateral_token_mint,
        token::authority = vault,
        seeds = [b"vaultWallet".as_ref(), payer.key.as_ref()],bump
    )]
    pub vault_wallet: Account<'info, token::TokenAccount>,
}

#[derive(Accounts)]
pub struct SellLong<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: Account<'info, PriceFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(mut, seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub long_token_mint: Account<'info, token::Mint>,
    pub mint_authority: SystemAccount<'info>,
    #[account(mut, associated_token::mint = long_token_mint, associated_token::authority = payer)]
    pub long_token_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    #[account(mut, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
    #[account(mut,
        token::mint = collateral_token_mint,
        token::authority = vault,
        seeds = [b"vaultWallet".as_ref(), payer.key.as_ref()],bump
    )]
    pub vault_wallet: Account<'info, token::TokenAccount>,
}

#[derive(Accounts)]
pub struct BuyShort<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: Account<'info, PriceFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(init, payer = payer, mint::decimals = 9, mint::authority = mint_authority)]
    pub short_token_mint: Account<'info, token::Mint>,
    pub mint_authority: SystemAccount<'info>,
    #[account(init_if_needed, payer = payer, associated_token::mint = short_token_mint, associated_token::authority = payer)]
    pub short_token_account: Account<'info, token::TokenAccount>,
    #[account(mut, seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    #[account(mut, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
    #[account(init_if_needed,
        payer = payer,
        token::mint = collateral_token_mint,
        token::authority = vault,
        seeds = [b"vault".as_ref(), payer.key.as_ref()],bump
    )]
    pub vault_wallet: Account<'info, token::TokenAccount>,
}

#[derive(Accounts)]
pub struct SellShort<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: Account<'info, PriceFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(mut, seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub short_token_mint: Account<'info, token::Mint>,
    pub mint_authority: SystemAccount<'info>,
    #[account(mut, associated_token::mint = short_token_mint, associated_token::authority = payer)]
    pub short_token_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    #[account(mut, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
    #[account(mut,
        token::mint = collateral_token_mint,
        token::authority = vault,
        seeds = [b"vault".as_ref(), payer.key.as_ref()],bump
    )]
    pub vault_wallet: Account<'info, token::TokenAccount>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    #[account(mut)]
    pub collateral_token_mint: Account<'info, token::Mint>,
    #[account(mut, associated_token::mint = collateral_token_mint, associated_token::authority = payer)]
    pub collateral_token_account: Account<'info, token::TokenAccount>,
    #[account(mut, seeds = [b"vault".as_ref(), payer.key.as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut,
        token::mint = collateral_token_mint,
        token::authority = vault,
        seeds = [b"vault".as_ref(), payer.key.as_ref()],bump
    )]
    pub vault_wallet: Account<'info, token::TokenAccount>
}

#[account]
pub struct Vault {
    pub amount: u64,
    pub price_entered: i64,
    pub position: Position,
    pub collateral_locked: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum Position {
    Long,
    Short,
}