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
class WithdrawInstructionData {
  const WithdrawInstructionData({
    required this.lpAmount,
    required this.minimumAmount0,
    required this.minimumAmount1,
  }) : discriminator = 4;

  final int discriminator;
  final BigInt lpAmount;
  final BigInt minimumAmount0;
  final BigInt minimumAmount1;
}

Encoder<WithdrawInstructionData> getWithdrawInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('lpAmount', getU64Encoder()),
    ('minimumAmount0', getU64Encoder()),
    ('minimumAmount1', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (WithdrawInstructionData value) => <String, Object?>{
      'discriminator': 4,
      'lpAmount': value.lpAmount,
      'minimumAmount0': value.minimumAmount0,
      'minimumAmount1': value.minimumAmount1,
    },
  );
}

Decoder<WithdrawInstructionData> getWithdrawInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('lpAmount', getU64Decoder()),
    ('minimumAmount0', getU64Decoder()),
    ('minimumAmount1', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'withdraw instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (WithdrawInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(4)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      WithdrawInstructionData(
        lpAmount: map['lpAmount']! as BigInt,
        minimumAmount0: map['minimumAmount0']! as BigInt,
        minimumAmount1: map['minimumAmount1']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<WithdrawInstructionData>(
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
      VariableSizeDecoder<WithdrawInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<WithdrawInstructionData, WithdrawInstructionData>
getWithdrawInstructionDataCodec() {
  return combineCodec(
    getWithdrawInstructionDataEncoder(),
    getWithdrawInstructionDataDecoder(),
  );
}

/// Creates a [Withdraw] instruction.
Instruction getWithdrawInstruction({
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
  required BigInt minimumAmount0,
  required BigInt minimumAmount1,
}) {
  final instructionData = WithdrawInstructionData(
    lpAmount: lpAmount,
    minimumAmount0: minimumAmount0,
    minimumAmount1: minimumAmount1,
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
    data: getWithdrawInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [Withdraw] instruction from raw instruction data.
WithdrawInstructionData parseWithdrawInstruction(Instruction instruction) {
  return getWithdrawInstructionDataDecoder().decode(instruction.data!);
}
