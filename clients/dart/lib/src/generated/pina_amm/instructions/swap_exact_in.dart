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
class SwapExactInInstructionData {
  const SwapExactInInstructionData({
    required this.amountIn,
    required this.minimumAmountOut,
  }) : discriminator = 5;

  final int discriminator;
  final BigInt amountIn;
  final BigInt minimumAmountOut;
}

Encoder<SwapExactInInstructionData> getSwapExactInInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('amountIn', getU64Encoder()),
    ('minimumAmountOut', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SwapExactInInstructionData value) => <String, Object?>{
      'discriminator': 5,
      'amountIn': value.amountIn,
      'minimumAmountOut': value.minimumAmountOut,
    },
  );
}

Decoder<SwapExactInInstructionData> getSwapExactInInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('amountIn', getU64Decoder()),
    ('minimumAmountOut', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'swapExactIn instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SwapExactInInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(5)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SwapExactInInstructionData(
        amountIn: map['amountIn']! as BigInt,
        minimumAmountOut: map['minimumAmountOut']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SwapExactInInstructionData>(
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
      VariableSizeDecoder<SwapExactInInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SwapExactInInstructionData, SwapExactInInstructionData>
getSwapExactInInstructionDataCodec() {
  return combineCodec(
    getSwapExactInInstructionDataEncoder(),
    getSwapExactInInstructionDataDecoder(),
  );
}

/// Creates a [SwapExactIn] instruction.
Instruction getSwapExactInInstruction({
  required Address programAddress,
  required Address trader,
  required Address pool,
  required Address inputToken,
  required Address outputToken,
  required Address inputVault,
  required Address outputVault,
  required Address inputTokenProgram,
  required Address outputTokenProgram,
  required BigInt amountIn,
  required BigInt minimumAmountOut,
}) {
  final instructionData = SwapExactInInstructionData(
    amountIn: amountIn,
    minimumAmountOut: minimumAmountOut,
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
    data: getSwapExactInInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SwapExactIn] instruction from raw instruction data.
SwapExactInInstructionData parseSwapExactInInstruction(
  Instruction instruction,
) {
  return getSwapExactInInstructionDataDecoder().decode(instruction.data!);
}
