use anchor_lang::prelude::*;

const OPEN_POSITION_DISCRIMINATOR: [u8; 8] = [77, 184, 74, 214, 112, 86, 241, 199];


pub fn open_position<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, OpenPosition<'info>>,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity: u128,
    amount_max_a: u64,
    amount_max_b: u64,
    with_metadata: bool,
    option_base_flag: u8,
    base_flag: bool,
) -> Result<()> {
    // 1. Build openPosition instruction:
    // 1.1. Build accounts:
    let mut accounts = Vec::with_capacity(22);
    accounts.push(AccountMeta::new(ctx.accounts.payer.key(), true));  // 1
    accounts.push(AccountMeta::new_readonly(ctx.accounts.position_nft_owner.key(), false));  // 2
    accounts.push(AccountMeta::new(ctx.accounts.position_nft_mint.key(), true));  // 3
    accounts.push(AccountMeta::new(ctx.accounts.position_nft_account.key(), false));  // 4
    accounts.push(AccountMeta::new(ctx.accounts.metadata_account.key(), false));  // 5
    accounts.push(AccountMeta::new(ctx.accounts.pool_id.key(), false));  // 6
    accounts.push(AccountMeta::new(ctx.accounts.protocol_position.key(), false));  // 7
    accounts.push(AccountMeta::new(ctx.accounts.tick_array_lower.key(), false));  // 8
    accounts.push(AccountMeta::new(ctx.accounts.tick_array_upper.key(), false));  // 9
    accounts.push(AccountMeta::new(ctx.accounts.personal_position.key(), false));  // 10
    accounts.push(AccountMeta::new(ctx.accounts.owner_token_account_a.key(), false));  // 11
    accounts.push(AccountMeta::new(ctx.accounts.owner_token_account_b.key(), false));  // 12
    accounts.push(AccountMeta::new(ctx.accounts.token_vault_a.key(), false));  // 13
    accounts.push(AccountMeta::new(ctx.accounts.token_vault_b.key(), false));  // 14
    accounts.push(AccountMeta::new_readonly(ctx.accounts.rent_program.key(), false));  // 15
    accounts.push(AccountMeta::new_readonly(ctx.accounts.system_program.key(), false));  // 16
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_program.key(), false));  // 17
    accounts.push(AccountMeta::new_readonly(ctx.accounts.associated_token_program.key(), false));  // 18
    accounts.push(AccountMeta::new_readonly(ctx.accounts.metadata_program.key(), false));  // 19
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false));  // 20
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_mint_a.key(), false));  // 21
    accounts.push(AccountMeta::new_readonly(ctx.accounts.token_mint_b.key(), false));  // 22

    // 1.2. Build data:
    let mut data: Vec<u8> = Vec::with_capacity(32);
    data.extend_from_slice(&OPEN_POSITION_DISCRIMINATOR);
    data.extend_from_slice(&tick_lower_index.to_le_bytes());
    data.extend_from_slice(&tick_upper_index.to_le_bytes());
    data.extend_from_slice(&tick_array_lower_start_index.to_le_bytes());
    data.extend_from_slice(&tick_array_upper_start_index.to_le_bytes());
    data.extend_from_slice(&liquidity.to_le_bytes());
    data.extend_from_slice(&amount_max_a.to_le_bytes());
    data.extend_from_slice(&amount_max_b.to_le_bytes());
    data.extend_from_slice(&[with_metadata as u8]);
    data.extend_from_slice(&option_base_flag.to_le_bytes());
    data.extend_from_slice(&[base_flag as u8]);

    // 1.3. Build instruction:
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts,
        data,
    };

    // 2. Invoke instruction:
    let account_infos: &mut Vec<AccountInfo<'_>> = &mut ctx.accounts.to_account_infos();
    anchor_lang::solana_program::program::invoke(
        &ix,
        account_infos,
    ).map_err(Into::into)
}


#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    /// CHECK:
    pub payer: AccountInfo<'info>,
    /// CHECK:
    pub position_nft_owner: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub position_nft_mint: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub position_nft_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub metadata_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub pool_id: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub protocol_position: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub tick_array_lower: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub tick_array_upper: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub personal_position: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub owner_token_account_a: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub owner_token_account_b: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub token_vault_a: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub token_vault_b: AccountInfo<'info>,
    /// CHECK:
    pub rent_program: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
    /// CHECK:
    pub token_program: AccountInfo<'info>,
    /// CHECK:
    pub associated_token_program: AccountInfo<'info>,
    /// CHECK:
    pub metadata_program: AccountInfo<'info>,
    /// CHECK:
    pub token_2022_program: AccountInfo<'info>,
    /// CHECK:
    pub token_mint_a: AccountInfo<'info>,
    /// CHECK:
    pub token_mint_b: AccountInfo<'info>,
}
