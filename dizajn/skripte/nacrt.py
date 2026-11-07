#!/usr/bin/env python3
"""Dijelovi prozora TerraSimplea: karta, trake, ploča i stablo slojeva.

Karta na nacrtima je ilustracija: mjesta i tokovi su izmišljeni. Mjere na
nacrtu računaju se iz crteža (M_PO_PX), pa se slažu s mjerilom u statusnoj traci.
"""
import random

from temelj import *  # noqa: F401,F403
from temelj import P, SJENA, add

W, H = 1440, 900
MENI = 40
STATUS = 28
PLOCA = 360
RUB = 12
MJERILO = 4500  # zaslon 96 dpi
M_PO_PX = 0.2646e-3 * MJERILO  # oko 1,19 m po pikselu


def broj_hr(v, dec=0):
    t = f"{v:,.{dec}f}"
    return t.replace(",", " ").replace(".", ",")


def opseg_m(t, zatvori=True):
    n = len(t)
    koraci = n if zatvori else n - 1
    return M_PO_PX * sum(((t[i][0] - t[(i + 1) % n][0]) ** 2 + (t[i][1] - t[(i + 1) % n][1]) ** 2) ** 0.5 for i in range(koraci))


def povrsina_ha(t):
    n = len(t)
    a = abs(sum(t[i][0] * t[(i + 1) % n][1] - t[(i + 1) % n][0] * t[i][1] for i in range(n))) / 2
    return a * M_PO_PX ** 2 / 10000


# ---------- karta ----------

def karta_vektori(m):
    """Obična podloga. Izmišljena mjesta, iste koordinate na svim zaslonima."""
    for put in [
        "M60 420 C140 380 240 400 300 470 C340 530 280 620 190 630 C100 640 20 540 60 420 Z",
        "M660 40 C760 10 900 30 960 90 C1000 150 900 210 800 190 C710 175 610 110 660 40 Z",
        "M720 600 C800 560 920 590 960 660 C990 730 880 800 780 780 C700 760 660 650 720 600 Z",
    ]:
        vektor(m, "šuma", put, fill=P["k_suma"])
    for put in [
        "M130 90 L250 70 L300 130 L270 210 L160 220 L110 160 Z",
        "M760 300 L870 280 L930 340 L900 420 L790 430 L740 370 Z",
        "M420 620 L510 600 L560 660 L520 730 L430 720 Z",
    ]:
        vektor(m, "naselje", put, fill=P["k_naselje"])
    for put, sw, boja, rub in [
        ("M-20 120 C200 170 480 80 700 250 C860 370 1100 330 1460 390", 5, P["k_glavna"], P["k_cesta_rub"]),
        ("M210 -20 C230 200 200 400 480 660 C560 730 600 800 620 860", 3.5, P["k_cesta"], P["k_cesta_rub"]),
        ("M830 -20 C840 120 830 260 830 360 C830 500 940 700 960 860", 3.5, P["k_cesta"], P["k_cesta_rub"]),
        ("M-20 700 C200 690 340 660 480 660", 3, P["k_cesta"], P["k_cesta_rub"]),
    ]:
        vektor(m, "cesta rub", put, stroke=rub, sw=sw + 2)
        vektor(m, "cesta", put, stroke=boja, sw=sw)
    vektor(m, "potok", "M300 -20 C340 90 420 160 470 240 C520 320 590 370 640 400", stroke=P["k_voda"], sw=3)
    vektor(m, "rijeka", "M-20 250 C120 300 260 330 400 420 C520 500 660 420 800 500 C940 580 1100 540 1460 620", stroke=P["k_voda"], sw=9)
    for t, x, y, s, st in [
        ("Novo Selo", 165, 132, 13, "Medium"), ("Brestovac", 790, 342, 13, "Medium"), ("Dolina", 450, 652, 13, "Medium"),
        ("Hrastova šuma", 730, 92, 12, "Regular"), ("Lug", 165, 500, 12, "Regular"),
    ]:
        tekst(m, t, x=x, y=y, vel=s, boja=P["k_natpis"], stil=st)
    tekst(m, "Rijeka", x=600, y=424, vel=12, boja=P["k_voda_natpis"], stil="Medium Italic")


