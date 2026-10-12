# Architecture

The Pina AMM is one program with two account types and ten instructions. This page explains how they fit together and why each design decision was made. For exact formulas see [math.md](math.md); for every account list see [instructions.md](instructions.md).

## Accounts

### `AmmConfig` — a fee tier

```text
PDA: [b"amm_config", index.to_le_bytes()]   (index: u16)
```

| Field                    | Type      | Meaning                                                                                        |
| ------------------------ | --------- | ---------------------------------------------------------------------------------------------- |
| `authority`              | `Address` | Updates the tier and collects protocol fees from every pool in it                              |
| `pool_creator_authority` | `Address` | The only signer allowed to create pools in this tier, or the default address for open creation |
| `trade_fee_rate`         | `u32`     | Fee on every swap, in parts per million                                                        |
| `protocol_fee_rate`      | `u32`     | Protocol's share of the trade fee, in parts per million of the trade fee                       |
| `creator_fee_rate`       | `u32`     | Extra fee paid to each pool's creator, in parts per million                                    |
| `index`                  | `u16`     | The tier's PDA seed                                                                            |
| `bump`                   | `u8`      | Canonical PDA bump                                                                             |

Only the program's **upgrade authority** can create a tier. The program reads the authority straight from the loader's program-data account, so there is no initialization race, no compiled-in admin key, and any team that deploys its own copy of this source automatically controls its own tiers. Each tier then has its own `authority`, which can be handed to a multisig.

`pool_creator_authority` cannot change after a tier is created, so a tier reserved for a launchpad stays reserved.

### `Pool` — one pair under one tier

```text
PDA: [b"pool", amm_config, mint_0, mint_1]   (mint_0 < mint_1, compared byte by byte)
```

| Field                                                     | Type      | Meaning                                                               |
| --------------------------------------------------------- | --------- | --------------------------------------------------------------------- |
| `amm_config`                                              | `Address` | The tier this pool was created under                                  |
| `creator`                                                 | `Address` | Recipient of this pool's creator fees                                 |
| `mint_0`, `mint_1`                                        | `Address` | The pair, in canonical order                                          |
| `vault_0`, `vault_1`                                      | `Address` | Pool-owned token accounts holding the reserves                        |
| `lp_mint`                                                 | `Address` | SPL Token mint for LP shares; its mint authority is the pool          |
| `lp_supply`                                               | `u64`     | Economic LP supply, including the locked minimum                      |
| `protocol_fees_0`, `protocol_fees_1`                      | `u64`     | Protocol fees accrued in each vault and not yet collected             |
| `creator_fees_0`, `creator_fees_1`                        | `u64`     | Creator fees accrued in each vault and not yet collected              |
| `trade_fee_rate`, `protocol_fee_rate`, `creator_fee_rate` | `u32`     | Rates snapshotted from the tier at creation                           |
| `creator_fee_mode`                                        | `u8`      | Which token pays the creator fee: `0` input, `1` token 0, `2` token 1 |
| `bump`                                                    | `u8`      | Canonical PDA bump                                                    |
| `price_0_cumulative_last`                                 | `u128`    | Time-weighted price of token 1 in token 0, Q64.64 price times seconds |
| `last_update_timestamp`                                   | `u64`     | Unix second the accumulator last advanced                             |

Sorting the mints gives every pair exactly one canonical pool per tier, so liquidity for a pair concentrates instead of fragmenting across duplicates.

### Data-free PDAs

| Address | Seeds                         | Owner                   | Purpose                               |
| ------- | ----------------------------- | ----------------------- | ------------------------------------- |
| Vault   | `[b"pool_vault", pool, mint]` | Token program of `mint` | Holds one side of the pool's reserves |
| LP mint | `[b"pool_lp_mint", pool]`     | SPL Token               | Issues LP shares, 9 decimals          |

The pool PDA itself is the owner of both vaults and the mint authority of the LP mint. There is no global authority account: each pool signs only for its own vaults.

