"""GEOS iz QGIS-a kaže je li lik ispravan, a CSV s odgovorima čita test `tests/presjek_qgis.rs`.

Svi likovi su izmišljeni. Stupci: ime, ispravan (1 ili 0), razlog iz GEOS-a, prstenovi.
Prstenovi su odvojeni s `|`, točke razmakom, a dužina i širina dvotočkom.
"""

import random
import sys

from qgis.core import Qgis, QgsApplication, QgsGeometry, QgsPointXY

RUCNI = {
    "kvadrat": [[(0, 0), (2, 0), (2, 2), (0, 2)]],
    "kvadrat obratno": [[(0, 2), (2, 2), (2, 0), (0, 0)]],
    "osmica": [[(0, 0), (2, 2), (2, 0), (0, 2)]],
    "dodir vrhom na rubu": [[(0, 0), (4, 0), (4, 4), (2, 0), (0, 4)]],
    "isti vrh dvaput": [[(0, 0), (2, 2), (4, 0), (4, 4), (2, 2), (0, 4)]],
    "siljak": [[(0, 0), (4, 0), (2, 0), (2, 2)]],
    "preklopljeni rubovi": [[(0, 0), (4, 0), (4, 2), (3, 2), (3, 0), (1, 0), (1, 2), (0, 2)]],
    "ponovljena tocka": [[(0, 0), (2, 0), (2, 0), (2, 2), (0, 2)]],
    "ravan vrh": [[(0, 0), (1, 0), (2, 0), (2, 2), (0, 2)]],
    "rupa unutra": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(2, 2), (4, 2), (4, 4), (2, 4)]],
    "rupa preko ruba": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(8, 2), (12, 2), (12, 4), (8, 4)]],
    "rupa dotice u tocki": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(5, 0), (6, 2), (4, 2)]],
    "rupa na rubu": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(2, 0), (4, 0), (4, 2), (2, 2)]],
    "dvije rupe se krizaju": [
        [(0, 0), (10, 0), (10, 10), (0, 10)],
        [(2, 2), (4, 2), (4, 4), (2, 4)],
        [(3, 3), (5, 3), (5, 5), (3, 5)],
    ],
    "dvije rupe se dodiruju u tocki": [
        [(0, 0), (10, 0), (10, 10), (0, 10)],
        [(2, 2), (4, 2), (4, 4), (2, 4)],
        [(4, 4), (6, 4), (6, 6), (4, 6)],
    ],
    "rupa s osmicom": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(2, 2), (4, 4), (4, 2), (2, 4)]],
    "rupa izvan lika": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(20, 2), (22, 2), (22, 4), (20, 4)]],
    "rupa kroz vrh lika": [[(0, 0), (10, 0), (10, 10), (0, 10)], [(8, 8), (12, 12), (8, 12)]],
    "trokut na pravcu": [[(0, 0), (1, 0), (2, 0)]],
    "parcela u Osijeku": [[(18.68, 45.54), (18.70, 45.54), (18.70, 45.56), (18.68, 45.56)]],
    "parcela s prekrizenim rubom": [[(18.68, 45.54), (18.70, 45.56), (18.70, 45.54), (18.68, 45.56)]],
}


def nasumicni(g, n):
    likovi = {}
    for i in range(n):
        mreza = g.random() < 0.5
        tocka = (lambda: (g.randrange(6), g.randrange(6))) if mreza else (
            lambda: (round(g.uniform(0, 10), 4), round(g.uniform(0, 10), 4)))
        vanjski = [tocka() for _ in range(3 + g.randrange(6))]
        prstenovi = [vanjski]
        if g.random() < 0.3:
            # Rupa unutar obuhvata lika, mala, da bude i ispravnih likova s rupom.
            x = min(t[0] for t in vanjski) + 1
            y = min(t[1] for t in vanjski) + 1
            prstenovi.append([(x, y), (x + 0.5, y), (x + 0.5, y + 0.5), (x, y + 0.5)])
        likovi[f"nasumican {i}"] = prstenovi
    return likovi


def razlika_tocaka(prsten):
    """Bar tri različite točke; inače to nije lik ni za GEOS ni za nas."""
    return len({tuple(t) for t in prsten}) >= 3


def main(izlaz, prefiks):
    QgsApplication.setPrefixPath(prefiks, True)
    app = QgsApplication([], False)
    app.initQgis()
    likovi = dict(RUCNI)
    likovi.update(nasumicni(random.Random(20261109), 400))
    redci = ["ime,ispravan,razlog,prstenovi"]
    for ime, prstenovi in likovi.items():
        if not all(razlika_tocaka(p) for p in prstenovi):
            continue
        g = QgsGeometry.fromPolygonXY([[QgsPointXY(x, y) for x, y in p] for p in prstenovi])
        greske = g.validateGeometry(Qgis.GeometryValidationEngine.Geos)
        razlog = greske[0].what().replace(",", " ") if greske else ""
        zapis = "|".join(" ".join(f"{x}:{y}" for x, y in p) for p in prstenovi)
        redci.append(f"{ime},{0 if greske else 1},{razlog},{zapis}")
    with open(izlaz, "w", encoding="utf-8") as f:
        f.write("\n".join(redci) + "\n")
    print(f"{len(redci) - 1} likova, od toga neispravnih:",
          sum(1 for r in redci[1:] if r.split(",")[1] == "0"))
    app.exitQgis()


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
