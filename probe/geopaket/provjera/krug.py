"""Cijeli krug s pravim QGIS-om, bez prozora: naša datoteka u QGIS-u, izmjena u QGIS-u kod
nas, QGIS-ova datoteka kod nas. Pokreće se Pythonom iz QGIS-a, vidi pokreni.sh."""
import json
import os
import shutil
import sqlite3
import subprocess
import sys

from qgis.core import (
    Qgis, QgsApplication, QgsCoordinateReferenceSystem, QgsCoordinateTransform, QgsFeature, QgsFeatureRequest,
    QgsField, QgsGeometry, QgsLineSymbol, QgsMapRendererCustomPainterJob, QgsMapSettings, QgsPointXY, QgsProject,
    QgsProviderRegistry, QgsRectangle, QgsSingleSymbolRenderer, QgsSymbolLayer, QgsVectorFileWriter, QgsVectorLayer,
    QgsCoordinateTransformContext,
)
from qgis.PyQt.QtCore import QSize, QVariant, Qt
from qgis.PyQt.QtGui import QColor, QImage, QPainter

PROBA, MAPA, VALIDATOR = sys.argv[1], sys.argv[2], sys.argv[3]
NAS = os.path.join(MAPA, "nas.gpkg")
TUDA = os.path.join(MAPA, "iz_qgisa.gpkg")
NASIPI = "Nasipi čđžšć"
palo = []
prolaza = 0


def provjeri(naziv, uvjet, podatak=""):
    global prolaza
    if uvjet:
        prolaza += 1
        print(f"  u redu  {naziv}")
    else:
        palo.append(naziv)
        print(f"  PALO    {naziv}  {podatak!r}")


def nalaz(tekst):
    print(f"  NALAZ   {tekst}")


def naslov(t):
    print(f"\n== {t}")


def program(*a, provjeri_kod=True):
    r = subprocess.run([PROBA, *a], capture_output=True, text=True)
    if provjeri_kod and r.returncode != 0:
        raise RuntimeError(f"{a}: {r.stderr}")
    return r


def citaj(put=NAS):
    return json.loads(program("citaj", put).stdout)


def sloj(put, tablica):
    return QgsVectorLayer(f"{put}|layername={tablica}", tablica, "ogr")


def validator(put):
    py = os.path.join(os.environ["QGIS_PREFIX_PATH"], "python3.12")
    r = subprocess.run([py, VALIDATOR, put], capture_output=True, text=True)
    redovi = [x for x in (r.stdout + r.stderr).splitlines() if x.strip() and "platform" not in x]
    return r.returncode, "\n".join(redovi)


def uz_datoteku(put):
    return sorted(x[len(os.path.basename(put)):] for x in os.listdir(os.path.dirname(put)) if x.startswith(os.path.basename(put)))


def otisak(put, tablica):
    c = sqlite3.connect(put)
    r = c.execute(f"select * from {tablica} order by 1").fetchall()
    c.close()
    return r


def blizu(a, b, t=1e-9):
    return abs(a - b) <= t


