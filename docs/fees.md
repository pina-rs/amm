# Fees

Every swap pays a **trade fee**, part of which can go to the protocol, and optionally a **creator fee**. All rates are in parts per million (`2_500` is 0.25%).

## Fee tiers

A fee tier (`AmmConfig`) fixes three rates:

| Rate                | Charged on                        | Paid to                           | Limit                   |
| ------------------- | --------------------------------- | --------------------------------- | ----------------------- |
| `trade_fee_rate`    | every swap's input                | LPs, minus the protocol share     | trade + creator ≤ 10%   |
| `protocol_fee_rate` | the trade fee                     | the tier authority, on collection | ≤ 100% of the trade fee |
| `creator_fee_rate`  | the input or output, by pool mode | the pool's creator, on collection | trade + creator ≤ 10%   |

Tiers are numbered by a `u16` index. A typical set might be:

| Index | Trade fee | Protocol share | Creator fee | Use                                                        |
| ----- | --------- | -------------- | ----------- | ---------------------------------------------------------- |
| 0     | 0.25%     | 16%            | 0           | Standard pairs                                             |
| 1     | 0.05%     | 16%            | 0           | Stable pairs                                               |
| 2     | 1.00%     | 16%            | 0           | Exotic pairs                                               |
| 100   | 0.25%     | 16%            | 0.50%       | Bonding-curve graduations, restricted to the curve program |

Only the program's upgrade authority can create a tier (`CreateConfig`). The tier's own `authority` can then change its rates or hand the tier to someone else (`UpdateConfig`), but **existing pools never change**: each pool copies the rates when it is created.

### Restricted tiers

Setting `pool_creator_authority` when creating a tier reserves it: `CreatePool` then requires that exact address to sign. A launchpad program sets it to one of its PDAs so it can create its graduation pools with a creator fee, and so nobody can create those pools first. Open tiers leave it as the default address; their `CreatePool` accepts any signer in that slot, normally the payer.

## Creator fees

Each pool has a `creator` (any address, chosen at creation) and a `creator_fee_mode`:

| Mode    | Value | The creator is paid in           |
| ------- | ----- | -------------------------------- |
| Input   | `0`   | whichever token the trader sells |
| Token 0 | `1`   | token 0, in both directions      |
| Token 1 | `2`   | token 1, in both directions      |

Pinning the creator fee to one token is how a launchpad pays a token's creator in SOL or USDC rather than in the token they launched: selling that token into the pool takes the creator fee from the quote output, and buying it takes the creator fee from the quote input. [math.md](math.md#fee-splits-by-creator-fee-mode) has the exact split.

The creator can hand the fee rights to another address with `SetPoolCreator`. Setting it to an address no one controls retires the creator fee for good.

## Accrual and collection

Fees are never transferred during a swap. The protocol and creator shares stay in the pool's vaults and are recorded in `protocol_fees_{0,1}` and `creator_fees_{0,1}`; they are excluded from the reserves, so they never move the price. The LPs' share is left in the reserves and compounds automatically.

| Instruction           | Signer                    | Pays out                                |
| --------------------- | ------------------------- | --------------------------------------- |
| `CollectProtocolFees` | the pool's tier authority | `protocol_fees_0` and `protocol_fees_1` |
| `CollectCreatorFees`  | the pool's creator        | `creator_fees_0` and `creator_fees_1`   |

Both take a maximum per token, so a caller can collect part of the balance; pass `u64::MAX` to collect everything. The recipient token accounts can belong to anyone.

From the CLI:

```sh
pina-amm fees collect-protocol --pool <POOL>   # signer: tier authority
pina-amm fees collect-creator  --pool <POOL>   # signer: pool creator
```

## Where fees show up

Every swap emits a `Swapped` event carrying `trade_fee`, `protocol_fee`, `creator_fee`, and `creator_fee_on_input` (`1` when the creator fee is in the input token). Collections emit `FeesCollected`. Indexers can reconstruct every fee from these events without reading account history.
