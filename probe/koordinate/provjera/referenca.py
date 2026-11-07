"""Pravi QGIS računa kontrolne točke i upisuje ih u CSV koji proba čita.

Rezultat QGIS-a je mjerilo, a ne proizvod ove probe. Točke su javni gradovi i
pravilna mreža preko Hrvatske, bez ikakvih podataka iz goCOP-a.
"""

import math
import random
import sqlite3
import struct
import sys
import tempfile
from pathlib import Path

from pyproj import Geod
from qgis.core import (
    QgsApplication,
    QgsCoordinateReferenceSystem as Crs,
    QgsCoordinateTransform as Pretvorba,
    QgsCoordinateTransformContext as Kontekst,
    QgsDatumTransform,
    QgsFeature,
    QgsField,
    QgsFields,
    QgsGeometry,
    QgsPointXY,
    QgsVectorFileWriter,
    QgsWkbTypes,
)

GRADOVI = {
    "Pula": (13.85, 44.87),
    "Rijeka": (14.44, 45.33),
    "Zagreb": (15.98, 45.81),
    "Zadar": (15.23, 44.12),
    "Split": (16.44, 43.51),
    "Dubrovnik": (18.09, 42.65),
    "Osijek": (18.69, 45.55),
    "Vukovar": (19.00, 45.35),
    "Ilok": (19.38, 45.22),
    "Cakovec": (16.43, 46.38),
}
WGS = Crs("EPSG:4326")


def tocke():
    """Gradovi, mreža preko Hrvatske i slučajne točke s istim sjemenom."""
    izlaz = list(GRADOVI.values())
    sirina = 42.3
    while sirina <= 46.61:
        duzina = 13.3
        while duzina <= 19.51:
            izlaz.append((round(duzina, 3), round(sirina, 3)))
            duzina += 0.4
        sirina += 0.4
    slucajni = random.Random(20261107)
    for _ in range(200):
        izlaz.append((slucajni.uniform(13.3, 19.5), slucajni.uniform(42.3, 46.6)))
    return izlaz


def pretvorba(iz, u, operacija=None):
    """QGIS-ova pretvorba, po želji s izabranom operacijom iz baze EPSG-a."""
    p = Pretvorba(Crs(iz), Crs(u), Kontekst())
    if operacija:
        izabrane = [
            o for o in QgsDatumTransform.operations(Crs(iz), Crs(u))
            if operacija in o.name
        ]
        assert len(izabrane) == 1, (iz, u, operacija, [o.name for o in izabrane])
        p.setCoordinateOperation(izabrane[0].proj)
    return p


def racunaj(p, x, y):
    t = p.transform(QgsPointXY(x, y))
    return t.x(), t.y()


def redak(pisac, grupa, x, y, a, b):
    pisac.write(f"{grupa},{x!r},{y!r},{a!r},{b!r}\n")


def smjerovi(pisac, ime, epsg, sve, operacija=None):
    """Iz WGS-a u sustav i natrag, s pravim QGIS-ovim rezultatom u oba smjera."""
    naprijed = pretvorba("EPSG:4326", epsg, operacija)
    natrag = pretvorba(epsg, "EPSG:4326", operacija)
    for lon, lat in sve:
        x, y = racunaj(naprijed, lon, lat)
        redak(pisac, f"4326>{ime}", lon, lat, x, y)
        a, b = racunaj(natrag, x, y)
        redak(pisac, f"{ime}>4326", x, y, a, b)


def medu_sustavima(pisac, iz, u, sve):
    """Izravno između dva projicirana sustava, kao kad QGIS slaže karte."""
    do_izvora = pretvorba("EPSG:4326", f"EPSG:{iz}")
    p = pretvorba(f"EPSG:{iz}", f"EPSG:{u}")
    for lon, lat in sve:
        x, y = racunaj(do_izvora, lon, lat)
        a, b = racunaj(p, x, y)
        redak(pisac, f"{iz}>{u}", x, y, a, b)
    p.transform(QgsPointXY(*racunaj(do_izvora, *GRADOVI["Osijek"])))
    return p.instantiatedCoordinateOperationDetails().name


def kut_okvira(pisac, sve):
    """Za koliko je sjever lista u HTRS96/TM zakrenut na ekranu u Mercatoru."""
    do_tm = pretvorba("EPSG:4326", "EPSG:3765")
    tm_u_ekran = pretvorba("EPSG:3765", "EPSG:3857")
    for lon, lat in sve:
        x, y = racunaj(do_tm, lon, lat)
        x0, y0 = racunaj(tm_u_ekran, x, y)
        x1, y1 = racunaj(tm_u_ekran, x, y + 1000.0)
        kut = math.degrees(math.atan2(x1 - x0, y1 - y0))
        redak(pisac, "kut", x, y, kut, 0.0)