def nas_u_qgisu():
    naslov("1. Naša datoteka u QGIS-u")
    kod, tekst = validator(NAS)
    print(f"  validator: kod {kod}; {tekst[:800]}")
    provjeri("službeni GDAL-ov validator nema primjedbi", kod == 0 and not tekst, tekst[:600])

    t = sloj(NAS, "tocke")
    n = sloj(NAS, NASIPI)
    p = sloj(NAS, "parcele")
    provjeri("sva tri sloja se otvaraju", t.isValid() and n.isValid() and p.isValid())
    podslojevi = [d.name() for d in QgsProviderRegistry.instance().querySublayers(NAS)]
    nalaz(f"QGIS pri otvaranju datoteke nudi: {podslojevi}")
    provjeri("sloj s dijakritikom i razmakom u imenu", NASIPI in podslojevi, podslojevi)
    provjeri("naše tablice QGIS ne nudi kao slojeve", not any("ts_" in x for x in podslojevi), podslojevi)
    provjeri("sustav točaka je 4326", t.crs().authid() == "EPSG:4326", t.crs().authid())
    provjeri("sustav linija i likova je 3765", n.crs().authid() == "EPSG:3765" and p.crs().authid() == "EPSG:3765")
    provjeri("vrste geometrije", (t.wkbType(), n.wkbType(), p.wkbType()) == (Qgis.WkbType.Point, Qgis.WkbType.LineString, Qgis.WkbType.Polygon), (t.wkbType(), n.wkbType(), p.wkbType()))
    provjeri("broj objekata 3, 3, 3", (t.featureCount(), n.featureCount(), p.featureCount()) == (3, 3, 3), (t.featureCount(), n.featureCount(), p.featureCount()))

    polja = {f.name(): f.typeName() for f in t.fields()}
    nalaz(f"stupci kako ih vidi QGIS: {polja}")
    prvi = next(t.getFeatures(QgsFeatureRequest().setFilterFid(1)))
    provjeri("hrvatska slova u vrijednosti", prvi["naziv"] == "Šećerana čćžšđ ČĆŽŠĐ", prvi["naziv"])
    provjeri("novi red i navodnici u opisu", prvi["opis"] == "Red\nnovi 'navodnik' \"dvostruki\" & <znak>", prvi["opis"])
    provjeri("cijeli broj i decimalni broj", prvi["broj"] == 1 and prvi["visina"] == 84.25)
    provjeri("datum", prvi["datum"].toString("yyyy-MM-dd") == "2026-11-07", prvi["datum"])
    provjeri("datum i vrijeme", prvi["vrijeme"].toString(Qt.ISODate).startswith("2026-11-07T10:20:30"), prvi["vrijeme"])
    provjeri("da/ne je pravi da/ne", t.fields().field("aktivno").type() == QVariant.Bool and prvi["aktivno"] is True, (t.fields().field("aktivno").typeName(), prvi["aktivno"]))
    g = prvi.geometry().asPoint()
    provjeri("točka nije zamijenila osi", blizu(g.x(), 18.68) and blizu(g.y(), 45.54), g.toString())

    s_rupom = next(p.getFeatures(QgsFeatureRequest().setFilterFid(1))).geometry()
    provjeri("lik ima rupu i točnu površinu", blizu(s_rupom.area(), 800 * 800 - 200 * 200, 1e-6), s_rupom.area())
    provjeri("geometrija je ispravna po QGIS-u", s_rupom.isGeosValid())
    e = n.extent()
    provjeri("obuhvat sloja", (e.xMinimum(), e.xMaximum(), e.yMinimum(), e.yMaximum()) == (670000, 672000, 5044300, 5044800), e.toString())

    obuhvat = QgsFeatureRequest().setFilterRect(QgsRectangle(670900, 5044700, 671100, 5044900))
    pogodak = [f["naziv"] for f in n.getFeatures(obuhvat)]
    provjeri("prostorni indeks nalazi samo gornju liniju", pogodak == ["Stil sloja"], pogodak)
    r = sqlite3.connect(NAS).execute(f'select count(*) from "rtree_{NASIPI}_geom"').fetchone()[0]
    provjeri("u indeksu su svi objekti", r == 3, r)

    naslov("2. Stil sloja i stil objekta u QGIS-u")
    rn = n.renderer()
    provjeri("stil se učitao sam, bez ikakve naredbe", rn.type() == "singleSymbol", rn.type())
    sn = rn.symbol().symbolLayer(0)
    provjeri("boja i debljina linije", sn.color().name() == "#0f766e" and blizu(sn.width(), 0.8), (sn.color().name(), sn.width()))
    provjeri("boja i veličina točke", (lambda s: s.color().name() == "#0f766e" and blizu(s.size(), 4.0))(t.renderer().symbol().symbolLayer(0)))
    provjeri("ispuna lika", p.renderer().symbol().symbolLayer(0).color().name() == "#fbbf24")
    provjeri("vlastiti stil dolazi iz stupca", sn.dataDefinedProperties().property(QgsSymbolLayer.PropertyStrokeColor).field() == "stil_boja")
    provjeri("natpisi su uključeni", n.labelsEnabled() and n.labeling() is not None and n.labeling().settings().fieldName == "naziv", n.labelsEnabled())
    skriveni = sorted(c.name for c in n.attributeTableConfig().columns() if c.hidden and c.name)
    provjeri("stupci stil_* skriveni su u tablici", skriveni == ["stil_boja", "stil_crta", "stil_debljina"], skriveni)
    provjeri("skriveni stupac ipak postoji", n.fields().indexOf("stil_boja") >= 0)
    return t, n, p


