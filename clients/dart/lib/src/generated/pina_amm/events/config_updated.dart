// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `ConfigUpdated`.
class ConfigUpdatedEvent extends PinaAmmEvent {
  const ConfigUpdatedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.ammConfig,
    required this.newAuthority,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
  });

  final int discriminator;
  final int migrationVersion;
  final Address ammConfig;
  final Address newAuthority;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;

  @override
  String get name => 'configUpdated';

  String toString() =>
      'ConfigUpdatedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, ammConfig: ${ammConfig}, newAuthority: ${newAuthority}, tradeFeeRate: ${tradeFeeRate}, protocolFeeRate: ${protocolFeeRate}, creatorFeeRate: ${creatorFeeRate})';
}

/// The discriminator this event is emitted under.
const configUpdatedEventDiscriminator = 6;

/// The discriminator bytes as stored at offset zero.
const List<int> _configUpdatedEventDiscriminatorBytes = [6];

/// The migration version this event decodes.
const configUpdatedEventMigrationVersion = 0;

/// Exact current byte length of a `ConfigUpdated` record, envelope included.
const configUpdatedEventSize = 78;

/// Decode one `ConfigUpdated` record.
ConfigUpdatedEvent decodeConfigUpdatedEvent(Uint8List data) {
  if (data.length != configUpdatedEventSize) {
    throw RangeError(
      'expected exactly ${configUpdatedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 6) {
    throw RangeError(
      'the provided bytes do not match the "ConfigUpdated" event discriminator',
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
  final (v4, c4) = getU32Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU32Decoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU32Decoder().read(data, cursor);
  cursor = c6;

  return ConfigUpdatedEvent(
    discriminator: v0,
    migrationVersion: v1,
    ammConfig: v2,
    newAuthority: v3,
    tradeFeeRate: v4,
    protocolFeeRate: v5,
    creatorFeeRate: v6,
  );
}

/// A decoded `ConfigUpdated` log record.
typedef DecodedConfigUpdatedEvent = ConfigUpdatedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
ConfigUpdatedEvent? parseConfigUpdatedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _configUpdatedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != configUpdatedEventMigrationVersion) {
    return null;
  }
  return decodeConfigUpdatedEvent(bytes);
}