def mjerilo_ekrana(pisac, sve):
    """Koliko je metara Mercatora na metar tla, istok-zapad i sjever-jug."""
    geod = Geod(ellps="WGS84")
    ekran = pretvorba("EPSG:4326", "EPSG:3857")
    korak = 0.01
    for lon, lat in sve:
        x0, y0 = racunaj(ekran, lon, lat)
        x1, _ = racunaj(ekran, lon + korak, lat)
        _, y1 = racunaj(ekran, lon, lat + korak)
        tlo_i = geod.inv(lon, lat, lon + korak, lat)[2]
        tlo_s = geod.inv(lon, lat, lon, lat + korak)[2]
        redak(pisac, "mjerilo", lon, lat, (x1 - x0) / tlo_i, (y1 - y0) / tlo_s)


def zapis_u_geopaket(epsg, mapa_izlaza):
    """Što QGIS stvarno upiše u GeoPackage za taj sustav: broj, definiciju, osi."""
    p = pretvorba("EPSG:4326", f"EPSG:{epsg}")
    x, y = racunaj(p, *GRADOVI["Osijek"])
    with tempfile.TemporaryDirectory() as mapa:
        put = str(Path(mapa) / "sloj.gpkg")
        polja = QgsFields()
        polja.append(QgsField("ime", 10))
        opcije = QgsVectorFileWriter.SaveVectorOptions()
        opcije.driverName = "GPKG"
        opcije.layerName = "sloj"
        pisac = QgsVectorFileWriter.create(
            put, polja, QgsWkbTypes.Point, Crs(f"EPSG:{epsg}"), Kontekst(), opcije
        )
        objekt = QgsFeature(polja)
        objekt.setGeometry(QgsGeometry.fromPointXY(QgsPointXY(x, y)))
        objekt.setAttribute("ime", "Osijek")
        pisac.addFeature(objekt)
        del pisac
        veza = sqlite3.connect(put)
        blob = veza.execute("select geom from sloj").fetchone()[0]
        broj, organizacija, definicija = veza.execute(
            "select s.srs_id, s.organization, s.definition from gpkg_spatial_ref_sys s"
            " join gpkg_contents c on c.srs_id = s.srs_id where c.table_name = 'sloj'"
        ).fetchone()
        veza.close()
    # Zaglavlje GeoPackagea: 8 bajtova, bez omotnice za točku; WKB: 5 + x, y.
    prvi, drugi = struct.unpack("<dd", blob[8 + 5 : 8 + 5 + 16])
    if abs(prvi - x) > 0.001 or abs(drugi - y) > 0.001:
        raise SystemExit(f"PAO redoslijed osi {epsg}: {prvi} {drugi} prema {x} {y}")
    (mapa_izlaza / f"wkt_{epsg}.txt").write_text(definicija)
    return (
        f"EPSG:{epsg} u GeoPackageu: broj {broj}, organizacija {organizacija}, "
        f"prva koordinata {prvi:.3f} (istok), druga {drugi:.3f} (sjever)"
    )


def stari_mgi():
    """Koliko se isti brojevi razlikuju kad datoteka kaže 31276 umjesto 3908."""
    izlaz = []
    do_hdks = pretvorba("EPSG:4326", "EPSG:3908")
    mgi = pretvorba("EPSG:31276", "EPSG:4326")
    hdks = pretvorba("EPSG:3908", "EPSG:4326")
    for ime, (lon, lat) in GRADOVI.items():
        x, y = racunaj(do_hdks, lon, lat)
        a = racunaj(mgi, x, y)
        b = racunaj(hdks, x, y)
        de = (a[0] - b[0]) * 111320 * math.cos(math.radians(lat))
        dn = (a[1] - b[1]) * 111320
        izlaz.append(f"{ime}: {math.hypot(de, dn):.1f} m")
    naziv = mgi.instantiatedCoordinateOperationDetails().name
    return naziv, izlaz


def main():
    izlaz = Path(sys.argv[1])
    app = QgsApplication([], False)
    app.initQgis()
    sve = tocke()
    with izlaz.open("w") as pisac:
        pisac.write("grupa,ulaz_x,ulaz_y,izlaz_a,izlaz_b\n")
        smjerovi(pisac, "3765", "EPSG:3765", sve)
        smjerovi(pisac, "3857", "EPSG:3857", sve)
        smjerovi(pisac, "3907", "EPSG:3907", sve)
        smjerovi(pisac, "3908", "EPSG:3908", sve)
        smjerovi(pisac, "3907#4", "EPSG:3907", sve, "(4)")
        smjerovi(pisac, "3908#4", "EPSG:3908", sve, "(4)")
        smjerovi(pisac, "8677", "EPSG:8677", sve)
        smjerovi(pisac, "8678", "EPSG:8678", sve)
        for iz, u in ((3765, 3857), (3857, 3765), (3765, 3908), (3908, 3765), (3908, 3857)):
            print(f"izravno {iz}>{u}:", medu_sustavima(pisac, iz, u, sve))
        kut_okvira(pisac, sve)
        mjerilo_ekrana(pisac, sve)
    print("točaka:", len(sve))
    for epsg in (3765, 3907, 3908, 8677, 8678, 3857):
        print(zapis_u_geopaket(epsg, izlaz.parent))
    naziv, razlike = stari_mgi()
    print("31276 u QGIS-u koristi:", naziv)
    print("  isti brojevi kao 31276 umjesto 3908:", ", ".join(razlike))
    app.exitQgis()


main()
