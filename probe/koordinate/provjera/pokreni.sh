#!/bin/sh
# Pravi QGIS računa kontrolne točke, a proba ih uspoređuje s našim računom.
set -eu
OVDJE=$(cd "$(dirname "$0")" && pwd)
A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
env PYTHONPATH="$R:$R/lib-dynload:$R/site-packages" \
    QT_QPA_PLATFORM=offscreen \
    QGIS_PREFIX_PATH="$A/MacOS" \
    PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" \
    GDAL_DATA="$A/Resources/qgis/gdal" \
    "$A/MacOS/python3.12" "$OVDJE/referenca.py" "$OVDJE/referenca.csv" 2>&1 \
    | grep -v -E "Could not find platform|^$" || true
cd "$OVDJE/.."
cargo test --quiet
cargo run --quiet --release -- usporedi "$OVDJE/referenca.csv"
