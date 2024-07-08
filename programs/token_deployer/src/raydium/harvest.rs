use anchor_lang::prelude::*;

const DECREASE_LIQUIDITY_V2_DISCRIMINATOR: [u8; 8] = [58, 127, 188, 62, 79, 82, 196, 96];


pub fn harvest<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, DecreaseLiquidityV2<'info>>,
) -> Result<()> {
    // 1. Build decreaseLiquidityV2 instruction:
    // 1.1. Build accounts:
    let mut accounts = Vec::with_capacity(16 + ctx.remaining_accounts.len());
    accounts.push(AccountMeta::new_readonly(ctx.accounts.position_nft_owner.key(), true));  // 1
    accounts.push(AccountMeta::new_readonly(ctx.accounts.position_nft_account.key(), false));  // 2
    accounts.push(AccountMeta::new(ctx.accounts.personal_position.key(), false));  // 3
    accounts.push(AccountMeta::new(ctx.accounts.pool_state.key(), false));  // 4
    accounts.push(AccountMeta::new(ctx.accounts.protocol_position.key(), false));  // 5
    accounts.push(AccountMeta::new(ctx.accounts.token_vault_0.key(), false));  // 6
    accounts.push(AccountMeta::new(ctx.accounts.token_vault_1.key(), false));  // 7
    accounts.push(AccountMeta::new(ctx.accounts.tick_array_lower.key(), false));  // 8
    accounts.push(AccountMeta::new(ctx.accounts.tick_array_upper.key(), false));  // 9
    accounts.push(AccountMeta::new(ctx.accounts.recipient_token_account_0.key(), false));  // 10
    accounts.push(AccountMeta::new(ctx.accounts.recipient_token_account_1.key(), false));  // 11
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_program.key(), false));  // 12
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false));  // 13
    accounts.push(AccountMeta::new_readonly(ctx.accounts.memo_program.key(), false));  // 14
    accounts.push(AccountMeta::new_readonly(ctx.accounts.vault_0_mint.key(), false));  // 15
    accounts.push(AccountMeta::new_readonly(ctx.accounts.vault_1_mint.key(), false));  // 16
    // Remaining accounts:
    let remaining_accounts = &ctx.remaining_accounts;
    // The first element is optional exTickArrayBitmap:
    // { pubkey: exTickArrayBitmap, isSigner: false, isWritable: true }
    let has_ex_tick_array_bitmap = remaining_accounts.len() % 3 == 1;
    if has_ex_tick_array_bitmap {
        accounts.push(AccountMeta::new(remaining_accounts[0].key(), false));
    }
    // The rest are triplets of reward info accounts:
    // { pubkey: i.poolRewardVault, isSigner: false, isWritable: true }
    // { pubkey: i.ownerRewardVault, isSigner: false, isWritable: true }
    // { pubkey: i.rewardMint, isSigner: false, isWritable: false }
    let start_index = if has_ex_tick_array_bitmap { 1 } else { 0 };
    accounts.extend(
        remaining_accounts[start_index..].iter().enumerate().map(|(i, a)| {
            if i % 3 == 2 {
                AccountMeta::new_readonly(a.key(), false)
            } else {
                AccountMeta::new(a.key(), false)
            }
        }),
    );

    // 1.2. Build data:
    let mut data: Vec<u8> = Vec::with_capacity(40);
    data.extend_from_slice(&DECREASE_LIQUIDITY_V2_DISCRIMINATOR);
    data.resize(40, 0);  // adding 32 zero-bytes repr struct {liquidity: u128 = 0, amount_0_max: u64 = 0, amount_1_max: u64 = 0}

    // 1.3. Build instruction:
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts,
        data,
    };

    // 2. Invoke instruction:
    let account_infos = &mut ctx.accounts.to_account_infos();
    account_infos.extend_from_slice(&ctx.remaining_accounts);
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        account_infos,
        ctx.signer_seeds,
    ).map_err(Into::into)
}


