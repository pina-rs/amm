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
class CollectCreatorFeesInstructionData {
  const CollectCreatorFeesInstructionData({
    required this.maximumAmount0,
    required this.maximumAmount1,
  }) : discriminator = 8;

  final int discriminator;
  final BigInt maximumAmount0;
  final BigInt maximumAmount1;
}

Encoder<CollectCreatorFeesInstructionData>
getCollectCreatorFeesInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('maximumAmount0', getU64Encoder()),
    ('maximumAmount1', getU64Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CollectCreatorFeesInstructionData value) => <String, Object?>{
      'discriminator': 8,
      'maximumAmount0': value.maximumAmount0,
      'maximumAmount1': value.maximumAmount1,
    },
  );
}

Decoder<CollectCreatorFeesInstructionData>
getCollectCreatorFeesInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('maximumAmount0', getU64Decoder()),
    ('maximumAmount1', getU64Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'collectCreatorFees instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (CollectCreatorFeesInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(8)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CollectCreatorFeesInstructionData(
        maximumAmount0: map['maximumAmount0']! as BigInt,
        maximumAmount1: map['maximumAmount1']! as BigInt,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CollectCreatorFeesInstructionData>(
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
      VariableSizeDecoder<CollectCreatorFeesInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CollectCreatorFeesInstructionData, CollectCreatorFeesInstructionData>
getCollectCreatorFeesInstructionDataCodec() {
  return combineCodec(
    getCollectCreatorFeesInstructionDataEncoder(),
    getCollectCreatorFeesInstructionDataDecoder(),
  );
}

/// Creates a [CollectCreatorFees] instruction.
Instruction getCollectCreatorFeesInstruction({
  required Address programAddress,
  required Address creator,
  required Address pool,
  required Address vault0,
  required Address vault1,
  required Address recipientToken0,
  required Address recipientToken1,
  required Address tokenProgram0,
  required Address tokenProgram1,
  required BigInt maximumAmount0,
  required BigInt maximumAmount1,
}) {
  final instructionData = CollectCreatorFeesInstructionData(
    maximumAmount0: maximumAmount0,
    maximumAmount1: maximumAmount1,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: creator, role: AccountRole.readonlySigner),
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: vault0, role: AccountRole.writable),
      AccountMeta(address: vault1, role: AccountRole.writable),
      AccountMeta(address: recipientToken0, role: AccountRole.writable),
      AccountMeta(address: recipientToken1, role: AccountRole.writable),
      AccountMeta(address: tokenProgram0, role: AccountRole.readonly),
      AccountMeta(address: tokenProgram1, role: AccountRole.readonly),
    ],
    data: getCollectCreatorFeesInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CollectCreatorFees] instruction from raw instruction data.
CollectCreatorFeesInstructionData parseCollectCreatorFeesInstruction(
  Instruction instruction,
) {
  return getCollectCreatorFeesInstructionDataDecoder().decode(
    instruction.data!,
  );
}
