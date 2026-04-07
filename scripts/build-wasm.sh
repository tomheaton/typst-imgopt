#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
project_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

cargo build --manifest-path "$project_root/Cargo.toml" --release --target wasm32-unknown-unknown
cp \
	"$project_root/target/wasm32-unknown-unknown/release/typst_imgopt.wasm" \
	"$project_root/typst/imgopt.wasm"
echo "Built typst/imgopt.wasm"
