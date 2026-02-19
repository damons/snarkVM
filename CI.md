# CI

## PR Workflows (on PR open)
5 CircleCI workflows: `cargo-workflow`, `circuit-workflow`, `console-workflow`, `ledger-workflow`, `synthesizer-workflow`. Each targets 15 min max runtime.

## Merge Workflow (on PR merge)
Runs slow/ignored tests and parameter downloads.

## Release Workflow (canary/testnet/mainnet branches)
Includes Windows testing.

## GitHub Actions
Benchmarks run separately via GitHub Actions.
