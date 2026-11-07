#!/bin/sh
# Pravi QGIS i PROJ računaju kontrolne mjere. Testovi čitaju zapisanu tablicu, pa QGIS za njih ne treba.
set -eu
OVDJE=$(cd "$(dirname "$0")" && pwd)
A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
env PYTHONPATH="$R:$R/lib-dynload:$R/site-packages" \
    QT_QPA_PLATFORM=offscreen \
    QGIS_PREFIX_PATH="$A/MacOS" \
    PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" \
    GDAL_DATA="$A/Resources/qgis/gdal" \
    "$A/MacOS/python3.12" "$OVDJE/referenca.py" "$OVDJE/../tests/podaci/kontrolne.csv" "$A/MacOS" 2>&1 \
    | grep -v -E "Could not find platform|^$" || true
