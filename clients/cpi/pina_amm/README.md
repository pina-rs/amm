# `pina_amm_cpi`

`no_std` CPI client for calling the [Pina AMM](https://github.com/pina-rs/amm) from another Solana program built with [Pina](https://github.com/pina-rs/pina). Generated from the program's IDL.

```toml
[dependencies]
pina_amm_cpi = "0.1"
```

```rust
use pina_amm_cpi::{ProgramAccount, SwapExactIn, SwapExactInIx};

let amm = ProgramAccount::try_new(self.amm_program)?;
SwapExactIn {
	trader: self.authority,
	pool: self.pool,
	input_token: self.token_0,
	output_token: self.token_1,
	input_vault: self.vault_0,
	output_vault: self.vault_1,
	input_token_program: self.token_program,
	output_token_program: self.token_program,
	ix: SwapExactInIx { amount_in, minimum_amount_out },
}
.invoke_signed(&amm, &[authority_seeds.to_signer().as_signer()])?;
```

Full guide, including creating pools from a launchpad: [docs/cpi.md](https://github.com/pina-rs/amm/blob/main/docs/cpi.md).
