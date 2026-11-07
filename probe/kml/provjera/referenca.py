#!/usr/bin/env python3
"""Neovisni izračun referentnih brojeva za KML i KMZ datoteke pomoću Pythona (expat + zipfile)
te GDAL alata `/Applications/QGIS-LTR.app/Contents/MacOS/ogrinfo` i `osgeo.ogr`.

VAŽNO ZA PRIVATNOST: Skripta nikada ne ispisuje nazive datoteka, mapa, objekata, opise ni koordinate.
Ispisuje isključivo sigurnu oznaku (`oznaka`) i cjelobrojne zbrojeve.
"""

import os
import re
import subprocess
import sys
import xml.parsers.expat
import zipfile
from osgeo import gdal, ogr

gdal.UseExceptions()
ogr.UseExceptions()

MAKS_VELICINA_STAVKE = 64 * 1024 * 1024
MAKS_UKUPNA_VELICINA = 128 * 1024 * 1024
MAKS_OMJER_KOMPRESIJE = 200
OGRINFO_PUT = "/Applications/QGIS-LTR.app/Contents/MacOS/ogrinfo"


def lokalno_ime(oznaka: str) -> str:
    if "}" in oznaka:
        return oznaka.rsplit("}", 1)[1]
    if ":" in oznaka:
        return oznaka.split(":", 1)[1]
    return oznaka


def sigurna_putanja_u_arhivi(ime: str) -> bool:
    if not ime or ime.startswith("/") or ime.startswith("\\") or "\x00" in ime:
        return False
    normalizirano = ime.replace("\\", "/")
    for dio in normalizirano.split("/"):
        if dio == "..":
            return False
    return True


def ucitaj_kml_iz_datoteke(put: str):
    """Vraća (kml_bajtovi, broj_slika_u_kmz) ili baca ValueError ako je datoteka neispravna."""
    if put.lower().endswith(".kmz"):
        if not zipfile.is_zipfile(put):
            raise ValueError("nije_zip")
        ukupno = 0
        kml_bajtovi = None
        slike = 0
        with zipfile.ZipFile(put, "r") as zf:
            for info in zf.infolist():
                if not sigurna_putanja_u_arhivi(info.filename):
                    raise ValueError("neocekivana_putanja")
                if info.is_dir():
                    continue
                if info.file_size > MAKS_VELICINA_STAVKE:
                    raise ValueError("prevelika_stavka")
                if info.compress_size > 0 and info.file_size > 1024 * 1024:
                    if info.file_size // info.compress_size > MAKS_OMJER_KOMPRESIJE:
                        raise ValueError("zip_bomba")
                procitano = zf.read(info.filename)
                if len(procitano) > MAKS_VELICINA_STAVKE:
                    raise ValueError("prevelika_stavka")
                ukupno += len(procitano)
                if ukupno > MAKS_UKUPNA_VELICINA:
                    raise ValueError("prevelika_arhiva")
                ime_malo = info.filename.lower()
                if ime_malo.endswith(".kml"):
                    if kml_bajtovi is None or ime_malo == "doc.kml":
                        kml_bajtovi = procitano
                elif ime_malo.endswith((".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tif", ".tiff", ".webp")):
                    slike += 1
        if kml_bajtovi is None:
            raise ValueError("nema_kml_u_kmz")
        return kml_bajtovi, slike
    else:
        with open(put, "rb") as f:
            return f.read(), 0


