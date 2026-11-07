#!/usr/bin/env python3
"""Slaže sve zaslone u ../TerraSimple.figcraft i osvježava otvoreni FigCraft.

Pokretanje: python3 izradi.py <mapa za radne datoteke> [port otvorenog FigCrafta]
"""
import json
import os
import subprocess
import sys

from nacrt import *  # noqa: F401,F403
from temelj import *  # noqa: F401,F403
from temelj import C, P, SJENA, add, tema

KORAK = W + 120


def stranica(ime):
    k = add("page.add", {"name": ime})
    add("page.select", {"id": k})


def mjere(r, popis):
    v = red(r, "mjere", gap=24, sizingH="fill")
    for a, b in popis:
        s = stup(v, "mjera · " + a, gap=2)
        tekst(s, a, vel=12, boja=P["sivo"])
        tekst(s, b, vel=14)
    return v


def klizac(r, oznaka, vrijednost, udio):
    s = stup(r, "klizač · " + oznaka, gap=6, sizingH="fill")
    g = red(s, "natpis", sizingH="fill")
    tekst(g, oznaka, vel=13, boja=P["sivo"])
    razmak(g)
    tekst(g, vrijednost, vel=13)
    t = okvir(s, "staza", w=PLOCA - 32, h=14, sizingH="fixed", sizingV="fixed")
    pravokutnik(t, "staza", 0, 5, PLOCA - 32, 4, P["visoko"], cornerRadius=2)
    pravokutnik(t, "ispuna", 0, 5, (PLOCA - 32) * udio, 4, P["glavna"], cornerRadius=2)
    krug(t, "ručka", (PLOCA - 32) * udio, 7, 7, P["glavna"])


def boje_stila(r, odabrana):
    s = stup(r, "boja", gap=8, sizingH="fill")
    naslov_polja(s, "Boja")
    g = red(s, "uzorci", gap=8)
    for b in ["#E8590C", "#C62828", "#8E24AA", "#1565C0", "#00838F", "#2E7D32", "#F9A825", "#37474F"]:
        o = okvir(g, "uzorak " + b, w=28, h=28, sizingH="fixed", sizingV="fixed")
        if b == odabrana:
            krug(o, "prsten", 14, 14, 14, "none", stroke=P["tekst"], strokeWeight=2)
        krug(o, "boja", 14, 14, 10, b)
    d = okvir(g, "druga boja", w=28, h=28, layout="horizontal", mainAlign="center", crossAlign="center", cornerRadius=14, stroke=P["rub"], strokeWeight=1, sizingH="fixed", sizingV="fixed")
    ikona(d, "plus", vel=16)


def redak_stila(r, opis, uzorak="poligon", boja=None):
    """Sažetak stila u ploči. Stil se bira na vlastitom zaslonu, da ploča ostane jednostavna."""
    g = red(r, "stil", gap=10, sizingH="fill")
    ikona(g, uzorak, vel=22, boja=boja or P["crtez"])
    s = stup(g, "natpisi", gap=1, sizingH="fill")
    tekst(s, "Stil", vel=14)
    tekst(s, opis, vel=12, boja=P["sivo"])
    gumb(g, "Promijeni", "tekst", ikona_ime="stil")


def okvir_mjere(r, veliko, dolje, napomena):
    f = stup(r, "nacrtano", gap=4, padding=[12, 14, 12, 14], cornerRadius=12, fill=P["nisko"], sizingH="fill")
    tekst(f, "Nacrtano", vel=12, boja=P["sivo"])
    tekst(f, veliko, vel=26)
    tekst(f, dolje, vel=13, boja=P["sivo"])
    tekst(f, napomena, vel=12, boja=P["blijedo"])


def glavni_sadrzaj(m, mh, lijevo=False):
    karta_vektori(m)
    moji_objekti(m, odabran=True, s_vrhovima=True)
    natpis_na_karti(m, "Parcela 1124/3", 440, 300)
    if lijevo:
        ploca_slojevi(m, mh, x=RUB)
        trazilica(m, x=RUB + PLOCA + 12)
        alati(m, x=RUB + PLOCA + 12)
        kontrole(m, W - RUB - 40, mh)
        kartica_objekta(m, mh, x=660)
        podloga_mjerilo(m, mh, desno=W - 300, lijevo=RUB + PLOCA + 12)
    else:
        trazilica(m)
        alati(m)
        kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
        ploca_slojevi(m, mh)
        kartica_objekta(m, mh)
        podloga_mjerilo(m, mh)


def zaslon_glavni(x, ime_teme):
    tema(ime_teme)
    _, m, mh = prozor(x, f"A · Glavni prozor, odabran objekt · {ime_teme}")
    glavni_sadrzaj(m, mh)


def zaslon_lijevo(x):
    tema("svijetla")
    _, m, mh = prozor(x, "A2 · Ploča premještena lijevo, kao u Google Earthu")
    glavni_sadrzaj(m, mh, lijevo=True)


