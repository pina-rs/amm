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
class Pool {
  const Pool({
    required this.ammConfig,
    required this.creator,
    required this.mint0,
    required this.mint1,
    required this.vault0,
    required this.vault1,
    required this.lpMint,
    required this.lpSupply,
    required this.protocolFees0,
    required this.protocolFees1,
    required this.creatorFees0,
    required this.creatorFees1,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
    required this.creatorFeeMode,
    required this.bump,
  }) : discriminator = 2,
       migrationVersion = 0;

  final int discriminator;
  final int migrationVersion;
  final Address ammConfig;
  final Address creator;
  final Address mint0;
  final Address mint1;
  final Address vault0;
  final Address vault1;
  final Address lpMint;
  final BigInt lpSupply;
  final BigInt protocolFees0;
  final BigInt protocolFees1;
  final BigInt creatorFees0;
  final BigInt creatorFees1;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;
  final int creatorFeeMode;
  final int bump;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Pool &&
          runtimeType == other.runtimeType &&
          discriminator == other.discriminator &&
          migrationVersion == other.migrationVersion &&
          ammConfig == other.ammConfig &&
          creator == other.creator &&
          mint0 == other.mint0 &&
          mint1 == other.mint1 &&
          vault0 == other.vault0 &&
          vault1 == other.vault1 &&
          lpMint == other.lpMint &&
          lpSupply == other.lpSupply &&
          protocolFees0 == other.protocolFees0 &&
          protocolFees1 == other.protocolFees1 &&
          creatorFees0 == other.creatorFees0 &&
          creatorFees1 == other.creatorFees1 &&
          tradeFeeRate == other.tradeFeeRate &&
          protocolFeeRate == other.protocolFeeRate &&
          creatorFeeRate == other.creatorFeeRate &&
          creatorFeeMode == other.creatorFeeMode &&
          bump == other.bump;

  @override
  int get hashCode => Object.hash(
    discriminator,
    migrationVersion,
    ammConfig,
    creator,
    mint0,
    mint1,
    vault0,
    vault1,
    lpMint,
    lpSupply,
    protocolFees0,
    protocolFees1,
    creatorFees0,
    creatorFees1,
    tradeFeeRate,
    protocolFeeRate,
    creatorFeeRate,
    creatorFeeMode,
    bump,
  );

  @override
  String toString() =>
      'Pool(discriminator: $discriminator, migrationVersion: $migrationVersion, ammConfig: $ammConfig, creator: $creator, mint0: $mint0, mint1: $mint1, vault0: $vault0, vault1: $vault1, lpMint: $lpMint, lpSupply: $lpSupply, protocolFees0: $protocolFees0, protocolFees1: $protocolFees1, creatorFees0: $creatorFees0, creatorFees1: $creatorFees1, tradeFeeRate: $tradeFeeRate, protocolFeeRate: $protocolFeeRate, creatorFeeRate: $creatorFeeRate, creatorFeeMode: $creatorFeeMode, bump: $bump)';
}

Encoder<Pool> getPoolEncoder() {
  final structEncoder = getStructEncoder(<(String, Encoder<Object?>)>[
    ('discriminator', getU8Encoder()),
    ('migrationVersion', getU8Encoder()),
    ('ammConfig', getAddressEncoder()),
    ('creator', getAddressEncoder()),
    ('mint0', getAddressEncoder()),
    ('mint1', getAddressEncoder()),
    ('vault0', getAddressEncoder()),
    ('vault1', getAddressEncoder()),
    ('lpMint', getAddressEncoder()),
    ('lpSupply', getU64Encoder()),
    ('protocolFees0', getU64Encoder()),
    ('protocolFees1', getU64Encoder()),
    ('creatorFees0', getU64Encoder()),
    ('creatorFees1', getU64Encoder()),
    ('tradeFeeRate', getU32Encoder()),
    ('protocolFeeRate', getU32Encoder()),
    ('creatorFeeRate', getU32Encoder()),
    ('creatorFeeMode', getU8Encoder()),
    ('bump', getU8Encoder()),
  ]);

  return transformEncoder(
    structEncoder,
    (Pool value) => <String, Object?>{
      'discriminator': 2,
      'migrationVersion': 0,
      'ammConfig': value.ammConfig,
      'creator': value.creator,
      'mint0': value.mint0,
      'mint1': value.mint1,
      'vault0': value.vault0,
      'vault1': value.vault1,
      'lpMint': value.lpMint,
      'lpSupply': value.lpSupply,
      'protocolFees0': value.protocolFees0,
      'protocolFees1': value.protocolFees1,
      'creatorFees0': value.creatorFees0,
      'creatorFees1': value.creatorFees1,
      'tradeFeeRate': value.tradeFeeRate,
      'protocolFeeRate': value.protocolFeeRate,
      'creatorFeeRate': value.creatorFeeRate,
      'creatorFeeMode': value.creatorFeeMode,
      'bump': value.bump,
    },
  );
}

