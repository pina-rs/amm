// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `PoolCreatorChanged`.
class PoolCreatorChangedEvent extends PinaAmmEvent {
  const PoolCreatorChangedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.previousCreator,
    required this.newCreator,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final Address previousCreator;
  final Address newCreator;

  @override
  String get name => 'poolCreatorChanged';

  String toString() =>
      'PoolCreatorChangedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, previousCreator: ${previousCreator}, newCreator: ${newCreator})';
}

/// The discriminator this event is emitted under.
const poolCreatorChangedEventDiscriminator = 7;

/// The discriminator bytes as stored at offset zero.
const List<int> _poolCreatorChangedEventDiscriminatorBytes = [7];

/// The migration version this event decodes.
const poolCreatorChangedEventMigrationVersion = 0;

/// Exact current byte length of a `PoolCreatorChanged` record, envelope included.
const poolCreatorChangedEventSize = 98;

/// Decode one `PoolCreatorChanged` record.
PoolCreatorChangedEvent decodePoolCreatorChangedEvent(Uint8List data) {
  if (data.length != poolCreatorChangedEventSize) {
    throw RangeError(
      'expected exactly ${poolCreatorChangedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 7) {
    throw RangeError(
      'the provided bytes do not match the "PoolCreatorChanged" event discriminator',
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

  return PoolCreatorChangedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    previousCreator: v3,
    newCreator: v4,
  );
}

/// A decoded `PoolCreatorChanged` log record.
typedef DecodedPoolCreatorChangedEvent = PoolCreatorChangedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
PoolCreatorChangedEvent? parsePoolCreatorChangedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _poolCreatorChangedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != poolCreatorChangedEventMigrationVersion) {
    return null;
  }
  return decodePoolCreatorChangedEvent(bytes);
}