def karta_orto(m):
    """Podloga nalik ortofotu: polja u trakama, nacrtana skriptom."""
    rnd = random.Random(7)
    boje = ["#8A9460", "#A0A36B", "#7E8F55", "#B5AE7C", "#94A06A", "#6F8350", "#C2B98A", "#88935E"]
    pravokutnik(m, "tlo", 0, 0, 1440, 832, "#7C8A58")
    kx, ky = 0.97, 0.24  # smjer parcela
    for i in range(-3, 11):
        for j in range(-3, 10):
            ox = i * 190 + j * -46
            oy = i * 46 + j * 130
            n = rnd.randint(3, 6)
            for t in range(n):
                a, b = t / n, (t + 1) / n
                d = 186
                x1, y1 = ox, oy + a * 126
                x2, y2 = ox, oy + b * 126 - 1.5
                vektor(m, "polje", f"M{x1:.0f} {y1:.0f} L{x1 + d * kx:.0f} {y1 + d * ky:.0f} L{x2 + d * kx:.0f} {y2 + d * ky:.0f} L{x2:.0f} {y2:.0f} Z", fill=rnd.choice(boje))
    vektor(m, "šuma", "M660 20 C780 -10 930 20 980 90 C1000 170 880 230 780 200 C690 180 600 90 660 20 Z", fill="#47603A")
    vektor(m, "šuma", "M60 420 C140 380 240 400 300 470 C340 530 280 620 190 630 C100 640 20 540 60 420 Z", fill="#4C663E")
    vektor(m, "cesta", "M-20 120 C200 170 480 80 700 250 C860 370 1100 330 1460 390", stroke="#CFC8B0", sw=4)
    vektor(m, "cesta", "M210 -20 C230 200 200 400 480 660 C560 730 600 800 620 860", stroke="#BDB69E", sw=3)
    vektor(m, "obala", "M-20 250 C120 300 260 330 400 420 C520 500 660 420 800 500 C940 580 1100 540 1460 620", stroke="#3D5A3C", sw=20)
    vektor(m, "rijeka", "M-20 250 C120 300 260 330 400 420 C520 500 660 420 800 500 C940 580 1100 540 1460 620", stroke="#3F6B7A", sw=11)


def natpis_na_karti(m, t, x, y):
    f = red(m, "natpis · " + t, x=x, y=y, padding=[3, 8, 3, 8], cornerRadius=999, fill=P["ploha"], stroke=P["rub"], strokeWeight=1)
    tekst(f, t, vel=12, stil="Medium")


def tamni_natpis(m, t, x, y):
    f = red(m, "mjera · " + t, x=x, y=y, padding=[4, 8, 4, 8], cornerRadius=6, fill=P["tamno_ploha"])
    tekst(f, t, vel=12, boja=P["na_tamnom"], stil="Medium")


PARCELA = [(420, 250), (560, 215), (640, 290), (610, 390), (470, 410), (395, 335)]
TRASA = "M120 300 C180 280 220 340 270 330 C320 320 340 290 395 335"
SUSJED1 = [(650, 220), (780, 250), (770, 350), (640, 380)]
SUSJED2 = [(285, 150), (400, 130), (440, 225), (315, 250)]
TOCKE = [(170, 240, "Zdenac"), (720, 470, "Ulaz na parcelu")]


def put_lika(tocke, zatvori=True):
    s = "M" + " L".join(f"{x} {y}" for x, y in tocke)
    return s + (" Z" if zatvori else "")


def moji_objekti(m, odabran=False, s_vrhovima=False):
    """Korisnikovi crteži: poligon, linija i točke."""
    vektor(m, "Poligon · Parcela 1126", put_lika(SUSJED1), fill=P["crtez_ploha"], stroke=P["crtez"], sw=1.75)
    vektor(m, "Poligon · Parcela 1118/2", put_lika(SUSJED2), fill=P["crtez_ploha"], stroke=P["crtez"], sw=1.75)
    vektor(m, "Poligon · Parcela 1124/3", put_lika(PARCELA), fill=P["crtez_ploha"], stroke=P["crtez"], sw=3 if odabran else 1.75)
    vektor(m, "Linija · Trasa odvodnje", TRASA, stroke="#1565C0", sw=3)
    for x, y, t in TOCKE:
        pribadaca(m, x, y)
        natpis_na_karti(m, t, x + 12, y - 30)
    if odabran:
        vektor(m, "isticanje", put_lika(PARCELA), stroke=P["ploha"], sw=1, dashes=[5, 4])
    if s_vrhovima:
        vrhovi(m, PARCELA)


