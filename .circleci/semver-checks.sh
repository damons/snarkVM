#! /bin/bash

# Ensure that the command is installed.
cargo install cargo-semver-checks@0.43.0 --locked

BASELINE_REV=65a079a87 # UPDATE ME ON NECESSARY BREAKING CHANGES

# Exclude CLI as it has been removed
cargo semver-checks --workspace --default-features \
  --exclude=snarkvm-cli --baseline-rev $BASELINE_REV