def zaslon_pomak_ploce(x):
    """Ploča se hvata za glavu i lijepi za lijevi ili desni rub. Program pamti gdje je ostala."""
    tema("svijetla")
    _, m, mh = prozor(x, "A4 · Ploča se povlači za glavu i lijepi za lijevi ili desni rub")
    karta_vektori(m)
    moji_objekti(m)
    pravokutnik(m, "mjesto lijevo", RUB, RUB, PLOCA, mh - 2 * RUB, P["glavna"] + "33" if len(P["glavna"]) == 7 else P["glavna"], cornerRadius=16, stroke=P["glavna"], strokeWeight=2, dashes=[8, 5])
    pravokutnik(m, "mjesto desno", W - RUB - PLOCA, RUB, PLOCA, mh - 2 * RUB, "none", cornerRadius=16, stroke=P["blijedo"], strokeWeight=1.5, dashes=[8, 5])
    trazilica(m, x=RUB + PLOCA + 12)
    alati(m, x=RUB + PLOCA + 12)
    kontrole(m, W - RUB - 40, mh)
    podloga_mjerilo(m, mh, desno=W - 300, lijevo=RUB + PLOCA + 12)
    ploca_slojevi(m, mh, x=470)
    tamni_natpis(m, "Pusti ovdje: ploča ide lijevo.", RUB + 80, mh // 2)


def zaslon_stablo(x):
    """Duboko stablo s dugim nazivima i stacionažom, kakvo je u pravom myplaces.kml."""
    tema("svijetla")
    _, m, mh = prozor(x, "A3 · Veliko stablo: duboke mape, dugi nazivi, stacionaža", naziv="Kanali")
    karta_vektori(m)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh)
    _, t = ploca(m, mh, "Slojevi")
    radnje_slojeva(t)
    f = red(t, "traži u slojevima", gap=8, padding=[8, 12, 8, 12], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fill")
    ikona(f, "filtar", vel=16)
    tekst(f, "Traži u slojevima", vel=14, boja=P["blijedo"])
    s = stup(t, "stablo", gap=0, sizingH="fill", sizingV="fill", clip=True)
    redak_stabla(s, 0, "mapa", "Objekti", otvoren=True)
    redak_stabla(s, 1, "mapa", "Kanali i vodotoci", otvoren=True)
    redak_stabla(s, 2, "linija", "Kanali I. reda", "42 objekta", otvoren=True, boja_ikone="#1565C0")
    for ime, st in [
        ("Kanal Sjever", "0+000 – 25+945"), ("Kanal Brijest", "0+000 – 0+795"),
        ("Crpna stanica Jug – potok Lipa, dugi naziv koji ne stane", "0+000 – 0+993"),
        ("Dovodni kanal za crpnu stanicu Podravlje", "0+000 – 40+976"), ("Kanal Hrast", "0+000 – 4+600"),
        ("Kanal Topola", "0+000 – 3+705"), ("Lateralni kanal Novo Selo – Dolina", "0+000 – 12+180"),
    ]:
        redak_stabla(s, 3, "linija", ime, st, boja_ikone="#1565C0")
    redak_stabla(s, 2, "linija", "Kanali II. reda", "38 objekata", otvoren=False, boja_ikone="#1565C0")
    redak_stabla(s, 1, "tocka", "Crpne stanice", "12 objekata", otvoren=False, boja_ikone="#C62828")
    redak_stabla(s, 0, "mapa", "Teren", "7 slojeva", otvoren=False, ukljucen=False)
    odjeljak_podloge(t)


def zaslon_crtanje(x):
    tema("svijetla")
    _, m, mh = prozor(x, "B · Crtanje poligona: mjere se vide odmah", podloga="DGU ortofoto")
    karta_orto(m)
    tocke = [(300, 230), (520, 190), (690, 290), (650, 450), (440, 480)]
    kursor = (330, 400)
    sve = tocke + [kursor]
    ha, op = povrsina_ha(sve), opseg_m(sve)
    vektor(m, "ploha", put_lika(sve), fill=P["crtez_ploha"])
    vektor(m, "nacrtano", put_lika(tocke, zatvori=False), stroke=P["crtez"], sw=2.5)
    vektor(m, "do pokazivača", f"M440 480 L{kursor[0]} {kursor[1]} L300 230", stroke="#FFFFFF", sw=1.75, dashes=[6, 5])
    vrhovi(m, tocke, srednji=False)
    krug(m, "pokazivač", kursor[0], kursor[1], 4, P["crtez"], stroke="#FFFFFF", strokeWeight=1.5)
    tamni_natpis(m, f"{broj_hr(ha, 2)} ha · {broj_hr(op)} m", kursor[0] + 14, kursor[1] - 12)
    trazilica(m)
    alati(m, aktivan="poligon")
    f = red(m, "radnje crtanja", x=RUB + 76, y=RUB + 56 + 102, gap=2, padding=[4, 4, 4, 4], cornerRadius=8, fill=P["tamno_ploha"])
    for t in ["Završi", "Odustani"]:
        g = red(f, t, padding=[6, 10, 6, 10], cornerRadius=6)
        tekst(g, t, vel=13, boja=P["na_tamnom"])
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh, zasluge="Ortofoto: © Državna geodetska uprava", svijetlo=True)
    p, t = ploca(m, mh, "Novi poligon", zatvori=False, natrag=True)
    polje(t, "Poligon 1", oznaka="Naziv (može i poslije)")
    okvir_mjere(t, f"{broj_hr(ha, 2)} ha", f"Opseg {broj_hr(op)} m · 6 vrhova", "Računato na elipsoidu WGS84")
    polje(t, "Parcele", oznaka="Sloj", ikona_ime="dolje")
    razdjelnik(t)
    redak_stila(t, "Kao sloj: narančasto, 0,7 mm")
    okvir(t, "razmak", w=1, h=1, sizingV="fill")
    p_ = stup(t, "kako se crta", gap=8, padding=[12, 14, 12, 14], cornerRadius=12, fill=P["nisko"], sizingH="fill")
    for a_, b_ in [("Lijevi klik", "dodaje točku"), ("Povlačenje", "pomiče točku"), ("Desni klik", "briše točku"), ("Dvoklik", "završava crtu ili lik"), ("Esc", "odustaje")]:
        g_ = red(p_, "pravilo · " + a_, gap=8, sizingH="fill")
        tekst(g_, a_, vel=13, stil="Medium", w=92)
        tekst(g_, b_, vel=13, boja=P["sivo"])
    d = podnozje(p)
    razmak(d)
    gumb(d, "Odustani", "tekst")
    gumb(d, "Završi")


def zaslon_svojstva(x):
    tema("svijetla")
    _, m, mh = prozor(x, "C · Uređivanje objekta: vrhovi se povlače odmah, bez posebnog načina", podloga="DGU ortofoto")
    karta_orto(m)
    vektor(m, "Poligon", put_lika(PARCELA), fill=P["crtez_ploha"], stroke=P["crtez"], sw=2.5)
    vrhovi(m, PARCELA)
    tamni_natpis(m, "Povuci točku. Desni klik je briše.", 650, 268)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh, zasluge="Ortofoto: © Državna geodetska uprava", svijetlo=True)
    p, t = ploca(m, mh, "Svojstva", natrag=True)
    polje(t, "Parcela 1124/3", oznaka="Naziv")
    s = stup(t, "opis", gap=6, sizingH="fill")
    naslov_polja(s, "Opis")
    f = stup(s, "unos", padding=[10, 12, 10, 12], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fill")
    tekst(f, "Oranica uz odvodni kanal. Provjeriti ulaz s južne strane.", vel=14, w=10, sizingH="fill", textAutoResize="height")
    polje(t, "Parcele", oznaka="Sloj", ikona_ime="dolje")
    mjere(t, [("Površina", f"{broj_hr(povrsina_ha(PARCELA), 2)} ha"), ("Opseg", f"{broj_hr(opseg_m(PARCELA))} m"), ("Vrhova", "6")])
    razdjelnik(t)
    redak_stila(t, "Kao sloj: narančasto, 0,7 mm")
    razdjelnik(t)
    g = red(t, "atributi", sizingH="fill")
    s = stup(g, "natpisi", gap=1)
    tekst(s, "Atributi", vel=14)
    tekst(s, "k. č. 1124/3 · oranica · još 3 polja", vel=12, boja=P["sivo"])
    razmak(g)
    gumb(g, "Tablica", "tekst", ikona_ime="tablica")
    okvir(t, "razmak", w=1, h=1, sizingV="fill")
    d = podnozje(p)
    gumb(d, "Obriši", "opasan", ikona_ime="kanta")
    razmak(d)
    gumb(d, "Gotovo")


def izbor_uzorka(r, naziv, odabran, crtaj):
    f = stup(r, "izbor · " + naziv, w=76, h=66, gap=6, padding=[10, 4, 6, 4], cornerRadius=8, crossAlign="center", sizingH="fixed", sizingV="fixed", fill=P["tint"] if odabran else "none", stroke="none" if odabran else P["rub"], strokeWeight=1)
    o = okvir(f, "uzorak", w=56, h=22, sizingH="fixed", sizingV="fixed")
    crtaj(o)
    tekst(f, naziv, vel=11, boja=P["na_tintu"] if odabran else P["sivo"], stil="Medium" if odabran else "Regular")


def izbor_vrste_crte(r, odabrana, boja=None):
    s = stup(r, "vrsta crte", gap=8, sizingH="fill")
    naslov_polja(s, "Vrsta crte")
    g = red(s, "vrste", gap=8)
    b = boja or P["tekst"]
    for ime, crtice in [("Puna", []), ("Isprekidana", [8, 5]), ("Točkasta", [1, 5]), ("Crta-točka", [10, 4, 1, 4])]:
        d = dict(dashes=crtice) if crtice else {}
        izbor_uzorka(g, ime, ime == odabrana, lambda o, d=d: vektor(o, "crta", "M2 11 L54 11", stroke=b, sw=2.5, **d))


def natpis_stila(t):
    g = red(t, "natpis", sizingH="fill")
    s = stup(g, "natpisi", gap=1, sizingH="fill")
    tekst(s, "Natpis na karti", vel=14)
    tekst(s, "Iz stupca Naziv, 3 mm", vel=12, boja=P["sivo"])
    prekidac(g, True)


def glava_stila(t):
    s = stup(t, "na što se odnosi", gap=8, sizingH="fill")
    cipovi(s, [("Cijeli sloj", True), "Samo ovaj objekt"])


def kraj_stila(p, t):
    okvir(t, "razmak", w=1, h=1, sizingV="fill")
    poruka(t, "Stil se sprema uz sloj, pa ga QGIS otvara s njim.")
    d = podnozje(p)
    gumb(d, "Vrati zadano", "tekst")
    razmak(d)
    gumb(d, "Gotovo")


def zaslon_stil_crte(x):
    tema("svijetla")
    _, m, mh = prozor(x, "C2 · Stil crte: vrsta, boja, debljina, natpis")
    karta_vektori(m)
    moji_objekti(m)
    vektor(m, "Trasa u novom stilu", TRASA, stroke="#FFFFFF", sw=6)
    vektor(m, "Trasa u novom stilu", TRASA, stroke="#1565C0", sw=3.5, dashes=[10, 6])
    natpis_na_karti(m, "Trasa odvodnje", 170, 345)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh)
    p, t = ploca(m, mh, "Stil · Trase", natrag=True)
    glava_stila(t)
    izbor_vrste_crte(t, "Isprekidana", "#1565C0")
    boje_stila(t, "#1565C0")
    klizac(t, "Debljina", "0,9 mm", 0.35)
    klizac(t, "Prozirnost", "0 %", 0.0)
    razdjelnik(t)
    natpis_stila(t)
    kraj_stila(p, t)