def pribadaca(m, x, y):
    vektor(m, "točka", f"M{x} {y} C{x - 3} {y - 8} {x - 9} {y - 11} {x - 9} {y - 18} C{x - 9} {y - 24} {x - 5} {y - 28} {x} {y - 28} C{x + 5} {y - 28} {x + 9} {y - 24} {x + 9} {y - 18} C{x + 9} {y - 11} {x + 3} {y - 8} {x} {y} Z", fill="#C62828", stroke="#FFFFFF", sw=1.5)
    krug(m, "sredina", x, y - 18, 3, "#FFFFFF")


def vrhovi(m, tocke, srednji=True):
    n = len(tocke)
    if srednji:
        for i in range(n):
            a, b = tocke[i], tocke[(i + 1) % n]
            krug(m, "dodaj vrh", (a[0] + b[0]) / 2, (a[1] + b[1]) / 2, 4, "#FFFFFFB3", stroke=P["crtez"], strokeWeight=1)
    for x, y in tocke:
        krug(m, "vrh", x, y, 5.5, "#FFFFFF", stroke=P["crtez"], strokeWeight=2)


def podloga_mjerilo(m, h, duljina="100 m", zasluge="© OpenFreeMap · OpenStreetMap suradnici", desno=None, svijetlo=False, lijevo=RUB):
    """Mjerilo dolje lijevo i zasluge dolje uz ploču."""
    b = "#FFFFFF" if svijetlo else P["tekst"]
    d = 100 / M_PO_PX
    pravokutnik(m, "mjerilo crta", lijevo + 4, h - 26, d, 2, b)
    pravokutnik(m, "mjerilo kraj", lijevo + 4, h - 32, 2, 8, b)
    pravokutnik(m, "mjerilo kraj", lijevo + 2 + d, h - 32, 2, 8, b)
    tekst(m, duljina, x=lijevo + 8, y=h - 46, vel=12, boja=b, stil="Medium")
    tekst(m, zasluge, x=(desno or 780), y=h - 22, vel=11, boja=b)


def kontrole(m, x, h, s_ispisom=False, slojevi=False):
    """Lebdeće kontrole: gore cijeli zaslon i ispis, dolje sjever, povećanje i slojevi."""
    g = stup(m, "kontrole gore", x=x, y=RUB, gap=0, cornerRadius=8, fill=P["ploha"], effects=SJENA(), clip=True)
    gumb_ikona(g, "zaslon", vel=40, naziv="Cijeli zaslon")
    razdjelnik(g, w=40)
    gumb_ikona(g, "ispis", vel=40, naziv="Ispis", ploha=P["tint"] if s_ispisom else None, boja=P["na_tintu"] if s_ispisom else None)
    v = 40 + 8 + 81 + (48 if slojevi else 0)
    d = stup(m, "kontrole dolje", x=x, y=h - RUB - v, gap=8)
    gumb_ikona(d, "sjever", vel=40, naziv="Sjever gore", fill=P["ploha"], effects=SJENA())
    z = stup(d, "povećanje", gap=0, cornerRadius=8, fill=P["ploha"], effects=SJENA(), clip=True)
    gumb_ikona(z, "plus", vel=40, naziv="Povećaj")
    razdjelnik(z, w=40)
    gumb_ikona(z, "minus", vel=40, naziv="Smanji")
    if slojevi:
        gumb_ikona(d, "slojevi", vel=40, naziv="Slojevi", fill=P["glavna"], boja=P["na_glavnoj"])


