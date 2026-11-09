#!/usr/bin/env bash
# Install the Pina AMM toolchain the way CI does, for a Linux cloud agent.
#
# The VM image has no systemd, so this uses Nix's single-user installer
# rather than the multi-user daemon. Devenv is pinned to the revision in
# devenv.lock. Re-running the script is safe: Nix, devenv, and the package
# managers all no-op once they match the lockfiles.
set -euo pipefail

cd "$(dirname "$0")/.."

if [ "$(uname -s)" != "Linux" ]; then
	echo "This installer supports Linux cloud agents. On macOS, install Nix and run the devenv commands in README.md." >&2
	exit 1
fi

if ! command -v curl >/dev/null 2>&1 || ! command -v xz >/dev/null 2>&1 || ! command -v git >/dev/null 2>&1; then
	sudo apt-get update
	sudo apt-get install -y --no-install-recommends ca-certificates curl git xz-utils
fi

if [ ! -x "${HOME}/.nix-profile/bin/nix" ]; then
	curl -fsSL https://nixos.org/nix/install | sh -s -- --no-daemon --yes
fi

# shellcheck disable=SC1091
. "${HOME}/.nix-profile/etc/profile.d/nix.sh"

# Single-user Nix reads this file for every shell, login or not. Flakes are
# required for the locked devenv install. The devenv cache is the one
# documented at https://devenv.sh/binary-caching/ ; `extra-*` adds it to the
# default cache.nixos.org substituter instead of replacing that list.
mkdir -p "${HOME}/.config/nix"
cat > "${HOME}/.config/nix/nix.conf" << 'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
extra-substituters = https://devenv.cachix.org
extra-trusted-public-keys = devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=
EOF

if ! nix config show substituters | grep -q 'https://devenv.cachix.org'; then
	echo "devenv.cachix.org is not configured as a substituter; refusing to build devenv from source." >&2
	exit 1
fi

devenv_revision="$(
	nix eval --impure --raw --expr '
    let
      lock = builtins.fromJSON (builtins.readFile ./devenv.lock);
    in
      lock.nodes.devenv.locked.rev
  '
)"

if ! nix profile list --json | grep -q "github:cachix/devenv/${devenv_revision}"; then
	if nix profile list --json | grep -q '"devenv"'; then
		nix profile remove devenv
	fi
	# `nix profile install` is a deprecated alias of `add`. The installable is
	# the devenv revision pinned in devenv.lock, which devenv.cachix.org
	# serves as a binary. A floating `nixpkgs#devenv` would not match CI.
	nix profile add --accept-flake-config "github:cachix/devenv/${devenv_revision}#devenv"
fi

# The Nix installer only edits ~/.profile. Login shells read that file, but
# later agent commands are not always login shells, so the binaries also
# live on the default PATH.
sudo ln -sfn "${HOME}/.nix-profile/bin/nix" /usr/local/bin/nix
sudo ln -sfn "${HOME}/.nix-profile/bin/devenv" /usr/local/bin/devenv

# Build and cache the locked shell (Agave, Pina, Node, Dart, Rust, …) before
# language packages are installed. Later `devenv shell` calls reuse it.
devenv shell -- true

devenv shell -- install:all

# Download crates and the SBF rustc. This does not compile the program.
devenv shell -- bash -c 'rustup toolchain install "${PINA_BPF_TOOLCHAIN}" --profile minimal'
devenv shell -- cargo fetch --locked
devenv shell -- cargo fetch --locked --manifest-path programs/pina_amm/tests/surfpool/Cargo.toml