def iscrtaj(slojevi, put):
    ms = QgsMapSettings()
    ms.setLayers(slojevi)
    ms.setDestinationCrs(QgsCoordinateReferenceSystem("EPSG:3765"))
    ms.setTransformContext(QgsCoordinateTransformContext())
    ms.setBackgroundColor(QColor("white"))
    ms.setOutputDpi(96)
    ms.setOutputSize(QSize(1200, 1200))
    ms.setExtent(QgsRectangle(669800, 5044000, 672200, 5047000))
    slika = QImage(ms.outputSize(), QImage.Format_ARGB32)
    slika.fill(QColor("white"))
    q = QPainter(slika)
    posao = QgsMapRendererCustomPainterJob(ms, q)
    posao.start()
    posao.waitForFinished()
    q.end()
    slika.save(put)
    return slika, ms


def piksel(slika, ms, x, y, crs=None):
    if crs:
        tr = QgsCoordinateTransform(QgsCoordinateReferenceSystem(crs), ms.destinationCrs(), QgsProject.instance())
        p = tr.transform(QgsPointXY(x, y))
        x, y = p.x(), p.y()
    p = ms.mapToPixel().transform(QgsPointXY(x, y))
    return int(round(p.x())), int(round(p.y()))


def boja(slika, xy):
    return slika.pixelColor(xy[0], xy[1]).name()


