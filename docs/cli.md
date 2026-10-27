# The `pina-amm` CLI

`pina-amm` drives every Pina AMM instruction from the terminal. It derives every PDA, vault, and associated token account, so you only supply the values you actually choose: a tier index, two mints, and amounts.

```sh
cargo install pina_amm_cli
pina-amm --help
```

## Global options

| Option          | Default                                          | Meaning                                                                                                           |
| --------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| `-u, --url`     | `devnet` (`$SOLANA_URL`)                         | `mainnet`, `devnet`, `testnet`, `localhost`, or an `https://` URL. Plain `http://` is only accepted for localhost |
| `-k, --keypair` | `~/.config/solana/id.json` (`$PINA_AMM_KEYPAIR`) | Signer and fee payer, in the Solana CLI's JSON format                                                             |
| `--simulate`    | off                                              | Simulate and print compute units instead of sending                                                               |
| `--json`        | off                                              | Print one JSON object per command                                                                                 |

Amounts are always **base units**, the integer a token account stores. One USDC with 6 decimals is `1000000`.

Quotes and `--simulate` requests use unsigned transactions with placeholder signatures and a placeholder blockhash. The RPC replaces the blockhash for simulation and skips signature verification; it receives no signature it could use to broadcast the transaction. Only transactions being sent are signed with a recent blockhash.

## Fee tiers

```sh
# Create tier 0: 0.25% trade fee, 16% of it to the protocol, no creator fee.
# The signer must be the program's upgrade authority.
pina-amm config create --index 0 --trade-fee-rate 2500 --protocol-fee-rate 160000

# A tier only a launchpad PDA may create pools in, with a 0.5% creator fee.
pina-amm config create --index 100 --trade-fee-rate 2500 --protocol-fee-rate 160000 \
  --creator-fee-rate 5000 --pool-creator-authority <LAUNCHPAD_PDA>

pina-amm config show --index 0
pina-amm config update --index 0 --trade-fee-rate 3000 --protocol-fee-rate 160000 --new-authority <MULTISIG>
```

Updating a tier never changes existing pools.

## Pools

```sh
pina-amm pool create --config 0 \
  --mint-a <MINT_A> --amount-a 1000000000 \
  --mint-b <MINT_B> --amount-b 2000000000 \
  --creator-fee-mode mint-b

pina-amm pool show --pool <POOL>
pina-amm pool set-creator --pool <POOL> --new-creator <ADDRESS>
```

You can pass the mints in either order; the CLI sorts them. `--creator-fee-mode` is `input` (default), `mint-a`, or `mint-b`. `pool show` prints the reserves (excluding accrued fees), the price of token 0 in token 1 base units, and the fees waiting to be collected.

## Trading

```sh
pina-amm quote --pool <POOL> --sell <MINT> --amount-in 1000000
pina-amm swap  --pool <POOL> --sell <MINT> --amount-in 1000000 --minimum-out 1800000
pina-amm swap  --pool <POOL> --sell <MINT> --amount-out 500000 --maximum-in 280000
```

`quote` simulates a swap and prints an advisory estimate supplied by the configured RPC. The endpoint can fabricate both logs and account data, even over HTTPS. Verify prices independently before choosing your limits; do not blindly copy an RPC quote into them.

`swap` requires your explicit absolute limit: `--minimum-out` with `--amount-in`, or `--maximum-in` with `--amount-out`. It signs those exact amounts without deriving a limit from simulation events. The program rejects execution outside your bounds. `--slippage-bps` has been removed. If the output token account does not exist yet, it is created in the same transaction.

## Liquidity

```sh
pina-amm deposit  --pool <POOL> --lp-amount 1000000 \
  --maximum-amount-0 800000 --maximum-amount-1 1600000
pina-amm withdraw --pool <POOL> --lp-amount 1000000 \
  --minimum-amount-0 600000 --minimum-amount-1 1200000
```

Both token limits are required, including under `--simulate`. Token 0 and token 1 refer to the pool's `mint_0` and `mint_1`, shown by `pool show`; they are sorted by address, not by the order passed to `pool create`. Deposits cap each token input and withdrawals set each minimum output. Missing token accounts are created first. Simulation results never change these bounds.

The amounts above are examples, not suggested prices. Swap results report the submitted `limits`; liquidity results report `limit_0`, `limit_1`, and their mint addresses. These are bounds, not execution amounts.

## Fees

```sh
pina-amm fees collect-protocol --pool <POOL>   # signer: the tier authority
pina-amm fees collect-creator  --pool <POOL>   # signer: the pool creator
```

Fees are paid to the signer's associated token accounts, which are created if needed.

## Scripting

With `--json`, every command prints exactly one JSON object on stdout, in one of two shapes.

Commands that act (`config create`, `config update`, `pool create`, `pool set-creator`, `quote`, `swap`, `deposit`, `withdraw`, and `fees ...`) print an envelope. `signature` is `null` for `quote` and under `--simulate`:

```sh
$ pina-amm --json quote --pool <POOL> --sell <MINT> --amount-in 1000000
{"action":"quote","details":{"amount_in":1000000,"amount_out":1813221,"compute_units":4682,...},"signature":null}
```

Commands that read (`config show` and `pool show`) print the decoded account fields at the top level, with no envelope:

```sh
$ pina-amm --json pool show --pool <POOL>
{"address":"...","lp_supply":2000000000,"reserve_0":1000000000,"reserve_1":4000000000,...}
```

The CLI exits with status 1 and a message on stderr when anything fails, including a failed simulation, which prints the program logs.
