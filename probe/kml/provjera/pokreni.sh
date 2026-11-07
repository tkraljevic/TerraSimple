#!/bin/sh
# Pokreće sve testove, izmišljene primjere, namjerne kvarove, neovisnu referencu
# (Python expat + GDAL ogrinfo) i mjerenje vremena/memorije u zasebnim procesima.
#
# Upotreba:
#   ./provjera/pokreni.sh [mapa_primjera] [stvarna_datoteka]...
#
# VAŽNO ZA PRIVATNOST: Za stvarne datoteke nikada se ne ispisuju stvarna imena datoteka
# (osim `myplaces.kml`), nazivi mapa, nazivi objekata, opisi ni koordinate.
set -u

OVDJE=$(cd "$(dirname "$0")" && pwd)
KORIJEN=$(cd "$OVDJE/.." && pwd)
MAPA="${1:-$KORIJEN/primjeri}"
if [ "$#" -ge 1 ]; then
    shift
fi

A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
REF_MAPA=$(mktemp -d "${TMPDIR:-/tmp}/proba_kml_ref.XXXXXX")
trap 'rm -rf "$REF_MAPA"' EXIT INT TERM

cd "$KORIJEN" || exit 1

echo "=== 1. Jedinični testovi, clippy i gradnja (release)"
cargo test --quiet || exit 1
cargo clippy --all-targets --quiet -- -D warnings || exit 1
cargo build --quiet --release || exit 1
PROBA="${CARGO_TARGET_DIR:-$KORIJEN/../target}/release/proba_kml"
if [ ! -x "$PROBA" ]; then
    PROBA="$KORIJEN/target/release/proba_kml"
fi

palo=0

echo "=== 2. Izrada i provjera izmišljenih primjera te namjernih kvarova"
"$PROBA" izradi-primjere "$MAPA" || exit 1
"$PROBA" provjeri-primjere "$MAPA" || { echo "  >>> provjera primjera nije prošla"; palo=$((palo + 1)); }

POPIS="$REF_MAPA/popis.tsv"
: > "$POPIS"
for ime in \
    sve_znacajke.kml \
    duboke_mape.kml \
    mijesani_slojevi.kml \
    arhiva_sa_slikama.kmz \
    ostecen.kml \
    prazna_geometrija.kml \
    zip_bomba.kmz \
    neocekivana_putanja.kmz
do
    oznaka="${ime%.*}"
    printf '%s\t%s\n' "$oznaka" "$MAPA/$ime" >> "$POPIS"
done

br_stvarnih=0
for f in "$@"; do
    if [ ! -f "$f" ]; then
        echo "  PALO    datoteka ne postoji"
        palo=$((palo + 1))
        continue
    fi
    baza=$(basename "$f")
    if [ "$baza" = "myplaces.kml" ]; then
        oznaka="myplaces.kml"
    else
        br_stvarnih=$((br_stvarnih + 1))
        oznaka="stvarna_${br_stvarnih}"
    fi
    printf '%s\t%s\n' "$oznaka" "$f" >> "$POPIS"
done

echo "=== 3. Neovisna referenca (Python expat + GDAL ogrinfo)"
while IFS="$(printf '\t')" read -r oznaka put; do
    env PYTHONDONTWRITEBYTECODE=1 PYTHONPATH="$R:$R/lib-dynload:$R/site-packages" \
        PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" \
        GDAL_DATA="$A/Resources/qgis/gdal" \
        "$A/MacOS/python3.12" "$OVDJE/referenca.py" "$REF_MAPA" "$oznaka" "$put" 2>&1 \
        | grep -v -E "Could not find platform|Warning 1|^$"
done < "$POPIS"

echo "=== 4. Usporedba našeg čitača s neovisnom referencom i GDAL ogrinfo"
while IFS="$(printf '\t')" read -r oznaka put; do
    "$PROBA" usporedi "$put" "$REF_MAPA/$oznaka.csv" "$oznaka" || {
        echo "  >>> $oznaka: usporedba nije prošla"
        palo=$((palo + 1))
    }
done < "$POPIS"

echo "=== 5. Mjerenje vremena i vršne memorije (svaka datoteka u zasebnom procesu)"
while IFS="$(printf '\t')" read -r oznaka put; do
    if grep -q '^neotvoriva' "$REF_MAPA/$oznaka.csv"; then
        continue
    fi
    "$PROBA" mjeri "$put" "$oznaka" || {
        echo "  >>> $oznaka: mjerenje nije prošlo"
        palo=$((palo + 1))
    }
    if [ "$oznaka" = "myplaces.kml" ] || [ "$oznaka" = "sve_znacajke" ]; then
        "$PROBA" mjeri-kml-crate "$put" "$oznaka" || {
            echo "  >>> $oznaka: mjerenje kml 0.15 nije prošlo"
            palo=$((palo + 1))
        }
    fi
done < "$POPIS"

echo "=== Sažetak: palih provjera = $palo"
exit "$palo"
