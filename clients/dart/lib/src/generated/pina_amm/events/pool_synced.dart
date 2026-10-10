// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `PoolSynced`.
class PoolSyncedEvent extends PinaAmmEvent {
  const PoolSyncedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.reserve0,
    required this.reserve1,
    required this.price0CumulativeLast,
    required this.lastUpdateTimestamp,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final BigInt reserve0;
  final BigInt reserve1;
  final BigInt price0CumulativeLast;
  final BigInt lastUpdateTimestamp;

  @override
  String get name => 'poolSynced';

  String toString() =>
      'PoolSyncedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, reserve0: ${reserve0}, reserve1: ${reserve1}, price0CumulativeLast: ${price0CumulativeLast}, lastUpdateTimestamp: ${lastUpdateTimestamp})';
}

/// The discriminator this event is emitted under.
const poolSyncedEventDiscriminator = 5;

/// The discriminator bytes as stored at offset zero.
const List<int> _poolSyncedEventDiscriminatorBytes = [5];

/// The migration version this event decodes.
const poolSyncedEventMigrationVersion = 0;

/// Exact current byte length of a `PoolSynced` record, envelope included.
const poolSyncedEventSize = 74;

/// Decode one `PoolSynced` record.
PoolSyncedEvent decodePoolSyncedEvent(Uint8List data) {
  if (data.length != poolSyncedEventSize) {
    throw RangeError(
      'expected exactly ${poolSyncedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 5) {
    throw RangeError(
      'the provided bytes do not match the "PoolSynced" event discriminator',
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
  final (v3, c3) = getU64Decoder().read(data, cursor);
  cursor = c3;
  final (v4, c4) = getU64Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU128Decoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU64Decoder().read(data, cursor);
  cursor = c6;

  return PoolSyncedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    reserve0: v3,
    reserve1: v4,
    price0CumulativeLast: v5,
    lastUpdateTimestamp: v6,
  );
}

/// A decoded `PoolSynced` log record.
typedef DecodedPoolSyncedEvent = PoolSyncedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
PoolSyncedEvent? parsePoolSyncedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _poolSyncedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != poolSyncedEventMigrationVersion) {
    return null;
  }
  return decodePoolSyncedEvent(bytes);
}