def alati(m, aktivan="odaberi", x=RUB):
    """Traka alata: uvijek na istom mjestu i uvijek s natpisom."""
    f = stup(m, "alati", x=x, y=RUB + 56, gap=2, padding=4, cornerRadius=12, fill=P["ploha"], effects=SJENA())
    for ime, t in [("odaberi", "Odaberi"), ("tocka", "Točka"), ("linija", "Linija"), ("poligon", "Poligon"), ("ravnalo", "Mjeri")]:
        a = ime == aktivan
        if ime == "ravnalo":
            razdjelnik(f)
        st = stup(f, "alat · " + t, gap=2, padding=[7, 4, 6, 4], cornerRadius=8, w=60, sizingH="fixed", crossAlign="center", fill=P["tint"] if a else "none")
        b = P["na_tintu"] if a else P["sivo"]
        ikona(st, ime, vel=20, boja=b)
        tekst(st, t, vel=11, boja=b, stil="Medium" if a else "Regular")
    return f


def trazilica(m, x=RUB):
    f = red(m, "tražilica", x=x, y=RUB, w=300, h=44, gap=10, padding=[0, 16, 0, 14], cornerRadius=999, fill=P["ploha"], effects=SJENA(), sizingH="fixed", sizingV="fixed")
    ikona(f, "trazi", vel=20)
    tekst(f, "Traži mjesto ili koordinate", vel=14, boja=P["blijedo"])


# ---------- prozor ----------

def traka_izbornika(r, naziv="Teren 2026", aktivan=None):
    f = red(r, "izbornik", gap=0, w=W, h=MENI, padding=[0, 8, 0, 12], fill=P["vrh"], sizingH="fixed", sizingV="fixed", strokeBottomWeight=1, stroke=P["rub"])
    z = red(f, "znak", gap=8, paddingRight=12)
    o = okvir(z, "oznaka", w=22, h=22, fill=P["glavna"], cornerRadius=6, sizingH="fixed", sizingV="fixed")
    vektor(o, "lik", "M11 4 L18 9 L15.5 18 L6.5 18 L4 9 Z", stroke=P["na_glavnoj"], sw=1.75)
    for t in ["Datoteka", "Uredi", "Crtanje", "Slojevi", "Prikaz", "Ispis", "Alati", "Pomoć"]:
        g = red(f, "izbornik · " + t, padding=[6, 10, 6, 10], cornerRadius=6, fill=P["odabrano"] if t == aktivan else "none")
        tekst(g, t, vel=13)
    razmak(f)
    tekst(f, naziv + ".gpkg", vel=13, stil="Medium")
    tekst(f, "  ·  spremljeno 20:41", vel=13, boja=P["sivo"])
    razmak(f)
    q = red(f, "gumb · Otvori u QGIS-u", padding=[4, 12, 4, 12], cornerRadius=999, stroke=P["rub"], strokeWeight=1, gap=6)
    tekst(q, "Otvori u QGIS-u", vel=12, stil="Medium", boja=P["glavna"])
    okvir(f, "razmak", w=8, h=1, sizingH="fixed")
    gumb_ikona(f, "ponisti", vel=32, ik=18, naziv="Poništi")
    gumb_ikona(f, "ponovi", vel=32, ik=18, naziv="Ponovi", boja=P["blijedo"])
    gumb_ikona(f, "mjesec", vel=32, ik=18, naziv="Tema")
    gumb_ikona(f, "pomoc", vel=32, ik=18, naziv="Pomoć")
    return f


def statusna_traka(r, h, podloga="OpenFreeMap", mjerilo="1 : 4 500"):
    f = red(r, "statusna traka", x=0, y=h - STATUS, gap=16, w=W, h=STATUS, padding=[0, 12, 0, 12], fill=P["vrh"], sizingH="fixed", sizingV="fixed", strokeTopWeight=1, stroke=P["rub"])
    tekst(f, "E 655 120   N 5 046 380", vel=12, features={"tnum": True})
    s = red(f, "koordinatni sustav", gap=4)
    tekst(s, "HTRS96/TM", vel=12, stil="Medium")
    ikona(s, "dolje", vel=14)
    s = red(f, "mjerilo", gap=4)
    tekst(s, mjerilo, vel=12, stil="Medium")
    ikona(s, "dolje", vel=14)
    razmak(f)
    s = red(f, "podloga", gap=6)
    ikona(s, "karta", vel=14)
    tekst(s, podloga, vel=12)
    s = red(f, "mreža", gap=6)
    ikona(s, "mreza", vel=14)
    tekst(s, "Mreža radi", vel=12)
    return f


