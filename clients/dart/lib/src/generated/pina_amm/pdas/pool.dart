// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'package:meta/meta.dart';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';

@immutable
class PoolSeeds {
  const PoolSeeds({
    required this.ammConfig,
    required this.mint0,
    required this.mint1,
  });

  final Address ammConfig;
  final Address mint0;
  final Address mint1;
}

/// Finds the program derived address for [Pool].
Future<(Address, int)> findPoolPda({
  required PoolSeeds seeds,
  required Address programAddress,
}) async {
  final seedValues = <Object>[
    'pool',
    getAddressEncoder().encode(seeds.ammConfig),
    getAddressEncoder().encode(seeds.mint0),
    getAddressEncoder().encode(seeds.mint1),
  ];

  return getProgramDerivedAddress(
    programAddress: programAddress,
    seeds: seedValues,
  );
}
