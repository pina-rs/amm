# Math

All arithmetic is integer, checked, and rounded in the pool's favour. The same rules run on chain (`programs/pina_amm/src/math.rs`) and are covered by property tests that try thousands of random reserves, amounts, and fee rates on every run.

## Notation

| Symbol | Meaning                                            |
| ------ | -------------------------------------------------- |
| `x`    | Reserve of the token the trader sells              |
| `y`    | Reserve of the token the trader buys               |
| `t`    | Trade fee rate, parts per million                  |
| `p`    | Protocol share of the trade fee, parts per million |
| `c`    | Creator fee rate, parts per million                |
| `D`    | `1_000_000`                                        |

Reserves exclude accrued fees: see [architecture.md](architecture.md#reserves).

## Rounding rules

| Quantity                             | Direction | Why                               |
| ------------------------------------ | --------- | --------------------------------- |
| Any fee                              | up        | The pool never undercharges       |
| Amount a trader receives             | down      | The pool never overpays           |
| Amount a trader pays                 | up        | The pool never undercharges       |
| Protocol and creator shares of a fee | down      | The remainder stays with LPs      |
| Tokens added for a deposit           | up        | Depositors cannot mint LP cheaply |
| Tokens returned for a withdrawal     | down      | Withdrawers cannot drain dust     |

After every swap the program also checks that `x * y` did not decrease. Rounding already guarantees it; the check turns any future arithmetic mistake into a failed transaction instead of a drained pool.

## Exact-input swaps

The trader sells exactly `a`.

**Creator fee taken from the input** (mode `Input`, or the mode names the input token):

```text
fee      = ceil(a * (t + c) / D)
creator  = floor(fee * c / (t + c))
trade    = fee - creator
out      = floor((a - fee) * y / (x + a - fee))
```

**Creator fee taken from the output** (the mode names the output token):

```text
trade    = ceil(a * t / D)
gross    = floor((a - trade) * y / (x + a - trade))
creator  = ceil(gross * c / D)
out      = gross - creator
```

In both cases `protocol = floor(trade * p / D)` and the LPs keep `trade - protocol`.

### Worked example

`x = 10,000,000`, `y = 20,000,000`, `t = 2,500` (0.25%), `c = 500` (0.05%), `p = 200,000` (20%), selling `a = 1,000,000` with the creator fee on the input:

```text
fee      = ceil(1,000,000 * 3,000 / 1,000,000)   = 3,000
creator  = floor(3,000 * 500 / 3,000)             = 500
trade    = 3,000 - 500                            = 2,500
protocol = floor(2,500 * 200,000 / 1,000,000)    = 500
out      = floor(997,000 * 20,000,000 / 10,997,000) = 1,813,221
```

After the swap the token-0 vault holds 1,000,000 more tokens, of which 500 are protocol fees and 500 are creator fees awaiting collection; the reserve grows by 999,000.

## Exact-output swaps

The trader buys exactly `b`. Each step inverts the exact-input formula and rounds up:

**Creator fee from the input:**

```text
net      = ceil(x * b / (y - b))
a        = ceil(net * D / (D - (t + c)))
fee      = a - net
creator  = floor(fee * c / (t + c))
trade    = fee - creator
```

**Creator fee from the output:**

```text
gross    = ceil(b * D / (D - c))
creator  = gross - b
net      = ceil(x * gross / (y - gross))
a        = ceil(net * D / (D - t))
trade    = a - net
```

Buying back the output of an exact-input trade never costs less than that trade paid; the property tests check this for random inputs.

## Fee splits by creator fee mode

| Mode        | Selling token 0                      | Selling token 1                      |
| ----------- | ------------------------------------ | ------------------------------------ |
| `0` Input   | creator fee in token 0               | creator fee in token 1               |
| `1` Token 0 | creator fee in token 0 (from input)  | creator fee in token 0 (from output) |
| `2` Token 1 | creator fee in token 1 (from output) | creator fee in token 1 (from input)  |

The trade fee, and therefore the protocol fee, is always taken from the input token.

## Liquidity

### First deposit

```text
lp_supply = floor(sqrt(amount_0 * amount_1))
minted    = lp_supply - 1,000
```

`lp_supply` must be greater than 1,000. The integer square root uses a shift-and-subtract method that is exact for every `u128` and avoids a software division per iteration on SBF.

### Deposit

Minting `lp` shares against supply `S` costs:

```text
amount_0 = ceil(reserve_0 * lp / S)
amount_1 = ceil(reserve_1 * lp / S)
```

### Withdraw

Burning `lp` shares returns:

```text
amount_0 = floor(reserve_0 * lp / S)
amount_1 = floor(reserve_1 * lp / S)
```

`lp` may not exceed `S - 1,000`, so the locked minimum always stays in the pool, and at least one of the two amounts must be positive.

## Time-weighted average price

Every pool sums its price over time so integrators can build manipulation-resistant average prices. The pool stores a cumulative accumulator, the Unix second it was last advanced, and the reserves it held at that moment (`reserve_0_last`, `reserve_1_last`). Every instruction that can change the reserves (`CreatePool`, both swaps, `Deposit`, `Withdraw`) and `SyncPool` runs one observation, exactly like Uniswap V2's `_update`:

```text
// 1. Price the elapsed interval with the reserves stored at the last observation.
price_0            = reserve_1_last * 2^64 / reserve_0_last      // Q64.64
price_0_cumulative += price_0 * (now - last_update)
last_update         = now
// 2. Apply the instruction to the live reserves.
// 3. Store the post-instruction reserves as the price of the next interval.
reserve_0_last, reserve_1_last = reserve_0, reserve_1
```

The accumulator never reads the live vault balances for time that has already passed. That matters because reserves are vault balances: a donation straight to a vault moves the live price without any program call, and if the next observation priced the whole interval since `last_update` at the live reserves, a donation followed by `SyncPool` in one transaction would rewrite history. With flash-deposited liquidity (deposits and withdrawals are feeless) an attacker could recover nearly all of the donation, so the manipulation would cost almost nothing. Storing the observed reserves closes that: a donation only enters the price at the next observation, and only for the time after it.

The pool stores reserves rather than a precomputed price because it is the cheaper choice. An observation divides only when the clock has moved since the last one, so every later instruction in the same second skips the `u128` division, and the post-instruction step is two `u64` stores. Both options take 16 bytes. These stored reserves feed the accumulator and nothing else: swaps, deposits, and withdrawals always price from the live vault balances.

To read the average price of token 0 over a window, sample the accumulator and the timestamp twice and divide:

```text
twap_0 = ((cumulative_b - cumulative_a) mod 2^256) / (timestamp_b - timestamp_a) / 2^64
```

The accumulator is a 256-bit unsigned integer in two little-endian `u128` fields: `price_0_cumulative_last` (low) and `price_0_cumulative_high_last` (high), with value `low + high * 2^128`. Decode both limbs before subtracting. For example, with TypeScript `bigint`:

```typescript
const decode = (low: bigint, high: bigint): bigint => low + (high << 128n);
const mask = (1n << 256n) - 1n;
const elapsed = timestampB - timestampA; // Require timestampB > timestampA.
const cumulativeA = decode(
	sampleA.price0CumulativeLast,
	sampleA.price0CumulativeHighLast,
);
const cumulativeB = decode(
	sampleB.price0CumulativeLast,
	sampleB.price0CumulativeHighLast,
);
const averageQ64 = ((cumulativeB - cumulativeA) & mask) / elapsed;
```

Keep the result in Q64.64 until the final conversion to avoid losing fractional precision. The encoded price is below `2^128`, and the sum of elapsed seconds in any supported timestamp window is below `2^64`. The full price-time delta is therefore below `2^192`, strictly below the `2^256` modulus. Subtraction recovers every bit even at the maximum reserve ratio over the full clock range. The old 128-bit accumulator did not have this headroom: `1 : 2^63` reserves held for two seconds lost an entire cycle and reported zero.

The value is token 1 per token 0; invert for the other direction. Notes:

- **Sample with `SyncPool`.** A stored accumulator is only current up to `last_update`. Put `SyncPool` and your read in the same transaction (or read `PoolSynced`), or extend a stale sample yourself: `cumulative + reserve_1_last * 2^64 / reserve_0_last * (now - last_update)`, which is exactly what the next observation will add.
- **Only the price standing at a second boundary counts.** Observations within the same Unix second accrue nothing, and the last one in that second sets the price for the next interval. Moving the price and restoring it inside one transaction, or inside one second, leaves the average untouched.
- **Donations count from the next observation onward.** Folding one in with `SyncPool` is legitimate and makes the stored price match the vault balances; it never changes what has already accrued.
- A pool with an empty side records no price, and a `last_update` of zero only anchors the clock.
- Use both limbs for stored samples, `PoolSynced` events, and counterfactual stale-sample extension. Using only the low limb can alias the delta to zero. The pool remains tradable at extreme ratios without saturating or discarding the elapsed integral.
- Take both samples after upgrading to the full-width schema. Migration initializes the high limb to zero; it cannot recover cycles already lost by the old accumulator. Never combine an old-schema sample or event with a full-width sample.

[security.md](security.md#time-weighted-price) describes what the average does and does not resist.

## Limits

| Limit                   | Value                                         |
| ----------------------- | --------------------------------------------- |
| Trade fee + creator fee | at most 100,000 ppm (10%)                     |
| Protocol share          | at most 1,000,000 ppm (100% of the trade fee) |
| Locked LP               | 1,000 units                                   |
| Amounts                 | `u64`; intermediate products use `u128`       |

Any result that does not fit its type fails with `MathOverflow` instead of wrapping.
