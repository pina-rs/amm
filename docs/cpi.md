# Calling the AMM from another program

`pina_amm_cpi` is a `no_std` crate for on-chain callers. Each instruction is a struct of `&AccountView` fields plus an `ix` struct of arguments. Field names follow the IDL, so numbered fields have no underscore before the digit (`mint0`, `vault1`); build it and call `.invoke(&program)` or `.invoke_signed(&program, signers)`.

```toml
[dependencies]
pina = { version = "0.23", default-features = false }
pina_amm_cpi = "0.1"
```

## Validate the program account first

```rust
use pina_amm_cpi::ProgramAccount;

let amm = ProgramAccount::try_new(self.amm_program)?; // checks address and executable flag
```

`ProgramAccount` is `pina::Program<PinaAmm>`: it refuses any account that is not the deployed AMM, so a caller cannot redirect your CPI to a look-alike program.

## Swap through the AMM

```rust
use pina_amm_cpi::{SwapExactIn, SwapExactInIx};

SwapExactIn {
	trader: self.vault_authority,
	pool: self.pool,
	input_token: self.program_token_0,
	output_token: self.program_token_1,
	input_vault: self.pool_vault_0,
	output_vault: self.pool_vault_1,
	input_token_program: self.token_program,
	output_token_program: self.token_program,
	ix: SwapExactInIx {
		amount_in,
		minimum_amount_out,
	},
}
.invoke_signed(&amm, &[vault_authority_seeds.to_signer().as_signer()])?;
```

The AMM validates the pool, its vaults, and both token programs itself, so your program only has to make sure the token accounts it passes are its own.

## Create pools from a launchpad

A launchpad that wants its pools to be uncontested asks the AMM's upgrade authority for a **restricted tier** whose `pool_creator_authority` is one of the launchpad's PDAs. It then signs `CreatePool` with that PDA:

```rust
use pina_amm_cpi::{CreatePool, CreatePoolIx};

CreatePool {
	payer: self.payer,
	depositor: self.launch,                 // owns the source token accounts
	pool_creator_authority: self.amm_authority, // the launchpad PDA the tier names
	amm_config: self.amm_config,
	mint0: self.mint_0,
	mint1: self.mint_1,
	pool: self.pool,
	lp_mint: self.lp_mint,
	vault0: self.pool_vault_0,
	vault1: self.pool_vault_1,
	depositor_token0: self.launch_vault_0,
	depositor_token1: self.launch_vault_1,
	lp_owner: self.launch,
	lp_owner_token: self.launch_lp_token,
	token_program0: self.token_program_0,
	token_program1: self.token_program_1,
	lp_token_program: self.token_program,
	associated_token_program: self.associated_token_program,
	system_program: self.system_program,
	ix: CreatePoolIx {
		amount0: amount_0,
		amount1: amount_1,
		creator: &creator,
		creator_fee_mode,
	},
}
.invoke_signed(&amm, &[launch_signer.as_signer(), authority_signer.as_signer()])?;
```

[`pina-rs/bonding_curve`](https://github.com/pina-rs/bonding_curve) does exactly this when a curve graduates. Its `Migrate` processor is a complete, tested example.

## Reading AMM accounts on chain

The crate's `accounts` module parses `AmmConfig` and `Pool` by discriminator only. Always check ownership first:

```rust
self.pool.assert_owner(&pina_amm_cpi::PINA_AMM_ID)?;
```

## Keeping the client in sync

The CPI crate is generated from the AMM's IDL. If you vendor it instead of depending on the published crate, regenerate it from the IDL of the AMM version you target with `pina import pina_amm --program-id pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV --idl <idl.json>`, which records the IDL hash it was built from.
