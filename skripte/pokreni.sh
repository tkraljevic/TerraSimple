#!/bin/sh
# Program radi i iz mape čije ime sadrži razmake.
cd -- "$(dirname -- "$0")" || exit 1
exec ./TerraSimple "$@"
