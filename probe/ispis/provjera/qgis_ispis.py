"""Isti list u pravom QGIS-u: A3, isti okvir, isto mjerilo, isti objekti.

Objekti stižu iz CSV-a koji piše proba. Podaci su izmišljeni, samo smješteni oko
Osijeka, bez ičega iz goCOP-a.
"""

import csv
import sys

from qgis.core import (
    QgsApplication,
    QgsCoordinateReferenceSystem,
    QgsFeature,
    QgsFillSymbol,
    QgsGeometry,
    QgsLayoutExporter,
    QgsLayoutItemMap,
    QgsLayoutPoint,
    QgsLayoutSize,
    QgsLineSymbol,
    QgsMarkerSymbol,
    QgsPrintLayout,
    QgsProject,
    QgsRectangle,
    QgsSingleSymbolRenderer,
    QgsUnitTypes,
    QgsVectorLayer,
)

MM = QgsUnitTypes.LayoutMillimeters


def simbol(r):
    boja, ispuna = r["boja"], r["ispuna"]
    debljina = r["debljina"]
    prozirnost = float(r["prozirnost"])
    if r["vrsta"] == "tocka":
        s = QgsMarkerSymbol.createSimple({
            "name": r["znak"], "color": boja, "size": debljina,
            "size_unit": "MM", "outline_style": "no",
        })
    elif r["vrsta"] == "crta":
        svojstva = {
            "line_color": boja, "line_width": debljina, "line_width_unit": "MM",
            "capstyle": "flat", "joinstyle": "miter",
        }
        if r["crtice"]:
            svojstva.update({
                "use_custom_dash": "1", "customdash": r["crtice"],
                "customdash_unit": "MM",
            })
        s = QgsLineSymbol.createSimple(svojstva)
    else:
        s = QgsFillSymbol.createSimple({
            "color": ispuna, "style": "solid", "outline_color": boja,
            "outline_width": debljina, "outline_width_unit": "MM",
            "joinstyle": "miter",
        })
    s.setOpacity(prozirnost)
    return s


def sloj(r):
    vrsta = {"tocka": "Point", "crta": "MultiLineString", "lik": "Polygon"}[r["vrsta"]]
    v = QgsVectorLayer(f"{vrsta}?crs=EPSG:3765", r["ime"], "memory")
    objekt = QgsFeature()
    objekt.setGeometry(QgsGeometry.fromWkt(r["wkt"]))
    assert not objekt.geometry().isNull(), r["wkt"]
    assert v.dataProvider().addFeatures([objekt])[0], r["ime"]
    v.updateExtents()
    v.setRenderer(QgsSingleSymbolRenderer(simbol(r)))
    return v


def main():
    mapa = sys.argv[1]
    ulaz, izlaz = f"{mapa}/objekti.csv", f"{mapa}/qgis.pdf"
    with open(f"{mapa}/list.csv", encoding="utf-8") as f:
        podaci = next(csv.DictReader(f, delimiter=";"))
    p = {k: float(v) for k, v in podaci.items() if k != "ime"}
    app = QgsApplication([], False)
    app.initQgis()
    projekt = QgsProject.instance()
    sustav = QgsCoordinateReferenceSystem("EPSG:3765")
    projekt.setCrs(sustav)
    with open(ulaz, encoding="utf-8") as f:
        redci = list(csv.DictReader(f, delimiter=";"))
    # U CSV-u je redoslijed crtanja, a QGIS traži prvo gornji sloj.
    slojevi = [sloj(r) for r in redci]
    projekt.addMapLayers(slojevi)

    nacrt = QgsPrintLayout(projekt)
    nacrt.initializeDefaults()
    nacrt.pageCollection().pages()[0].setPageSize(QgsLayoutSize(p["sirina"], p["visina"], MM))
    karta = QgsLayoutItemMap(nacrt)
    karta.attemptMove(QgsLayoutPoint(p["x"], p["y"], MM))
    karta.attemptResize(QgsLayoutSize(p["okvir_s"], p["okvir_v"], MM))
    karta.setCrs(sustav)
    karta.setLayers(list(reversed(slojevi)))
    karta.setExtent(QgsRectangle(p["zapad"], p["jug"], p["istok"], p["sjever"]))
    nacrt.addLayoutItem(karta)
    # Bez sustava karte QGIS bi tiho javio besmisleno mjerilo.
    if abs(karta.scale() - 5000) > 0.5:
        raise SystemExit(f"QGIS kaže za mjerilo: 1 : {karta.scale():.3f}")
    print(f"QGIS kaže za mjerilo: 1 : {karta.scale():.3f}")
    print(f"QGIS kaže za okvir: {karta.extent().toString(1)}")

    postavke = QgsLayoutExporter.PdfExportSettings()
    postavke.dpi = 600
    postavke.forceVectorOutput = True
    postavke.rasterizeWholeImage = False
    postavke.simplifyGeometries = False
    rezultat = QgsLayoutExporter(nacrt).exportToPdf(izlaz, postavke)
    if rezultat != QgsLayoutExporter.Success:
        raise SystemExit(f"QGIS nije izvezao PDF: {rezultat}")
    app.exitQgis()


main()
