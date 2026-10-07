// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `PoolCreated`.
class PoolCreatedEvent extends PinaAmmEvent {
  const PoolCreatedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.ammConfig,
    required this.creator,
    required this.mint0,
    required this.mint1,
    required this.lpMint,
    required this.amount0,
    required this.amount1,
    required this.lpSupply,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final Address ammConfig;
  final Address creator;
  final Address mint0;
  final Address mint1;
  final Address lpMint;
  final BigInt amount0;
  final BigInt amount1;
  final BigInt lpSupply;

  @override
  String get name => 'poolCreated';

  String toString() =>
      'PoolCreatedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, ammConfig: ${ammConfig}, creator: ${creator}, mint0: ${mint0}, mint1: ${mint1}, lpMint: ${lpMint}, amount0: ${amount0}, amount1: ${amount1}, lpSupply: ${lpSupply})';
}

/// The discriminator this event is emitted under.
const poolCreatedEventDiscriminator = 1;

/// The discriminator bytes as stored at offset zero.
const List<int> _poolCreatedEventDiscriminatorBytes = [1];

/// The migration version this event decodes.
const poolCreatedEventMigrationVersion = 0;

/// Exact current byte length of a `PoolCreated` record, envelope included.
const poolCreatedEventSize = 218;

/// Decode one `PoolCreated` record.
PoolCreatedEvent decodePoolCreatedEvent(Uint8List data) {
  if (data.length != poolCreatedEventSize) {
    throw RangeError(
      'expected exactly ${poolCreatedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 1) {
    throw RangeError(
      'the provided bytes do not match the "PoolCreated" event discriminator',
    );
  }
  final (v1, c1) = getU8Decoder().read(data, cursor);
  cursor = c1;
  if (v1 != 0) {
    throw RangeError(
      v1 < 0
          ? 'event migration version mismatch: expected 0, received $v1 (decode it with the event for that version)'
          : 'event migration version mismatch: expected 0, received $v1 (the log was written by a newer program; upgrade this client)',
    );
  }
  final (v2, c2) = getAddressDecoder().read(data, cursor);
  cursor = c2;
  final (v3, c3) = getAddressDecoder().read(data, cursor);
  cursor = c3;
  final (v4, c4) = getAddressDecoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getAddressDecoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getAddressDecoder().read(data, cursor);
  cursor = c6;
  final (v7, c7) = getAddressDecoder().read(data, cursor);
  cursor = c7;
  final (v8, c8) = getU64Decoder().read(data, cursor);
  cursor = c8;
  final (v9, c9) = getU64Decoder().read(data, cursor);
  cursor = c9;
  final (v10, c10) = getU64Decoder().read(data, cursor);
  cursor = c10;

  return PoolCreatedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    ammConfig: v3,
    creator: v4,
    mint0: v5,
    mint1: v6,
    lpMint: v7,
    amount0: v8,
    amount1: v9,
    lpSupply: v10,
  );
}

/// A decoded `PoolCreated` log record.
typedef DecodedPoolCreatedEvent = PoolCreatedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
PoolCreatedEvent? parsePoolCreatedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _poolCreatedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != poolCreatedEventMigrationVersion) {
    return null;
  }
  return decodePoolCreatedEvent(bytes);
}
