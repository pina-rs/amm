# Instruction reference

Every instruction's data is a one-byte discriminator followed by its fields, packed little-endian with no padding. Accounts are listed in order; `w` is writable and `s` is signer. The generated clients build these lists for you.

| #   | Instruction                                                          | Signer            | Summary                                    |
| --- | -------------------------------------------------------------------- | ----------------- | ------------------------------------------ |
| 0   | [`CreateConfig`](#createconfig)                                      | upgrade authority | Create a fee tier                          |
| 1   | [`UpdateConfig`](#updateconfig)                                      | tier authority    | Change a tier's authority and future rates |
| 2   | [`CreatePool`](#createpool)                                          | depositor         | Create a pool and make the first deposit   |
| 3   | [`Deposit`](#deposit-and-withdraw)                                   | LP                | Add liquidity proportionally               |
| 4   | [`Withdraw`](#deposit-and-withdraw)                                  | LP                | Remove liquidity proportionally            |
| 5   | [`SwapExactIn`](#swapexactin-and-swapexactout)                       | trader            | Sell an exact amount                       |
| 6   | [`SwapExactOut`](#swapexactin-and-swapexactout)                      | trader            | Buy an exact amount                        |
| 7   | [`CollectProtocolFees`](#collectprotocolfees-and-collectcreatorfees) | tier authority    | Pay out protocol fees                      |
| 8   | [`CollectCreatorFees`](#collectprotocolfees-and-collectcreatorfees)  | pool creator      | Pay out creator fees                       |
| 9   | [`SetPoolCreator`](#setpoolcreator)                                  | pool creator      | Hand over creator-fee rights               |
| 255 | `Migrate`                                                            | —                 | Pina's reserved account-migration route    |

## `CreateConfig`

| Data                     | Type      | Notes                            |
| ------------------------ | --------- | -------------------------------- |
| `index`                  | `u16`     | Tier index and PDA seed          |
| `trade_fee_rate`         | `u32`     | ppm                              |
| `protocol_fee_rate`      | `u32`     | ppm of the trade fee             |
| `creator_fee_rate`       | `u32`     | ppm                              |
| `authority`              | `Address` | Must not be the default address  |
| `pool_creator_authority` | `Address` | Default address for an open tier |

| # | Account             |      | Notes                                       |
| - | ------------------- | ---- | ------------------------------------------- |
| 0 | `payer`             | w, s | Pays rent                                   |
| 1 | `upgrade_authority` | s    | Must match the program-data account         |
| 2 | `program_data`      |      | `[program_id]` under the upgradeable loader |
| 3 | `amm_config`        | w    | `[b"amm_config", index]`, created           |
| 4 | `system_program`    |      |                                             |

Errors: `Unauthorized`, `InvalidProgramData`, `InvalidFeeRates`.

## `UpdateConfig`

| Data                                                      | Type      | Notes                                 |
| --------------------------------------------------------- | --------- | ------------------------------------- |
| `new_authority`                                           | `Address` | Pass the current authority to keep it |
| `trade_fee_rate`, `protocol_fee_rate`, `creator_fee_rate` | `u32`     | Rates for future pools                |

| # | Account      |   |                        |
| - | ------------ | - | ---------------------- |
| 0 | `authority`  | s | Current tier authority |
| 1 | `amm_config` | w |                        |

Errors: `Unauthorized`, `InvalidFeeRates`. Existing pools keep their snapshotted rates.

## `CreatePool`

| Data               | Type      | Notes                               |
| ------------------ | --------- | ----------------------------------- |
| `amount_0`         | `u64`     | Initial token 0, non-zero           |
| `amount_1`         | `u64`     | Initial token 1, non-zero           |
| `creator`          | `Address` | Recipient of creator fees           |
| `creator_fee_mode` | `u8`      | `0` input, `1` token 0, `2` token 1 |

| #  | Account                    |      | Notes                                                  |
| -- | -------------------------- | ---- | ------------------------------------------------------ |
| 0  | `payer`                    | w, s | Pays rent for everything created                       |
| 1  | `depositor`                | s    | Owns the source token accounts; may equal `payer`      |
| 2  | `pool_creator_authority`   | s    | The tier's restriction, or any signer for open tiers   |
| 3  | `amm_config`               |      |                                                        |
| 4  | `mint_0`                   |      | Smaller address                                        |
| 5  | `mint_1`                   |      | Larger address                                         |
| 6  | `pool`                     | w    | `[b"pool", amm_config, mint_0, mint_1]`                |
| 7  | `lp_mint`                  | w    | `[b"pool_lp_mint", pool]`                              |
| 8  | `vault_0`                  | w    | `[b"pool_vault", pool, mint_0]`                        |
| 9  | `vault_1`                  | w    | `[b"pool_vault", pool, mint_1]`                        |
| 10 | `depositor_token_0`        | w    |                                                        |
| 11 | `depositor_token_1`        | w    |                                                        |
| 12 | `lp_owner`                 |      | Receives the initial LP                                |
| 13 | `lp_owner_token`           | w    | `lp_owner`'s associated LP account, created if missing |
| 14 | `token_program_0`          |      | Owner of `mint_0`                                      |
| 15 | `token_program_1`          |      | Owner of `mint_1`                                      |
| 16 | `lp_token_program`         |      | SPL Token                                              |
| 17 | `associated_token_program` |      |                                                        |
| 18 | `system_program`           |      |                                                        |

Errors: `InvalidMintOrder`, `UnsupportedMint`, `PoolAccountMismatch`, `PoolCreatorNotAuthorized`, `InsufficientInitialLiquidity`, `ZeroAmount`, `InvalidCreatorFeeMode`. Emits `PoolCreated`.

## `Deposit` and `Withdraw`

| Data                                    | Type  | Notes                        |
| --------------------------------------- | ----- | ---------------------------- |
| `lp_amount`                             | `u64` | LP to mint or burn, non-zero |
| `maximum_amount_0` / `minimum_amount_0` | `u64` | Slippage limit for token 0   |
| `maximum_amount_1` / `minimum_amount_1` | `u64` | Slippage limit for token 1   |

| #  | Account            |   |           |
| -- | ------------------ | - | --------- |
| 0  | `owner`            | s |           |
| 1  | `pool`             | w |           |
| 2  | `vault_0`          | w |           |
| 3  | `vault_1`          | w |           |
| 4  | `lp_mint`          | w |           |
| 5  | `owner_token_0`    | w |           |
| 6  | `owner_token_1`    | w |           |
| 7  | `owner_lp_token`   | w |           |
| 8  | `token_program_0`  |   |           |
| 9  | `token_program_1`  |   |           |
| 10 | `lp_token_program` |   | SPL Token |

Errors: `PoolAccountMismatch`, `SlippageExceeded`, `InsufficientLiquidity`, `ZeroAmount`, `VaultAccountingMismatch`. Emits `LiquidityChanged`.

## `SwapExactIn` and `SwapExactOut`

| Data (`SwapExactIn`) | Type            |
| -------------------- | --------------- |
| `amount_in`          | `u64`, non-zero |
| `minimum_amount_out` | `u64`           |

| Data (`SwapExactOut`) | Type            |
| --------------------- | --------------- |
| `amount_out`          | `u64`, non-zero |
| `maximum_amount_in`   | `u64`           |

| # | Account                |   | Notes                                                 |
| - | ---------------------- | - | ----------------------------------------------------- |
| 0 | `trader`               | s | Owns `input_token`                                    |
| 1 | `pool`                 | w |                                                       |
| 2 | `input_token`          | w | Token being sold                                      |
| 3 | `output_token`         | w | Receives the token being bought; may belong to anyone |
| 4 | `input_vault`          | w | The pool's vault for the sold token                   |
| 5 | `output_vault`         | w | The pool's vault for the bought token                 |
| 6 | `input_token_program`  |   |                                                       |
| 7 | `output_token_program` |   |                                                       |

The direction is implied by the vaults. Errors: `PoolAccountMismatch`, `SlippageExceeded`, `InsufficientLiquidity`, `ZeroAmount`, `InvariantViolation`, `VaultAccountingMismatch`. Emits `Swapped`.

## `CollectProtocolFees` and `CollectCreatorFees`

| Data               | Type  | Notes                          |
| ------------------ | ----- | ------------------------------ |
| `maximum_amount_0` | `u64` | `u64::MAX` collects everything |
| `maximum_amount_1` | `u64` | `u64::MAX` collects everything |

`CollectProtocolFees` accounts: `authority` (s), `amm_config`, `pool` (w), `vault_0` (w), `vault_1` (w), `recipient_token_0` (w), `recipient_token_1` (w), `token_program_0`, `token_program_1`.

`CollectCreatorFees` accounts: `creator` (s), `pool` (w), then the same six vault, recipient, and program accounts.

Errors: `Unauthorized`, `PoolAccountMismatch`. Emits `FeesCollected`.

## `SetPoolCreator`

| Data          | Type      |
| ------------- | --------- |
| `new_creator` | `Address` |

Accounts: `creator` (s), `pool` (w). Errors: `Unauthorized`.

## `SyncPool`

Advances a pool's time-weighted price accumulator to now, using the price in force since the last advance. Permissionless: anyone may sync any pool. Swaps advance the same accumulator with their pre-trade price, so `SyncPool` matters most after a donation or a quiet interval. See [math.md](math.md#time-weighted-average-price) for the sampling formula.

| Account           | Type          | Signer | Writable | Note                      |
| ----------------- | ------------- | ------ | -------- | ------------------------- |
| `pool`            | Pool          |        | ✓        | The pool to sync          |
| `vault_0`         | Token account |        |          | The pool's token 0 vault  |
| `vault_1`         | Token account |        |          | The pool's token 1 vault  |
| `token_program_0` | Program       |        |          | Program that owns token 0 |
| `token_program_1` | Program       |        |          | Program that owns token 1 |

Emits `PoolSynced`. Errors: the pool and vault mismatches from the other read paths.

## Events

| Event                | Fields                                                                                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PoolCreated`        | `pool`, `amm_config`, `creator`, `mint_0`, `mint_1`, `lp_mint`, `amount_0`, `amount_1`, `lp_supply`                                                       |
| `Swapped`            | `pool`, `trader`, `zero_for_one`, `amount_in`, `amount_out`, `trade_fee`, `protocol_fee`, `creator_fee`, `creator_fee_on_input`, `reserve_0`, `reserve_1` |
| `LiquidityChanged`   | `pool`, `owner`, `is_deposit`, `lp_amount`, `amount_0`, `amount_1`, `lp_supply`                                                                           |
| `FeesCollected`      | `pool`, `collector`, `is_protocol`, `amount_0`, `amount_1`                                                                                                |
| `PoolSynced`         | `pool`, `reserve_0`, `reserve_1`, `price_0_cumulative_last`, `last_update_timestamp`                                                                      |
| `ConfigUpdated`      | `amm_config`, `new_authority`, `trade_fee_rate`, `protocol_fee_rate`, `creator_fee_rate`                                                                  |
| `PoolCreatorChanged` | `pool`, `previous_creator`, `new_creator`                                                                                                                 |

Flags are `u8`: `1` for true and `0` for false. Decode them with the generated `parsePinaAmmEventsFromLogs` (TypeScript and Dart), which only attributes records the AMM itself emitted.

## Errors

| Code | Name                           | Meaning                                                             |
| ---- | ------------------------------ | ------------------------------------------------------------------- |
| 0    | `InvalidFeeRates`              | Trade plus creator fee above 10%, or protocol share above 100%      |
| 1    | `Unauthorized`                 | The signer is not the required authority                            |
| 2    | `InvalidProgramData`           | The program-data account is wrong or the program is not upgradeable |
| 3    | `InvalidMintOrder`             | Mints are equal or not in ascending order                           |
| 4    | `UnsupportedMint`              | Token program or Token-2022 extension not supported                 |
| 5    | `PoolAccountMismatch`          | A vault, LP mint, or tier does not belong to the pool               |
| 6    | `InvalidCreatorFeeMode`        | Mode is not 0, 1, or 2                                              |
| 7    | `InsufficientInitialLiquidity` | The first deposit mints no more than the locked minimum             |
| 8    | `ZeroAmount`                   | An amount is zero or rounds to nothing                              |
| 9    | `SlippageExceeded`             | The result is worse than the caller's limit                         |
| 10   | `InsufficientLiquidity`        | The pool cannot pay the amount                                      |
| 11   | `MathOverflow`                 | A value overflowed its type                                         |
| 12   | `InvariantViolation`           | The constant product would decrease                                 |
| 13   | `VaultAccountingMismatch`      | A vault holds less than its accrued fees                            |
| 14   | `PoolCreatorNotAuthorized`     | The tier restricts pool creation to another signer                  |
| 15   | `DefaultCreator`               | The pool creator is the default address, which can never sign       |

Pina's own framework errors use codes `0xFFFF0000` and above, for example `DuplicateMutableAccount` when the same writable account appears twice.