#[derive(Accounts)]
pub struct DecreaseLiquidityV2<'info> {
    /// CHECK:
    pub position_nft_owner: AccountInfo<'info>,
    /// CHECK:
    pub position_nft_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub personal_position: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub pool_state: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub protocol_position: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub token_vault_0: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub token_vault_1: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub tick_array_lower: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub tick_array_upper: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub recipient_token_account_0: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub recipient_token_account_1: AccountInfo<'info>,
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    /// CHECK:
    pub token_2022_program: AccountInfo<'info>,
    /// CHECK:
    pub memo_program: AccountInfo<'info>,
    /// CHECK:
    pub vault_0_mint: AccountInfo<'info>,
    /// CHECK:
    pub vault_1_mint: AccountInfo<'info>,
}

// #[derive(Accounts)]
// pub struct DecreaseLiquidityV2<'info> {
//     /// The position owner or delegated authority
//     pub nft_owner: Signer<'info>,

//     /// The token account for the tokenized position
//     #[account(
//         constraint = nft_account.mint == personal_position.nft_mint,
//         token::token_program = token_program,
//     )]
//     pub nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

//     /// Decrease liquidity for this position
//     #[account(mut, constraint = personal_position.pool_id == pool_state.key())]
//     pub personal_position: Box<Account<'info, PersonalPositionState>>,

//     #[account(mut)]
//     pub pool_state: AccountLoader<'info, PoolState>,

//     #[account(
//         mut,
//         seeds = [
//             POSITION_SEED.as_bytes(),
//             pool_state.key().as_ref(),
//             &personal_position.tick_lower_index.to_be_bytes(),
//             &personal_position.tick_upper_index.to_be_bytes(),
//         ],
//         bump,
//         constraint = protocol_position.pool_id == pool_state.key(),
//     )]
//     pub protocol_position: Box<Account<'info, ProtocolPositionState>>,

//     /// Token_0 vault
//     #[account(
//         mut,
//         constraint = token_vault_0.key() == pool_state.load()?.token_vault_0
//     )]
//     pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

//     /// Token_1 vault
//     #[account(
//         mut,
//         constraint = token_vault_1.key() == pool_state.load()?.token_vault_1
//     )]
//     pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

//     /// Stores init state for the lower tick
//     #[account(mut, constraint = tick_array_lower.load()?.pool_id == pool_state.key())]
//     pub tick_array_lower: AccountLoader<'info, TickArrayState>,

//     /// Stores init state for the upper tick
//     #[account(mut, constraint = tick_array_upper.load()?.pool_id == pool_state.key())]
//     pub tick_array_upper: AccountLoader<'info, TickArrayState>,

//     /// The destination token account for receive amount_0
//     #[account(
//         mut,
//         token::mint = token_vault_0.mint
//     )]
//     pub recipient_token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,

//     /// The destination token account for receive amount_1
//     #[account(
//         mut,
//         token::mint = token_vault_1.mint
//     )]
//     pub recipient_token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,

//     /// SPL program to transfer out tokens
//     pub token_program: Program<'info, Token>,
//     /// Token program 2022
//     pub token_program_2022: Program<'info, Token2022>,

//     /// memo program
//     /// CHECK:
//     #[account(
//         address = spl_memo::id()
//     )]
//     pub memo_program: UncheckedAccount<'info>,

//     /// The mint of token vault 0
//     #[account(
//         address = token_vault_0.mint
//     )]
//     pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

//     /// The mint of token vault 1
//     #[account(
//         address = token_vault_1.mint
//     )]
//     pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,
//     // remaining account
//     // #[account(
//     //     seeds = [
//     //         POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
//     //         pool_state.key().as_ref(),
//     //     ],
//     //     bump
//     // )]
//     // pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,
// }

// const remainingAccounts = [
//     ...(exTickArrayBitmap ? [{ pubkey: exTickArrayBitmap, isSigner: false, isWritable: true }] : []),
//     ...rewardAccounts
//       .map((i) => [
//         { pubkey: i.poolRewardVault, isSigner: false, isWritable: true },
//         { pubkey: i.ownerRewardVault, isSigner: false, isWritable: true },
//         { pubkey: i.rewardMint, isSigner: false, isWritable: false },
//       ])
//       .flat(),
//   ]