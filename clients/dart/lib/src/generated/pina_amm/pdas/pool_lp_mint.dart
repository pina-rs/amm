// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';

@immutable
class PoolLpMintSeeds {
  const PoolLpMintSeeds({required this.pool});

  final Address pool;
}

/// Finds the program derived address for [PoolLpMint].
Future<(Address, int)> findPoolLpMintPda({
  required PoolLpMintSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'pool_lp_mint',
    getAddressEncoder().encode(seeds.pool),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