def zaslon_stil_tocke(x):
    tema("svijetla")
    _, m, mh = prozor(x, "C3 · Stil točke: ikona, boja, veličina, natpis")
    karta_vektori(m)
    moji_objekti(m)
    for tx, ty, ime in TOCKE:
        krug(m, "pozadina ikone", tx, ty - 14, 16, "#FFFFFF", stroke="#1565C0", strokeWeight=1.5)
        ikona(m, "kap", vel=20, boja="#1565C0", x=tx - 10, y=ty - 24)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh)
    p, t = ploca(m, mh, "Stil · Točke", natrag=True)
    glava_stila(t)
    s = stup(t, "ikona", gap=8, sizingH="fill")
    naslov_polja(s, "Ikona")
    g = red(s, "ikone", gap=8, wrap=True, sizingH="fill", crossAlign="min")
    for ime in ["tocka", "krug", "kvadrat", "trokut", "zvijezda", "kriz", "kap", "valovi", "zastava", "kuca", "upozorenje", "info"]:
        o = ime == "kap"
        f = okvir(g, "ikona · " + ime, w=48, h=44, layout="horizontal", mainAlign="center", crossAlign="center", cornerRadius=8, sizingH="fixed", sizingV="fixed", fill=P["tint"] if o else "none", stroke="none" if o else P["rub"], strokeWeight=1)
        ikona(f, ime, vel=22, boja=P["na_tintu"] if o else P["sivo"])
    gumb(s, "Učitaj svoju ikonu (PNG ili SVG)", "tekst", ikona_ime="uvezi", padding=[6, 8, 4, 0])
    boje_stila(t, "#1565C0")
    klizac(t, "Veličina", "6 mm", 0.4)
    razdjelnik(t)
    natpis_stila(t)
    kraj_stila(p, t)


def zaslon_stil_poligona(x):
    tema("svijetla")
    _, m, mh = prozor(x, "C4 · Stil poligona: ispuna, rub, prozirnost")
    karta_vektori(m)
    moji_objekti(m, odabran=True)
    natpis_na_karti(m, "Parcela 1124/3", 440, 300)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    podloga_mjerilo(m, mh)
    p, t = ploca(m, mh, "Stil · Parcele", natrag=True)
    glava_stila(t)
    s = stup(t, "ispuna", gap=8, sizingH="fill")
    naslov_polja(s, "Ispuna")
    g = red(s, "vrste", gap=8)
    def puna(o):
        pravokutnik(o, "uzorak", 6, 2, 44, 18, P["crtez_ploha"], stroke=P["crtez"], strokeWeight=1.5)
    def srafura(o):
        pravokutnik(o, "uzorak", 6, 2, 44, 18, "none", stroke=P["crtez"], strokeWeight=1.5)
        for i in range(0, 5):
            vektor(o, "crta", f"M{10 + i * 8} 20 L{18 + i * 8} 2", stroke=P["crtez"], sw=1)
    def bez(o):
        pravokutnik(o, "uzorak", 6, 2, 44, 18, "none", stroke=P["crtez"], strokeWeight=1.5)
    izbor_uzorka(g, "Puna", True, puna)
    izbor_uzorka(g, "Šrafura", False, srafura)
    izbor_uzorka(g, "Bez ispune", False, bez)
    boje_stila(t, "#E8590C")
    klizac(t, "Prozirnost ispune", "75 %", 0.75)
    razdjelnik(t)
    izbor_vrste_crte(t, "Puna", "#E8590C")
    klizac(t, "Debljina ruba", "0,7 mm", 0.28)
    razdjelnik(t)
    natpis_stila(t)
    kraj_stila(p, t)


def zaslon_mjerenje(x):
    tema("svijetla")
    _, m, mh = prozor(x, "D · Mjerenje: ništa se ne sprema dok korisnik to ne zatraži")
    karta_vektori(m)
    moji_objekti(m)
    tocke = [(180, 560), (400, 500), (620, 560), (900, 620)]
    vektor(m, "mjera", put_lika(tocke, zatvori=False), stroke=P["tekst"], sw=2.5, dashes=[8, 5])
    vrhovi(m, tocke, srednji=False)
    for i in range(len(tocke) - 1):
        a, b = tocke[i], tocke[i + 1]
        tamni_natpis(m, f"{broj_hr(opseg_m([a, b], zatvori=False))} m", (a[0] + b[0]) // 2 - 22, (a[1] + b[1]) // 2 - 32)
    trazilica(m)
    alati(m, aktivan="ravnalo")
    ukupno = sum(round(opseg_m([tocke[i], tocke[i + 1]], zatvori=False)) for i in range(len(tocke) - 1))
    f = stup(m, "ploča mjerenja", x=RUB + 76, y=RUB + 56, w=280, gap=10, padding=14, cornerRadius=12, fill=P["ploha"], effects=SJENA(), sizingH="fixed")
    g = red(f, "glava", gap=8, sizingH="fill")
    cip(g, "Udaljenost", True)
    cip(g, "Površina")
    razmak(g)
    gumb_ikona(g, "x", vel=28, ik=18, naziv="Zatvori (Esc)")
    tekst(f, f"{broj_hr(ukupno)} m", vel=26)
    tekst(f, "Ukupno, 3 dijela. Računato na elipsoidu WGS84.", vel=12, boja=P["sivo"], w=252, textAutoResize="height")
    d = red(f, "radnje", gap=0, sizingH="fill")
    gumb(d, "Spremi kao liniju", "tekst", ikona_ime="linija")
    razmak(d)
    gumb(d, "Očisti", "tekst")
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh)
    ploca_slojevi(m, mh)
    podloga_mjerilo(m, mh)


