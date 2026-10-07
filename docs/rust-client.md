# Rust client

`pina_amm_client` is the off-chain Rust client, generated from the program's IDL. It builds `solana_instruction::Instruction` values, decodes accounts and events, and exposes the program's errors.

```toml
[dependencies]
pina_amm_client = "0.1"
solana-instruction = "3.4"
solana-pubkey = "4"
```

For on-chain callers use [`pina_amm_cpi`](cpi.md) instead; it is `no_std` and builds CPIs rather than transactions.

## Layout

| Module         | Contents                                                      |
| -------------- | ------------------------------------------------------------- |
| crate root     | `PINA_AMM_ID`                                                 |
| `instructions` | One account struct and one `*InstructionData` per instruction |
| `accounts`     | `AmmConfig` and `Pool` decoders and PDA helpers               |
| `events`       | `PoolCreated`, `Swapped`, `LiquidityChanged`, `FeesCollected` |
| `errors`       | `PinaAmmError`                                                |

## Derive addresses

```rust
use pina_amm_client::PINA_AMM_ID;
use pina_amm_client::accounts::{AmmConfig, Pool};
use solana_pubkey::Pubkey;

let (config, _) = AmmConfig::find_pda(0);
let (mint_0, mint_1) = if mint_a < mint_b { (mint_a, mint_b) } else { (mint_b, mint_a) };
let (pool, _) = Pool::find_pda(&config, &mint_0, &mint_1);

let vault_0 = Pubkey::find_program_address(
	&[b"pool_vault", pool.as_ref(), mint_0.as_ref()],
	&PINA_AMM_ID,
)
.0;
let lp_mint = Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &PINA_AMM_ID).0;
```

`Pubkey`'s `Ord` compares bytes, which is exactly the program's mint order.

## Read a pool

```rust
use pina_amm_client::PINA_AMM_ID;
use pina_amm_client::accounts::Pool;

let account = rpc.get_account(&pool)?;
assert_eq!(account.owner, PINA_AMM_ID, "not a Pina AMM account");
let state = Pool::from_bytes(&account.data)?;

println!("lp supply {}", state.lp_supply.get());
println!("vaults {} {}", state.vault0, state.vault1);
println!("accrued protocol fees {} {}", state.protocol_fees0.get(), state.protocol_fees1.get());
```

`from_bytes` checks the discriminator, schema version, and length. It cannot know where the bytes came from, so check the owner first, as above. Numeric fields are little-endian wrappers; call `.get()` to read them.

## Swap

```rust
use pina_amm_client::instructions::{SwapExactIn, SwapExactInInstructionData};

let data = SwapExactInInstructionData::new(|data| {
	data.amount_in.set(1_000_000);
	data.minimum_amount_out.set(minimum_out);
})?;
let instruction = SwapExactIn::new(
	trader,
	pool,
	trader_token_0, // input: selling token 0
	trader_token_1, // output
	state.vault0,
	state.vault1,
	token_program_0,
	token_program_1,
)
.instruction(data);
```

Every account struct is plain data, so you can also build it with a struct literal and change a field before calling `.instruction(...)`.

## Create a pool

```rust
use pina_amm_client::instructions::{CreatePool, CreatePoolInstructionData};

let data = CreatePoolInstructionData::new(|data| {
	data.amount0.set(1_000_000_000);
	data.amount1.set(2_000_000_000);
	data.creator = creator;
	data.creator_fee_mode = 0;
})?;
let instruction = CreatePool::new(
	payer,
	depositor,
	payer, // pool-creator authority: any signer for an open tier
	config,
	mint_0,
	mint_1,
	lp_mint,
	vault_0,
	vault_1,
	depositor_token_0,
	depositor_token_1,
	lp_owner,
	lp_owner_token, // lp_owner's associated token account for lp_mint
	token_program_0,
	token_program_1,
)
.instruction(data);
```

`CreatePool::new` derives the pool PDA and fills in the SPL Token, associated token, and system program addresses.

## Events

The Rust events module decodes one record at a time. Find the AMM's `Program data:` lines in a transaction's logs, base64-decode them, and match on the first byte:

| First byte | Event              |
| ---------- | ------------------ |
| `1`        | `PoolCreated`      |
| `2`        | `Swapped`          |
| `3`        | `LiquidityChanged` |
| `4`        | `FeesCollected`    |

```rust
use pina_amm_client::events::Swapped;

if bytes.first() == Some(&2) {
	let swap = Swapped::from_bytes(&bytes)?;
	println!("{} in, {} out", swap.amount_in.get(), swap.amount_out.get());
}
```

Only attribute a `Program data:` line to the AMM while the AMM is the innermost invoked program in the log frames; the CLI's `program_events` function (`crates/pina_amm_cli/src/context.rs`) is a complete reference implementation.

## A complete reference

The end-to-end suite in `programs/pina_amm/tests/surfpool` uses this client for every instruction against the real program, including fee tiers, pool creation, both swap modes, liquidity, and fee collection. It is the most complete example of the client in use.
