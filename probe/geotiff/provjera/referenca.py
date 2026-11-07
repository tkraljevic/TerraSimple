"""GDAL čita iste isječke kao proba, a proba ih mora dobiti jednake.

Upotreba: referenca.py <mapa izlaza> <datoteka>...
Za svaku datoteku piše <ime>.csv i sirove isječke <ime>_<n>.raw u mapu izlaza.
Stvarne datoteke se samo čitaju i ne upisuju se u repozitorij.
"""

import sys
from pathlib import Path

import numpy as np
from osgeo import gdal

gdal.UseExceptions()
VELICINA = 256


def rgb(ds, x, y, s, v, razina):
    """Isječak kao RGB, s paletom raširenom u boje, kako ga vraća i proba."""
    def pojas(i):
        b = ds.GetRasterBand(i)
        return b if razina == 0 else b.GetOverview(razina - 1)

    prvi = ds.GetRasterBand(1)
    if prvi.GetColorTable() is not None:
        tablica = prvi.GetColorTable()
        boje = np.array([tablica.GetColorEntry(i)[:3] for i in range(tablica.GetCount())], dtype=np.uint8)
        return boje[pojas(1).ReadAsArray(x, y, s, v)]
    if ds.RasterCount >= 3:
        return np.stack([pojas(i).ReadAsArray(x, y, s, v) for i in (1, 2, 3)], axis=2)
    a = pojas(1).ReadAsArray(x, y, s, v)
    return np.stack([a, a, a], axis=2)


def obradi(put, mapa):
    ime = Path(put).stem
    try:
        ds = gdal.Open(put)
    except RuntimeError as g:
        (mapa / f"{ime}.csv").write_text(f"neotvoriva;{str(g)[:60]}\n")
        print(f"{ime}: GDAL je ne otvara")
        return
    prvi = ds.GetRasterBand(1)
    gt = ds.GetGeoTransform(can_return_null=True)
    srs = ds.GetSpatialRef()
    epsg = srs.GetAuthorityCode(None) if srs is not None else None
    redci = [f"geo;{';'.join(repr(v) for v in gt) if gt else ''};{epsg or ''}"]
    razine = [(ds.RasterXSize, ds.RasterYSize)]
    for i in range(prvi.GetOverviewCount()):
        o = prvi.GetOverview(i)
        razine.append((o.XSize, o.YSize))
    for i, (s, v) in enumerate(razine):
        redci.append(f"razina;{i};{s};{v}")
    n = 0
    for razina, (s, v) in enumerate(razine):
        uzorci = [(0, 0), (s // 2 - 100, v // 2 - 100), (s - VELICINA // 2, v - VELICINA // 2)] if razina == 0 else [(s // 3, v // 3)]
        for x, y in uzorci:
            x, y = max(0, x), max(0, y)
            w, h = min(VELICINA, s - x), min(VELICINA, v - y)
            podaci = rgb(ds, x, y, w, h, razina)
            (mapa / f"{ime}_{n}.raw").write_bytes(podaci.tobytes())
            redci.append(f"isjecak;{razina};{x};{y};{w};{h};{ime}_{n}.raw")
            n += 1
    (mapa / f"{ime}.csv").write_text("\n".join(redci) + "\n")
    print(f"{ime}: {ds.RasterXSize}x{ds.RasterYSize}, razina {len(razine)}, isječaka {n}")


def main():
    mapa = Path(sys.argv[1])
    mapa.mkdir(parents=True, exist_ok=True)
    for put in sys.argv[2:]:
        obradi(put, mapa)


main()
