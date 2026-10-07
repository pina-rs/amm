// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

@immutable
class AmmConfigSeeds {
  const AmmConfigSeeds({required this.index});

  final int index;
}

/// Finds the program derived address for [AmmConfig].
Future<(Address, int)> findAmmConfigPda({
  required AmmConfigSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'amm_config',
    getU16Encoder().encode(seeds.index),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
