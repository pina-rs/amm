# `pina_amm_client`

Rust client for the [Pina AMM](https://github.com/pina-rs/amm), a permissionless constant-product market maker on Solana. Generated from the program's IDL: instruction builders, account decoders, PDA helpers, events, and error codes.

```toml
[dependencies]
pina_amm_client = "0.1"
```

```rust
use pina_amm_client::instructions::{SwapExactIn, SwapExactInInstructionData};

let data = SwapExactInInstructionData::new(|data| {
	data.amount_in.set(1_000_000);
	data.minimum_amount_out.set(990_000);
})?;
let instruction = SwapExactIn::new(
	trader, pool, input_token, output_token, input_vault, output_vault,
	input_token_program, output_token_program,
)
.instruction(data);
```

Decoders check discriminators and schema versions; check an account's owner against `PINA_AMM_ID` before trusting it. Full guide: [docs/rust-client.md](https://github.com/pina-rs/amm/blob/main/docs/rust-client.md).