def zaslon_tablica(x, ime="E · Atributna tablica: odabir u tablici i na karti je isti"):
    tema("svijetla")
    vis = 300
    r, m, mh = prozor(x, ime, map_h=H - MENI - STATUS - vis)
    karta_vektori(m)
    moji_objekti(m, odabran=True, s_vrhovima=True)
    natpis_na_karti(m, "Parcela 1124/3", 440, 300)
    trazilica(m)
    alati(m)
    kontrole(m, W - RUB - 40, mh, slojevi=True)
    podloga_mjerilo(m, mh, desno=W - 300)
    dovrsi_tablicu(r, vis)
    return r


def zaslon_izvoz_tablice(x):
    r = zaslon_tablica(x, "E2 · Izvoz tablice u Excel ili CSV")
    d = dijalog(r, 560, 540, "Izvoz tablice")
    s_ = stup(d, "format", gap=8, sizingH="fill")
    naslov_polja(s_, "Format")
    cipovi(s_, [("Excel (.xlsx)", True), "CSV"])
    polje(d, "Parcele.xlsx", oznaka="Datoteka", ikona_ime="mapa")
    s_ = stup(d, "retci", gap=8, sizingH="fill")
    naslov_polja(s_, "Retci")
    cipovi(s_, [("Svih 12", True), "Samo odabrani (1)", "Samo filtrirani"])
    g = red(d, "koordinate", sizingH="fill")
    st = stup(g, "natpisi", gap=1, sizingH="fill")
    tekst(st, "Dodaj koordinate točaka", vel=14)
    tekst(st, "Samo za čitanje. Crteži se tablicom ne mijenjaju.", vel=12, boja=P["sivo"])
    prekidac(g, False)
    poruka(d, "Stupac fid povezuje retke s objektima na karti. Ostavite ga kakav jest pa ćete izmjene moći vratiti u sloj.")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Izvezi")


def zaslon_uvoz_izmjena(x):
    """Prije primjene korisnik vidi svaku izmjenu. Bez ovoga se u sloj tiho upiše pogrešan redak."""
    r = zaslon_tablica(x, "E3 · Uvoz izmjena: prije primjene vidi se što se mijenja")
    d = dijalog(r, 760, 680, "Uvoz izmjena u sloj Parcele")
    polje(d, "Parcele.xlsx", oznaka="Datoteka", ikona_ime="mapa")
    g = red(d, "sažetak", gap=8, sizingH="fill")
    for broj, opis in [("3", "retka se mijenjaju"), ("8", "redaka je isto"), ("1", "redak ima grešku"), ("1", "novi stupac")]:
        k = stup(g, "broj · " + opis, gap=1, padding=[8, 12, 8, 12], cornerRadius=8, fill=P["nisko"], sizingH="fill")
        tekst(k, broj, vel=20)
        tekst(k, opis, vel=12, boja=P["sivo"])
    naslov_polja(d, "Izmjene")
    t = okvir(d, "popis izmjena", w=712, h=176, cornerRadius=8, stroke=P["rub"], strokeWeight=1, clip=True, sizingH="fill", sizingV="fixed", fill=P["ploha"])
    stupci = [("Objekt", 12), ("Stupac", 200), ("Bilo je", 330), ("Bit će", 520)]
    pravokutnik(t, "zaglavlje", 0, 0, 760, 30, P["nisko"])
    for ime, cx in stupci:
        tekst(t, ime, x=cx, y=8, vel=12, stil="Medium")
    izmjene = [
        ("Parcela 1124/3", "Kultura", "oranica", "livada"),
        ("Parcela 1131/1", "Napomena", "Uz potok", "Uz potok, pristup s juga"),
        ("Parcela 1140/4", "Pregledano", "—", "4. 11. 2026."),
        ("Parcela 1132", "Pregledano", "3. 6. 2025.", "35. 6. 2025."),
    ]
    y = 30
    for i, (a, b, c, e) in enumerate(izmjene):
        gr = i == 3
        if gr:
            pravokutnik(t, "greška", 0, y, 760, 36, P["upoz_ploha"])
        tekst(t, a, x=12, y=y + 10, vel=13)
        tekst(t, b, x=200, y=y + 10, vel=13, boja=P["sivo"])
        tekst(t, c, x=330, y=y + 10, vel=13, boja=P["sivo"])
        tekst(t, e, x=520, y=y + 10, vel=13, stil="Medium")
        if gr:
            ikona(t, "upozorenje", vel=16, x=650, y=y + 10, boja=P["upoz"])
        pravokutnik(t, "crta", 0, y + 35, 760, 1, P["visoko"])
        y += 36
    stavka_pregleda(d, "pazi", "Parcela 1132: datum „35. 6. 2025.” nije valjan", "Taj se redak preskače, a ostali se primjenjuju.")
    stavka_pregleda(d, "pazi", "Novi stupac „Izvođač” dodat će se u sloj", "U stupcu su tekstovi. Može se i isključiti.")
    stavka_pregleda(d, "da", "Crteži i stupci koje računa program ostaju netaknuti", "Obrisani redak u datoteci ne briše objekt iz sloja.")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    tekst(g, "Izmjena se može poništiti (Ctrl+Z).", vel=13, boja=P["sivo"])
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Primijeni 3 izmjene")