def prebroji_xml(kml_bajtovi: bytes, slike_u_kmz: int) -> dict:
    stog = []
    dubina_mapa = 0
    dubina_spremnika = 0
    maks_dubina_mapa = 0
    maks_dubina_spremnika = 0

    brojevi = {
        "dokumenti": 0,
        "mape": 0,
        "najveca_dubina_mapa": 0,
        "najveca_dubina_spremnika": 0,
        "objekti": 0,
        "objekti_s_nazivom": 0,
        "objekti_s_opisom": 0,
        "mape_s_opisom": 0,
        "tocke": 0,
        "linije": 0,
        "poligoni": 0,
        "rupe_poligona": 0,
        "tragovi": 0,
        "ukupno_vrhova": 0,
        "vrhovi_s_nenultom_visinom": 0,
        "stilovi": 0,
        "mape_stilova": 0,
        "mrezne_veze": 0,
        "slikovni_prekrivaci": 0,
        "pogledi_kamere": 0,
        "vremenski_zapisi": 0,
        "prosireni_podaci": 0,
        "slike_u_kmz": slike_u_kmz,
        "unutarnje_granice_tag": 0,
    }

    # Stog stanja za praćenje teksta i provjeru nepraznih geometrija
    tekst_stog = []
    geom_vrhovi_stog = []

    def pocetak(ime, atributi):
        nonlocal dubina_mapa, dubina_spremnika, maks_dubina_mapa, maks_dubina_spremnika
        lok = lokalno_ime(ime)
        roditelj = stog[-1] if stog else ""
        stog.append(lok)
        tekst_stog.append([])

        if lok == "Document":
            brojevi["dokumenti"] += 1
            dubina_spremnika += 1
            if dubina_spremnika > maks_dubina_spremnika:
                maks_dubina_spremnika = dubina_spremnika
        elif lok == "Folder":
            brojevi["mape"] += 1
            dubina_mapa += 1
            dubina_spremnika += 1
            if dubina_mapa > maks_dubina_mapa:
                maks_dubina_mapa = dubina_mapa
            if dubina_spremnika > maks_dubina_spremnika:
                maks_dubina_spremnika = dubina_spremnika
        elif lok == "Placemark":
            brojevi["objekti"] += 1
        elif lok == "Point":
            brojevi["tocke"] += 1
            geom_vrhovi_stog.append([0])
        elif lok == "LineString":
            brojevi["linije"] += 1
            geom_vrhovi_stog.append([0])
        elif lok == "Polygon":
            brojevi["poligoni"] += 1
            geom_vrhovi_stog.append([0])
        elif lok == "innerBoundaryIs":
            brojevi["unutarnje_granice_tag"] += 1
        elif lok == "LinearRing":
            geom_vrhovi_stog.append([0])
            if roditelj == "innerBoundaryIs":
                brojevi["rupe_poligona"] += 1
            elif roditelj != "outerBoundaryIs":
                brojevi["linije"] += 1
        elif lok == "Track":
            brojevi["tragovi"] += 1
            geom_vrhovi_stog.append([0])
        elif lok == "Style":
            brojevi["stilovi"] += 1
        elif lok == "StyleMap":
            brojevi["mape_stilova"] += 1
        elif lok == "NetworkLink":
            brojevi["mrezne_veze"] += 1
        elif lok == "GroundOverlay":
            brojevi["slikovni_prekrivaci"] += 1
        elif lok in ("LookAt", "Camera"):
            if roditelj in ("Document", "Folder", "Placemark"):
                brojevi["pogledi_kamere"] += 1
        elif lok in ("TimeStamp", "TimeSpan"):
            if roditelj in ("Document", "Folder", "Placemark", "LookAt", "Camera"):
                brojevi["vremenski_zapisi"] += 1
        elif lok == "ExtendedData":
            brojevi["prosireni_podaci"] += 1

    def kraj(ime):
        nonlocal dubina_mapa, dubina_spremnika
        lok = lokalno_ime(ime)
        stog.pop()
        roditelj = stog[-1] if stog else ""
        sadrzaj = "".join(tekst_stog.pop()).strip()

        if lok == "Document":
            dubina_spremnika -= 1
        elif lok == "Folder":
            dubina_mapa -= 1
            dubina_spremnika -= 1
        elif lok == "name" and roditelj == "Placemark" and sadrzaj:
            brojevi["objekti_s_nazivom"] += 1
        elif lok == "description" and sadrzaj:
            if roditelj == "Placemark":
                brojevi["objekti_s_opisom"] += 1
            elif roditelj in ("Folder", "Document"):
                brojevi["mape_s_opisom"] += 1
        elif lok == "coordinates":
            # Samo koordinate unutar geometrija (ne unutar gx:LatLonQuad u GroundOverlayu)
            u_geometriji = any(
                g in stog for g in ("Point", "LineString", "LinearRing", "Polygon")
            )
            if u_geometriji:
                n_vrhova = 0
                for token in sadrzaj.split():
                    dijelovi = token.split(",")
                    if len(dijelovi) < 2:
                        raise ValueError("neispravna_koordinata")
                    float(dijelovi[0])
                    float(dijelovi[1])
                    if len(dijelovi) >= 3 and dijelovi[2] != "":
                        z = float(dijelovi[2])
                        if abs(z) > 1e-9:
                            brojevi["vrhovi_s_nenultom_visinom"] += 1
                    n_vrhova += 1
                brojevi["ukupno_vrhova"] += n_vrhova
                for g in geom_vrhovi_stog:
                    g[0] += n_vrhova
        elif lok == "coord" and "Track" in stog:
            dijelovi = sadrzaj.split()
            if len(dijelovi) < 2:
                raise ValueError("neispravna_gx_coord")
            float(dijelovi[0])
            float(dijelovi[1])
            if len(dijelovi) >= 3:
                z = float(dijelovi[2])
                if abs(z) > 1e-9:
                    brojevi["vrhovi_s_nenultom_visinom"] += 1
            brojevi["ukupno_vrhova"] += 1
            for g in geom_vrhovi_stog:
                g[0] += 1
        elif lok in ("Point", "LineString", "Polygon", "LinearRing", "Track"):
            n_vrh = geom_vrhovi_stog.pop()[0]
            if lok == "Point" and n_vrh == 0:
                raise ValueError("prazna_tocka")
            if lok in ("Polygon", "LinearRing") and n_vrh < 3:
                raise ValueError("prekratak_prsten")
            if lok == "Track" and n_vrh == 0:
                raise ValueError("prazan_trag")

    def tekst(podaci):
        if tekst_stog:
            tekst_stog[-1].append(podaci)

    p = xml.parsers.expat.ParserCreate()
    p.StartElementHandler = pocetak
    p.EndElementHandler = kraj
    p.CharacterDataHandler = tekst
    p.Parse(kml_bajtovi, True)

    brojevi["najveca_dubina_mapa"] = maks_dubina_mapa
    brojevi["najveca_dubina_spremnika"] = maks_dubina_spremnika
    return brojevi


