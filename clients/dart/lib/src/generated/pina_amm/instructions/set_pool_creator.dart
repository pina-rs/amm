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
class SetPoolCreatorInstructionData {
  const SetPoolCreatorInstructionData({required this.newCreator})
    : discriminator = 9;

  final int discriminator;
  final Address newCreator;
}

Encoder<SetPoolCreatorInstructionData>
getSetPoolCreatorInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('newCreator', getAddressEncoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (SetPoolCreatorInstructionData value) => <String, Object?>{
      'discriminator': 9,
      'newCreator': value.newCreator,
    },
  );
}

Decoder<SetPoolCreatorInstructionData>
getSetPoolCreatorInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('newCreator', getAddressDecoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'setPoolCreator instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (SetPoolCreatorInstructionData, int) readTopLevel(
    Uint8List bytes,
    int offset,
  ) {
    getConstantDecoder(getU8Encoder().encode(9)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      SetPoolCreatorInstructionData(newCreator: map['newCreator']! as Address),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<SetPoolCreatorInstructionData>(
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
      VariableSizeDecoder<SetPoolCreatorInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<SetPoolCreatorInstructionData, SetPoolCreatorInstructionData>
getSetPoolCreatorInstructionDataCodec() {
  return combineCodec(
    getSetPoolCreatorInstructionDataEncoder(),
    getSetPoolCreatorInstructionDataDecoder(),
  );
}

/// Creates a [SetPoolCreator] instruction.
Instruction getSetPoolCreatorInstruction({
  required Address programAddress,
  required Address creator,
  required Address pool,
  required Address newCreator,
}) {
  final instructionData = SetPoolCreatorInstructionData(newCreator: newCreator);

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: creator, role: AccountRole.readonlySigner),
      AccountMeta(address: pool, role: AccountRole.writable),
    ],
    data: getSetPoolCreatorInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [SetPoolCreator] instruction from raw instruction data.
SetPoolCreatorInstructionData parseSetPoolCreatorInstruction(
  Instruction instruction,
) {
  return getSetPoolCreatorInstructionDataDecoder().decode(instruction.data!);
}
