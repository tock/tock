#!/usr/bin/env bash

set -e

# Verify that we're running in the base directory
if [ ! -x tools/run_cargo_fmt.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

# Peg a rustfmt version while things are unstable
#
# Note: We install a local copy of rustfmt so as not to interfere with any
# other use of rustfmt on the machine
RUSTFMT_VERSION=0.6.0

# For CI, want to install to a cached travis directory
if [[ "$CI" == "true" ]]; then
	LOCAL_CARGO=$HOME/local_cargo
else
	LOCAL_CARGO=$(pwd)/tools/local_cargo
fi

mkdir -p $LOCAL_CARGO

# Check if we actually need to do anything
needs_install=false
if [[ ! -x $LOCAL_CARGO/bin/rustfmt ]]; then
	needs_install=true
elif [[ $($LOCAL_CARGO/bin/rustfmt --version | perl -pe '($_)=/([0-9]+([.][0-9]+)+)/') != "$RUSTFMT_VERSION" ]]; then
	needs_install=true
fi

if $needs_install; then
	echo "INFO: rustfmt v$RUSTFMT_VERSION not installed. Installing."
	echo "(This will take a few minutes)"
	echo ""

	pushd $LOCAL_CARGO
	mkdir -p .cargo
	cat > .cargo/config <<EOL
[build]
rustflags = [
"-C", "link-arg=-Xlinker",
"-C", "link-arg=-rpath",
"-C", "link-arg=$(rustc --print sysroot)/lib",
]
EOL
	cargo install --root . --vers $RUSTFMT_VERSION --force rustfmt-nightly || exit
	echo ""
	echo "rustfmt v$RUSTFMT_VERSION install complete."
	echo ""
	popd
fi

# Put local cargo format on PATH
PATH="$LOCAL_CARGO/bin:$PATH"
