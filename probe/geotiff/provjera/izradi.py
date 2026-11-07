"""Izrađuje izmišljene ispitne GeoTIFF-ove preko GDAL-a iz QGIS-a.

Velike slike od oko 2,4 GB razlikuju se samo po tome kako su složene, jer o tome ovisi
hoće li se otvarati bez čekanja. Male slike provjeravaju položaj i BigTIFF.
Upotreba: izradi.py <mapa>
"""

import shutil
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
from osgeo import gdal, osr

gdal.UseExceptions()
STRANICA = 28300
KORAK_M = 0.5
ZAPAD = 664000.0
SJEVER = 5053000.0
KOPIJA = ["BIGTIFF=IF_NEEDED", "NUM_THREADS=ALL_CPUS"]


def isjecak(x0, y0, sirina, visina):
    """Boja ovisi o položaju, uz malo šuma, pa se sažima slično kao prava snimka."""
    x = np.arange(x0, x0 + sirina, dtype=np.float32)[None, :]
    y = np.arange(y0, y0 + visina, dtype=np.float32)[:, None]
    sum_ = np.random.default_rng([x0, y0]).integers(-6, 7, (visina, sirina), dtype=np.int16)
    r = 128 + 70 * np.sin(x / 211) + 35 * np.sin(y / 37 + x / 97) + sum_
    g = 128 + 70 * np.sin(y / 173) + 35 * np.cos(x / 53 - y / 83) + sum_
    b = 110 + 50 * np.sin((x + y) / 331) + 20 * np.cos(y / 29) + sum_
    rgb = np.stack([r, g, b]).clip(0, 255).astype(np.uint8)
    # Svakih 1 000 piksela crta, svakih 5 000 debela: vidi se i pri velikom smanjenju.
    for korak, debljina in ((1000, 4), (5000, 90)):
        rgb[:, :, ((np.arange(x0, x0 + sirina)) % korak) < debljina] = 255
        rgb[:, ((np.arange(y0, y0 + visina)) % korak) < debljina, :] = 255
    # Crveni trokut samo gore lijevo: tako se vidi je li slika okrenuta.
    xx = np.arange(x0, x0 + sirina)[None, :]
    yy = np.arange(y0, y0 + visina)[:, None]
    trokut = (xx + yy) < 6000
    rgb[0][trokut], rgb[1][trokut], rgb[2][trokut] = 220, 30, 30
    return rgb


def sustav(epsg):
    s = osr.SpatialReference()
    s.ImportFromEPSG(epsg)
    return s.ExportToWkt()


def izradi_golu(put):
    ds = gdal.GetDriverByName("GTiff").Create(str(put), STRANICA, STRANICA, 3, gdal.GDT_Byte, ["INTERLEAVE=PIXEL"])
    ds.SetGeoTransform((ZAPAD, KORAK_M, 0, SJEVER, 0, -KORAK_M))
    ds.SetProjection(sustav(3765))
    for y in range(0, STRANICA, 256):
        visina = min(256, STRANICA - y)
        ds.WriteRaster(0, y, STRANICA, visina, isjecak(0, y, STRANICA, visina).tobytes(), band_list=[1, 2, 3])
    ds = None


def prevedi(izvor, cilj, *opcije, format_="GTiff"):
    gdal.Translate(str(cilj), str(izvor), format=format_, creationOptions=[*opcije])


def piramide(put, *opcije, ro=False):
    gdal.SetConfigOption("COMPRESS_OVERVIEW", "DEFLATE")
    ds = gdal.Open(str(put), gdal.GA_ReadOnly if ro else gdal.GA_Update)
    ds.BuildOverviews("AVERAGE", [2, 4, 8, 16, 32, 64])
    ds = None
    # Postavka je globalna i inače smeta sljedećoj izradi, koja piramide sažima JPEG-om.
    gdal.SetConfigOption("COMPRESS_OVERVIEW", None)


def mala(put, epsg, zapad, sjever, *opcije, metapodaci=None):
    n = 2000
    ds = gdal.GetDriverByName("GTiff").Create(str(put), n, n, 3, gdal.GDT_Byte, [*opcije])
    ds.SetGeoTransform((zapad, KORAK_M, 0, sjever, 0, -KORAK_M))
    ds.SetProjection(sustav(epsg))
    if metapodaci:
        ds.SetMetadata(metapodaci)
    ds.WriteRaster(0, 0, n, n, isjecak(0, 0, n, n).tobytes(), band_list=[1, 2, 3])
    ds = None