def dovrsi_tablicu(korijen, vis):
    t = okvir(korijen, "atributna tablica", x=0, y=H - STATUS - vis, w=W, h=vis, fill=P["vrh"], clip=True, strokeTopWeight=1, stroke=P["rub"])
    g = red(t, "glava tablice", x=0, y=0, w=W, h=48, gap=10, padding=[0, 12, 0, 16], sizingH="fixed", sizingV="fixed")
    tekst(g, "Sloj Parcele", vel=15, stil="Medium")
    tekst(g, "1 od 12 odabrano", vel=13, boja=P["sivo"])
    f = red(g, "filtar", gap=8, w=240, padding=[6, 10, 6, 10], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fixed")
    ikona(f, "filtar", vel=16)
    tekst(f, "Filtriraj retke", vel=13, boja=P["blijedo"])
    razmak(g)
    gumb(g, "Novo polje", "tekst", ikona_ime="plus")
    gumb(g, "Izvezi", "tekst", ikona_ime="izvezi")
    gumb(g, "Uvezi izmjene", "tekst", ikona_ime="uvezi")
    gumb_ikona(g, "zaslon", vel=32, ik=18, naziv="Zaseban prozor")
    gumb_ikona(g, "x", vel=32, ik=20, naziv="Zatvori")
    stupci = [("Naziv", 16, 240), ("Vrsta", 256, 100), ("K. č.", 356, 110), ("Kultura", 466, 150), ("ƒ Površina (ha)", 616, 140), ("ƒ Opseg (m)", 756, 120), ("Pregledano", 876, 130), ("Napomena", 1006, 420)]
    y = 48
    pravokutnik(t, "zaglavlje", 0, y, W, 32, P["nisko"])
    for ime, cx, _ in stupci:
        izr = ime.startswith("ƒ")
        tekst(t, ime, x=cx, y=y + 8, vel=12, stil="Medium", boja=P["sivo"] if izr else P["tekst"])
    ikona(t, "dolje", vel=14, x=16 + 42, y=y + 9)
    redovi = [
        ("Parcela 1118/2", "Poligon", "1118/2", "oranica", "2,31", "604", "12. 5. 2025.", ""),
        ("Parcela 1124/3", "Poligon", "1124/3", "oranica", broj_hr(povrsina_ha(PARCELA), 2), broj_hr(opseg_m(PARCELA)), "12. 5. 2025.", "Provjeriti ulaz s južne strane"),
        ("Parcela 1126", "Poligon", "1126", "livada", "1,87", "548", "12. 5. 2025.", ""),
        ("Parcela 1131/1", "Poligon", "1131/1", "voćnjak", "0,96", "402", "3. 6. 2025.", "Uz potok"),
        ("Parcela 1132", "Poligon", "1132", "oranica", "4,10", "846", "3. 6. 2025.", ""),
        ("Parcela 1140/4", "Poligon", "1140/4", "šuma", "6,72", "1 138", "—", "Nije pregledano"),
    ]
    y += 32
    for i, red_ in enumerate(redovi):
        o = i == 1
        pravokutnik(t, "redak", 0, y, W, 30, P["tint"] if o else "none")
        pravokutnik(t, "crta", 0, y + 29, W, 1, P["visoko"])
        vr = red_[:4] + red_[4:6] + red_[6:]
        for j, (_, cx, _s) in enumerate(stupci):
            izr = j in (4, 5)
            tekst(t, vr[j], x=cx, y=y + 7, vel=13, boja=(P["na_tintu"] if o else (P["sivo"] if izr else P["tekst"])), features={"tnum": True})
        y += 30
    # ćelija koja se upravo uređuje
    c = okvir(t, "ćelija u uređivanju", x=460, y=48 + 32 + 60 + 30, w=150, h=30, fill=P["ploha"], stroke=P["glavna"], strokeWeight=2, cornerRadius=2)
    tekst(c, "voćnjak", x=6, y=7, vel=13)
    tekst(t, "ƒ znači da stupac računa program iz crteža, pa se ne upisuje ručno. Dvoklik uređuje ćeliju.", x=16, y=vis - 22, vel=12, boja=P["sivo"])


def dijalog(r, w, h, naslov):
    pravokutnik(r, "prekrivač", 0, 0, W, H, P["prekrivac"])
    f = stup(r, "prozorčić · " + naslov, x=(W - w) // 2, y=(H - h) // 2, w=w, h=h, gap=14, padding=24, cornerRadius=16, fill=P["vrh"], effects=SJENA(), sizingH="fixed", sizingV="fixed", clip=True)
    tekst(f, naslov, vel=20)
    return f


def stavka_pregleda(r, vrsta, naslov, opis=None):
    ik, b = {"da": ("kvacica", "#2E7D32"), "pazi": ("upozorenje", P["upoz"]), "ne": ("x", P["greska"])}[vrsta]
    f = red(r, "stavka · " + naslov, gap=12, sizingH="fill", crossAlign="min", padding=[4, 0, 4, 0])
    ikona(f, ik, vel=20, boja=b, sir=2)
    s = stup(f, "natpisi", gap=2, sizingH="fill")
    tekst(s, naslov, vel=14, w=10, sizingH="fill", textAutoResize="height")
    if opis:
        tekst(s, opis, vel=13, boja=P["sivo"], w=10, sizingH="fill", textAutoResize="height")


def zaslon_izvoz(x):
    tema("svijetla")
    r, m, mh = prozor(x, "K · Izvoz: prije spremanja korisnik vidi što se gubi")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 600, 560, "Izvoz u KML za Google Earth")
    polje(d, "Teren 2026.kml", oznaka="Datoteka", ikona_ime="mapa")
    razdjelnik(d)
    tekst(d, "Što ide u datoteku", vel=13, boja=P["sivo"])
    stavka_pregleda(d, "da", "Sve geometrije, nazivi, opisi i mape", "12 poligona, 1 linija i 2 točke.")
    stavka_pregleda(d, "da", "Atributi", "5 polja ide u ExtendedData.")
    stavka_pregleda(d, "pazi", "Koordinate se pretvaraju u WGS84", "Projekt je u HTRS96/TM, a KML poznaje samo WGS84.")
    stavka_pregleda(d, "pazi", "Isprekidane crte postaju pune", "KML nema isprekidane crte. Boja i širina ostaju.")
    stavka_pregleda(d, "ne", "Predlošci ispisa ne idu u KML", "Ostaju u projektu Teren 2026.gpkg.")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Izvezi KML")


def zaslon_crs(x):
    tema("svijetla")
    r, m, mh = prozor(x, "L · Uvoz bez koordinatnog sustava: program pita, ne pretpostavlja")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 600, 560, "U kojem su koordinatnom sustavu podaci?")
    tekst(d, "Datoteka nasipi_stari.shp nema zapisan koordinatni sustav. Odaberite sustav u kojem je nastala, pa provjerite pregled na karti.", vel=14, boja=P["sivo"], w=10, sizingH="fill", textAutoResize="height")
    for naslov, opis, o in [
        ("HTRS96/TM", "EPSG:3765 · službeni sustav u Hrvatskoj", False),
        ("HDKS, zona 6", "EPSG:3908 · stari planovi istočno od 16° 30′ · točnost oko 1 m", True),
        ("HDKS, zona 5", "EPSG:3907 · stari planovi zapadno od 16° 30′ · točnost oko 1 m", False),
        ("WGS84", "EPSG:4326 · GPS i Google Earth", False),
    ]:
        f = red(d, "izbor · " + naslov, gap=12, padding=[10, 12, 10, 12], cornerRadius=8, sizingH="fill", fill=P["tint"] if o else "none", stroke=P["rub"] if not o else "none", strokeWeight=1)
        k = okvir(f, "oznaka izbora", w=20, h=20, sizingH="fixed", sizingV="fixed")
        krug(k, "prsten", 10, 10, 9, "none", stroke=P["glavna"] if o else P["blijedo"], strokeWeight=2)
        if o:
            krug(k, "točka", 10, 10, 5, P["glavna"])
        s = stup(f, "natpisi", gap=1)
        tekst(s, naslov, vel=14, stil="Medium")
        tekst(s, opis, vel=12, boja=P["sivo"])
    tekst(d, "Drugi sustav (EPSG broj)…", vel=14, boja=P["glavna"], stil="Medium")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Uvezi")


def zaslon_novi_sloj(x):
    tema("svijetla")
    r, m, mh = prozor(x, "F · Novi sloj: dovoljni su naziv i vrsta")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 520, 480, "Novi sloj")
    polje(d, "Bunari", oznaka="Naziv")
    s = stup(d, "vrsta", gap=8, sizingH="fill")
    naslov_polja(s, "Što će sloj sadržavati")
    cipovi(s, [("Točke", True), ("Linije", False, "linija"), ("Poligone", False, "poligon")])
    polje(d, "Teren 2026", oznaka="U mapu", ikona_ime="dolje")
    g = red(d, "napredno", gap=6)
    ikona(g, "desno", vel=18)
    tekst(g, "Polja i koordinatni sustav", vel=14)
    tekst(d, "Polja se mogu dodati i kasnije, u tablici. Sloj se sprema u Teren 2026.gpkg, pa ga otvara i QGIS.", vel=13, boja=P["sivo"], w=10, sizingH="fill", textAutoResize="height")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Napravi sloj")


def zaslon_izbornik_sloja(x):
    tema("svijetla")
    r, m, mh = prozor(x, "F2 · Desni klik na sloj: stil, tablica, premještanje u mapu")
    glavni_sadrzaj(m, mh)
    f = stup(r, "izbornik sloja", x=W - RUB - PLOCA - 170, y=MENI + 190, w=250, gap=0, padding=[6, 0, 6, 0], cornerRadius=10, fill=P["ploha"], effects=SJENA(), sizingH="fixed")
    for st in [("stil", "Stil…"), ("tablica", "Atributna tablica"), None, ("trazi", "Približi na sloj"), ("premjesti", "Premjesti u mapu…"), ("olovka", "Preimenuj"), None, ("izvezi", "Izvezi sloj…"), None, ("kanta", "Obriši sloj")]:
        if st is None:
            razdjelnik(f)
            continue
        g = red(f, "stavka · " + st[1], gap=10, padding=[7, 14, 7, 14], sizingH="fill", fill=P["odabrano"] if st[1].startswith("Stil") else "none")
        opasno = st[0] == "kanta"
        ikona(g, st[0], vel=18, boja=P["greska"] if opasno else None)
        tekst(g, st[1], vel=14, boja=P["greska"] if opasno else P["tekst"])
    tamni_natpis(r, "Slojevi se u mapu mogu i povući mišem.", W - RUB - PLOCA - 170 - 190, MENI + 480)


