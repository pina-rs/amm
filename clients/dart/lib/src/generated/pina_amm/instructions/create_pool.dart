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
class CreatePoolInstructionData {
  const CreatePoolInstructionData({
    required this.amount0,
    required this.amount1,
    required this.creator,
    required this.creatorFeeMode,
  }) : discriminator = 2;

  final int discriminator;
  final BigInt amount0;
  final BigInt amount1;
  final Address creator;
  final int creatorFeeMode;
}

Encoder<CreatePoolInstructionData> getCreatePoolInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('amount0', getU64Encoder()),
    ('amount1', getU64Encoder()),
    ('creator', getAddressEncoder()),
    ('creatorFeeMode', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CreatePoolInstructionData value) => <String, Object?>{
      'discriminator': 2,
      'amount0': value.amount0,
      'amount1': value.amount1,
      'creator': value.creator,
      'creatorFeeMode': value.creatorFeeMode,
    },
  );
}

Decoder<CreatePoolInstructionData> getCreatePoolInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('amount0', getU64Decoder()),
    ('amount1', getU64Decoder()),
    ('creator', getAddressDecoder()),
    ('creatorFeeMode', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'createPool instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (CreatePoolInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(2)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CreatePoolInstructionData(
        amount0: map['amount0']! as BigInt,
        amount1: map['amount1']! as BigInt,
        creator: map['creator']! as Address,
        creatorFeeMode: map['creatorFeeMode']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CreatePoolInstructionData>(
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
      VariableSizeDecoder<CreatePoolInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CreatePoolInstructionData, CreatePoolInstructionData>
getCreatePoolInstructionDataCodec() {
  return combineCodec(
    getCreatePoolInstructionDataEncoder(),
    getCreatePoolInstructionDataDecoder(),
  );
}

/// Creates a [CreatePool] instruction.
Instruction getCreatePoolInstruction({
  required Address programAddress,
  required Address payer,
  required Address depositor,
  required Address poolCreatorAuthority,
  required Address ammConfig,
  required Address mint0,
  required Address mint1,
  required Address pool,
  required Address lpMint,
  required Address vault0,
  required Address vault1,
  required Address depositorToken0,
  required Address depositorToken1,
  required Address lpOwner,
  required Address lpOwnerToken,
  required Address tokenProgram0,
  required Address tokenProgram1,
  required Address lpTokenProgram,
  required Address associatedTokenProgram,
  required Address systemProgram,
  required BigInt amount0,
  required BigInt amount1,
  required Address creator,
  required int creatorFeeMode,
}) {
  final instructionData = CreatePoolInstructionData(
    amount0: amount0,
    amount1: amount1,
    creator: creator,
    creatorFeeMode: creatorFeeMode,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: payer, role: AccountRole.writableSigner),
      AccountMeta(address: depositor, role: AccountRole.readonlySigner),
      AccountMeta(
        address: poolCreatorAuthority,
        role: AccountRole.readonlySigner,
      ),
      AccountMeta(address: ammConfig, role: AccountRole.readonly),
      AccountMeta(address: mint0, role: AccountRole.readonly),
      AccountMeta(address: mint1, role: AccountRole.readonly),
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: lpMint, role: AccountRole.writable),
      AccountMeta(address: vault0, role: AccountRole.writable),
      AccountMeta(address: vault1, role: AccountRole.writable),
      AccountMeta(address: depositorToken0, role: AccountRole.writable),
      AccountMeta(address: depositorToken1, role: AccountRole.writable),
      AccountMeta(address: lpOwner, role: AccountRole.readonly),
      AccountMeta(address: lpOwnerToken, role: AccountRole.writable),
      AccountMeta(address: tokenProgram0, role: AccountRole.readonly),
      AccountMeta(address: tokenProgram1, role: AccountRole.readonly),
      AccountMeta(address: lpTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: associatedTokenProgram, role: AccountRole.readonly),
      AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getCreatePoolInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CreatePool] instruction from raw instruction data.
CreatePoolInstructionData parseCreatePoolInstruction(Instruction instruction) {
  return getCreatePoolInstructionDataDecoder().decode(instruction.data!);
}
