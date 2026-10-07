// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';

@immutable
class SwapExactOutInstructionData {
  const SwapExactOutInstructionData({
    required this.amountOut,
    required this.maximumAmountIn,
  }) : discriminator = 6;

  final int discriminator;
  final BigInt amountOut;
  final BigInt maximumAmountIn;
}

Encoder<SwapExactOutInstructionData> getSwapExactOutInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('amountOut', getU64Encoder()),
    ('maximumAmountIn', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SwapExactOutInstructionData value) => <String, Object?>{
      'discriminator': 6,
      'amountOut': value.amountOut,
      'maximumAmountIn': value.maximumAmountIn,
    },
  );
}

Decoder<SwapExactOutInstructionData> getSwapExactOutInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('amountOut', getU64Decoder()),
    ('maximumAmountIn', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'swapExactOut instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SwapExactOutInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(6)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SwapExactOutInstructionData(
        amountOut: map['amountOut']! as BigInt,
        maximumAmountIn: map['maximumAmountIn']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SwapExactOutInstructionData>(
        fixedSize: structDecoder.fixedSize,
        read: (bytes, offset) {
          final bytesLength = bytes.length - offset;
          if (bytesLength != structDecoder.fixedSize) {
            throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
          }
          return readTopLevel(bytes, offset);
        },
      ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<SwapExactOutInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SwapExactOutInstructionData, SwapExactOutInstructionData>
getSwapExactOutInstructionDataCodec() {
  return combineCodec(
    getSwapExactOutInstructionDataEncoder(),
    getSwapExactOutInstructionDataDecoder(),
  );
}

/// Creates a [SwapExactOut] instruction.
Instruction getSwapExactOutInstruction({
  required Address programAddress,
  required Address trader,
  required Address pool,
  required Address inputToken,
  required Address outputToken,
  required Address inputVault,
  required Address outputVault,
  required Address inputTokenProgram,
  required Address outputTokenProgram,
  required BigInt amountOut,
  required BigInt maximumAmountIn,
}) {
  final instructionData = SwapExactOutInstructionData(
    amountOut: amountOut,
    maximumAmountIn: maximumAmountIn,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: trader, role: AccountRole.readonlySigner),
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: inputToken, role: AccountRole.writable),
      AccountMeta(address: outputToken, role: AccountRole.writable),
      AccountMeta(address: inputVault, role: AccountRole.writable),
      AccountMeta(address: outputVault, role: AccountRole.writable),
      AccountMeta(address: inputTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: outputTokenProgram, role: AccountRole.readonly),
    ],
    data: getSwapExactOutInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SwapExactOut] instruction from raw instruction data.
SwapExactOutInstructionData parseSwapExactOutInstruction(
  Instruction instruction,
) {
  return getSwapExactOutInstructionDataDecoder().decode(instruction.data!);
}
