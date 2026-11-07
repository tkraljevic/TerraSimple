"""Mjeri oba PDF-a preko slike iz Popplera, koji nema veze ni s krillom ni s QGIS-om.

Upotreba: analiza.py <radna mapa> [podloga.jpg]
Radna mapa drži nas.pdf, qgis.pdf, objekti.csv i list.csv.
"""

import csv
import re
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image

Image.MAX_IMAGE_PIXELS = None
DPI = 600
MM_PO_PX = 25.4 / DPI
GRANICA_MM = 0.1
# QGIS-ov PDF i sam je do oko 0,05 % drukčijeg mjerila, što je 0,15 mm na 280 mm. Zato je
# prema njemu granica šira, a stroga granica vrijedi prema računu.
GRANICA_QGIS_MM = 0.2
TEKSTOVI = [
    "Šećerana Čađavica Žažina Đakovo Vođinci Ćićarija",
    "ČĆŽŠĐ čćžšđ",
    "Ispisati u stvarnoj veličini (100 %)",
]
palo = []
LIST = {}


def provjera(uvjet, tekst):
    print(("  u redu  " if uvjet else "  PALO    ") + tekst)
    if not uvjet:
        palo.append(tekst)


def naredba(*a):
    return subprocess.run(a, capture_output=True, text=True, check=True).stdout


def boja(tekst):
    return np.array([int(tekst[i : i + 2], 16) for i in (1, 3, 5)], dtype=np.int16)


def u_mm(e, n):
    return LIST["x"] + (e - LIST["zapad"]) / 5, LIST["y"] + (LIST["sjever"] - n) / 5


def od_kuta(desno, dolje):
    """Položaj na papiru za točku zadanu od gornjeg lijevog kuta karte, u metrima."""
    return LIST["x"] + desno / 5, LIST["y"] + dolje / 5


def tocke_wkt(wkt):
    brojevi = re.findall(r"(-?\d+(?:\.\d+)?) (-?\d+(?:\.\d+)?)", wkt)
    return [u_mm(float(a), float(b)) for a, b in brojevi]


def ocekivano(r):
    """Očekivani okvir u mm iz same geometrije. Kosa crta nema jednostavan okvir."""
    t = tocke_wkt(r["wkt"])
    x0, x1 = min(p[0] for p in t), max(p[0] for p in t)
    y0, y1 = min(p[1] for p in t), max(p[1] for p in t)
    d = float(r["debljina"]) / 2
    if r["vrsta"] in ("lik", "tocka") or t[0] == t[-1]:
        return x0 - d, y0 - d, x1 + d, y1 + d
    if y0 == y1:
        return x0, y0 - d, x1, y1 + d
    if x0 == x1:
        return x0 - d, y0, x1 + d, y1
    return None


def okvir(slika, b, snap=14):
    """Okvir svih piksela te boje, u mm."""
    maska = np.abs(slika.astype(np.int16) - b).max(axis=2) <= snap
    if not maska.any():
        return None, maska
    ys, xs = np.where(maska)
    k = MM_PO_PX
    return (xs.min() * k, ys.min() * k, (xs.max() + 1) * k, (ys.max() + 1) * k), maska


def slika_stranice(pdf, stem, mapa, stranica=1):
    izlaz = str(Path(mapa) / stem)
    naredba("pdftoppm", "-r", str(DPI), "-f", str(stranica), "-l", str(stranica), "-singlefile",
            "-aaVector", "no", "-png", pdf, izlaz)
    return np.asarray(Image.open(izlaz + ".png").convert("RGB"))


def papir(pdf):
    info = naredba("pdfinfo", pdf)
    m = re.search(r"Page size:\s+([\d.]+) x ([\d.]+) pts", info)
    return float(m.group(1)) * 25.4 / 72, float(m.group(2)) * 25.4 / 72


def nas_pdf(pdf):
    print("naš PDF, sadržaj:")
    tekst = naredba("pdftotext", "-layout", pdf, "-")
    for t in TEKSTOVI:
        provjera(t in tekst, f"tekst se može kopirati: {t}")
    fontovi = naredba("pdffonts", pdf).splitlines()[2:]
    provjera(len(fontovi) > 0, f"fontova: {len(fontovi)}")
    for f in fontovi:
        polja = f.split()
        provjera(polja[-5] == "yes" and polja[-4] == "yes" and polja[-3] == "yes",
                 f"font ugrađen, podskup, s Unicodeom: {polja[0]}")
    slike = naredba("pdfimages", "-f", "1", "-l", "1", "-list", pdf).splitlines()[2:]
    provjera(len(slike) == 0, f"prva stranica bez ijedne rasterske slike ({len(slike)})")


def crtice(maska, y_mm):
    red = maska[int(y_mm / MM_PO_PX)]
    promjene = np.flatnonzero(np.diff(red.astype(np.int8)))
    if red[0]:
        promjene = np.insert(promjene, 0, -1)
    pocetci = promjene[0::2] + 1
    krajevi = promjene[1::2] + 1
    return [(k - p) * MM_PO_PX for p, k in zip(pocetci, krajevi)]