def prozor(page_x, ime, h=H, y=60, ima_statusa=True, map_h=None, naziv="Teren 2026", mjerilo="1 : 4 500", podloga="OpenFreeMap", izbornik=None):
    r = novi_okvir(page_x, ime, W, h, y=y)
    traka_izbornika(r, naziv, izbornik)
    mh = map_h or h - MENI - STATUS
    m = okvir(r, "karta", x=0, y=MENI, w=W, h=mh, fill=P["k_zemlja"], clip=True)
    if ima_statusa:
        statusna_traka(r, h, podloga=podloga, mjerilo=mjerilo)
    return r, m, mh


def ploca(m, mh, naslov, natrag=False, zatvori=True, gumbi_glave=None, x=None, hvataljka=False):
    """Ploča desno. Jedna je i mijenja sadržaj: Slojevi, Crtanje, Svojstva, Ispis."""
    f = stup(m, "ploča · " + naslov, x=x if x is not None else W - RUB - PLOCA, y=RUB, w=PLOCA, h=mh - 2 * RUB, gap=0, cornerRadius=16, fill=P["vrh"], effects=SJENA(), sizingH="fixed", sizingV="fixed", clip=True)
    g = red(f, "glava", gap=4, padding=[12, 8, 8, 16], sizingH="fill")
    if natrag:
        gumb_ikona(g, "natrag", vel=32, ik=20, naziv="Natrag", boja=P["tekst"])
    elif hvataljka:
        ikona(g, "hvataljka", vel=18, boja=P["blijedo"])
    tekst(g, naslov, vel=20)
    razmak(g)
    for ime, nz in (gumbi_glave or []):
        gumb_ikona(g, ime, vel=32, ik=18, naziv=nz)
    if zatvori:
        gumb_ikona(g, "x", vel=32, ik=20, naziv="Zatvori")
    t = stup(f, "sadržaj", gap=14, padding=[4, 16, 12, 16], sizingH="fill", sizingV="fill", clip=True)
    return f, t


def podnozje(f):
    return red(f, "podnožje", gap=8, padding=[10, 16, 14, 16], sizingH="fill")


# ---------- ploča Slojevi ----------

def redak_stabla(r, razina, ime_ikone, t, opis=None, otvoren=None, ukljucen=True, odabran=False, oznaka=None, boja_ikone=None, brava=False):
    d = dict(gap=6, padding=[5, 6, 5, 4 + razina * 18], cornerRadius=8, sizingH="fill")
    if odabran:
        d["fill"] = P["tint"]
    f = red(r, "redak · " + t, **d)
    if otvoren is None:
        okvir(f, "uvlaka", w=18, h=18, sizingH="fixed", sizingV="fixed")
    else:
        ikona(f, "dolje" if otvoren else "desno", vel=18)
    ikona(f, ime_ikone, vel=18, boja=boja_ikone)
    s = stup(f, "natpisi", gap=0, sizingH="fill")
    tekst(s, t, vel=14, stil="Medium" if otvoren is not None else "Regular", boja=P["na_tintu"] if odabran else P["tekst"], w=250, h=20, sizingH="fill", sizingV="fixed", textAutoResize="truncate", maxLines=1)
    if opis:
        tekst(s, opis, vel=12, boja=P["na_tintu"] if odabran else P["sivo"], w=250, h=16, sizingH="fill", sizingV="fixed", textAutoResize="truncate", maxLines=1)
    if oznaka:
        o = red(f, "oznaka formata", padding=[1, 6, 1, 6], cornerRadius=4, stroke=P["rub"], strokeWeight=1)
        tekst(o, oznaka, vel=11, boja=P["sivo"], stil="Medium")
    if brava:
        ikona(f, "brava", vel=14)
    prekidac(f, ukljucen)
    return f


def radnje_slojeva(t):
    """Novi sloj, uvoz postojećeg i nova mapa: uvijek vidljivi, s natpisom."""
    g = red(t, "radnje slojeva", gap=6, sizingH="fill")
    gumb(g, "Novi sloj", "obrub", ikona_ime="plus", padding=[6, 12, 6, 10], gap=4)
    gumb(g, "Uvezi sloj", "obrub", ikona_ime="uvezi", padding=[6, 12, 6, 10], gap=4)
    gumb(g, "Mapa", "obrub", ikona_ime="nova_mapa", padding=[6, 12, 6, 10], gap=4)