def debljina_i_crta(slika, ms, y_mape, boja_crte):
    """Debljina poprijeko i udio obojenih piksela uzduž crte, ispod natpisa."""
    x, y = piksel(slika, ms, 670500, y_mape)
    gusto = lambda c: abs(c.red() - QColor(boja_crte).red()) + abs(c.green() - QColor(boja_crte).green()) + abs(c.blue() - QColor(boja_crte).blue()) < 60
    debljine = []
    uzduz = 0
    x0, x1 = piksel(slika, ms, 670100, y_mape)[0], piksel(slika, ms, 670900, y_mape)[0]
    for a in range(x0, x1):
        if gusto(slika.pixelColor(a, y)):
            uzduz += 1
            d = sum(gusto(slika.pixelColor(a, b)) for b in range(y - 20, y + 21))
            debljine.append(d)
    return (sorted(debljine)[len(debljine) // 2] if debljine else 0), uzduz / (x1 - x0)


def natpisi_na_liniji(slika, ms):
    """Tamni pikseli teksta u pojasu uz liniju. Oznaka „uključeno” ne znači da se natpis vidi."""
    y = piksel(slika, ms, 670500, 5044800)[1]
    x0, x1 = piksel(slika, ms, 670000, 5044800)[0], piksel(slika, ms, 672000, 5044800)[0]
    pojas = [b for b in range(y - 25, y + 26) if abs(b - y) > 4]  # bez same linije
    tamnih = sum(1 for a in range(x0, x1) for b in pojas if slika.pixelColor(a, b).lightness() < 90)
    provjeri("natpis linije se stvarno vidi", tamnih > 30, tamnih)


def slika_u_qgisu(t, n, p):
    naslov("3. Kako to QGIS stvarno iscrta (uzorci piksela)")
    put = os.path.join(MAPA, "qgis_iscrtano.png")
    slika, ms = iscrtaj([t, n, p], put)
    print(f"  slika: {put}")

    d, udio = debljina_i_crta(slika, ms, 5044800, "#0f766e")
    provjeri("linija sa stilom sloja je tirkizna i puna", boja(slika, piksel(slika, ms, 670500, 5044800)) == "#0f766e" and udio > 0.98, (boja(slika, piksel(slika, ms, 670500, 5044800)), udio))
    provjeri("debljina 0,8 mm je oko 3 piksela na 96 dpi", 2 <= d <= 4, d)
    d2, udio2 = debljina_i_crta(slika, ms, 5044300, "#dc2626")
    provjeri("linija s vlastitim stilom je crvena i deblja", 5 <= d2 <= 7, d2)
    provjeri("vlastita crta je isprekidana", 0.4 < udio2 < 0.95, udio2)

    provjeri("ispuna lika iz stila sloja", boja(slika, piksel(slika, ms, 670150, 5045150)) == "#fbbf24", boja(slika, piksel(slika, ms, 670150, 5045150)))
    # Natpis lika stoji na sredini, pa uzorak ide u kut rupe.
    provjeri("rupa u liku je prazna", boja(slika, piksel(slika, ms, 670320, 5045320)) == "#ffffff", boja(slika, piksel(slika, ms, 670320, 5045320)))
    provjeri("vlastita ispuna lika", boja(slika, piksel(slika, ms, 671400, 5045400)) == "#2563eb", boja(slika, piksel(slika, ms, 671400, 5045400)))

    a = piksel(slika, ms, 18.68, 45.54, "EPSG:4326")
    b = piksel(slika, ms, 18.69, 45.54, "EPSG:4326")
    provjeri("točka sa stilom sloja", boja(slika, a) == "#0f766e", boja(slika, a))
    # Natpis stoji preko sredine točke, pa uzorak ide u kut znaka.
    u_kutu = boja(slika, (b[0] + 12, b[1] + 12))
    provjeri("točka s vlastitim stilom je crvena", u_kutu == "#dc2626", u_kutu)
    u_krugu = boja(slika, (a[0] + 5, a[1] + 5))
    izvan = boja(slika, (b[0] + 20, b[1] + 20))
    provjeri("vlastiti znak je kvadrat od 8 mm, a ne veći", izvan == "#ffffff", izvan)
    provjeri("znak sloja je manji krug", u_kutu != u_krugu, u_krugu)

    # Objekti bez vlastitog stila iscrtani odmah iza objekta s njim ne smiju ništa preuzeti.
    c = piksel(slika, ms, 18.70, 45.54, "EPSG:4326")
    uz_treću = boja(slika, (c[0] + 6, c[1] + 6))
    provjeri("treća točka je opet krug, a ne preuzeti kvadrat", uz_treću == "#ffffff", uz_treću)
    d3, udio3 = debljina_i_crta(slika, ms, 5044550, "#0f766e")
    provjeri("linija iza vlastitog stila je opet tanka i puna", 2 <= d3 <= 4 and udio3 > 0.98, (d3, udio3))
    provjeri("lik iza vlastitog stila ima ispunu sloja", boja(slika, piksel(slika, ms, 672000, 5045050)) == "#fbbf24", boja(slika, piksel(slika, ms, 672000, 5045050)))
    def rub(x_mape, y_mape):
        x, y = piksel(slika, ms, x_mape, y_mape)
        return sum(1 for a in range(x - 8, x + 9) if slika.pixelColor(a, y).name() not in ("#ffffff", "#fbbf24"))
    prvi, zadnji = rub(670000, 5045100), rub(671900, 5045100)
    provjeri("rub tog lika jednak je rubu prvog lika, a ne preuzet", prvi == zadnji, (prvi, zadnji))
    natpisi_na_liniji(slika, ms)


def izmjene_u_qgisu(t, n, p):
    naslov("4. Izmjene u QGIS-u")
    prije = (otisak(NAS, "gpkg_ts_stablo"), otisak(NAS, "gpkg_ts_stanje"))

    t.startEditing()
    t.changeAttributeValue(1, t.fields().indexOf("naziv"), "Izmijenjeno u QGIS-u čćž")
    t.deleteFeature(3)
    provjeri("spremanje točaka", t.commitChanges(), t.commitErrors())

    n.startEditing()
    provjeri("pomak vrha", n.moveVertex(671000, 5044850, 1, 1))
    provjeri("spremanje linija", n.commitChanges(), n.commitErrors())

    p.startEditing()
    novi = QgsFeature(p.fields())
    novi.setGeometry(QgsGeometry.fromWkt("Polygon((670000 5046000, 670500 5046000, 670500 5046300, 670000 5046000))"))
    novi["naziv"] = "Dodano u QGIS-u"
    provjeri("dodavanje lika", p.addFeature(novi))
    provjeri("spremanje likova", p.commitChanges(), p.commitErrors())
    provjeri("novi stupac", p.dataProvider().addAttributes([QgsField("napomena", QVariant.String)]))
    p.updateFields()

    sim = QgsLineSymbol.createSimple({"line_color": "#7c3aed", "line_width": "1.2", "line_style": "dot"})
    n.setRenderer(QgsSingleSymbolRenderer(sim))
    n.saveStyleToDatabaseV2("Iz QGIS-a", "", True, "")
    print(f"  uz datoteku dok je QGIS otvoren: {uz_datoteku(NAS)}")
    v = citaj()
    provjeri("program vidi izmjene dok je QGIS još otvoren", v["slojevi"]["tocke"]["objekti"][0]["naziv"] == "Izmijenjeno u QGIS-u čćž")
    kopija = os.path.join(MAPA, "kopija_samo_gpkg.gpkg")
    shutil.copyfile(NAS, kopija)
    c = sqlite3.connect(kopija)
    u_kopiji = c.execute("select count(*) from parcele").fetchone()[0]
    c.close()
    u_izvorniku = len(v["slojevi"]["parcele"]["objekti"])
    nalaz(f"kopija samo .gpkg dok je QGIS otvoren: {u_kopiji} likova, u stvarnosti {u_izvorniku}, -wal {os.path.getsize(NAS + '-wal') if os.path.exists(NAS + '-wal') else 0} bajtova")
    provjeri("naše tablice su netaknute", prije == (otisak(NAS, "gpkg_ts_stablo"), otisak(NAS, "gpkg_ts_stanje")))


def izmjene_kod_nas():
    naslov("5. Što program vidi nakon QGIS-a")
    v = citaj()
    objekti = v["slojevi"]["tocke"]["objekti"]
    provjeri("izmijenjena vrijednost", objekti[0]["naziv"] == "Izmijenjeno u QGIS-u čćž", objekti[0]["naziv"])
    provjeri("obrisani objekt je nestao", [o["fid"] for o in objekti] == [1, 2], [o["fid"] for o in objekti])
    crta = v["slojevi"][NASIPI]["objekti"][0]["_geom"]["wkt"]
    provjeri("pomaknuti vrh", "671000 5044850" in crta, crta)
    likovi = v["slojevi"]["parcele"]["objekti"]
    provjeri("dodani lik s novim fid-om", likovi[-1]["naziv"] == "Dodano u QGIS-u" and likovi[-1]["fid"] == 4, likovi[-1])
    provjeri("novi stupac se vidi", ["napomena", "TEXT(0)"] in v["slojevi"]["parcele"]["stupci"] or any(s[0] == "napomena" for s in v["slojevi"]["parcele"]["stupci"]), v["slojevi"]["parcele"]["stupci"])
    st = v["slojevi"][NASIPI]["stil"]
    provjeri("novi stil iz QGIS-a", st.get("naziv") == "Iz QGIS-a" and st["opcije"]["line_color"].startswith("124,58,237") and st["opcije"]["line_width"] == "1.2" and st["opcije"]["line_style"] == "dot", st)
    nalaz(f"redaka stila za sloj: {st.get('stilova')}, a dnevnik datoteke je {v['dnevnik']}")
    otvoren = sloj(NAS, NASIPI)  # sloj mora živjeti dok se čita njegov simbol
    ponovo = otvoren.renderer().symbol().symbolLayer(0).color().name()
    provjeri("QGIS pri ponovnom otvaranju uzima svoj noviji stil", ponovo == "#7c3aed", ponovo)
    return v


def qgisova_datoteka():
    naslov("6. Datoteka koju je napravio QGIS")
    crs = "crs=EPSG:3765"
    podaci = [
        ("Lik", f"MultiPolygon?{crs}", ["MultiPolygon(((670000 5046000, 670100 5046000, 670100 5046100, 670000 5046000)),((670200 5046000, 670300 5046000, 670300 5046100, 670200 5046000)))"]),
        ("Linija3D", f"LineStringZ?{crs}", ["LineStringZ(670000 5046000 84.5, 670100 5046100 85.5)"]),
        ("Točka", "Point?crs=EPSG:4326", ["Point(18.68 45.54)", None]),
        ("Krivulja", f"CompoundCurve?{crs}", ["CompoundCurve(CircularString(670000 5046000, 670050 5046050, 670100 5046000))"]),
    ]
    prvi = True
    for ime, opis, geometrije in podaci:
        m = QgsVectorLayer(f"{opis}&field=naziv:string&field=broj:integer", ime, "memory")
        for i, w in enumerate(geometrije):
            f = QgsFeature(m.fields())
            if w:
                f.setGeometry(QgsGeometry.fromWkt(w))
            f["naziv"] = f"{ime} {i} čćž"
            m.dataProvider().addFeature(f)
        o = QgsVectorFileWriter.SaveVectorOptions()
        o.driverName = "GPKG"
        o.layerName = ime
        o.actionOnExistingFile = QgsVectorFileWriter.CreateOrOverwriteFile if prvi else QgsVectorFileWriter.CreateOrOverwriteLayer
        r = QgsVectorFileWriter.writeAsVectorFormatV3(m, TUDA, QgsCoordinateTransformContext(), o)
        provjeri(f"QGIS piše sloj {ime}", r[0] == 0, r)
        prvi = False
    c = sqlite3.connect(TUDA)
    c.executescript("CREATE TABLE tuda_tablica (id INTEGER PRIMARY KEY, tekst TEXT); INSERT INTO tuda_tablica VALUES (1, 'ne diraj');")
    c.commit()
    c.close()

    v = citaj(TUDA)
    sl = v["slojevi"]
    lik = sl["Lik"]["objekti"][0]["_geom"]
    provjeri("program čita višedijelni lik", lik["wkt"] and "MULTIPOLYGON" in lik["wkt"].upper(), lik)
    d3 = sl["Linija3D"]["objekti"][0]["_geom"]
    provjeri("visina se može pročitati", "84.5" in (d3["wkt_3d"] or ""), d3)
    nalaz(f"ista linija bez posebnog traženja visine: wkt={d3['wkt']}; geo_types={d3['geo']}")
    provjeri("omotnica 3D linije", d3["omotnica"] == [670000.0, 5046000.0, 670100.0, 5046100.0], d3["omotnica"])
    tocka = sl["Točka"]["objekti"]
    provjeri("točka bez zapisane omotnice", tocka[0]["_geom"]["omotnica"] == [18.68, 45.54, 18.68, 45.54], tocka[0]["_geom"])
    nalaz(f"QGIS objekt bez geometrije piše kao: {tocka[1]['_geom']}")
    provjeri("objekt bez geometrije ne ruši čitanje", tocka[1]["_geom"]["prazna"] and tocka[1]["_geom"]["omotnica"] is None, tocka[1])
    uredi = sloj(TUDA, "Točka")
    uredi.startEditing()
    uredi.changeGeometry(2, QgsGeometry())
    provjeri("QGIS briše geometriju objekta", uredi.commitChanges(), uredi.commitErrors())
    del uredi
    bez = citaj(TUDA)["slojevi"]["Točka"]["objekti"][1]["_geom"]
    nalaz(f"nakon brisanja geometrije u QGIS-u zapis je: {bez}")
    provjeri("ni objekt s obrisanom geometrijom ne ruši čitanje", bez is None or bez["omotnica"] is None, bez)
    k = sl["Krivulja"]["objekti"][0]["_geom"]
    nalaz(f"krivulja: wkt={k['wkt']} greška_wkt={k['greska_wkt']} greška_geo={k['greska_geo']}")

    naslov("7. Program dopisuje u QGIS-ovu datoteku")
    prije_tuda = otisak(TUDA, "tuda_tablica")
    fid = program("dodaj", TUDA, "Lik", "670500", "5046500", "Dodao program čćž").stdout.strip()
    fid2 = program("dodaj", TUDA, "Točka", "18.71", "45.55", "Dodao program").stdout.strip()
    program("izmijeni", TUDA, "Linija3D", "1", "naziv", "Promijenio program")
    nalaz(f"dodani fid: {fid}, {fid2}")
    l = sloj(TUDA, "Lik")
    tt = sloj(TUDA, "Točka")
    provjeri("QGIS vidi novi objekt", l.featureCount() == 2 and tt.featureCount() == 3, (l.featureCount(), tt.featureCount()))
    provjeri("brojač objekata je točan", sqlite3.connect(TUDA).execute("select feature_count from gpkg_ogr_contents where table_name='Lik'").fetchone()[0] == 2)
    pogodak = [f["naziv"] for f in l.getFeatures(QgsFeatureRequest().setFilterRect(QgsRectangle(670490, 5046490, 670600, 5046600)))]
    provjeri("prostorni indeks nalazi novi objekt", pogodak == ["Dodao program čćž"], pogodak)
    provjeri("obuhvat sloja se proširio", l.extent().xMaximum() >= 670600, l.extent().toString())
    prom = next(sloj(TUDA, "Linija3D").getFeatures())
    provjeri("izmjena vrijednosti ne dira visinu", prom["naziv"] == "Promijenio program" and prom.geometry().constGet().zAt(0) == 84.5, prom.geometry().asWkt())
    provjeri("tuđa tablica je netaknuta", prije_tuda == otisak(TUDA, "tuda_tablica"))
    kod, tekst = validator(TUDA)
    provjeri("validator i nakon našeg pisanja", kod == 0 and not tekst, tekst[:600])


def istodobno():
    import time
    naslov("8. Program i QGIS nad istom datotekom")
    put = os.path.join(MAPA, "istodobno.gpkg")
    program("pisi", put)
    q = sloj(put, "parcele")
    naziv = q.fields().indexOf("naziv")
    q.startEditing()
    q.changeAttributeValue(1, naziv, "QGIS spremio prvi")
    provjeri("QGIS sprema", q.commitChanges(), q.commitErrors())
    nalaz(f"uz datoteku nakon prvog spremanja u QGIS-u: {uz_datoteku(put)}")
    v = citaj(put)
    nalaz(f"dnevnik kako ga vidi program: {v['dnevnik']}")
    provjeri("program čita QGIS-ovu izmjenu dok je QGIS otvoren", v["slojevi"]["parcele"]["objekti"][0]["naziv"] == "QGIS spremio prvi")
    program("dodaj", put, "parcele", "670000", "5046500", "Dodao program dok je QGIS otvoren")
    q.reload()
    pregled = [f["naziv"] for f in q.getFeatures()]
    provjeri("QGIS vidi naš novi objekt nakon osvježavanja", "Dodao program dok je QGIS otvoren" in pregled, pregled)

    for sekundi in (0.2, 1, 4):
        q.startEditing()
        q.changeAttributeValue(1, naziv, f"QGIS uz zaključavanje {sekundi}")
        drzac = subprocess.Popen([PROBA, "drzi", put, str(sekundi)], stdout=subprocess.PIPE, text=True)
        drzac.stdout.readline()
        a = time.time()
        uspjeh = q.commitChanges()
        trajalo = time.time() - a
        nalaz(f"program drži pisanje {sekundi} s, QGIS sprema: uspjeh={uspjeh}, trajalo {trajalo:.2f} s, {q.commitErrors() if not uspjeh else ''}")
        drzac.wait()
        if not uspjeh:
            q.rollBack()

    # Sloj ostaje u uređivanju nakon spremanja, kao gumb „Spremi izmjene” u QGIS-u.
    q.startEditing()
    q.changeAttributeValue(1, naziv, "QGIS spremio, još uređuje")
    nov = QgsFeature(q.fields())
    nov.setGeometry(QgsGeometry.fromWkt("Polygon((670000 5046600, 670100 5046600, 670100 5046700, 670000 5046600))"))
    q.addFeature(nov)
    provjeri("QGIS sprema i ostaje u uređivanju", q.commitChanges(False), q.commitErrors())
    nalaz(f"uz datoteku dok je sloj u uređivanju: {uz_datoteku(put)}, -wal {os.path.getsize(put + '-wal') if os.path.exists(put + '-wal') else 0} bajtova")
    kopija = os.path.join(MAPA, "kopija_u_uredjivanju.gpkg")
    shutil.copyfile(put, kopija)
    c = sqlite3.connect(kopija)
    u_kopiji = c.execute("select count(*) from parcele").fetchone()[0]
    c.close()
    stvarno = len(citaj(put)["slojevi"]["parcele"]["objekti"])
    nalaz(f"kopija samo .gpkg u uređivanju: {u_kopiji} likova, stvarno {stvarno}")
    q.rollBack()

    # Otvoreni program mora sam primijetiti izmjenu iz QGIS-a.
    pazi = subprocess.Popen([PROBA, "pazi", put, "6"], stdout=subprocess.PIPE, text=True)
    pazi.stdout.readline()
    q.startEditing()
    q.changeAttributeValue(1, naziv, "QGIS, program gleda")
    a = time.time()
    q.commitChanges()
    izlaz = pazi.stdout.readline().strip()
    provjeri("otvoreni program primijeti spremanje u QGIS-u", izlaz == "promjena", izlaz)
    nalaz(f"primijetio za {time.time() - a:.2f} s")
    pazi.wait()

    # Drugi drži pisanje: program ne smije odmah odustati.
    drugi = sqlite3.connect(put, isolation_level=None)
    drugi.execute("BEGIN IMMEDIATE")
    drzac = subprocess.Popen([PROBA, "izmijeni", put, "parcele", "3", "naziv", "Program je čekao"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    a = time.time()
    time.sleep(1)
    drugi.execute("COMMIT")
    drugi.close()
    izlaz, greska = drzac.communicate()
    provjeri("program pričeka tuđe pisanje i uspije", drzac.returncode == 0, greska)
    nalaz(f"čekao oko {time.time() - a:.2f} s")

    # Isti objekt s obje strane: tko će pobijediti, i saznaje li gubitnik.
    q.startEditing()
    q.changeAttributeValue(2, naziv, "QGIS, nije još spremio")
    a = time.time()
    r = program("izmijeni", put, "parcele", "2", "naziv", "Program", provjeri_kod=False)
    nalaz(f"program piše dok QGIS ima nespremljeno: kod {r.returncode}, trajalo {time.time() - a:.2f} s, {r.stderr.strip()}")
    provjeri("QGIS sprema nakon toga", q.commitChanges(), q.commitErrors())
    konacno = citaj(put)["slojevi"]["parcele"]["objekti"][1]["naziv"]
    nalaz(f"isti objekt mijenjali oba, ostalo je: {konacno!r}, a nitko nije dobio upozorenje")

    del q
    QgsApplication.processEvents()
    nalaz(f"uz datoteku nakon što je QGIS zatvorio sloj: {uz_datoteku(put)}")
    c = sqlite3.connect(put)
    nalaz(f"dnevnik nakon zatvaranja: {c.execute('pragma journal_mode').fetchone()[0]}")
    c.close()


def main():
    if os.path.exists(MAPA):
        shutil.rmtree(MAPA)
    os.makedirs(MAPA)
    app = QgsApplication([], False)
    app.initQgis()
    print(f"QGIS {Qgis.QGIS_VERSION}")
    program("pisi", NAS)
    t, n, p = nas_u_qgisu()
    slika_u_qgisu(t, n, p)
    izmjene_u_qgisu(t, n, p)
    del t, n, p
    izmjene_kod_nas()
    kod, tekst = validator(NAS)
    provjeri("validator nakon izmjena u QGIS-u", kod == 0 and not tekst, tekst[:600])
    podslojevi = [d.name() for d in QgsProviderRegistry.instance().querySublayers(NAS)]
    nalaz(f"nakon izmjena QGIS nudi: {podslojevi}")
    qgisova_datoteka()
    istodobno()
    print(f"\nProšlo {prolaza}, palo {len(palo)}")
    for x in palo:
        print(f"  - {x}")
    sys.stdout.flush()
    os._exit(1 if palo else 0)


main()