def provjeri_podlogu(pdf, podloga, mapa):
    """Podloga mora ležati točno u okviru: pomak za piksel mora dati goru sliku."""
    print("druga stranica, podloga:")
    izvor = np.asarray(Image.open(podloga).convert("L")).astype(np.int16)
    izlaz = str(Path(mapa) / "str2_200")
    naredba("pdftoppm", "-r", "200", "-f", "2", "-l", "2", "-singlefile", "-png", pdf, izlaz)
    stranica = np.asarray(Image.open(izlaz + ".png").convert("L")).astype(np.int16)
    x0, y0 = round(LIST["x"] * 200 / 25.4), round(LIST["y"] * 200 / 25.4)
    # Uzorak je pojas između žutog i plavog lika, gdje prolaze samo tanke crte mreže. Zato
    # se gleda medijan razlike, na koji te crte ne utječu.
    (v0, v1), (s0, s1) = (157, 630), (819, 1008)
    rezultati = {}
    for dy in range(-3, 4):
        for dx in range(-3, 4):
            isjecak = stranica[y0 + v0 + dy : y0 + v1 + dy, x0 + s0 + dx : x0 + s1 + dx]
            rezultati[(dx, dy)] = float(np.median(np.abs(isjecak - izvor[v0:v1, s0:s1])))
    najbolji = min(rezultati, key=rezultati.get)
    provjera(max(abs(najbolji[0]), abs(najbolji[1])) <= 1,
             f"podloga leži u okviru: najbolje uz pomak {najbolji} piksela od 0,127 mm, razlika {rezultati[najbolji]:.1f}, a uz (3, 3) {rezultati[(3, 3)]:.1f}")
    naopako = float(np.median(np.abs(stranica[y0 + v0 : y0 + v1, x0 + s0 : x0 + s1] - izvor[v0:v1, s0:s1][::-1])))
    provjera(rezultati[najbolji] < naopako / 2, f"podloga nije okrenuta naopako: {rezultati[najbolji]:.1f} prema {naopako:.1f}")