def zaslon_uvoz_sloja(x):
    tema("svijetla")
    r, m, mh = prozor(x, "G · Uvoz postojećeg sloja: GeoPackage može imati više slojeva")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 600, 560, "Uvoz sloja")
    polje(d, "nasipi_sektor.gpkg", oznaka="Datoteka", ikona_ime="mapa")
    s = stup(d, "popis", gap=4, sizingH="fill")
    naslov_polja(s, "Slojevi u datoteci")
    for ik, ime, op, uk in [
        ("linija", "nasipi", "128 linija · HTRS96/TM", True),
        ("tocka", "objekti", "340 točaka · HTRS96/TM", True),
        ("poligon", "obuhvati", "16 poligona · HTRS96/TM", False),
    ]:
        f = red(s, "sloj · " + ime, gap=10, padding=[8, 12, 8, 12], cornerRadius=8, stroke=P["rub"], strokeWeight=1, sizingH="fill")
        k = okvir(f, "kućica", w=20, h=20, cornerRadius=4, fill=P["glavna"] if uk else "none", stroke="none" if uk else P["blijedo"], strokeWeight=2, sizingH="fixed", sizingV="fixed")
        if uk:
            ikona(k, "kvacica", vel=20, boja=P["na_glavnoj"], sir=2.5)
        ikona(f, ik, vel=18)
        st = stup(f, "natpisi", gap=1)
        tekst(st, ime, vel=14)
        tekst(st, op, vel=12, boja=P["sivo"])
    polje(d, "Uvezeno", oznaka="U mapu", ikona_ime="dolje")
    s = stup(d, "način", gap=8, sizingH="fill")
    naslov_polja(s, "Kako ga koristiti")
    cipovi(s, [("Čitaj iz izvorne datoteke", True), "Kopiraj u projekt"])
    tekst(s, "Izvorna datoteka se ne mijenja bez potvrde. Promjene u QGIS-u vide se i ovdje.", vel=13, boja=P["sivo"], w=10, sizingH="fill", textAutoResize="height")
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Uvezi 2 sloja")


def kartice_podloge(d, odabrana):
    g = red(d, "način dodavanja", gap=8)
    for t, ik in [("Datoteka", "uvezi"), ("WMS", "mreza"), ("WMTS", "mreza"), ("XYZ", "mreza")]:
        cip(g, t, t == odabrana, ik)


