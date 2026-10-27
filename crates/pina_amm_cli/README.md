# `pina-amm`

Command-line interface for the [Pina AMM](https://github.com/pina-rs/amm). It derives every PDA, vault, and token account, quotes trades by simulation, and requires explicit absolute limits for swaps and liquidity changes. RPC quotes are advisory; verify prices independently before choosing your bounds.

```sh
cargo install pina_amm_cli

pina-amm -u mainnet pool show --pool <POOL>
pina-amm -u mainnet quote --pool <POOL> --sell <MINT> --amount-in 1000000
pina-amm -u mainnet swap  --pool <POOL> --sell <MINT> --amount-in 1000000 --minimum-out 1800000
pina-amm -u mainnet deposit --pool <POOL> --lp-amount 1000000 --maximum-amount-0 800000 --maximum-amount-1 1600000
```

Every command accepts `--json` for scripting and `--simulate` for a dry run. Full reference: [docs/cli.md](https://github.com/pina-rs/amm/blob/main/docs/cli.md).
