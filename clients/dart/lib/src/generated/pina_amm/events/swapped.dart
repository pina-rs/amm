// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `Swapped`.
class SwappedEvent extends PinaAmmEvent {
  const SwappedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.pool,
    required this.trader,
    required this.zeroForOne,
    required this.amountIn,
    required this.amountOut,
    required this.tradeFee,
    required this.protocolFee,
    required this.creatorFee,
    required this.creatorFeeOnInput,
    required this.reserve0,
    required this.reserve1,
  });

  final int discriminator;
  final int migrationVersion;
  final Address pool;
  final Address trader;
  final int zeroForOne;
  final BigInt amountIn;
  final BigInt amountOut;
  final BigInt tradeFee;
  final BigInt protocolFee;
  final BigInt creatorFee;
  final int creatorFeeOnInput;
  final BigInt reserve0;
  final BigInt reserve1;

  @override
  String get name => 'swapped';

  String toString() =>
      'SwappedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, pool: ${pool}, trader: ${trader}, zeroForOne: ${zeroForOne}, amountIn: ${amountIn}, amountOut: ${amountOut}, tradeFee: ${tradeFee}, protocolFee: ${protocolFee}, creatorFee: ${creatorFee}, creatorFeeOnInput: ${creatorFeeOnInput}, reserve0: ${reserve0}, reserve1: ${reserve1})';
}

/// The discriminator this event is emitted under.
const swappedEventDiscriminator = 2;

/// The discriminator bytes as stored at offset zero.
const List<int> _swappedEventDiscriminatorBytes = [2];

/// The migration version this event decodes.
const swappedEventMigrationVersion = 0;

/// Exact current byte length of a `Swapped` record, envelope included.
const swappedEventSize = 124;

/// Decode one `Swapped` record.
SwappedEvent decodeSwappedEvent(Uint8List data) {
  if (data.length != swappedEventSize) {
    throw RangeError(
      'expected exactly ${swappedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 2) {
    throw RangeError(
      'the provided bytes do not match the "Swapped" event discriminator',
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
  final (v9, c9) = getU64Decoder().read(data, cursor);
  cursor = c9;
  final (v10, c10) = getU8Decoder().read(data, cursor);
  cursor = c10;
  final (v11, c11) = getU64Decoder().read(data, cursor);
  cursor = c11;
  final (v12, c12) = getU64Decoder().read(data, cursor);
  cursor = c12;

  return SwappedEvent(
    discriminator: v0,
    migrationVersion: v1,
    pool: v2,
    trader: v3,
    zeroForOne: v4,
    amountIn: v5,
    amountOut: v6,
    tradeFee: v7,
    protocolFee: v8,
    creatorFee: v9,
    creatorFeeOnInput: v10,
    reserve0: v11,
    reserve1: v12,
  );
}

/// A decoded `Swapped` log record.
typedef DecodedSwappedEvent = SwappedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
SwappedEvent? parseSwappedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _swappedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != swappedEventMigrationVersion) {
    return null;
  }
  return decodeSwappedEvent(bytes);
}
