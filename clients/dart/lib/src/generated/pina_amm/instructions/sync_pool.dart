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
class SyncPoolInstructionData {
  const SyncPoolInstructionData() : discriminator = 10;

  final int discriminator;
}

Encoder<SyncPoolInstructionData> getSyncPoolInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SyncPoolInstructionData value) => <String, Object?>{'discriminator': 10},
  );
}

Decoder<SyncPoolInstructionData> getSyncPoolInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'syncPool instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SyncPoolInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(10)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (SyncPoolInstructionData(), newOffset);
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SyncPoolInstructionData>(
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
      VariableSizeDecoder<SyncPoolInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SyncPoolInstructionData, SyncPoolInstructionData>
getSyncPoolInstructionDataCodec() {
  return combineCodec(
    getSyncPoolInstructionDataEncoder(),
    getSyncPoolInstructionDataDecoder(),
  );
}

/// Creates a [SyncPool] instruction.
Instruction getSyncPoolInstruction({
  required Address programAddress,
  required Address pool,
  required Address vault0,
  required Address vault1,
  required Address tokenProgram0,
  required Address tokenProgram1,
}) {
  final instructionData = SyncPoolInstructionData();

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: pool, role: AccountRole.writable),
      AccountMeta(address: vault0, role: AccountRole.readonly),
      AccountMeta(address: vault1, role: AccountRole.readonly),
      AccountMeta(address: tokenProgram0, role: AccountRole.readonly),
      AccountMeta(address: tokenProgram1, role: AccountRole.readonly),
    ],
    data: getSyncPoolInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SyncPool] instruction from raw instruction data.
SyncPoolInstructionData parseSyncPoolInstruction(Instruction instruction) {
  return getSyncPoolInstructionDataDecoder().decode(instruction.data!);
}
