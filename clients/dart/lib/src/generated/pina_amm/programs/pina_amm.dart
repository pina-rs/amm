// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';
import 'package:solana_kit_errors/solana_kit_errors.dart';
import 'package:solana_kit_instructions/solana_kit_instructions.dart';

import '../instructions/instructions.dart';

/// The address of the PinaAmm program.
const pinaAmmProgramAddress = Address(
  'pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV',
);

/// Known accounts for the PinaAmm program.
enum PinaAmmAccount { ammConfig, pool }

/// Known instructions for the PinaAmm program.
enum PinaAmmInstruction {
  createConfig,
  updateConfig,
  createPool,
  deposit,
  withdraw,
  swapExactIn,
  swapExactOut,
  collectProtocolFees,
  collectCreatorFees,
  setPoolCreator,
  syncPool,
}

/// Identifies the type of a PinaAmm instruction.
PinaAmmInstruction identifyPinaAmmInstruction(Uint8List data) {
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return PinaAmmInstruction.createConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return PinaAmmInstruction.updateConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return PinaAmmInstruction.createPool;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return PinaAmmInstruction.deposit;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return PinaAmmInstruction.withdraw;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return PinaAmmInstruction.swapExactIn;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return PinaAmmInstruction.swapExactOut;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return PinaAmmInstruction.collectProtocolFees;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return PinaAmmInstruction.collectCreatorFees;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return PinaAmmInstruction.setPoolCreator;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return PinaAmmInstruction.syncPool;
  }

  throw SolanaError(SolanaErrorCode.programClientsFailedToIdentifyInstruction, {
    'instructionData': data,
    'programName': 'pinaAmm',
  });
}

/// A parsed instruction from the PinaAmm program.
sealed class ParsedPinaAmmInstruction {
  const ParsedPinaAmmInstruction(this.instructionType);

  final PinaAmmInstruction instructionType;
}

/// A parsed CreateConfig instruction.
final class ParsedCreateConfig extends ParsedPinaAmmInstruction {
  const ParsedCreateConfig({required this.data})
    : super(PinaAmmInstruction.createConfig);

  final CreateConfigInstructionData data;
}

/// A parsed UpdateConfig instruction.
final class ParsedUpdateConfig extends ParsedPinaAmmInstruction {
  const ParsedUpdateConfig({required this.data})
    : super(PinaAmmInstruction.updateConfig);

  final UpdateConfigInstructionData data;
}

/// A parsed CreatePool instruction.
final class ParsedCreatePool extends ParsedPinaAmmInstruction {
  const ParsedCreatePool({required this.data})
    : super(PinaAmmInstruction.createPool);

  final CreatePoolInstructionData data;
}

/// A parsed Deposit instruction.
final class ParsedDeposit extends ParsedPinaAmmInstruction {
  const ParsedDeposit({required this.data}) : super(PinaAmmInstruction.deposit);

  final DepositInstructionData data;
}

/// A parsed Withdraw instruction.
final class ParsedWithdraw extends ParsedPinaAmmInstruction {
  const ParsedWithdraw({required this.data})
    : super(PinaAmmInstruction.withdraw);

  final WithdrawInstructionData data;
}

/// A parsed SwapExactIn instruction.
final class ParsedSwapExactIn extends ParsedPinaAmmInstruction {
  const ParsedSwapExactIn({required this.data})
    : super(PinaAmmInstruction.swapExactIn);

  final SwapExactInInstructionData data;
}

/// A parsed SwapExactOut instruction.
final class ParsedSwapExactOut extends ParsedPinaAmmInstruction {
  const ParsedSwapExactOut({required this.data})
    : super(PinaAmmInstruction.swapExactOut);

  final SwapExactOutInstructionData data;
}

/// A parsed CollectProtocolFees instruction.
final class ParsedCollectProtocolFees extends ParsedPinaAmmInstruction {
  const ParsedCollectProtocolFees({required this.data})
    : super(PinaAmmInstruction.collectProtocolFees);

  final CollectProtocolFeesInstructionData data;
}

/// A parsed CollectCreatorFees instruction.
final class ParsedCollectCreatorFees extends ParsedPinaAmmInstruction {
  const ParsedCollectCreatorFees({required this.data})
    : super(PinaAmmInstruction.collectCreatorFees);

  final CollectCreatorFeesInstructionData data;
}

/// A parsed SetPoolCreator instruction.
final class ParsedSetPoolCreator extends ParsedPinaAmmInstruction {
  const ParsedSetPoolCreator({required this.data})
    : super(PinaAmmInstruction.setPoolCreator);

  final SetPoolCreatorInstructionData data;
}

/// A parsed SyncPool instruction.
final class ParsedSyncPool extends ParsedPinaAmmInstruction {
  const ParsedSyncPool({required this.data})
    : super(PinaAmmInstruction.syncPool);

  final SyncPoolInstructionData data;
}

/// Parses a PinaAmm instruction.
ParsedPinaAmmInstruction parsePinaAmmInstruction(Instruction instruction) {
  return switch (identifyPinaAmmInstruction(instruction.data ?? Uint8List(0))) {
    PinaAmmInstruction.createConfig => ParsedCreateConfig(
      data: parseCreateConfigInstruction(instruction),
    ),
    PinaAmmInstruction.updateConfig => ParsedUpdateConfig(
      data: parseUpdateConfigInstruction(instruction),
    ),
    PinaAmmInstruction.createPool => ParsedCreatePool(
      data: parseCreatePoolInstruction(instruction),
    ),
    PinaAmmInstruction.deposit => ParsedDeposit(
      data: parseDepositInstruction(instruction),
    ),
    PinaAmmInstruction.withdraw => ParsedWithdraw(
      data: parseWithdrawInstruction(instruction),
    ),
    PinaAmmInstruction.swapExactIn => ParsedSwapExactIn(
      data: parseSwapExactInInstruction(instruction),
    ),
    PinaAmmInstruction.swapExactOut => ParsedSwapExactOut(
      data: parseSwapExactOutInstruction(instruction),
    ),
    PinaAmmInstruction.collectProtocolFees => ParsedCollectProtocolFees(
      data: parseCollectProtocolFeesInstruction(instruction),
    ),
    PinaAmmInstruction.collectCreatorFees => ParsedCollectCreatorFees(
      data: parseCollectCreatorFeesInstruction(instruction),
    ),
    PinaAmmInstruction.setPoolCreator => ParsedSetPoolCreator(
      data: parseSetPoolCreatorInstruction(instruction),
    ),
    PinaAmmInstruction.syncPool => ParsedSyncPool(
      data: parseSyncPoolInstruction(instruction),
    ),
  };
}
