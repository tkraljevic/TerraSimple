#!/usr/bin/env bash
# SPDX-License-Identifier: EUPL-1.2
set -euo pipefail

: "${TERRASIMPLE_PMTILES:?Zadaj stazu PMTiles datoteke u TERRASIMPLE_PMTILES}"

KORIJEN="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$KORIJEN"

echo "=== 1. Provjera statičke analize (cargo clippy -D warnings) ==="
cargo clippy --all-targets -- -D warnings

echo "=== 2. Jedinični testovi i testovi namjernog kvara (cargo test) ==="
cargo test

echo "=== 3. Snimanje svijetle i tamne teme (OpenFreeMap) u stvarnom prozoru ==="
if [[ ! -f "snimke/svijetla_tema.png" || ! -f "snimke/tamna_tema.png" || "${PONOVNO_SNIMI:-0}" == "1" ]]; then
    cargo run --release -- --snimi-teme
fi

echo "=== 4. Cjelovita provjera svih 5 pitanja i namjernih kvarova (--provjeri-sve) ==="
cargo run --release -- --provjeri-sve

echo "=== SVE PROVJERE U probe/walkers-dodaci USPJEŠNO ZAVRŠENE ==="
