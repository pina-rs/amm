// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `LiquidityChanged`.
class LiquidityChangedEvent extends PinaAmmEvent {
  const LiquidityChangedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.owner,
    required this.isDeposit,
    required this.lpAmount,
    required this.amount0,
    required this.amount1,
    required this.lpSupply,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final Address owner;
  final int isDeposit;
  final BigInt lpAmount;
  final BigInt amount0;
  final BigInt amount1;
  final BigInt lpSupply;

  @override
  String get name => 'liquidityChanged';

  String toString() =>
      'LiquidityChangedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, owner: ${owner}, isDeposit: ${isDeposit}, lpAmount: ${lpAmount}, amount0: ${amount0}, amount1: ${amount1}, lpSupply: ${lpSupply})';
}

/// The discriminator this event is emitted under.
const liquidityChangedEventDiscriminator = 3;

/// The discriminator bytes as stored at offset zero.
const List<int> _liquidityChangedEventDiscriminatorBytes = [3];

/// The migration version this event decodes.
const liquidityChangedEventMigrationVersion = 0;

/// Exact current byte length of a `LiquidityChanged` record, envelope included.
const liquidityChangedEventSize = 99;

/// Decode one `LiquidityChanged` record.
LiquidityChangedEvent decodeLiquidityChangedEvent(Uint8List data) {
  if (data.length != liquidityChangedEventSize) {
    throw RangeError(
      'expected exactly ${liquidityChangedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 3) {
    throw RangeError(
      'the provided bytes do not match the "LiquidityChanged" event discriminator',
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
  final (v7, c7) = getU64Decoder().read(data, cursor);
  cursor = c7;
  final (v8, c8) = getU64Decoder().read(data, cursor);
  cursor = c8;

  return LiquidityChangedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    owner: v3,
    isDeposit: v4,
    lpAmount: v5,
    amount0: v6,
    amount1: v7,
    lpSupply: v8,
  );
}

/// A decoded `LiquidityChanged` log record.
typedef DecodedLiquidityChangedEvent = LiquidityChangedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
LiquidityChangedEvent? parseLiquidityChangedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _liquidityChangedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != liquidityChangedEventMigrationVersion) {
    return null;
  }
  return decodeLiquidityChangedEvent(bytes);
}
