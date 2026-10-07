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
class CreateConfigInstructionData {
  const CreateConfigInstructionData({
    required this.index,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
    required this.authority,
    required this.poolCreatorAuthority,
  }) : discriminator = 0;

  final int discriminator;
  final int index;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;
  final Address authority;
  final Address poolCreatorAuthority;
}

Encoder<CreateConfigInstructionData> getCreateConfigInstructionDataEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('index', getU16Encoder()),
    ('tradeFeeRate', getU32Encoder()),
    ('protocolFeeRate', getU32Encoder()),
    ('creatorFeeRate', getU32Encoder()),
    ('authority', getAddressEncoder()),
    ('poolCreatorAuthority', getAddressEncoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (CreateConfigInstructionData value) => <String, Object?>{
      'discriminator': 0,
      'index': value.index,
      'tradeFeeRate': value.tradeFeeRate,
      'protocolFeeRate': value.protocolFeeRate,
      'creatorFeeRate': value.creatorFeeRate,
      'authority': value.authority,
      'poolCreatorAuthority': value.poolCreatorAuthority,
    },
  );
}

Decoder<CreateConfigInstructionData> getCreateConfigInstructionDataDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('index', getU16Decoder()),
    ('tradeFeeRate', getU32Decoder()),
    ('protocolFeeRate', getU32Decoder()),
    ('creatorFeeRate', getU32Decoder()),
    ('authority', getAddressDecoder()),
    ('poolCreatorAuthority', getAddressDecoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'createConfig instruction decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (CreateConfigInstructionData, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(0)).read(bytes, offset + 0);
    final (map, newOffset) = structDecoder.read(bytes, offset);
    if (newOffset != bytes.length) {
      throwInvalidByteLength(newOffset - offset, bytes.length - offset);
    }

    return (
      CreateConfigInstructionData(
        index: map['index']! as int,
        tradeFeeRate: map['tradeFeeRate']! as int,
        protocolFeeRate: map['protocolFeeRate']! as int,
        creatorFeeRate: map['creatorFeeRate']! as int,
        authority: map['authority']! as Address,
        poolCreatorAuthority: map['poolCreatorAuthority']! as Address,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() =>
      FixedSizeDecoder<CreateConfigInstructionData>(
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
      VariableSizeDecoder<CreateConfigInstructionData>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<CreateConfigInstructionData, CreateConfigInstructionData>
getCreateConfigInstructionDataCodec() {
  return combineCodec(
    getCreateConfigInstructionDataEncoder(),
    getCreateConfigInstructionDataDecoder(),
  );
}

/// Creates a [CreateConfig] instruction.
Instruction getCreateConfigInstruction({
  required Address programAddress,
  required Address payer,
  required Address upgradeAuthority,
  required Address programData,
  required Address ammConfig,
  required Address systemProgram,
  required int index,
  required int tradeFeeRate,
  required int protocolFeeRate,
  required int creatorFeeRate,
  required Address authority,
  required Address poolCreatorAuthority,
}) {
  final instructionData = CreateConfigInstructionData(
    index: index,
    tradeFeeRate: tradeFeeRate,
    protocolFeeRate: protocolFeeRate,
    creatorFeeRate: creatorFeeRate,
    authority: authority,
    poolCreatorAuthority: poolCreatorAuthority,
  );

  return Instruction(
    programAddress: programAddress,
    accounts: [
      AccountMeta(address: payer, role: AccountRole.writableSigner),
      AccountMeta(address: upgradeAuthority, role: AccountRole.readonlySigner),
      AccountMeta(address: programData, role: AccountRole.readonly),
      AccountMeta(address: ammConfig, role: AccountRole.writable),
      AccountMeta(address: systemProgram, role: AccountRole.readonly),
    ],
    data: getCreateConfigInstructionDataEncoder().encode(instructionData),
  );
}

/// Parses a [CreateConfig] instruction from raw instruction data.
CreateConfigInstructionData parseCreateConfigInstruction(
  Instruction instruction,
) {
  return getCreateConfigInstructionDataDecoder().decode(instruction.data!);
}
