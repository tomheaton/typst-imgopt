#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

if ! command -v typst >/dev/null 2>&1; then
  echo "Typst CLI is not installed."
  echo "Install Typst and re-run this script."
  exit 1
fi

"$script_dir/build-wasm.sh"

# Build optimised and baseline PDFs from equivalent example documents.
typst compile --root "$project_root" \
  "$project_root/examples/main.typ" \
  "$project_root/examples/main.pdf"
typst compile --root "$project_root" \
  "$project_root/examples/main-no-opt.typ" \
  "$project_root/examples/main-no-opt.pdf"

opt_bytes=$(wc -c < "$project_root/examples/main.pdf" | tr -d '[:space:]')
base_bytes=$(wc -c < "$project_root/examples/main-no-opt.pdf" | tr -d '[:space:]')

printf "Optimised PDF   : %s bytes\n" "$opt_bytes"
printf "Unoptimised PDF : %s bytes\n" "$base_bytes"

if [ "$base_bytes" -gt 0 ]; then
  saved=$((base_bytes - opt_bytes))
  pct=$(awk "BEGIN { printf \"%.2f\", (1 - ($opt_bytes / $base_bytes)) * 100 }")
  printf "Saved: %s bytes (%s%%)\n" "$saved" "$pct"
fi
