mod raydium;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
use anchor_spl::token::{self, CloseAccount, InitializeAccount, Mint, MintTo, SetAuthority, Token, TokenAccount};
use anchor_spl::token_2022::Token2022;
use anchor_lang::system_program::{self, CreateAccount, Transfer};
use mpl_token_metadata::types::DataV2;
use raydium::*;
use solana_program::{pubkey, pubkey::Pubkey};
use spl_token::instruction::AuthorityType;

// CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK (mainnet)
// devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH  (devnet)
pub const CLMM_PROGRAM_ID: Pubkey = pubkey!("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH");
// 8iv4cFhk7s7SCysedKgBC2haqEfvx8HmJ75PP5pmjn1b (mainnet)
// 5TLHYfEXHmNCctnCurLiPzLkwW46gxECePQ9BTwGYfvD (devnet)
pub const HARVESTER: Pubkey = pubkey!("5TLHYfEXHmNCctnCurLiPzLkwW46gxECePQ9BTwGYfvD");

const TOKEN_ACCOUNT_SPACE: u64 = 165;
const MIN_LIQUIDITY_PCT_OF_SUPPLY: u64 = 95;  // At least 95% of total supply must be provided to liquidity pool
const FEE_AMOUNT: u64 = 400_000_000;  // 0.4 SOL

declare_id!("RoCKeTyq1Wf8Co6e2x1RQCC6uxRkZrQ3mDJUEoQryhh");

