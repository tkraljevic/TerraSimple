#!/bin/sh
# Program crta list, QGIS crta isti list, a Poppler oba pretvara u sliku koja se mjeri.
# Radi za A4 i A3. Podloga se uzima ako postoji provjera/dof_a4.jpg ili dof_a3.jpg.
# Upotreba: pokreni.sh <radna mapa>
set -eu
OVDJE=$(cd "$(dirname "$0")" && pwd)
RADNA=$1
A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
qgis_python() {
    stanje=0
    env PYTHONPATH="$R:$R/lib-dynload:$R/site-packages" \
        QT_QPA_PLATFORM=offscreen \
        QGIS_PREFIX_PATH="$A/MacOS" \
        PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" \
        GDAL_DATA="$A/Resources/qgis/gdal" \
        "$A/MacOS/python3.12" "$@" > "$RADNA/izlaz.txt" 2>&1 || stanje=$?
    grep -v -E "Could not find platform|Application path|^$" "$RADNA/izlaz.txt" || true
    return "$stanje"
}
cd "$OVDJE/.."
cargo test --quiet
for FORMAT in a4 a3; do
    mkdir -p "$RADNA/$FORMAT"
    PODLOGA=""
    if [ -f "$OVDJE/dof_$FORMAT.jpg" ]; then PODLOGA="$OVDJE/dof_$FORMAT.jpg"; fi
    cargo run --quiet --release -- "$FORMAT" "$RADNA/$FORMAT" $PODLOGA
    qgis_python "$OVDJE/qgis_ispis.py" "$RADNA/$FORMAT"
    qgis_python "$OVDJE/analiza.py" "$RADNA/$FORMAT" $PODLOGA
done
