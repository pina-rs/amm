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
class DepositInstructionData {
  const DepositInstructionData({
    required this.lpAmount,
    required this.maximumAmount0,
    required this.maximumAmount1,
  }) : discriminator = 3;

  final int discriminator;
  final BigInt lpAmount;
  final BigInt maximumAmount0;
  final BigInt maximumAmount1;
}

Encoder<DepositInstructionData> getDepositInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('lpAmount', getU64Encoder()),
    ('maximumAmount0', getU64Encoder()),
    ('maximumAmount1', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (DepositInstructionData value) => <String, Object?>{
      'discriminator': 3,
      'lpAmount': value.lpAmount,
      'maximumAmount0': value.maximumAmount0,
      'maximumAmount1': value.maximumAmount1,
    },
  );
}

Decoder<DepositInstructionData> getDepositInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('lpAmount', getU64Decoder()),
    ('maximumAmount0', getU64Decoder()),
    ('maximumAmount1', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'deposit instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (DepositInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(3)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      DepositInstructionData(
        lpAmount: map['lpAmount']! as BigInt,
        maximumAmount0: map['maximumAmount0']! as BigInt,
        maximumAmount1: map['maximumAmount1']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<DepositInstructionData>(
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
      VariableSizeDecoder<DepositInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<DepositInstructionData, DepositInstructionData>
getDepositInstructionDataCodec() {
  return combineCodec(
    getDepositInstructionDataEncoder(),
    getDepositInstructionDataDecoder(),
  );
}

/// Creates a [Deposit] instruction.
Instruction getDepositInstruction({
  required Address programAddress,
  required Address owner,
  required Address pool,
  required Address vault0,
  required Address vault1,
  required Address lpMint,
  required Address ownerToken0,
  required Address ownerToken1,
  required Address ownerLpToken,
  required Address tokenProgram0,
  required Address tokenProgram1,
  required Address lpTokenProgram,
  required BigInt lpAmount,
  required BigInt maximumAmount0,
  required BigInt maximumAmount1,
}) {
  final instructionData = DepositInstructionData(
    lpAmount: lpAmount,
    maximumAmount0: maximumAmount0,
    maximumAmount1: maximumAmount1,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: owner, role: AccountRole.readonlySigner),
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: vault0, role: AccountRole.writable),
      AccountMeta(address: vault1, role: AccountRole.writable),
      AccountMeta(address: lpMint, role: AccountRole.writable),
      AccountMeta(address: ownerToken0, role: AccountRole.writable),
      AccountMeta(address: ownerToken1, role: AccountRole.writable),
      AccountMeta(address: ownerLpToken, role: AccountRole.writable),
      AccountMeta(address: tokenProgram0, role: AccountRole.readonly),
      AccountMeta(address: tokenProgram1, role: AccountRole.readonly),
      AccountMeta(address: lpTokenProgram, role: AccountRole.readonly),
    ],
    data: getDepositInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [Deposit] instruction from raw instruction data.
DepositInstructionData parseDepositInstruction(Instruction instruction) {
  return getDepositInstructionDataDecoder().decode(instruction.data!);
}
