#!/bin/sh
# Cijeli krug s QGIS-om iz /Applications. Rezultati idu u mapu iz prvog argumenta, koja se briše na početku.
set -eu
MAPA=${1:?radna mapa}
OVDJE=$(cd "$(dirname "$0")" && pwd)
A=/Applications/QGIS-LTR.app/Contents
R=$A/Resources/python3.11
PROBA=$OVDJE/../../target/release/proba_geopaket
export PYTHONPATH="$R:$R/lib-dynload:$R/site-packages"
export PYTHONDONTWRITEBYTECODE=1
export QT_QPA_PLATFORM=offscreen
export QGIS_PREFIX_PATH="$A/MacOS"
export PROJ_DATA="$A/Resources/qgis/proj" PROJ_LIB="$A/Resources/qgis/proj" GDAL_DATA="$A/Resources/qgis/gdal"
exec "$A/MacOS/python3.12" "$OVDJE/krug.py" "$PROBA" "$MAPA" "$R/site-packages/osgeo_utils/samples/validate_gpkg.py"
