use anchor_lang::prelude::*;

const CREATE_POOL_DISCRIMINATOR: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];


pub fn create_pool<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreatePool<'info>>,
    sqrt_price_x64: u128,
) -> Result<()> {
    // 1. Build createPool instruction:
    // 1.1. Build accounts:
    let mut accounts = Vec::with_capacity(13);
    accounts.push(AccountMeta::new(ctx.accounts.pool_creator.key(), true));  // 1
    accounts.push(AccountMeta::new_readonly(ctx.accounts.amm_config_id.key(), false));  // 2
    accounts.push(AccountMeta::new(ctx.accounts.pool_id.key(), false));  // 3
    accounts.push(AccountMeta::new_readonly(ctx.accounts.mint_a.key(), false));  // 4
    accounts.push(AccountMeta::new_readonly(ctx.accounts.mint_b.key(), false));  // 5
    accounts.push(AccountMeta::new(ctx.accounts.mint_vault_a.key(), false));  // 6
    accounts.push(AccountMeta::new(ctx.accounts.mint_vault_b.key(), false));  // 7
    accounts.push(AccountMeta::new(ctx.accounts.observation_id.key(), false));  // 8
    accounts.push(AccountMeta::new(ctx.accounts.ex_tick_array_bitmap.key(), false));  // 9
    accounts.push(AccountMeta::new_readonly(ctx.accounts.mint_program_id_a.key(), false));  // 10
    accounts.push(AccountMeta::new_readonly(ctx.accounts.mint_program_id_b.key(), false));  // 11
    accounts.push(AccountMeta::new_readonly(ctx.accounts.system_program.key(), false));  // 12
    accounts.push(AccountMeta::new_readonly(ctx.accounts.rent_program.key(), false));  // 13

    // 1.2. Build data:
    let mut data: Vec<u8> = Vec::with_capacity(32);
    let start_time: u64 = 0;
    data.extend_from_slice(&CREATE_POOL_DISCRIMINATOR);
    data.extend_from_slice(&sqrt_price_x64.to_le_bytes());
    data.extend_from_slice(&start_time.to_le_bytes());

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
pub struct CreatePool<'info> {
    #[account(mut)]
    /// CHECK:
    pub pool_creator: AccountInfo<'info>,
    /// CHECK:
    pub amm_config_id: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub pool_id: AccountInfo<'info>,
    /// CHECK:
    pub mint_a: AccountInfo<'info>,
    /// CHECK:
    pub mint_b: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub mint_vault_a: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub mint_vault_b: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub observation_id: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    pub ex_tick_array_bitmap: AccountInfo<'info>,
    /// CHECK:
    pub mint_program_id_a: AccountInfo<'info>,
    /// CHECK:
    pub mint_program_id_b: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
    /// CHECK:
    pub rent_program: AccountInfo<'info>,
}
