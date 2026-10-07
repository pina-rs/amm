// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `FeesCollected`.
class FeesCollectedEvent extends PinaAmmEvent {
  const FeesCollectedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.collector,
    required this.isProtocol,
    required this.amount0,
    required this.amount1,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final Address collector;
  final int isProtocol;
  final BigInt amount0;
  final BigInt amount1;

  @override
  String get name => 'feesCollected';

  String toString() =>
      'FeesCollectedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, collector: ${collector}, isProtocol: ${isProtocol}, amount0: ${amount0}, amount1: ${amount1})';
}

/// The discriminator this event is emitted under.
const feesCollectedEventDiscriminator = 4;

/// The discriminator bytes as stored at offset zero.
const List<int> _feesCollectedEventDiscriminatorBytes = [4];

/// The migration version this event decodes.
const feesCollectedEventMigrationVersion = 0;

/// Exact current byte length of a `FeesCollected` record, envelope included.
const feesCollectedEventSize = 83;

/// Decode one `FeesCollected` record.
FeesCollectedEvent decodeFeesCollectedEvent(Uint8List data) {
  if (data.length != feesCollectedEventSize) {
    throw RangeError(
      'expected exactly ${feesCollectedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 4) {
    throw RangeError(
      'the provided bytes do not match the "FeesCollected" event discriminator',
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
  final (v4, c4) = getU8Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU64Decoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU64Decoder().read(data, cursor);
  cursor = c6;

  return FeesCollectedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    collector: v3,
    isProtocol: v4,
    amount0: v5,
    amount1: v6,
  );
}

/// A decoded `FeesCollected` log record.
typedef DecodedFeesCollectedEvent = FeesCollectedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
FeesCollectedEvent? parseFeesCollectedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _feesCollectedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != feesCollectedEventMigrationVersion) {
    return null;
  }
  return decodeFeesCollectedEvent(bytes);
}
