#!/bin/sh
# Pokreće samo QGIS-ov Python, bez prozora i bez instalacije.
set -eu
OVDJE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
KORIJEN=$(CDPATH= cd -- "$OVDJE/../.." && pwd)
A=${QGIS_APP:-/Applications/QGIS-LTR.app/Contents}
R="$A/Resources/python3.11"
export PYTHONPATH="$R:$R/lib-dynload:$R/site-packages"
export PYTHONDONTWRITEBYTECODE=1 QT_QPA_PLATFORM=offscreen QGIS_PREFIX_PATH="$A/MacOS"
export PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" GDAL_DATA="$A/Resources/qgis/gdal"
export CARGO_TARGET_DIR="$KORIJEN/target"
exec "$A/MacOS/python3.12" "$OVDJE/krug.py" "$KORIJEN/target/debug/examples/krug" "$KORIJEN/.quality/qgis" "$R/site-packages/osgeo_utils/samples/validate_gpkg.py"
