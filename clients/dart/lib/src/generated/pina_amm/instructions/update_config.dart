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
class UpdateConfigInstructionData {
  const UpdateConfigInstructionData({
    required this.newAuthority,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
  }) : discriminator = 1;

  final int discriminator;
  final Address newAuthority;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;
}

Encoder<UpdateConfigInstructionData> getUpdateConfigInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('newAuthority', getAddressEncoder()),
    ('tradeFeeRate', getU32Encoder()),
    ('protocolFeeRate', getU32Encoder()),
    ('creatorFeeRate', getU32Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (UpdateConfigInstructionData value) => <String, Object?>{
      'discriminator': 1,
      'newAuthority': value.newAuthority,
      'tradeFeeRate': value.tradeFeeRate,
      'protocolFeeRate': value.protocolFeeRate,
      'creatorFeeRate': value.creatorFeeRate,
    },
  );
}

Decoder<UpdateConfigInstructionData> getUpdateConfigInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('newAuthority', getAddressDecoder()),
    ('tradeFeeRate', getU32Decoder()),
    ('protocolFeeRate', getU32Decoder()),
    ('creatorFeeRate', getU32Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'updateConfig instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (UpdateConfigInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(1)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      UpdateConfigInstructionData(
        newAuthority: map['newAuthority']! as Address,
        tradeFeeRate: map['tradeFeeRate']! as int,
        protocolFeeRate: map['protocolFeeRate']! as int,
        creatorFeeRate: map['creatorFeeRate']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<UpdateConfigInstructionData>(
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
      VariableSizeDecoder<UpdateConfigInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<UpdateConfigInstructionData, UpdateConfigInstructionData>
getUpdateConfigInstructionDataCodec() {
  return combineCodec(
    getUpdateConfigInstructionDataEncoder(),
    getUpdateConfigInstructionDataDecoder(),
  );
}

/// Creates a [UpdateConfig] instruction.
Instruction getUpdateConfigInstruction({
  required Address programAddress,
  required Address authority,
  required Address ammConfig,
  required Address newAuthority,
  required int tradeFeeRate,
  required int protocolFeeRate,
  required int creatorFeeRate,
}) {
  final instructionData = UpdateConfigInstructionData(
    newAuthority: newAuthority,
    tradeFeeRate: tradeFeeRate,
    protocolFeeRate: protocolFeeRate,
    creatorFeeRate: creatorFeeRate,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: authority, role: AccountRole.readonlySigner),
      AccountMeta(address: ammConfig, role: AccountRole.writable),
    ],
    data: getUpdateConfigInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [UpdateConfig] instruction from raw instruction data.
UpdateConfigInstructionData parseUpdateConfigInstruction(
  Instruction instruction,
) {
  return getUpdateConfigInstructionDataDecoder().decode(instruction.data!);
}
