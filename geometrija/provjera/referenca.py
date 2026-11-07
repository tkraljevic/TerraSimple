"""QGIS i PROJ računaju kontrolne mjere i upisuju ih u CSV koji testovi čitaju.

Mjerilo je tuđi račun, a ne naš. Svi likovi su izmišljeni ili od javnih koordinata gradova.
Stupci: ime, vrsta, točke, Karneyjev račun iz PROJ-a (duljina ili opseg, površina) i QGIS.
"""

import math
import random
import sys

from pyproj import Geod
from qgis.core import (
    QgsApplication,
    QgsCoordinateReferenceSystem as Crs,
    QgsCoordinateTransformContext as Kontekst,
    QgsDistanceArea,
    QgsGeometry,
    QgsPointXY,
)

OSIJEK = (18.69, 45.55)
ZAGREB = (15.98, 45.81)
SPLIT = (16.44, 43.51)
DUBROVNIK = (18.09, 42.65)
PULA = (13.85, 44.87)
CAKOVEC = (16.43, 46.38)
ILOK = (19.38, 45.22)

# Primjer iz uputa GeographicLiba, koji ima objavljen rezultat.
ANTARKTIKA = [(-58, -63.1), (-74, -72.9), (-102, -71.9), (-102, -74.9), (-131, -74.3),
              (-163, -77.5), (163, -77.4), (172, -71.7), (140, -65.9), (113, -65.7),
              (88, -66.6), (59, -66.9), (25, -69.8), (-4, -70.0), (-14, -71.0),
              (-33, -77.3), (-46, -77.9), (-61, -74.7)]


def pomak(sredina, dx_m, dy_m):
    return (sredina[0] + dx_m / (111_320 * 0.70), sredina[1] + dy_m / 111_132)


def pravokutnik(sredina, sirina_m, visina_m):
    return [pomak(sredina, x, y) for x, y in
            [(0, 0), (sirina_m, 0), (sirina_m, visina_m), (0, visina_m)]]


CRTE = {
    "osijek-zagreb": [OSIJEK, ZAGREB],
    "zagreb-split-dubrovnik": [ZAGREB, SPLIT, DUBROVNIK],
    "jfk-lhr": [(-73.8, 40.6), (-0.5, 51.6)],
    "gotovo-antipodi": [(0, 0), (179.5, 0.5)],
    "preko-180": [(179, 0), (-179, 0)],
    "preko-pola": [(0, 89), (180, 89)],
    "centimetar": [OSIJEK, pomak(OSIJEK, 0.01, 0)],
    "ponovljena-tocka": [OSIJEK, OSIJEK, ZAGREB, ZAGREB],
    "tam-i-natrag": [OSIJEK, ZAGREB, OSIJEK],
    "jedna-tocka": [OSIJEK],
    "prazna": [],
}

PARCELA = [pomak(OSIJEK, x, y) for x, y in
           [(0, 0), (250, -40), (310, 120), (220, 260), (40, 230), (-30, 110)]]
KV = pravokutnik(OSIJEK, 1000, 1000)
RUPA = pravokutnik(pomak(OSIJEK, 250, 250), 500, 500)
RUPA2 = pravokutnik(pomak(OSIJEK, 800, 800), 100, 100)

LIKOVI = {
    "parcela": [PARCELA],
    "parcela-obrnuto": [PARCELA[::-1]],
    "kvadrat-1km": [KV],
    "kvadrat-1km-obrnuto": [KV[::-1]],
    "kvadrat-1m": [pravokutnik(OSIJEK, 1, 1)],
    "uska-traka": [pravokutnik(OSIJEK, 50_000, 2)],
    "kvadrat-100km": [pravokutnik(OSIJEK, 100_000, 100_000)],
    "hrvatska-grubo": [[PULA, CAKOVEC, ILOK, DUBROVNIK, SPLIT, ZAGREB]],
    "sa-rupom": [KV, RUPA],
    "sa-dvije-rupe": [KV, RUPA, RUPA2],
    "rupa-obrnuto": [KV, RUPA[::-1]],
    "preko-180-lik": [[(179, 10), (-179, 10), (-179, 12), (179, 12)]],
    "sjever": [[(10, 78), (30, 78), (30, 80), (10, 80)]],
    "antarktika": [ANTARKTIKA],
    "osmica": [[(18, 45), (18.1, 45.1), (18.1, 45), (18, 45.1)]],
    "osmica-nejednaka": [[(18, 45), (18.2, 45.1), (18.2, 45), (18, 45.05)]],
    "na-crti": [[(18, 45), (18.1, 45), (18.2, 45)]],
    "dvije-tocke": [[(18, 45), (18.1, 45)]],
    "prazan": [[]],
}

sluc = random.Random(20261107)
for i in range(120):
    n = sluc.randint(3, 24)
    sredina = (sluc.uniform(13.5, 19.4), sluc.uniform(42.5, 46.5))
    r = 10 ** sluc.uniform(1, 5)
    tocke = [pomak(sredina, r * sluc.uniform(0.5, 1) * math.cos(2 * math.pi * k / n),
                   r * sluc.uniform(0.5, 1) * math.sin(2 * math.pi * k / n)) for k in range(n)]
    LIKOVI[f"slucajni-{i:03}"] = [tocke]


def zapis(prstenovi):
    return "|".join(" ".join(f"{t[0]!r}:{t[1]!r}" for t in p) for p in prstenovi)


def glavni(staza):
    QgsApplication.setPrefixPath(sys.argv[2], True)
    app = QgsApplication([], False)
    app.initQgis()
    g = Geod(ellps="WGS84")
    d = QgsDistanceArea()
    d.setSourceCrs(Crs("EPSG:4326"), Kontekst())
    if not d.setEllipsoid("EPSG:7030"):
        raise SystemExit("QGIS nije prihvatio elipsoid")

    def xy(p):
        return [QgsPointXY(*t) for t in p]

    def pp(p):
        if len(p) < 3:
            return 0.0, 0.0
        a, o = g.polygon_area_perimeter([t[0] for t in p], [t[1] for t in p])
        return o, abs(a)

    with open(staza, "w", encoding="utf-8") as f:
        f.write("ime,vrsta,tocke,karney_d,karney_p,qgis_d,qgis_p\n")
        for ime, t in CRTE.items():
            k = g.line_length([x[0] for x in t], [x[1] for x in t]) if len(t) > 1 else 0.0
            q = d.measureLength(QgsGeometry.fromPolylineXY(xy(t))) if len(t) > 1 else 0.0
            f.write(f"{ime},crta,{zapis([t])},{k:.6f},0,{q:.6f},0\n")
        for ime, prst in LIKOVI.items():
            mjere = [pp(p) for p in prst]
            o = sum(m[0] for m in mjere)
            a = mjere[0][1] - sum(m[1] for m in mjere[1:])
            if len(prst[0]) >= 3:
                q = QgsGeometry.fromPolygonXY([xy(p) for p in prst])
                qo, qa = d.measurePerimeter(q), d.measureArea(q)
            else:
                qo, qa = 0.0, 0.0
            f.write(f"{ime},lik,{zapis(prst)},{o:.6f},{a:.6f},{qo:.6f},{qa:.6f}\n")
    app.exitQgis()


glavni(sys.argv[1])
