// Auto-generated. Do not edit.
// ignore_for_file: type=lint

export 'event_log.dart';
export 'pool_created.dart';
export 'swapped.dart';
export 'liquidity_changed.dart';
export 'fees_collected.dart';

import 'event_log.dart';
import 'pool_created.dart';
import 'swapped.dart';
import 'liquidity_changed.dart';
import 'fees_collected.dart';

/// The program whose invocation frames emit the events decoded here.
const pinaAmmEventSourceAddress = 'pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV';

final _programInvokeLog = RegExp(r'^Program (\S+) invoke \[\d+\]$');
final _programExitLog = RegExp(r'^Program (\S+) (?:success|failed: .*)$');

/// Decode every `Program data:` line this program emitted in a transaction's
/// logs.
///
/// [logs] must be the complete, ordered log messages of one transaction. The
/// parser follows the runtime's `Program <address> invoke [n]` and
/// `Program <address> success` / `failed` frames and decodes a data line only
/// while [programAddress] is the innermost invoked program. Any program can
/// write a `Program data:` line with this program's discriminator, so data
/// lines from other programs (including ones this program invokes through CPI)
/// and lines outside any frame are skipped rather than trusted.
///
/// Unrelated lines are skipped. A line this program emitted that names an event
/// but carries a version no generated event describes throws instead of being
/// silently dropped. The per-event `parse*FromLog` helpers decode one line
/// without this attribution and are only safe for data already known to come
/// from this program.
List<PinaAmmEvent> parsePinaAmmEventsFromLogs(
  List<String> logs, {
  String programAddress = pinaAmmEventSourceAddress,
}) {
  final discovered = <PinaAmmEvent>[];
  final frames = <String>[];
  for (final log in logs) {
    final invoke = _programInvokeLog.firstMatch(log);
    if (invoke != null) {
      frames.add(invoke.group(1)!);
      continue;
    }
    if (_programExitLog.hasMatch(log)) {
      if (frames.isNotEmpty) {
        frames.removeLast();
      }
      continue;
    }
    if (frames.isEmpty || frames.last != programAddress) {
      continue;
    }
    final poolCreated = parsePoolCreatedEventFromLog(log);
    if (poolCreated != null) {
      discovered.add(poolCreated);
      continue;
    }
    final swapped = parseSwappedEventFromLog(log);
    if (swapped != null) {
      discovered.add(swapped);
      continue;
    }
    final liquidityChanged = parseLiquidityChangedEventFromLog(log);
    if (liquidityChanged != null) {
      discovered.add(liquidityChanged);
      continue;
    }
    final feesCollected = parseFeesCollectedEventFromLog(log);
    if (feesCollected != null) {
      discovered.add(feesCollected);
      continue;
    }
    final unknownVersion = _unrecognizedEventVersion(log);
    if (unknownVersion != null) {
      throw RangeError(unknownVersion);
    }
  }
  return discovered;
}

/// Explain a `Program data:` line that names a migration-aware event but that
/// no generated event claimed, or return null for an unrelated line.
String? _unrecognizedEventVersion(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null) {
    return null;
  }
  if (bytes.length >= 1 && bytes[0] == 1) {
    return bytes.length < 2
        ? 'event "poolCreated" log is too short for its version envelope'
        : 'event "poolCreated" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 2) {
    return bytes.length < 2
        ? 'event "swapped" log is too short for its version envelope'
        : 'event "swapped" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 3) {
    return bytes.length < 2
        ? 'event "liquidityChanged" log is too short for its version envelope'
        : 'event "liquidityChanged" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 4) {
    return bytes.length < 2
        ? 'event "feesCollected" log is too short for its version envelope'
        : 'event "feesCollected" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  return null;
}