#[program]
mod token_deployer {
    use super::*;
    
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_token(
        ctx: Context<CreateTokenAccounts>,
        amount: u64,
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        // Mint supply to payer:
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                },
            ),
            amount,
        )?;
        // Add metadata:
        create_metadata_accounts_v3(
			CpiContext::new(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.token_metadata_account.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent_program.to_account_info(),
                },
			),
			DataV2 {
                name: name,
                symbol: symbol,
                uri: uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
			},
			false,  // Is mutable
			true,   // Update authority is signer
			None,   // Collection details
	    )?;
        // Revoke mint authority:
        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    current_authority: ctx.accounts.payer.to_account_info(),
                    account_or_mint: ctx.accounts.token_mint.to_account_info(),
                },
            ),
            AuthorityType::MintTokens,
            None,
        )?;
        Ok(())
    }

    pub fn add_to_raydium(
        ctx: Context<AddToRaydiumAccounts>,
        sqrt_price_x64: u128,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity: u128,
        token_amount_max: u64,
        wsol_amount_max: u64,
    ) -> Result<()> {
        let token_balance_at_start = ctx.accounts.token_account.amount;
        // Create, initialize and fund temporary WSOL account:
        anchor_lang::system_program::create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.wsol_account.to_account_info(),
                },
            ),
            ctx.accounts.rent_program.minimum_balance(TOKEN_ACCOUNT_SPACE as usize) + wsol_amount_max,
            TOKEN_ACCOUNT_SPACE,
            &ctx.accounts.token_program.key(),
        )?;
        token::initialize_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InitializeAccount {
                    account: ctx.accounts.wsol_account.to_account_info(),
                    mint: ctx.accounts.wsol_mint.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                    rent: ctx.accounts.rent_program.to_account_info(),
                }
            )
        )?;
        // Create pool:
        raydium::create_pool(
            CpiContext::new(
                ctx.accounts.clmm_program.to_account_info(),
                raydium::CreatePool {
                    pool_creator: ctx.accounts.payer.to_account_info(),
                    amm_config_id: ctx.accounts.amm_config_id.to_account_info(),
                    pool_id: ctx.accounts.pool_id.to_account_info(),
                    mint_a: ctx.accounts.token_mint.to_account_info(),
                    mint_b: ctx.accounts.wsol_mint.to_account_info(),
                    mint_vault_a: ctx.accounts.token_vault.to_account_info(),
                    mint_vault_b: ctx.accounts.wsol_vault.to_account_info(),
                    observation_id: ctx.accounts.observation_id.to_account_info(),
                    ex_tick_array_bitmap: ctx.accounts.ex_tick_array_bitmap.to_account_info(),
                    mint_program_id_a: ctx.accounts.token_program.to_account_info(),
                    mint_program_id_b: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent_program: ctx.accounts.rent_program.to_account_info(),
                },
            ),
            sqrt_price_x64,
        )?;
        // Open position:
        raydium::open_position(
            CpiContext::new(
                ctx.accounts.clmm_program.to_account_info(),
                raydium::OpenPosition {
                    payer: ctx.accounts.payer.to_account_info(),
                    position_nft_owner: ctx.accounts.position_nft_owner_pda.to_account_info(),
                    position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
                    position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
                    metadata_account: ctx.accounts.metadata_account.to_account_info(),
                    pool_id: ctx.accounts.pool_id.to_account_info(),
                    protocol_position: ctx.accounts.protocol_position.to_account_info(),
                    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                    personal_position: ctx.accounts.personal_position.to_account_info(),
                    owner_token_account_a: ctx.accounts.token_account.to_account_info(),
                    owner_token_account_b: ctx.accounts.wsol_account.to_account_info(),
                    token_vault_a: ctx.accounts.token_vault.to_account_info(),
                    token_vault_b: ctx.accounts.wsol_vault.to_account_info(),
                    rent_program: ctx.accounts.rent_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                    metadata_program: ctx.accounts.metadata_program.to_account_info(),
                    token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
                    token_mint_a: ctx.accounts.token_mint.to_account_info(),
                    token_mint_b: ctx.accounts.wsol_mint.to_account_info(),
                },
            ),
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            liquidity,
            token_amount_max,
            wsol_amount_max,
            true,   // with_metadata
            0,      // option_base_flag
            false,  // base_flag
        )?;
        // Close temporary SOL account:
        token::close_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                CloseAccount {
                    account: ctx.accounts.wsol_account.to_account_info(),
                    destination: ctx.accounts.payer.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            )
        )?;
        // Fee:
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.fee_receiver.to_account_info(),
                },
            ),
            FEE_AMOUNT
        )?;
        // Require that at least 95% of total supply was provided into liquidity
        ctx.accounts.token_account.reload()?;
        let token_amount_spent: u64 = token_balance_at_start - ctx.accounts.token_account.amount;
        require!(token_amount_spent >= ctx.accounts.token_mint.supply / 100 * MIN_LIQUIDITY_PCT_OF_SUPPLY, CustomError::InsufficientLiquidityProvided);
        Ok(())
    }

    pub fn harvest_fees<'info>(ctx: Context<'_, '_, '_, 'info, HarvestAccounts<'info>>) -> Result<()> {
        // Create, initialize and fund temporary WSOL account:
        anchor_lang::system_program::create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                CreateAccount {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.wsol_recipient_account.to_account_info(),
                },
            ),
            ctx.accounts.rent_program.minimum_balance(TOKEN_ACCOUNT_SPACE as usize),
            TOKEN_ACCOUNT_SPACE,
            &ctx.accounts.token_program.key(),
        )?;
        token::initialize_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InitializeAccount {
                    account: ctx.accounts.wsol_recipient_account.to_account_info(),
                    mint: ctx.accounts.wsol_mint.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                    rent: ctx.accounts.rent_program.to_account_info(),
                }
            )
        )?;
        // Harvest:
        let harvest_accounts: DecreaseLiquidityV2<'_> = raydium::DecreaseLiquidityV2 {
            position_nft_owner: ctx.accounts.position_nft_owner_pda.to_account_info(),
            position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
            personal_position: ctx.accounts.personal_position.to_account_info(),
            pool_state: ctx.accounts.pool_id.to_account_info(),
            protocol_position: ctx.accounts.protocol_position.to_account_info(),
            token_vault_0: ctx.accounts.token_vault.to_account_info(),
            token_vault_1: ctx.accounts.wsol_vault.to_account_info(),
            tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
            tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
            recipient_token_account_0: ctx.accounts.token_recipient_account.to_account_info(),
            recipient_token_account_1: ctx.accounts.wsol_recipient_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            memo_program: ctx.accounts.memo_program.to_account_info(),
            vault_0_mint: ctx.accounts.token_mint.to_account_info(),
            vault_1_mint: ctx.accounts.wsol_mint.to_account_info(),
        };

        let bump = ctx.bumps.position_nft_owner_pda;
        let seeds = &[b"position_nft_owner_pda".as_ref(), &[bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx: CpiContext<'_, '_, '_, '_, DecreaseLiquidityV2<'_>> = CpiContext::new_with_signer(
            ctx.accounts.clmm_program.to_account_info(),
            harvest_accounts,
            signer_seeds,
        ).with_remaining_accounts(ctx.remaining_accounts.to_vec());

        raydium::harvest(cpi_ctx)?;

        // Close temporary WSOL account:
        token::close_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                CloseAccount {
                    account: ctx.accounts.wsol_recipient_account.to_account_info(),
                    destination: ctx.accounts.payer.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            )
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        payer=payer,
        seeds=[b"position_nft_owner_pda"],
        bump,
        space=8
    )]
    /// CHECK: only used as a signing PDA
    position_nft_owner_pda: AccountInfo<'info>,

    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent_program: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateTokenAccounts<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = payer,
    )]
    token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = token_mint, 
        associated_token::authority = payer
    )]
    token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: checked in CPI
    token_metadata_account: UncheckedAccount<'info>,
  
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    metadata_program: Program<'info, Metadata>,
    rent_program: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddToRaydiumAccounts<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    token_mint: Account<'info, Mint>,
    #[account(mut)]
    token_account: Account<'info, TokenAccount>,

    /// CHECK: checked in CPI
    amm_config_id: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    pool_id: UncheckedAccount<'info>,

    /// CHECK: checked in CPI
    wsol_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: checked in CPI
    token_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    wsol_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    observation_id: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    ex_tick_array_bitmap: UncheckedAccount<'info>,

    #[account(mut)]
    position_nft_mint: Signer<'info>,
    #[account(mut, seeds=[b"position_nft_owner_pda"], bump)]
    /// CHECK: only used as a signing PDA
    position_nft_owner_pda: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    position_nft_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    metadata_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    protocol_position: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    tick_array_lower: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    tick_array_upper: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    personal_position: AccountInfo<'info>,
    #[account(mut)]
    wsol_account: Signer<'info>,

    #[account(mut, address = HARVESTER)]
    /// CHECK: checked by address
    fee_receiver: UncheckedAccount<'info>,
    
    #[account(address = CLMM_PROGRAM_ID)]
    /// CHECK: checked by address
    clmm_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_2022_program: Program<'info, Token2022>,
    metadata_program: Program<'info, Metadata>,
    rent_program: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct HarvestAccounts<'info> {
    #[account(mut, seeds=[b"position_nft_owner_pda"], bump)]
    /// CHECK: only used as a signing PDA
    position_nft_owner_pda: AccountInfo<'info>,
    #[account(mut,
        associated_token::mint = position_nft_mint,
        associated_token::authority = position_nft_owner_pda,
    )]
    position_nft_account: Box<Account<'info, TokenAccount>>,
    position_nft_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    /// CHECK: checked in CPI
    personal_position: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    pool_id: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    protocol_position: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    token_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    wsol_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    tick_array_lower: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in CPI
    tick_array_upper: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer=payer,
        associated_token::mint=token_mint,
        associated_token::authority=payer,
    )]
    token_recipient_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    wsol_recipient_account: Signer<'info>,
    
    token_mint: Account<'info, Mint>,
    wsol_mint: Account<'info, Mint>,

    #[account(mut, address = HARVESTER)]
    payer: Signer<'info>,
    
    #[account(address = CLMM_PROGRAM_ID)]
    /// CHECK: checked by address
    clmm_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_2022_program: Program<'info, Token2022>,
    /// CHECK: checked in CPI
    memo_program: UncheckedAccount<'info>,
    rent_program: Sysvar<'info, Rent>,
}

#[error_code]
pub enum CustomError {
    #[msg("At least 95% of token supply must be provided to liquidity pool")]
    InsufficientLiquidityProvided
}