def mala_paleta(put, *opcije):
    """Paleta kao u starim skenovima, s dva načina slaganja i sažimanja."""
    n = 2000
    ds = gdal.GetDriverByName("GTiff").Create(str(put), n, n, 1, gdal.GDT_Byte, ["PHOTOMETRIC=PALETTE", *opcije])
    ds.SetGeoTransform((ZAPAD, KORAK_M, 0, SJEVER, 0, -KORAK_M))
    ds.SetProjection(sustav(3765))
    tablica = gdal.ColorTable()
    for i in range(256):
        tablica.SetColorEntry(i, (i, 255 - i, (i * 7) % 256, 255))
    ds.GetRasterBand(1).SetRasterColorTable(tablica)
    ds.GetRasterBand(1).WriteRaster(0, 0, n, n, isjecak(0, 0, n, n)[0].tobytes())
    ds = None


def korak(ime, posao):
    if GOTOVO(ime):
        return
    pocetak = time.time()
    posao()
    print(f"{ime}: {time.time() - pocetak:.0f} s", flush=True)


def main():
    mapa = Path(sys.argv[1])
    mapa.mkdir(parents=True, exist_ok=True)
    global GOTOVO
    GOTOVO = lambda ime: (mapa / f"{ime}.tif").exists()
    gola = mapa / "gola_trake.tif"
    korak("gola_trake", lambda: izradi_golu(gola))
    korak("jedan_trak", lambda: prevedi(gola, mapa / "jedan_trak.tif", f"BLOCKYSIZE={STRANICA}"))
    korak("lzw_trake", lambda: prevedi(gola, mapa / "lzw_trake.tif", "COMPRESS=LZW", "PREDICTOR=2", *KOPIJA))
    plocice = mapa / "plocice_bez_piramida.tif"
    korak("plocice_bez_piramida", lambda: prevedi(gola, plocice, "TILED=YES", "BLOCKXSIZE=512", "BLOCKYSIZE=512", "COMPRESS=DEFLATE", *KOPIJA))
    s_piramidama = mapa / "plocice_piramide.tif"
    def p():
        subprocess.run(["cp", "-c", str(plocice), str(s_piramidama)], check=True)
        piramide(s_piramidama)
    korak("plocice_piramide", p)
    def v():
        vanjska = mapa / "gola_vanjske_piramide.tif"
        subprocess.run(["cp", "-c", str(gola), str(vanjska)], check=True)
        piramide(vanjska, ro=True)
    korak("gola_vanjske_piramide", v)
    korak("jpeg_cog", lambda: gdal.Translate(str(mapa / "jpeg_cog.tif"), str(gola), format="COG", creationOptions=["COMPRESS=JPEG", "QUALITY=85", "BLOCKSIZE=512", "NUM_THREADS=ALL_CPUS", "OVERVIEW_RESAMPLING=AVERAGE"]))
    mala(mapa / "mala_bigtiff.tif", 3765, ZAPAD, SJEVER, "BIGTIFF=YES", "TILED=YES", "COMPRESS=DEFLATE")
    mala(mapa / "mala_hdks.tif", 8678, 6554000.0, 5046000.0, "COMPRESS=LZW")
    mala(mapa / "mala_tocka.tif", 3765, ZAPAD, SJEVER, metapodaci={"AREA_OR_POINT": "Point"})
    mala_paleta(mapa / "mala_paleta_lzw.tif", "COMPRESS=LZW")
    mala_paleta(mapa / "mala_paleta_packbits.tif", "COMPRESS=PACKBITS")
    mala_paleta(mapa / "mala_paleta_ploce.tif", "TILED=YES", "COMPRESS=PACKBITS")
    mala_paleta(mapa / "mala_paleta_gola.tif")
    for f in sorted(mapa.glob("*")):
        print(f"{f.name}: {f.stat().st_size / 1e9:.3f} GB (na disku {f.stat().st_blocks * 512 / 1e9:.3f})", flush=True)
    print("slobodno:", shutil.disk_usage(mapa).free // 10**9, "GB")


main()