def odjeljak_podloge(t, odabrana="Karta", popis=("Karta", "DGU ortofoto", "ortofoto_2024.tif")):
    """Podloga je na dnu ploče, kao i na karti: ispod svih slojeva."""
    razdjelnik(t)
    s = stup(t, "podloga", gap=8, sizingH="fill")
    naslov_polja(s, "Podloga")
    cipovi(s, [(c, c == odabrana) for c in popis])
    g = red(s, "dodaj podlogu", gap=0, sizingH="fill")
    gumb(g, "Učitaj datoteku", "tekst", ikona_ime="uvezi", padding=[4, 8, 4, 0])
    gumb(g, "Dodaj WMS", "tekst", ikona_ime="mreza", padding=[4, 8, 4, 8])


def sadrzaj_slojeva(t, odabrana_parcela=True):
    radnje_slojeva(t)
    s = stup(t, "stablo", gap=0, sizingH="fill", sizingV="fill", clip=True)
    redak_stabla(s, 0, "mapa", "Teren 2026", "3 sloja", otvoren=True)
    redak_stabla(s, 1, "poligon", "Parcele", "12 objekata", otvoren=True, boja_ikone=P["crtez"])
    redak_stabla(s, 2, "poligon", "Parcela 1118/2", "2,31 ha", boja_ikone=P["crtez"])
    redak_stabla(s, 2, "poligon", "Parcela 1124/3", f"{broj_hr(povrsina_ha(PARCELA), 2)} ha", odabran=odabrana_parcela, boja_ikone=P["crtez"])
    redak_stabla(s, 2, "poligon", "Parcela 1126", "1,87 ha", boja_ikone=P["crtez"])
    redak_stabla(s, 1, "linija", "Trase", "1 objekt", otvoren=False, boja_ikone="#1565C0")
    redak_stabla(s, 1, "tocka", "Točke", "2 objekta", otvoren=False, boja_ikone="#C62828")
    redak_stabla(s, 0, "mapa", "Parcele 2025", "2 sloja", otvoren=False, ukljucen=False)
    redak_stabla(s, 0, "mapa", "Uvezeno", "2 sloja", otvoren=True)
    redak_stabla(s, 1, "datoteka", "obilazak", "23 objekta", oznaka="KML", otvoren=False)
    redak_stabla(s, 1, "baza", "nasipi", "samo za čitanje", oznaka="GPKG", brava=True, ukljucen=False, otvoren=False)
    odjeljak_podloge(t)


def ploca_slojevi(m, mh, x=None):
    f, t = ploca(m, mh, "Slojevi", x=x, hvataljka=True)
    sadrzaj_slojeva(t)
    return f


def kartica_objekta(m, mh, x=330):
    f = stup(m, "kartica odabranog objekta", x=x, y=mh - 188, w=440, gap=12, padding=[14, 16, 8, 16], cornerRadius=16, fill=P["ploha"], effects=SJENA(), sizingH="fixed")
    g = red(f, "glava", gap=10, sizingH="fill", crossAlign="min")
    ikona(g, "poligon", vel=22, boja=P["crtez"])
    s = stup(g, "naziv", gap=1, sizingH="fill")
    tekst(s, "Parcela 1124/3", vel=16, stil="Medium")
    tekst(s, "Poligon · sloj Parcele, mapa Teren 2026", vel=12, boja=P["sivo"])
    gumb_ikona(g, "x", vel=28, ik=18, naziv="Zatvori")
    v = red(f, "mjere", gap=28, sizingH="fill")
    for a, b in [("Površina", f"{broj_hr(povrsina_ha(PARCELA), 2)} ha"), ("Opseg", f"{broj_hr(opseg_m(PARCELA))} m"), ("Vrhova", "6"), ("Kultura", "oranica")]:
        s = stup(v, "mjera · " + a, gap=2)
        tekst(s, a, vel=12, boja=P["sivo"])
        tekst(s, b, vel=14)
    d = red(f, "radnje", gap=0, sizingH="fill")
    razmak(d)
    gumb(d, "Atributi", "tekst")
    gumb(d, "Izvezi", "tekst")
    gumb(d, "Uredi", "tekst")


