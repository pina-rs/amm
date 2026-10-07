// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';

@immutable
class PoolVaultSeeds {
  const PoolVaultSeeds({required this.pool, required this.mint});

  final Address pool;
  final Address mint;
}

/// Finds the program derived address for [PoolVault].
Future<(Address, int)> findPoolVaultPda({
  required PoolVaultSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'pool_vault',
    getAddressEncoder().encode(seeds.pool),
    getAddressEncoder().encode(seeds.mint),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
