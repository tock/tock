#!/usr/bin/env bash


# Ask rustup to pick the latest version that will work.
# This requires rustup >= 1.20.0.
echo "Updating rustc to latest compatible version..."
rustup update nightly --allow-downgrade --component cargo --component clippy --component llvm-tools-preview --component miri --component rls --component rust-analysis --component rust-docs --component rust-src --component rust-std --component rustc --component rustfmt

# # Rerun the command so that it prints out the version it installed. We then have
# # to extract that from the output. If there is a better way to do this then we
# # should update this.
# RUSTUP_NIGHTLY_VERSION=`rustup update nightly 2>&1`
# BEST_DATE=`echo $RUSTUP_NIGHTLY_VERSION | sed 's/.* \([0-9]*-[0-9]*-[0-9]*\).*/\1/g'`

# I just do not know how to get rustup to tell us the version of the toolchain
# it decided on with the format required for `rust-toolchain`. That the dates
# are off-by-one day is annoying. I'm resorting to just asking the user.


echo "Please enter the version of Rust to use."
echo "It is probably just one day later than whatever was printed out above."
echo ""
read -p "Date string: " BEST_DATE

# Nightly version string
NIGHTLY=nightly-$BEST_DATE

echo Updating Rust to $NIGHTLY

# Set the Rust version in rust-toolchain file.
echo $NIGHTLY > rust-toolchain

# Update all relevant files with the new version string.
# Note, x-platform `sed -i` has odd, but particular syntax
# https://stackoverflow.com/questions/5694228/sed-in-place-flag-that-works-both-on-mac-bsd-and-linux
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" .vscode/settings.json
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" doc/Getting_Started.md
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" rust-toolchain
sed -i._SED_HACK "s/nightly-[0-9]*-[0-9]*-[0-9]*/${NIGHTLY}/g" tools/netlify-build.sh
sed -i._SED_HACK "s/[0-9]*-[0-9]*-[0-9]*/${BEST_DATE}/g" shell.nix

find . -name '*._SED_HACK' -delete
