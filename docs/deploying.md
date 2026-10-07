# Deploying

This is the runbook for deploying the program and creating its fee tiers. Deploying is a manual, deliberate step: no script, hook, or CI job deploys automatically.

## Keys

| Key               | Purpose                                                              | Where it lives                                                                 |
| ----------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Program keypair   | The program's address, `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV` | Offline backup; locally at `target/deploy/pina_amm-keypair.json` (git-ignored) |
| Upgrade authority | Upgrades the program and creates fee tiers                           | A hardware wallet or multisig                                                  |
| Fee payer         | Pays deployment rent and fees                                        | Any funded wallet                                                              |

Never commit a keypair. `pina keys show` compares the local keypair with `declare_id!` in `programs/pina_amm/src/lib.rs`.

## Deploy

```sh
export PINA_AMM_PROGRAM_KEYPAIR=~/secure/pina_amm-keypair.json
export PINA_AMM_UPGRADE_AUTHORITY=~/secure/upgrade-authority.json
export PINA_AMM_DEPLOY_KEYPAIR=~/.config/solana/id.json

devenv shell deploy:program devnet
devenv shell deploy:program mainnet-beta --allow-mainnet
```

`deploy:program` wraps `pina deploy --build`, which builds the artifact, prints the complete plan, asks for confirmation, and **records the migration publication** in `programs/pina_amm/migrations/publications.json`. Commit that file after every deployment: it freezes the schema versions now live on chain, so a later change to an account layout must ship a migration instead of silently rewriting history. Never deploy with `solana program deploy` directly.

Before upgrading a live deployment, rehearse it against recent traffic:

```sh
devenv shell deploy:program mainnet-beta --allow-mainnet --rehearse
```

## Create fee tiers

Tiers can only be created by the upgrade authority:

```sh
pina-amm -u mainnet -k ~/secure/upgrade-authority.json \
  config create --index 0 --trade-fee-rate 2500 --protocol-fee-rate 160000 \
  --authority <TIER_MULTISIG>
```

Pass `--authority` so day-to-day tier management and protocol-fee collection move off the upgrade key immediately.

### A tier for a bonding curve

The [bonding curve](https://github.com/pina-rs/bonding_curve) migrates graduated launches into the AMM. It needs a tier reserved for its AMM authority PDA so nobody can create its pools first:

```sh
# The curve's AMM authority: PDA [b"amm_authority"] under the curve program
# CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9, which is
# ALk5JUXnbaYwVeHG4ymAVVfPKW1xaUTCrSyrvKiq7CGZ.
pina-amm -u mainnet -k ~/secure/upgrade-authority.json \
  config create --index 100 --trade-fee-rate 2500 --protocol-fee-rate 160000 \
  --creator-fee-rate 5000 --pool-creator-authority ALk5JUXnbaYwVeHG4ymAVVfPKW1xaUTCrSyrvKiq7CGZ \
  --authority <TIER_MULTISIG>
```

Launchpad operators then pass tier 100's address (printed by `pina-amm config show --index 100`) as `--amm-config` when they create a curve configuration.

## Verify

```sh
pina-amm -u mainnet config show --index 0
pina build --verify     # deterministic build to compare with the deployed bytes
```

## Running your own deployment

Nothing in the program is tied to this repository's deployment. To run a separate instance, generate a new identity with `pina keys new`, run `pina migrations create` to rebind the migration history to the new address, regenerate the clients, and deploy. Your upgrade authority then controls your tiers.
