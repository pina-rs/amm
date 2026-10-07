# pina_amm

Dart and Flutter client for the [Pina AMM](https://github.com/pina-rs/amm), a permissionless constant-product market maker on Solana, for [`solana_kit`](https://pub.dev/packages/solana_kit). Generated from the program's IDL.

```yaml
dependencies:
  pina_amm: ^0.1.0
```

```dart
import 'package:pina_amm/pina_amm.dart';
import 'package:solana_kit/solana_kit.dart';

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
  minimumAmountOut: BigInt.from(990000),
);
```

Full guide: [docs/dart.md](https://github.com/pina-rs/amm/blob/main/docs/dart.md).
