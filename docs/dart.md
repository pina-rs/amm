# Dart and Flutter client

`pina_amm` is the Dart client for the Pina AMM, generated from the program's IDL for [`solana_kit`](https://pub.dev/packages/solana_kit). It works in Flutter apps and Dart servers alike.

```yaml
dependencies:
  pina_amm: ^0.1.0
```

```dart
import 'package:pina_amm/pina_amm.dart';
```

## What is exported

| Kind         | Examples                                                                              |
| ------------ | ------------------------------------------------------------------------------------- |
| Program      | `pinaAmmProgramAddress`, `ParsedPinaAmmInstruction`                                   |
| Instructions | `getSwapExactInInstruction`, `getCreatePoolInstruction`, `getDepositInstruction`, ... |
| Accounts     | `Pool`, `AmmConfig`, `decodePool`, `getPoolDecoder`, ...                              |
| PDAs         | `findAmmConfigPda`, `findPoolPda`, `findPoolVaultPda`, `findPoolLpMintPda`            |
| Events       | `parsePinaAmmEventsFromLogs`, `PinaAmmEvent` subclasses                               |
| Errors       | `pinaAmmErrorSlippageExceeded` and the other code constants                           |

## Find a pool

```dart
final (ammConfig, _) = await findAmmConfigPda(
  seeds: AmmConfigSeeds(index: 0),
  programAddress: pinaAmmProgramAddress,
);
final (pool, _) = await findPoolPda(
  seeds: PoolSeeds(ammConfig: ammConfig, mint0: mint0, mint1: mint1),
  programAddress: pinaAmmProgramAddress,
);
```

`mint0` must be the mint whose 32 address bytes sort first. Compare `getAddressEncoder().encode(address)` byte by byte; do not compare base58 strings.

## Read a pool

```dart
final encoded = await fetchEncodedAccount(rpc, pool);
if (encoded.programAddress != pinaAmmProgramAddress) {
  throw StateError('not a Pina AMM account');
}
final state = decodePool(encoded).data;
print('lp supply ${state.lpSupply}');
```

Generated decoders check the discriminator and schema version but not the owner, so always check `programAddress` first when an address comes from outside your app.

## Swap

```dart
final instruction = getSwapExactInInstruction(
  programAddress: pinaAmmProgramAddress,
  trader: wallet.address,
  pool: pool,
  inputToken: walletToken0,
  outputToken: walletToken1,
  inputVault: state.vault0,
  outputVault: state.vault1,
  inputTokenProgram: tokenProgramAddress,
  outputTokenProgram: tokenProgramAddress,
  amountIn: BigInt.from(1000000),
  minimumAmountOut: minimumOut,
);
```

Selling token 1 swaps the token accounts, vaults, and token programs. `getSwapExactOutInstruction` takes `amountOut` and `maximumAmountIn`. Quote with the formulas in [math.md](math.md), or simulate the transaction and read the `Swapped` event.

## Events

```dart
final events = parsePinaAmmEventsFromLogs(transaction.meta!.logMessages!);
for (final event in events) {
  switch (event) {
    case SwappedEvent(:final amountIn, :final amountOut):
      print('$amountIn in, $amountOut out');
    default:
      break;
  }
}
```

Pass every log line of one transaction, in order, so records are attributed only to the AMM.

## Flutter notes

- Keep `BigInt` end to end; token amounts exceed JavaScript-safe and Dart `int` ranges on the web.
- Show users the quote _and_ the minimum they will accept, derived with the same slippage rule the CLI uses: `minimum = quote * (10000 - bps) ~/ 10000`.
- Map error codes such as `pinaAmmErrorSlippageExceeded` to product language in your app; the generated messages are written for developers.