Decoder<Pool> getPoolDecoder() {
  final structDecoder = getStructDecoder(<(String, Decoder<Object?>)>[
    ('discriminator', getU8Decoder()),
    ('migrationVersion', getU8Decoder()),
    ('ammConfig', getAddressDecoder()),
    ('creator', getAddressDecoder()),
    ('mint0', getAddressDecoder()),
    ('mint1', getAddressDecoder()),
    ('vault0', getAddressDecoder()),
    ('vault1', getAddressDecoder()),
    ('lpMint', getAddressDecoder()),
    ('lpSupply', getU64Decoder()),
    ('protocolFees0', getU64Decoder()),
    ('protocolFees1', getU64Decoder()),
    ('creatorFees0', getU64Decoder()),
    ('creatorFees1', getU64Decoder()),
    ('tradeFeeRate', getU32Decoder()),
    ('protocolFeeRate', getU32Decoder()),
    ('creatorFeeRate', getU32Decoder()),
    ('creatorFeeMode', getU8Decoder()),
    ('bump', getU8Decoder()),
  ]);

  Never throwInvalidByteLength(int expected, int bytesLength) {
    throw SolanaError(SolanaErrorCode.codecsInvalidByteLength, {
      'codecDescription': 'pool account decoder',
      'expected': expected,
      'bytesLength': bytesLength,
    });
  }

  (Pool, int) readTopLevel(Uint8List bytes, int offset) {
    getConstantDecoder(getU8Encoder().encode(2)).read(bytes, offset + 0);
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
      Pool(
        ammConfig: map['ammConfig']! as Address,
        creator: map['creator']! as Address,
        mint0: map['mint0']! as Address,
        mint1: map['mint1']! as Address,
        vault0: map['vault0']! as Address,
        vault1: map['vault1']! as Address,
        lpMint: map['lpMint']! as Address,
        lpSupply: map['lpSupply']! as BigInt,
        protocolFees0: map['protocolFees0']! as BigInt,
        protocolFees1: map['protocolFees1']! as BigInt,
        creatorFees0: map['creatorFees0']! as BigInt,
        creatorFees1: map['creatorFees1']! as BigInt,
        tradeFeeRate: map['tradeFeeRate']! as int,
        protocolFeeRate: map['protocolFeeRate']! as int,
        creatorFeeRate: map['creatorFeeRate']! as int,
        creatorFeeMode: map['creatorFeeMode']! as int,
        bump: map['bump']! as int,
      ),
      newOffset,
    );
  }

  return switch (structDecoder) {
    FixedSizeDecoder<Map<String, Object?>>() => FixedSizeDecoder<Pool>(
      fixedSize: structDecoder.fixedSize,
      read: (bytes, offset) {
        final bytesLength = bytes.length - offset;
        if (bytesLength < structDecoder.fixedSize) {
          throwInvalidByteLength(structDecoder.fixedSize, bytesLength);
        }
        return readTopLevel(bytes, offset);
      },
    ),
    VariableSizeDecoder<Map<String, Object?>>() => VariableSizeDecoder<Pool>(
      read: readTopLevel,
      maxSize: structDecoder.maxSize,
    ),
  };
}

Codec<Pool, Pool> getPoolCodec() {
  return combineCodec(getPoolEncoder(), getPoolDecoder());
}

Account<Pool> decodePool(EncodedAccount encodedAccount) {
  return decodeAccount(encodedAccount, getPoolDecoder());
}

/// The account schema version this client was generated from.
const int poolMigrationVersion = 0;

/// Cheap envelope check for fetched `Pool` bytes: returns true only when
/// the bytes carry this account's discriminator and a migration version older
/// than this client's schema — exactly the accounts [getMigrateInstruction]
/// can bring current. Decoding reports every other mismatch.
bool poolNeedsMigration(List<int> data) {
  if (data.length < 2) {
    return false;
  }
  if (data[0] != 2) {
    return false;
  }
  return data[1] < 0;
}
