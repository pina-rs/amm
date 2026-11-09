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

mkdir -p "${HOME}/.config/nix"
cat > "${HOME}/.config/nix/nix.conf" << 'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
substituters = https://cache.nixos.org https://devenv.cachix.org
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=
EOF

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
	nix profile add --accept-flake-config "github:cachix/devenv/${devenv_revision}#devenv"
fi

# The Nix installer only edits ~/.profile. Login shells read that file, but
# later agent commands are not always login shells, so the binaries also
# live on the default PATH.
sudo ln -sfn "${HOME}/.nix-profile/bin/nix" /usr/local/bin/nix
sudo ln -sfn "${HOME}/.nix-profile/bin/devenv" /usr/local/bin/devenv

devenv shell -- install:all

# Download crates and the SBF rustc. This does not compile the program.
devenv shell -- bash -c 'rustup toolchain install "${PINA_BPF_TOOLCHAIN}" --profile minimal'
devenv shell -- cargo fetch --locked
devenv shell -- cargo fetch --locked --manifest-path programs/pina_amm/tests/surfpool/Cargo.toml
