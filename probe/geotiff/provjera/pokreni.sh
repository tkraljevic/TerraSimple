#!/bin/sh
# Usporedba s GDAL-om, pa mjerenje vremena i memorije, svaka datoteka u svom procesu.
# Upotreba: pokreni.sh <mapa s izmišljenim slikama> [stvarna datoteka]...
# Stvarne datoteke se samo čitaju, a ništa od njih ne ide u repozitorij.
set -u
OVDJE=$(cd "$(dirname "$0")" && pwd)
MAPA=$1
shift
A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
REF="$MAPA/ref"
SLIKE="$MAPA/slike"
mkdir -p "$REF" "$SLIKE"
cd "$OVDJE/.." || exit 1
cargo test --quiet || exit 1
cargo build --quiet --release || exit 1
PROBA="${CARGO_TARGET_DIR:-../target}/release/proba_geotiff"
palo=0
: > "$MAPA/popis.txt"
for f in "$MAPA"/*.tif; do printf '%s\n' "$f" >> "$MAPA/popis.txt"; done
for f in "$@"; do printf '%s\n' "$f" >> "$MAPA/popis.txt"; done

echo "=== referenca GDAL-a"
while IFS= read -r f; do
    env PYTHONPATH="$R:$R/lib-dynload:$R/site-packages" \
        PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" \
        GDAL_DATA="$A/Resources/qgis/gdal" \
        "$A/MacOS/python3.12" "$OVDJE/referenca.py" "$REF" "$f" 2>&1 \
        | grep -v -E "Could not find platform|Warning 1|^$"
done < "$MAPA/popis.txt"

echo "=== usporedba s GDAL-om"
while IFS= read -r f; do
    ime=$(basename "$f"); ime=${ime%.*}
    echo "$ime"
    "$PROBA" usporedi "$f" "$REF/$ime.csv" || { echo "  >>> usporedba nije prošla"; palo=$((palo + 1)); }
done < "$MAPA/popis.txt"

echo "=== mjerenje"
while IFS= read -r f; do
    ime=$(basename "$f"); ime=${ime%.*}
    # Datoteka koju ni GDAL ne otvara već je provjerena u usporedbi.
    if grep -q '^neotvoriva' "$REF/$ime.csv"; then continue; fi
    "$PROBA" prolaz "$f" "$SLIKE" || { echo "  >>> $ime: mjerenje nije prošlo"; palo=$((palo + 1)); }
done < "$MAPA/popis.txt"

echo "=== sve pročitati"
while IFS= read -r f; do
    case "$f" in
        *gola_trake*|*lzw_trake*|*plocice_bez*) "$PROBA" cijela "$f" ;;
    esac
done < "$MAPA/popis.txt"
echo "palih datoteka: $palo"
exit "$palo"
