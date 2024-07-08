# Rocket Launcher program for Solana

Deployed at [RoCKeTyq1Wf8Co6e2x1RQCC6uxRkZrQ3mDJUEoQryhh](https://solscan.io/account/RoCKeTyq1Wf8Co6e2x1RQCC6uxRkZrQ3mDJUEoQryhh)

This contract implements three methods:
* `create_token`
* `add_to_raydium`
* `harvest_fees`

## `create_token`
Creates SPL token and mints supply to the payer

Args:
* `amount` - token amount to mint to the payer
* `name` - token name
* `symbol` - token symbol
* `uri` - URI pointing to token metadata JSON file

Description:
* Creates an SPL token and mints `amount` to payer
* Creates Metaplex metadata account with `name`, `symbol`, and `uri`
* Revokes mint authority

Details:
* Metadata URI points to a JSON file with information about token, including both on-chain (name, symbol) and off-chain (description, image URI)
* Token is created with decimals=6 and with freeze authority revoked

## `add_to_raydium`
Creates TOKEN/WSOL Raydium pool and opens liquidity position

Args:
* `sqrt_price_x64` - initial token price
* `tick_lower_index` - lower price tick of a liquidity position
* `tick_upper_index` - upper price tick of a liquidity position
* `tick_array_lower_start_index` - tick array lower index (derived from `tick_lower_index`)
* `tick_array_upper_start_index` - tick array upper index (derived from `tick_upper_index`)
* `liquidity` - liquidity parameter for a liquidity position
* `token_amount_max` - maximum amount of tokens to provide for a liquidity position
* `wsol_amount_max` - maximum amount of WSOL to provide for a liquidity position

Description:
* Creates TOKEN/WSOL 1% Raydium CLMM pool with initial price of `sqrt_price_x64`
* Opens liquidity position with parameters above
* Fixed fee (0.4 SOL) is transferred from the payer

Details:
* At least 95% of total token supply must be provided to liquidity position
* NFT representing liquidity position is minted to the program's derived address (to avoid rugging)
* Observation account must be created within the same transaction before instruction execution

## `harvest_fees`
Harvests fees associated with liquidity position

Details:
* Harvests both accumulated token and WSOL fees
* Only able to harvest fees from a single position (`position_nft_mint` must be provided in accounts)