def prebroji_rupe_geometrije_ogr(geom) -> int:
    if geom is None:
        return 0
    ime = geom.GetGeometryName()
    if ime == "POLYGON":
        n = geom.GetGeometryCount()
        return max(0, n - 1)
    rupe = 0
    for i in range(geom.GetGeometryCount()):
        rupe += prebroji_rupe_geometrije_ogr(geom.GetGeometryRef(i))
    return rupe


def provjeri_s_ogrinfo_i_ogr(put: str):
    """Pokreće `/Applications/QGIS-LTR.app/Contents/MacOS/ogrinfo -ro -al -so` i `osgeo.ogr`."""
    try:
        proc = subprocess.run(
            [OGRINFO_PUT, "-ro", "-al", "-so", put],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode != 0:
            return 0, 0, 0, 0
        brojevi_slojeva = [
            int(m.group(1))
            for m in re.finditer(r"^Feature Count:\s*(\d+)", proc.stdout, re.MULTILINE)
        ]
        ogr_slojevi = len(brojevi_slojeva)
        ogr_objekti = sum(brojevi_slojeva)

        ds = ogr.Open(put, 0)
        if ds is None:
            return 0, 0, 0, 0
        ogr_rupe = 0
        for i in range(ds.GetLayerCount()):
            sloj = ds.GetLayerByIndex(i)
            sloj.ResetReading()
            for feat in sloj:
                ogr_rupe += prebroji_rupe_geometrije_ogr(feat.GetGeometryRef())
        return 1, ogr_slojevi, ogr_objekti, ogr_rupe
    except Exception:
        return 0, 0, 0, 0


def glavna():
    if len(sys.argv) < 4:
        sys.exit(2)
    mapa_ref = sys.argv[1]
    oznaka = sys.argv[2]
    put = sys.argv[3]
    izlaz_csv = os.path.join(mapa_ref, f"{oznaka}.csv")

    try:
        kml_bajtovi, slike_u_kmz = ucitaj_kml_iz_datoteke(put)
        brojevi = prebroji_xml(kml_bajtovi, slike_u_kmz)
    except Exception as e:
        with open(izlaz_csv, "w", encoding="utf-8") as f:
            f.write("neotvoriva;1\n")
        print(f"  ref     {oznaka:<24} -> neispravna datoteka ({type(e).__name__})")
        return

    ogr_otvara, ogr_slojevi, ogr_objekti, ogr_rupe = provjeri_s_ogrinfo_i_ogr(put)
    brojevi["ogr_otvara"] = ogr_otvara
    brojevi["ogr_slojevi"] = ogr_slojevi
    brojevi["ogr_objekti"] = ogr_objekti
    brojevi["ogr_rupe"] = ogr_rupe

    with open(izlaz_csv, "w", encoding="utf-8") as f:
        for kljuc, vrij in brojevi.items():
            f.write(f"{kljuc};{vrij}\n")

    print(
        f"  ref     {oznaka:<24} -> obj={brojevi['objekti']}, vrhova={brojevi['ukupno_vrhova']}, rupe={brojevi['rupe_poligona']} (ogrinfo: slojeva={ogr_slojevi}, obj={ogr_objekti}, rupe={ogr_rupe})"
    )


if __name__ == "__main__":
    glavna()
