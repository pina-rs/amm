// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `ConfigCreated`.
class ConfigCreatedEvent extends PinaAmmEvent {
  const ConfigCreatedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.ammConfig,
    required this.authority,
    required this.poolCreatorAuthority,
    required this.tradeFeeRate,
    required this.protocolFeeRate,
    required this.creatorFeeRate,
    required this.index,
  });

  final int discriminator;
  final int migrationVersion;
  final Address ammConfig;
  final Address authority;
  final Address poolCreatorAuthority;
  final int tradeFeeRate;
  final int protocolFeeRate;
  final int creatorFeeRate;
  final int index;

  @override
  String get name => 'configCreated';

  String toString() =>
      'ConfigCreatedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, ammConfig: ${ammConfig}, authority: ${authority}, poolCreatorAuthority: ${poolCreatorAuthority}, tradeFeeRate: ${tradeFeeRate}, protocolFeeRate: ${protocolFeeRate}, creatorFeeRate: ${creatorFeeRate}, index: ${index})';
}

/// The discriminator this event is emitted under.
const configCreatedEventDiscriminator = 8;

/// The discriminator bytes as stored at offset zero.
const List<int> _configCreatedEventDiscriminatorBytes = [8];

/// The migration version this event decodes.
const configCreatedEventMigrationVersion = 0;

/// Exact current byte length of a `ConfigCreated` record, envelope included.
const configCreatedEventSize = 112;

/// Decode one `ConfigCreated` record.
ConfigCreatedEvent decodeConfigCreatedEvent(Uint8List data) {
  if (data.length != configCreatedEventSize) {
    throw RangeError(
      'expected exactly ${configCreatedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 8) {
    throw RangeError(
      'the provided bytes do not match the "ConfigCreated" event discriminator',
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
  final (v5, c5) = getU32Decoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU32Decoder().read(data, cursor);
  cursor = c6;
  final (v7, c7) = getU32Decoder().read(data, cursor);
  cursor = c7;
  final (v8, c8) = getU16Decoder().read(data, cursor);
  cursor = c8;

  return ConfigCreatedEvent(
    discriminator: v0,
    migrationVersion: v1,
    ammConfig: v2,
    authority: v3,
    poolCreatorAuthority: v4,
    tradeFeeRate: v5,
    protocolFeeRate: v6,
    creatorFeeRate: v7,
    index: v8,
  );
}

/// A decoded `ConfigCreated` log record.
typedef DecodedConfigCreatedEvent = ConfigCreatedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
ConfigCreatedEvent? parseConfigCreatedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _configCreatedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != configCreatedEventMigrationVersion) {
    return null;
  }
  return decodeConfigCreatedEvent(bytes);
}
