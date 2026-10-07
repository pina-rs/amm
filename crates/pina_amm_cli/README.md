# `pina-amm`

Command-line interface for the [Pina AMM](https://github.com/pina-rs/amm). It derives every PDA, vault, and token account, quotes trades by simulating them, and sends them with a slippage limit.

```sh
cargo install pina_amm_cli

pina-amm -u mainnet pool show --pool <POOL>
pina-amm -u mainnet quote --pool <POOL> --sell <MINT> --amount-in 1000000
pina-amm -u mainnet swap  --pool <POOL> --sell <MINT> --amount-in 1000000 --slippage-bps 50
pina-amm -u mainnet deposit --pool <POOL> --lp-amount 1000000
```

Every command accepts `--json` for scripting and `--simulate` for a dry run. Full reference: [docs/cli.md](https://github.com/pina-rs/amm/blob/main/docs/cli.md).