def zaslon_podloga_wms(x):
    tema("svijetla")
    r, m, mh = prozor(x, "H · Podloga preko WMS-a")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 620, 620, "Dodaj podlogu")
    kartice_podloge(d, "WMS")
    s = stup(d, "adresa", gap=6, sizingH="fill")
    naslov_polja(s, "Adresa poslužitelja")
    g = red(s, "red", gap=8, sizingH="fill")
    f = red(g, "unos", padding=[10, 12, 10, 12], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fill")
    tekst(f, "https://geoportal.dgu.hr/wms", vel=14)
    gumb(g, "Dohvati", "obrub")
    s = stup(d, "slojevi", gap=4, sizingH="fill")
    naslov_polja(s, "Slojevi na poslužitelju")
    for ime, op, o in [("DOF", "Digitalni ortofoto", True), ("HOK", "Hrvatska osnovna karta", False), ("TK25", "Topografska karta 1 : 25 000", False)]:
        f = red(s, "sloj · " + ime, gap=10, padding=[8, 12, 8, 12], cornerRadius=8, sizingH="fill", fill=P["tint"] if o else "none", stroke="none" if o else P["rub"], strokeWeight=1)
        k = okvir(f, "kućica", w=20, h=20, cornerRadius=4, fill=P["glavna"] if o else "none", stroke="none" if o else P["blijedo"], strokeWeight=2, sizingH="fixed", sizingV="fixed")
        if o:
            ikona(k, "kvacica", vel=20, boja=P["na_glavnoj"], sir=2.5)
        st = stup(f, "natpisi", gap=1)
        tekst(st, ime, vel=14, stil="Medium")
        tekst(st, op, vel=12, boja=P["sivo"])
    polje(d, "DGU ortofoto", oznaka="Naziv podloge")
    polje(d, "© Državna geodetska uprava", oznaka="Zasluge (vide se na karti i ispisu)")
    razmak_v = okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Dodaj podlogu")


def zaslon_podloga_datoteka(x):
    tema("svijetla")
    r, m, mh = prozor(x, "I · Podloga iz datoteke: radi i bez interneta")
    glavni_sadrzaj(m, mh)
    d = dijalog(r, 620, 620, "Dodaj podlogu")
    kartice_podloge(d, "Datoteka")
    z = stup(d, "mjesto za datoteku", gap=8, padding=28, cornerRadius=12, stroke=P["rub"], strokeWeight=1.5, dashes=[6, 4], sizingH="fill", crossAlign="center")
    ikona(z, "uvezi", vel=28)
    tekst(z, "Povucite datoteku ovamo", vel=15, stil="Medium")
    tekst(z, "GeoTIFF, PMTiles, MBTiles ili slika s koordinatama", vel=13, boja=P["sivo"])
    gumb(z, "Odaberi datoteku", "obrub")
    f = red(d, "odabrana datoteka", gap=12, padding=[10, 12, 10, 12], cornerRadius=8, fill=P["nisko"], sizingH="fill")
    ikona(f, "slika", vel=22)
    st = stup(f, "natpisi", gap=1, sizingH="fill")
    tekst(st, "ortofoto_2024.tif", vel=14, stil="Medium")
    tekst(st, "GeoTIFF · 2,4 GB · HTRS96/TM · piksel 0,5 m", vel=12, boja=P["sivo"])
    ikona(f, "kvacica", vel=20, boja="#2E7D32", sir=2)
    polje(d, "Ortofoto 2024", oznaka="Naziv podloge")
    s = stup(d, "kako", gap=8, sizingH="fill")
    naslov_polja(s, "Datoteka u projektu")
    cipovi(s, [("Samo poveznica", True), "Kopiraj uz projekt"])
    okvir(d, "razmak", w=1, h=1, sizingV="fill")
    g = red(d, "gumbi", gap=8, sizingH="fill")
    razmak(g)
    gumb(g, "Odustani", "tekst")
    gumb(g, "Dodaj podlogu")


def zaslon_izbornik(x):
    tema("svijetla")
    r, m, mh = prozor(x, "J · Izbornik Datoteka: projekt je GeoPackage, a QGIS je jedan klik", izbornik="Datoteka")
    glavni_sadrzaj(m, mh)
    f = stup(r, "izbornik Datoteka", x=46, y=MENI - 2, w=300, gap=0, padding=[6, 0, 6, 0], cornerRadius=10, fill=P["ploha"], effects=SJENA(), sizingH="fixed")
    for st in [
        ("Novi projekt", "Ctrl+N"), ("Otvori…", "Ctrl+O"), ("Spremi", "Ctrl+S"), ("Spremi kao…", "Ctrl+Shift+S"), None,
        ("Uvezi sloj…", ""), ("Izvezi sloj…", ""), ("Dodaj podlogu…", ""), None,
        ("Otvori u QGIS-u", ""), ("Prikaži datoteku u mapi", ""), None,
        ("Ispis…", "Ctrl+P"),
    ]:
        if st is None:
            razdjelnik(f)
            continue
        g = red(f, "stavka · " + st[0], padding=[7, 14, 7, 14], sizingH="fill", fill=P["odabrano"] if st[0].endswith("QGIS-u") else "none")
        tekst(g, st[0], vel=14)
        razmak(g)
        if st[1]:
            tekst(g, st[1], vel=12, boja=P["sivo"])


def zaslon_ispis(x):
    tema("svijetla")
    _, m, mh = prozor(x, "M · Ispis: A3 položeno u mjerilu 1 : 2 000")
    karta_vektori(m)
    moji_objekti(m)
    # A3 u 1 : 2 000 na zaslonu 1 : 4 500 (0,2646 mm po pikselu)
    ow, oh = round(420 / 0.2646 * 2000 / MJERILO), round(297 / 0.2646 * 2000 / MJERILO)
    ox, oy = 150, 110
    desno = W
    pravokutnik(m, "izvan okvira", 0, 0, desno, oy, P["prekrivac"])
    pravokutnik(m, "izvan okvira", 0, oy + oh, desno, mh - oy - oh, P["prekrivac"])
    pravokutnik(m, "izvan okvira", 0, oy, ox, oh, P["prekrivac"])
    pravokutnik(m, "izvan okvira", ox + ow, oy, desno - ox - ow, oh, P["prekrivac"])
    pravokutnik(m, "okvir ispisa", ox, oy, ow, oh, "none", stroke=P["glavna"], strokeWeight=2.5)
    for kx, ky in [(ox, oy), (ox + ow, oy), (ox, oy + oh), (ox + ow, oy + oh)]:
        pravokutnik(m, "kut", kx - 5, ky - 5, 10, 10, "#FFFFFF", stroke=P["glavna"], strokeWeight=2)
    f = red(m, "natpis okvira", x=ox, y=oy - 38, gap=10, padding=[6, 12, 6, 12], cornerRadius=6, fill=P["glavna"])
    tekst(f, "A3 položeno · 1 : 2 000", vel=13, stil="Medium", boja=P["na_glavnoj"])
    tekst(f, "Povucite okvir ili njegov kut.", vel=12, boja=P["na_glavnoj"])
    # legenda i naslov iz predloška, na papiru
    l = stup(m, "naslov i legenda", x=ox + 14, y=oy + 14, gap=3, padding=[8, 10, 8, 10], cornerRadius=4, fill="#FFFFFFF2")
    tekst(l, "Teren 2026", vel=13, stil="Medium", boja="#1A1C1E")
    tekst(l, "Parcele, odvodnja i točke", vel=11, boja="#44474F")
    tekst(l, "6. 11. 2026. · HTRS96/TM", vel=11, boja="#44474F")
    ikona(m, "sjever", vel=26, x=ox + ow - 40, y=oy + 14, boja="#1A1C1E")
    kontrole(m, W - RUB - PLOCA - 12 - 40, mh, s_ispisom=True)
    podloga_mjerilo(m, mh, svijetlo=True)
    p, t = ploca(m, mh, "Ispis karte")
    polje(t, "Moj A3 · naslov i legenda", oznaka="Predložak", ikona_ime="dolje")
    gumb(t, "Uredi predložak", "tekst", ikona_ime="olovka")
    s = stup(t, "format", gap=8, sizingH="fill")
    naslov_polja(s, "Format papira")
    cipovi(s, ["A4", ("A3", True), "A2", "A1", "A0", "Drugo"])
    s = stup(t, "položaj", gap=8, sizingH="fill")
    naslov_polja(s, "Položaj papira")
    cipovi(s, [("Uspravno", False, "uspravno"), ("Položeno", True)])
    s = stup(t, "mjerilo", gap=8, sizingH="fill")
    naslov_polja(s, "Mjerilo (obavezno)")
    cipovi(s, ["1 : 500", "1 : 1 000", ("1 : 2 000", True), "1 : 5 000", "1 : 10 000", "Upiši"])
    tekst(s, f"Na papiru 1 cm je 20 m. Okvir pokriva {broj_hr(0.42 * 2000)} × {broj_hr(0.297 * 2000)} m.", vel=12, boja=P["sivo"], w=10, sizingH="fill", textAutoResize="height")
    polje(t, "Teren 2026", oznaka="Naslov")
    okvir(t, "razmak", w=1, h=1, sizingV="fill")
    poruka(t, "Podloga je u ovom mjerilu preslabe razlučivosti pa će na papiru biti mutnija. Crteži i tekst ostaju oštri.", "upozorenje")
    d = podnozje(p)
    razmak(d)
    gumb(d, "Ispiši", "tekst", ikona_ime="ispis")
    gumb(d, "Izvezi PDF")


def zaslon_predlozak(x):
    """Uređivač predloška: prozor se mijenja, jer se ne radi o karti nego o papiru."""
    tema("svijetla")
    r = novi_okvir(x, "N · Uređivač predloška ispisa", W, H)
    g = red(r, "traka uređivača", x=0, y=0, w=W, h=48, gap=4, padding=[0, 12, 0, 8], fill=P["vrh"], sizingH="fixed", sizingV="fixed", strokeBottomWeight=1, stroke=P["rub"])
    gumb_ikona(g, "natrag", vel=36, naziv="Natrag na ispis", boja=P["tekst"])
    tekst(g, "Moj A3 · naslov i legenda", vel=15, stil="Medium")
    okvir(g, "razmak", w=16, h=1, sizingH="fixed")
    for i, n in [("ponisti", "Poništi"), ("ponovi", "Ponovi"), ("poravnaj", "Poravnaj"), ("ispred", "Ispred ili iza"), ("brava", "Zaključaj")]:
        gumb_ikona(g, i, vel=36, ik=18, naziv=n)
    razmak(g)
    tekst(g, "A3 položeno · 420 × 297 mm", vel=13, boja=P["sivo"])
    okvir(g, "razmak", w=12, h=1, sizingH="fixed")
    gumb(g, "Spremi predložak", padding=[8, 16, 8, 16])
    pod = okvir(r, "radna površina", x=0, y=48, w=W, h=H - 48, fill=P["visoko"], clip=True)
    a = stup(pod, "dodaj na papir", x=RUB, y=RUB, gap=2, padding=4, cornerRadius=12, fill=P["ploha"], effects=SJENA())
    for ik, t in [("karta", "Karta"), ("tekst", "Tekst"), ("legenda", "Legenda"), ("mjerilo", "Mjerilo"), ("sjever", "Sjever"), ("slika", "Slika"), ("crta", "Crta"), ("okvir", "Okvir"), ("kalendar", "Datum"), ("info", "Izvor")]:
        st = stup(a, "dodaj · " + t, gap=2, padding=[7, 4, 6, 4], cornerRadius=8, w=60, sizingH="fixed", crossAlign="center")
        ikona(st, ik, vel=20)
        tekst(st, t, vel=11, boja=P["sivo"])
    # list A3, 2 px po mm
    lx, ly, lw, lh = 112, 92, 840, 594
    pravokutnik(pod, "list", lx, ly, lw, lh, "#FFFFFF", effects=SJENA())
    pm = okvir(pod, "okvir karte", x=lx + 20, y=ly + 20, w=lw - 40, h=lh - 110, fill=P["k_zemlja"], clip=True)
    karta_vektori(pm)
    moji_objekti(pm)
    pravokutnik(pod, "odabran okvir", lx + 20, ly + 20, lw - 40, lh - 110, "none", stroke=P["glavna"], strokeWeight=2)
    for kx, ky in [(lx + 20, ly + 20), (lx + lw - 20, ly + 20), (lx + 20, ly + lh - 90), (lx + lw - 20, ly + lh - 90), (lx + lw / 2, ly + 20), (lx + lw / 2, ly + lh - 90), (lx + 20, ly + 20 + (lh - 110) / 2), (lx + lw - 20, ly + 20 + (lh - 110) / 2)]:
        pravokutnik(pod, "ručka", kx - 4, ky - 4, 8, 8, "#FFFFFF", stroke=P["glavna"], strokeWeight=1.5)
    tekst(pod, "Teren 2026", x=lx + 20, y=ly + lh - 78, vel=22, stil="Medium", boja="#1A1C1E")
    tekst(pod, "Parcele i odvodnja · {datum}", x=lx + 20, y=ly + lh - 48, vel=12, boja="#44474F")
    tekst(pod, "Izvor: OpenFreeMap · {koordinatni sustav}", x=lx + 20, y=ly + lh - 30, vel=11, boja="#74777F")
    lg = stup(pod, "legenda", x=lx + 400, y=ly + lh - 80, gap=4)
    for b, t in [(P["crtez"], "Parcele"), ("#1565C0", "Trasa odvodnje"), ("#C62828", "Točke")]:
        o = red(lg, "stavka legende", gap=8)
        pravokutnik(o, "uzorak", 0, 0, 18, 4, b)
        tekst(o, t, vel=11, boja="#1A1C1E")
    mj = okvir(pod, "grafičko mjerilo", x=lx + 600, y=ly + lh - 62, w=180, h=40)
    pravokutnik(mj, "crta", 0, 18, 180, 3, "#1A1C1E")
    pravokutnik(mj, "polovina", 0, 18, 90, 3, "#74777F")
    tekst(mj, "0", x=0, y=0, vel=10, boja="#1A1C1E")
    tekst(mj, "200 m", x=150, y=0, vel=10, boja="#1A1C1E")
    tekst(mj, "{mjerilo}", x=0, y=24, vel=10, boja="#44474F")
    ikona(pod, "sjever", vel=30, x=lx + lw - 56, y=ly + lh - 66, boja="#1A1C1E")
    p = stup(pod, "ploča · Svojstva", x=W - RUB - 340, y=RUB, w=340, h=H - 48 - 2 * RUB, gap=14, padding=16, cornerRadius=16, fill=P["vrh"], effects=SJENA(), sizingH="fixed", sizingV="fixed", clip=True)
    tekst(p, "Okvir karte", vel=20)
    s = stup(p, "položaj", gap=8, sizingH="fill")
    naslov_polja(s, "Položaj i veličina (mm)")
    for par in [[("X", "10"), ("Y", "10")], [("Širina", "400"), ("Visina", "242")]]:
        rr = red(s, "par", gap=8, sizingH="fill")
        for a_, b_ in par:
            f = red(rr, "unos " + a_, gap=8, padding=[8, 10, 8, 10], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fill")
            tekst(f, a_, vel=12, boja=P["sivo"])
            razmak(f)
            tekst(f, b_, vel=14)
    polje(p, "Odabire se pri ispisu", oznaka="Mjerilo", ikona_ime="dolje")
    polje(p, "Trenutačna podloga", oznaka="Podloga", ikona_ime="dolje")
    g2 = red(p, "okvir crte", sizingH="fill")
    s2 = stup(g2, "natpisi", gap=1, sizingH="fill")
    tekst(s2, "Crta oko karte", vel=14)
    tekst(s2, "0,5 mm, crna", vel=12, boja=P["sivo"])
    prekidac(g2, True)
    g2 = red(p, "zaključano", sizingH="fill")
    tekst(g2, "Zaključaj položaj", vel=14)
    razmak(g2)
    prekidac(g2, False)
    razdjelnik(p)
    naslov_polja(p, "Polja koja program sam popunjava")
    cipovi(p, ["{naslov}", "{datum}", "{mjerilo}", "{koordinatni sustav}", "{autor}"])
    okvir(p, "razmak", w=1, h=1, sizingV="fill")
    poruka(p, "Veličine su u milimetrima na papiru, pa ispis ne ovisi o zaslonu.")


def main():
    radna = sys.argv[1] if len(sys.argv) > 1 else "."
    port = sys.argv[2] if len(sys.argv) > 2 else None
    os.makedirs(radna, exist_ok=True)
    cilj = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "TerraSimple.figcraft"))

    add("page.rename", {"name": "1 · Glavni prozor"})
    zaslon_glavni(0, "svijetla")
    zaslon_glavni(KORAK, "tamna")
    zaslon_lijevo(2 * KORAK)
    zaslon_pomak_ploce(3 * KORAK)
    zaslon_stablo(4 * KORAK)
    stranica("2 · Crtanje i mjerenje")
    zaslon_crtanje(0)
    zaslon_svojstva(KORAK)
    zaslon_stil_crte(2 * KORAK)
    zaslon_stil_tocke(3 * KORAK)
    zaslon_stil_poligona(4 * KORAK)
    zaslon_mjerenje(5 * KORAK)
    stranica("3 · Atributna tablica")
    zaslon_tablica(0)
    zaslon_izvoz_tablice(KORAK)
    zaslon_uvoz_izmjena(2 * KORAK)
    stranica("4 · Slojevi i podloge")
    zaslon_novi_sloj(0)
    zaslon_izbornik_sloja(KORAK)
    zaslon_uvoz_sloja(2 * KORAK)
    zaslon_podloga_wms(3 * KORAK)
    zaslon_podloga_datoteka(4 * KORAK)
    stranica("5 · Datoteke i QGIS")
    zaslon_izbornik(0)
    zaslon_izvoz(KORAK)
    zaslon_crs(2 * KORAK)
    stranica("6 · Ispis")
    zaslon_ispis(0)
    zaslon_predlozak(KORAK)
    add("page.select", {"index": 0})

    skripta = os.path.join(radna, "run.json")
    with open(skripta, "w", encoding="utf-8") as f:
        json.dump([{"command": "command.batch", "params": {"commands": C, "label": "Nacrt TerraSimplea"}}], f, ensure_ascii=False)
    cli = os.path.expanduser("~/.local/share/figcraft/current/figcraft-cli")
    iz = subprocess.run([cli, "run", skripta, "--out", cilj], capture_output=True, text=True)
    if iz.returncode != 0:
        sys.exit(iz.stderr[-2000:] or iz.stdout[-2000:])
    print("naredbi:", len(C))
    print("\n".join(iz.stdout.splitlines()[-1:])[:200])
    if port:
        osvjezi(cli, port)


def osvjezi(cli, port):
    """Otvoreni FigCraft ponovno učita datoteku s diska."""
    p = subprocess.Popen([cli, "mcp", "--attach", port], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    def posalji(o):
        p.stdin.write(json.dumps(o) + "\n")
        p.stdin.flush()
    posalji({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "nacrt", "version": "1"}}})
    p.stdout.readline()
    posalji({"jsonrpc": "2.0", "method": "notifications/initialized"})
    posalji({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "run_command", "arguments": {"command": "file.revert", "params": {}}}})
    print("osvježeno:", p.stdout.readline().strip()[:200])
    p.stdin.close()
    p.terminate()


if __name__ == "__main__":
    main()
