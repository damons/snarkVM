#!/bin/bash
# Detect affected snarkVM crates from a list of file paths
# Usage: detect-crates.sh < files.txt
# Or:    echo "path/to/file.rs" | detect-crates.sh

set -euo pipefail

# Map top-level directories to crate names
declare -A CRATE_MAP=(
  ["algorithms"]="snarkvm-algorithms"
  ["circuit"]="snarkvm-circuit"
  ["console"]="snarkvm-console"
  ["curves"]="snarkvm-curves"
  ["fields"]="snarkvm-fields"
  ["ledger"]="snarkvm-ledger"
  ["metrics"]="snarkvm-metrics"
  ["parameters"]="snarkvm-parameters"
  ["synthesizer"]="snarkvm-synthesizer"
  ["utilities"]="snarkvm-utilities"
  ["vm"]="snarkvm"
  ["wasm"]="snarkvm-wasm"
)

# Read file paths from stdin or argument
if [[ $# -gt 0 ]]; then
  INPUT="$1"
else
  INPUT=$(cat)
fi

# Extract unique crates
echo "$INPUT" | while read -r filepath; do
  # Skip empty lines
  [[ -z "$filepath" ]] && continue
  
  # Extract top-level directory
  dir=$(echo "$filepath" | cut -d'/' -f1)
  
  # Map to crate name
  if [[ -n "${CRATE_MAP[$dir]:-}" ]]; then
    echo "${CRATE_MAP[$dir]}"
  fi
done | sort -u
