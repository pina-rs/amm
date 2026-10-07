import 'dart:typed_data';

import 'package:pina_amm/pina_amm.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';
import 'package:test/test.dart';

void main() {
  test('exports the deployed program address', () {
    expect(
      pinaAmmProgramAddress,
      const Address('pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV'),
    );
  });

  test('builds a swap with the program account order and data layout', () {
    final trader = const Address('11111111111111111111111111111111');
    final instruction = getSwapExactInInstruction(
      programAddress: pinaAmmProgramAddress,
      trader: trader,
      pool: trader,
      inputToken: trader,
      outputToken: trader,
      inputVault: trader,
      outputVault: trader,
      inputTokenProgram: trader,
      outputTokenProgram: trader,
      amountIn: BigInt.from(1000),
      minimumAmountOut: BigInt.from(990),
    );

    expect(instruction.accounts, hasLength(8));
    expect(instruction.accounts!.first.role, AccountRole.readonlySigner);
    final data = instruction.data!;
    expect(data, hasLength(17));
    expect(data[0], 5);
    final view = ByteData.sublistView(Uint8List.fromList(data));
    expect(view.getUint64(1, Endian.little), 1000);
    expect(view.getUint64(9, Endian.little), 990);
  });
}