def main():
    mapa = sys.argv[1]
    nas, qgis = f"{mapa}/nas.pdf", f"{mapa}/qgis.pdf"
    with open(f"{mapa}/list.csv", encoding="utf-8") as f:
        red = next(csv.DictReader(f, delimiter=";"))
    LIST.update({k: (v if k == "ime" else float(v)) for k, v in red.items()})
    print(f"=== list {LIST['ime']}")
    with open(f"{mapa}/objekti.csv", encoding="utf-8") as f:
        redci = list(csv.DictReader(f, delimiter=";"))
    s, v = papir(nas)
    provjera(abs(s - LIST["sirina"]) < 0.01 and abs(v - LIST["visina"]) < 0.01, f"naš list {s:.3f} x {v:.3f} mm")
    s, v = papir(qgis)
    print(f"  QGIS-ov list je {s:.3f} x {v:.3f} mm (samo za znanje)")
    nas_pdf(nas)
    a = slika_stranice(nas, "nas", mapa)
    b = slika_stranice(qgis, "qgis", mapa)
    print(f"  slike na {DPI} dpi: naša {a.shape[:2]}, QGIS-ova {b.shape[:2]}")
    print(f"{'objekt':<18}{'naš, mm':<36}{'QGIS prema našem':>18}{'naš prema računu':>16}{'QGIS prema računu':>16}")
    najvise_q = najvise_r = najvise_qr = 0.0
    maske = {}
    rub_karte = (LIST["x"], LIST["y"], LIST["x"] + LIST["okvir_s"], LIST["y"] + LIST["okvir_v"])
    for r in redci:
        if not r["mjeri"]:
            continue
        o_a, maska = okvir(a, boja(r["boja"]))
        o_b, maska_q = okvir(b, boja(r["boja"]))
        maske[r["ime"]] = (maska, maska_q)
        if o_a is None or o_b is None:
            provjera(False, f"{r['ime']}: boje nema na slici")
            continue
        d_q = max(abs(x - y) for x, y in zip(o_a, o_b))
        cilj = ocekivano(r)
        if r["ime"] == "Okvir":
            # Crta leži točno na rubu karte, pa QGIS njezinu vanjsku polovicu odreže. Zato
            # se uspoređuje s rubom karte, a ne s našim okvirom koji crta cijelu debljinu.
            d_rub = max(abs(x - y) for x, y in zip(o_b, rub_karte))
            provjera(d_rub <= GRANICA_QGIS_MM, f"QGIS reže okvir na rubu karte, odstupanje {d_rub:.3f} mm")
            d_q = 0.0
            d_qr = float("nan")
        else:
            d_qr = max(abs(x - y) for x, y in zip(o_b, cilj)) if cilj else float("nan")
        d_r = max(abs(x - y) for x, y in zip(o_a, cilj)) if cilj else float("nan")
        if cilj:
            najvise_r = max(najvise_r, d_r)
        if d_qr == d_qr:
            najvise_qr = max(najvise_qr, d_qr)
        najvise_q = max(najvise_q, d_q)
        tekst = " ".join(f"{v:7.2f}" for v in o_a)
        print(f"{r['ime']:<18}{tekst:<36}{d_q:>18.3f}{d_r:>16.3f}{d_qr:>16.3f}")
    provjera(najvise_r <= GRANICA_MM, f"naš list prema računu: najviše {najvise_r:.3f} mm (granica {GRANICA_MM})")
    provjera(najvise_q <= GRANICA_QGIS_MM, f"naš list prema QGIS-u: najviše {najvise_q:.3f} mm (granica {GRANICA_QGIS_MM})")
    print(f"  QGIS-ov list prema računu: najviše {najvise_qr:.3f} mm (samo za znanje)")

    # Mreža 100 m mora biti svakih 20 mm, cijelom širinom lista.
    mreza = next(r for r in redci if r["ime"] == "Mreža")
    broj_crta = round(LIST["okvir_s"] / 20) - 1
    for ime, slika in (("naš", a), ("QGIS", b)):
        m = np.abs(slika.astype(np.int16) - boja(mreza["boja"])).max(axis=2) <= 8
        # Likovi pokrivaju dio crte, zato prag nije pola visine. Vodoravne crte mreže
        # daju oko 12 %, pa 25 % razlikuje okomite crte od ostalog.
        stupci = m.mean(axis=0) > 0.25
        rub = np.flatnonzero(np.diff(stupci.astype(np.int8)) == 1) * MM_PO_PX
        razmaci = np.diff(rub)
        provjera(len(rub) == broj_crta and abs(razmaci.mean() - 20) < 0.02 and np.abs(razmaci - 20).max() < 0.1,
                 f"mreža {ime}: {len(rub)} crta od {broj_crta}, razmak prosječno {razmaci.mean():.3f} mm, najgori {np.abs(razmaci - 20).max():.3f} mm od 20")

    # Crtice: 4 mm crta, 2 mm razmak.
    r_crt = next(r for r in redci if r["crtice"])
    y = tocke_wkt(r_crt["wkt"])[0][1]
    for ime, indeks in (("naš", 0), ("QGIS", 1)):
        d = crtice(maske[r_crt["ime"]][indeks], y)
        unutarnje = d[:-1]
        provjera(abs(np.mean(unutarnje) - 4) < 0.1, f"crtice {ime}: {len(d)} komada, prosječno {np.mean(unutarnje):.2f} mm")

    # Prozirnost: pola plave i pola ljubičaste, između crta mreže.
    xm, ym = od_kuta(890, 290)
    x, y = int(xm / MM_PO_PX), int(ym / MM_PO_PX)
    p_a, p_b = tuple(int(v) for v in a[y, x]), tuple(int(v) for v in b[y, x])
    provjera(max(abs(i - j) for i, j in zip(p_a, p_b)) <= 6, f"prozirni lik na plavom: naš {p_a}, QGIS {p_b}")

    # Rupa u liku mora ostati prazna.
    xm, ym = od_kuta(290, 290)
    x, y = int(xm / MM_PO_PX), int(ym / MM_PO_PX)
    provjera(a[y, x].min() >= 250, f"rupa u liku je prazna: {tuple(int(v) for v in a[y, x])}")

    # Ništa ne smije izaći iz okvira po širini ni prijeći najmanji rub pisača od 5 mm.
    stupci_t = np.flatnonzero((a.min(axis=2) < 200).any(axis=0))
    redovi_t = np.flatnonzero((a.min(axis=2) < 200).any(axis=1))
    lijevo, desno = stupci_t.min() * MM_PO_PX, (stupci_t.max() + 1) * MM_PO_PX
    dolje = (redovi_t.max() + 1) * MM_PO_PX
    provjera(lijevo >= LIST["x"] - 0.5 and desno <= LIST["x"] + LIST["okvir_s"] + 0.5,
             f"sve je unutar širine okvira: od {lijevo:.1f} do {desno:.1f} mm")
    provjera(dolje <= LIST["visina"] - 5, f"donji rub teksta {dolje:.1f} mm, list {LIST['visina']:.0f} mm")

    if len(sys.argv) > 2:
        provjeri_podlogu(nas, sys.argv[2], mapa)

    Image.fromarray(a).resize((a.shape[1] // 5, a.shape[0] // 5), Image.LANCZOS).save(Path(mapa) / "nas_umanjeno.png")
    if palo:
        print(f"PALO: {len(palo)}")
        for p in palo:
            print(" -", p)
        raise SystemExit(1)
    print("PROŠLO")


main()
