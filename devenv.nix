{ pkgs, inputs, ... }:
let
  custom = inputs.ifiokjr-nixpkgs.packages.${pkgs.stdenv.hostPlatform.system};
in
{
  packages = with pkgs; [
    actionlint
    cargo-deny
    curl
    custom.agave
    custom.monochange
    custom.pina
    dart
    dprint
    git
    libiconv
    nixfmt
    nodejs_24
    pnpm
    rustup
    zlib
  ];

  env = {
    PINA_BPF_TOOLCHAIN = "nightly-2025-11-20";
    SBF_TOOLS_VERSION = "v1.54";
  };

  scripts = {
    "install:all".exec = ''
      set -euo pipefail
      pnpm install --frozen-lockfile
      (cd clients/dart && dart pub get)
    '';

    "build:program".exec = ''
      set -euo pipefail
      RUST_LOG=error pina build --project programs/pina_amm
    '';

    "build:clients".exec = ''
      set -euo pipefail
      cargo build --workspace --locked
      pnpm --dir clients/typescript/pina_amm build
    '';

    # Regenerate every client from the program IDL. Generated sources are
    # never edited by hand; manifests and entrypoints are preserved.
    "generate:clients".exec = ''
      set -euo pipefail
      pina migrations check --project programs/pina_amm
      # Pina refuses to write through symlinks, and pnpm links the TypeScript
      # client's dependencies into its directory. Remove them first and
      # reinstall afterwards.
      rm -rf -- clients/typescript/pina_amm/node_modules
      pina generate --project programs/pina_amm --output clients --npx node
      pnpm install --frozen-lockfile >/dev/null
      dart format clients/dart >/dev/null
      dprint fmt --allow-no-files 'clients/**/*.{ts,json,toml}' >/dev/null
    '';

    "check:clients".exec = ''
      set -euo pipefail
      generate:clients
      if ! git diff --quiet -- clients; then
        echo "Generated clients drifted from the program. Run generate:clients and commit the result." >&2
        git --no-pager diff --stat -- clients >&2
        exit 1
      fi
    '';

    "test:unit".exec = ''
      set -euo pipefail
      cargo test --workspace --all-features --locked
      pnpm --dir clients/typescript/pina_amm test
      (cd clients/dart && dart test)
    '';

    # The end-to-end suite deploys the real SBF artifact to an offline
    # Surfnet and drives it through the generated Rust client and the CLI.
    "test:surfpool".exec = ''
      set -euo pipefail
      build:program
      cargo build -p pina_amm_cli --locked
      PINA_SBF_ARTIFACT="$PWD/target/deploy/pina_amm.so" \
        PINA_AMM_CLI="$PWD/target/debug/pina-amm" \
        cargo test \
          --manifest-path programs/pina_amm/tests/surfpool/Cargo.toml \
          --locked \
          -- \
          --ignored \
          --nocapture
    '';

    "lint:all".exec = ''
      set -euo pipefail
      pina migrations check --project programs/pina_amm
      if [ "$(uname -s)-$(uname -m)" = "Darwin-x86_64" ]; then
        echo "Skipping Pina security lints: no Intel macOS lint driver is published."
      else
        rustup toolchain install --profile minimal
        if [ "$(uname -s)" = "Darwin" ]; then
          # Pina's lint driver puts the rustup toolchain's libLLVM on the dyld
          # path, which the Nix clang linker then loads and aborts on. Link
          # host build scripts with the system linker for this call only.
          CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc \
            pina lint --project programs/pina_amm
        else
          pina lint --project programs/pina_amm
        fi
      fi
      cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
      cargo clippy \
        --manifest-path programs/pina_amm/tests/surfpool/Cargo.toml \
        --all-targets \
        --locked \
        -- \
        -D warnings
      RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
      dprint check
      actionlint
      pnpm --dir clients/typescript/pina_amm check
      (cd clients/dart && dart analyze --fatal-infos --fatal-warnings)
      monochange check
    '';

    "fix:format".exec = ''
      set -euo pipefail
      dprint fmt
      dart format clients/dart >/dev/null
    '';

    "security:audit".exec = ''
      set -euo pipefail
      cargo deny check -D warnings advisories bans sources
      pnpm audit --audit-level high
    '';

    # Everything CI runs, in the order it fails fastest.
    "verify:all".exec = ''
      set -euo pipefail
      lint:all
      check:clients
      test:unit
      test:surfpool
      security:audit
    '';

    # Manual release step: never call from another script, hook, or CI job.
    # `pina deploy` records the migration publication receipt that freezes the
    # deployed schema versions; never deploy with `solana program deploy`
    # directly. See docs/deploying.md for the runbook.
    "deploy:program".exec = ''
      set -euo pipefail
      cluster="''${1:?usage: deploy:program <devnet|mainnet-beta> [extra pina deploy flags]}"
      shift
      pina deploy \
        --project programs/pina_amm \
        --build \
        --cluster "$cluster" \
        --program-keypair "''${PINA_AMM_PROGRAM_KEYPAIR:?set PINA_AMM_PROGRAM_KEYPAIR to the program-id keypair file}" \
        --upgrade-authority "''${PINA_AMM_UPGRADE_AUTHORITY:?set PINA_AMM_UPGRADE_AUTHORITY to the upgrade-authority keypair file}" \
        --payer "''${PINA_AMM_DEPLOY_KEYPAIR:?set PINA_AMM_DEPLOY_KEYPAIR to the fee-payer keypair file}" \
        "$@"
    '';
  };

  enterShell = ''
    export PATH="$PWD/node_modules/.bin:$PATH"
  '';
}