Every account carries a one-byte discriminator followed by a one-byte schema version written by [`pina migrations`](https://github.com/pina-rs/pina), so the layouts can evolve after deployment through Pina's reserved `Migrate` instruction.

## Lifecycle

```text
upgrade authority ──CreateConfig──▶ AmmConfig ──UpdateConfig (future pools only)
                                       │
anyone (or the tier's                  │
pool-creator authority) ──CreatePool──▶ Pool + vaults + LP mint
                                       │
traders ──────────SwapExactIn / SwapExactOut──▶ reserves move, fees accrue
LPs ──────────────Deposit / Withdraw──────────▶ LP minted or burned
tier authority ───CollectProtocolFees─────────▶ protocol fees paid out
pool creator ─────CollectCreatorFees / SetPoolCreator
```

`CreatePool` makes the first deposit in the same instruction, so a pool never exists without liquidity. The geometric mean of the deposit, `floor(sqrt(amount_0 * amount_1))`, becomes the LP supply; 1,000 units of it are counted but never minted. Those locked units guarantee no withdrawal can empty a pool, which removes the classic first-depositor share-inflation attack.

## Reserves

Reserves are never stored. A pool's reserve of a token is:

```text
reserve = vault balance - protocol fees accrued - creator fees accrued
```

This has three consequences:

1. **Donations go to LPs.** Tokens sent straight to a vault raise the reserve, which raises the value of every LP share.
2. **Fees never price trades.** Accrued protocol and creator fees sit in the vaults but are excluded from pricing until they are collected.
3. **No reserve can drift from reality.** There is no stored number that can disagree with the token account.

The liquidity providers' share of the trade fee is simply left in the vault, so it compounds into the reserves automatically.

## LP accounting

`lp_supply` is the program's own count of LP shares, not the LP mint's supply. Deposits add to it and withdrawals subtract from it. LP burned directly through the token program, as the [bonding curve](https://github.com/pina-rs/bonding_curve) does at migration, leaves `lp_supply` unchanged, so the burned share of the pool stays locked forever instead of being redistributed to other LPs.

## Token programs and extensions

Each mint may use SPL Token or Token-2022. Token-2022 mints may carry only extensions that cannot change how many tokens a transfer moves or who can move them:

| Allowed                                                  | Rejected                                                                              |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Metadata pointer, token metadata                         | Transfer fee                                                                          |
| Group pointer, group, group member pointer, group member | Transfer hook                                                                         |
| Interest-bearing config, scaled UI amount                | Permanent delegate, pausable                                                          |
|                                                          | Non-transferable, default account state, confidential transfers, mint close authority |

Extensions are fixed when a mint is initialized, so checking them once in `CreatePool` keeps every later transfer exact. The program therefore uses plain `Transfer` CPIs, which need neither mint accounts nor decimals and keep the swap account list to eight entries.

LP mints are always SPL Token with 9 decimals. Vaults are 165-byte token accounts; none of the allowed extensions needs an account extension.

## Front-running and griefing

| Threat                                                                                 | Mitigation                                                                                                                  |
| -------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Someone pre-funds a vault or LP-mint address so `CreateAccount` fails                  | Vaults and the LP mint are created by topping up, allocating, and assigning, which works on a pre-funded address            |
| Someone creates a launchpad's pool first, at a bad price or with themselves as creator | The launchpad's tier sets `pool_creator_authority`, so only the launchpad can create pools in it                            |
| A first depositor inflates the share price to round later deposits to zero             | 1,000 LP units are locked at creation                                                                                       |
| A malicious "token program" account fakes a transfer                                   | Every token CPI checks the program is SPL Token or Token-2022, and vault balances are read through the owning token program |
| A tier authority raises fees on existing pools                                         | Pools snapshot rates at creation; tier updates only reach new pools                                                         |

[security.md](security.md) lists the full threat model and the invariants the test suite checks.

## What is deliberately missing

- **No pause or admin withdrawal.** The upgrade authority can always ship a fix; a pause switch would only add a way to freeze users' funds.
- **No on-chain oracle.** Integrations that need a time-weighted price can derive one from `Swapped` events.
- **No transfer-fee tokens.** Supporting them would require gross-up math on every path and fee-aware slippage; rejecting them keeps every transfer exact.
- **No open-time gate.** Launch timing belongs to whatever creates the pool, such as the bonding curve.
