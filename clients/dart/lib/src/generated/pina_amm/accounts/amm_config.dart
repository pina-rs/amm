// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:meta/meta.dart';
import 'package:solana_kit_accounts/solana_kit_accounts.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';

@immutable
class AmmConfig {
  const AmmConfig({
    required this.authority,
    required this.poolCreatorAuthority,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
    required this.index,
    required this.bump,
  }) : discriminator = 1,
       migrationVersion = 0;

  final int discriminator;
  final int migrationVersion;
  final Address authority;
  final Address poolCreatorAuthority;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;
  final int index;
  final int bump;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AmmConfig &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          migrationVersion == other.migrationVersion &&
          authority == other.authority &&
          poolCreatorAuthority == other.poolCreatorAuthority &&
          tradeFeeRate == other.tradeFeeRate &&
          protocolFeeRate == other.protocolFeeRate &&
          creatorFeeRate == other.creatorFeeRate &&
          index == other.index &&
          bump == other.bump;

  @override
  int get hashCode => Object.hash(
    discriminator,
    migrationVersion,
    authority,
    poolCreatorAuthority,
    tradeFeeRate,
    protocolFeeRate,
    creatorFeeRate,
    index,
    bump,
  );

  @override
  String toString() =>
      'AmmConfig(discriminator: $discriminator, migrationVersion: $migrationVersion, authority: $authority, poolCreatorAuthority: $poolCreatorAuthority, tradeFeeRate: $tradeFeeRate, protocolFeeRate: $protocolFeeRate, creatorFeeRate: $creatorFeeRate, index: $index, bump: $bump)';
}

Encoder<AmmConfig> getAmmConfigEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('migrationVersion', getU8Encoder()),
    ('authority', getAddressEncoder()),
    ('poolCreatorAuthority', getAddressEncoder()),
    ('tradeFeeRate', getU32Encoder()),
    ('protocolFeeRate', getU32Encoder()),
    ('creatorFeeRate', getU32Encoder()),
    ('index', getU16Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (AmmConfig value) => <String, Object?>{
      'discriminator': 1,
      'migrationVersion': 0,
      'authority': value.authority,
      'poolCreatorAuthority': value.poolCreatorAuthority,
      'tradeFeeRate': value.tradeFeeRate,
      'protocolFeeRate': value.protocolFeeRate,
      'creatorFeeRate': value.creatorFeeRate,
      'index': value.index,
      'bump': value.bump,
    },
  );
}

Decoder<AmmConfig> getAmmConfigDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('migrationVersion', getU8Decoder()),
    ('authority', getAddressDecoder()),
    ('poolCreatorAuthority', getAddressDecoder()),
    ('tradeFeeRate', getU32Decoder()),
    ('protocolFeeRate', getU32Decoder()),
    ('creatorFeeRate', getU32Decoder()),
    ('index', getU16Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'ammConfig account decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (AmmConfig, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(1)).read(bytes, offset + 0);
    final (storedMigrationVersion, _) = getU8Decoder().read(bytes, offset + 1);
    if (storedMigrationVersion != 0) {
      throw StateError(
        storedMigrationVersion < 0
            ? 'migration version mismatch: expected 0, received $storedMigrationVersion (the data predates this client; migrate it by sending a transaction to the program, or decode it with a client generated from an older IDL)'
            : 'migration version mismatch: expected 0, received $storedMigrationVersion (the data was written by a newer program; upgrade this client)',
      );
    }
    final (map, newOffset) = structDecoder.read(bytes, offset);

    return (
      AmmConfig(
        authority: map['authority']! as Address,
        poolCreatorAuthority: map['poolCreatorAuthority']! as Address,
        tradeFeeRate: map['tradeFeeRate']! as int,
        protocolFeeRate: map['protocolFeeRate']! as int,
        creatorFeeRate: map['creatorFeeRate']! as int,
        index: map['index']! as int,
        bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() => FixedSizeDecoder<AmmConfig>(
      fixedSize: structDecoder.fixedSize,
      read: (bytes, offset) {
        final bytesLength = bytes.length - offset;
        if (bytesLength < structDecoder.fixedSize) {
          throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
        }
        return readTopLevel(bytes, offset);
      },
    ),
    VariableSizeDecoder<Map<String, Object?>>() =>
      VariableSizeDecoder<AmmConfig>(
        read: readTopLevel,
        maxSize: structDecoder.maxSize,
      ),
  };
}

Codec<AmmConfig, AmmConfig> getAmmConfigCodec() {
  return combineCodec(getAmmConfigEncoder(), getAmmConfigDecoder());
}

Account<AmmConfig> decodeAmmConfig(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getAmmConfigDecoder());
}

/// The account schema version this client was generated from.
const int ammConfigMigrationVersion = 0;

/// Cheap envelope check for fetched `AmmConfig` bytes: returns true only when
/// the bytes carry this account's discriminator and a migration version older
/// than this client's schema — exactly the accounts [getMigrateInstruction]
/// can bring current. Decoding reports every other mismatch.
bool ammConfigNeedsMigration(List<int> data) {
  if (data.length < 2) {
    return false;
  }
  if (data[0] != 1) {
    return false;
  }
  return data[1] < 0;
}
