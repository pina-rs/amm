# `pina_amm`

The on-chain Pina AMM program: a permissionless constant-product market maker with curated fee tiers and built-in creator fees. It is deployed at `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV` and is not published to a registry.

| Path                  | Contents                                                 |
| --------------------- | -------------------------------------------------------- |
| `src/state.rs`        | `AmmConfig` and `Pool` layouts, PDA seeds, fee constants |
| `src/instructions.rs` | Instruction discriminators and data layouts              |
| `src/processors/`     | Account lists and handlers                               |
| `src/math.rs`         | Checked swap and liquidity math with property tests      |
| `src/events.rs`       | Events written to the transaction log                    |
| `src/errors.rs`       | Error codes                                              |
| `migrations/`         | Pina's schema history; never edit by hand                |
| `tests/surfpool/`     | End-to-end suite against the compiled program            |

```sh
devenv shell build:program   # target/deploy/pina_amm.so and target/idl/pina_amm.json
cargo test -p pina_amm       # unit and property tests
devenv shell test:surfpool   # end-to-end suite
```

See the [repository README](../../README.md) and [docs/](../../docs/readme.md).
