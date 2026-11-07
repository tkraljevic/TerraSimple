"""Zajednički dijelovi nacrta: naredbe za FigCraft, boje, ikone i sastavnice.

Sve boje su u PALETE, pa se tema i glavna boja mijenjaju na jednom mjestu.
Geometrija je dijelom preuzeta ili prilagođena iz zbirke Lucide.
Usporedba svih ikona i obavijesti o licencama nalaze se uz resurse ikona.
"""
import re

C = []  # naredbe jednog command.batch

PALETE = {
    "svijetla": dict(
        vrh="#FAF9FD", ploha="#FFFFFF", nisko="#F4F3F7", visoko="#E9E8EC", odabrano="#E3E2E6",
        rub="#C4C6D0", tekst="#1A1C1E", sivo="#44474F", blijedo="#74777F",
        glavna="#0B5C56", na_glavnoj="#FFFFFF", tint="#CFE8E3", na_tintu="#00302C",
        crtez="#E8590C", crtez_ploha="#E8590C40", upoz="#A16207", upoz_ploha="#FEF3C7",
        greska="#BA1A1A", sjena="#00000026", tamno_ploha="#1E1F23", na_tamnom="#FFFFFF", prekrivac="#00000059",
        k_zemlja="#EEF0E8", k_suma="#D3E3C1", k_naselje="#E5E1D9", k_voda="#7DB4E0", k_cesta="#FFFFFF",
        k_cesta_rub="#D4D1C7", k_glavna="#F3D08A", k_natpis="#50535A", k_voda_natpis="#3B6E9C",
    ),
    "tamna": dict(
        vrh="#1E1F23", ploha="#26272C", nisko="#121316", visoko="#303136", odabrano="#3A3B41",
        rub="#44474F", tekst="#E3E2E6", sivo="#C4C6D0", blijedo="#8E9099",
        glavna="#7FD4C8", na_glavnoj="#00302C", tint="#1F4A45", na_tintu="#CFE8E3",
        crtez="#FF8A4C", crtez_ploha="#FF8A4C40", upoz="#FBBF24", upoz_ploha="#3D2F0A",
        greska="#FFB4AB", sjena="#00000066", tamno_ploha="#E3E2E6", na_tamnom="#1A1C1E", prekrivac="#00000080",
        k_zemlja="#1D211E", k_suma="#25301F", k_naselje="#2C2C2A", k_voda="#2F6085", k_cesta="#494C49",
        k_cesta_rub="#2F312F", k_glavna="#6B5A33", k_natpis="#B8BBC2", k_voda_natpis="#8DBDE6",
    ),
}
P = dict(PALETE["svijetla"])


def tema(ime):
    P.clear()
    P.update(PALETE[ime])


def add(cmd, params):
    C.append({"command": cmd, "params": params})
    return {"$step": len(C) - 1}


def _roditelj(p, r):
    if r is not None:
        p["parent"] = r
    p["select"] = False
    return p


def okvir(r, ime, x=0, y=0, w=None, h=None, **pr):
    p = {"type": "frame", "name": ime, "x": x, "y": y, "props": dict({"fill": "none"}, **pr)}
    if w is not None:
        p["width"] = w
    if h is not None:
        p["height"] = h
    return add("node.create", _roditelj(p, r))


def red(r, ime="red", gap=8, **pr):
    """Vodoravni auto layout."""
    d = dict(layout="horizontal", gap=gap, sizingH="hug", sizingV="hug", crossAlign="center")
    d.update(pr)
    return okvir(r, ime, **d)


def stup(r, ime="stup", gap=4, **pr):
    """Okomiti auto layout."""
    d = dict(layout="vertical", gap=gap, sizingH="hug", sizingV="hug")
    d.update(pr)
    return okvir(r, ime, **d)


def razmak(r):
    """Gura ono iza sebe do kraja reda."""
    return okvir(r, "razmak", w=1, h=1, sizingH="fill")


def tekst(r, t, x=0, y=0, vel=14, boja=None, stil="Regular", w=None, h=None, **pr):
    d = {"fontFamily": "Inter", "fontStyle": stil, "fontSize": vel, "fill": boja or P["tekst"]}
    d.update(pr)
    p = {"type": "text", "name": t[:40], "text": t, "x": x, "y": y, "props": d}
    if w is not None:
        p["width"] = w
    k = add("node.create", _roditelj(p, r))
    if h is not None:
        # Visina se upisuje tek nakon stvaranja: tekst je širok w, pa bi inače ostao visok kao da je prelomljen.
        add("node.set", {"id": k, "height": h})
    return k


def vektor(r, ime, path, fill="none", stroke=None, sw=1.0, **pr):
    d = {"fill": fill, "stroke": stroke or "none", "strokeWeight": sw, "strokeCap": "round", "strokeJoin": "round"}
    d.update(pr)
    return add("node.create", _roditelj({"type": "vector", "name": ime, "path": path, "x": 0, "y": 0, "props": d}, r))


def pravokutnik(r, ime, x, y, w, h, fill, **pr):
    d = {"fill": fill}
    d.update(pr)
    return add("node.create", _roditelj({"type": "rectangle", "name": ime, "x": x, "y": y, "width": w, "height": h, "props": d}, r))


def krug(r, ime, cx, cy, rad, fill, **pr):
    d = {"fill": fill}
    d.update(pr)
    return add("node.create", _roditelj({"type": "ellipse", "name": ime, "x": cx - rad, "y": cy - rad, "width": 2 * rad, "height": 2 * rad, "props": d}, r))


# Ikone: p = crta (samo apsolutni M, L, C, Z), c = krug, e = elipsa, r = pravokutnik.
IKONE = {
    "trazi": [("c", 11, 11, 8), ("p", "M21 21 L16.7 16.7")],
    "x": [("p", "M18 6 L6 18"), ("p", "M6 6 L18 18")],
    "desno": [("p", "M9 6 L15 12 L9 18")],
    "dolje": [("p", "M6 9 L12 15 L18 9")],
    "natrag": [("p", "M19 12 L5 12"), ("p", "M12 19 L5 12 L12 5")],
    "mapa": [("p", "M4 5 L9 5 L11 8 L20 8 C21 8 22 9 22 10 L22 18 C22 19 21 20 20 20 L4 20 C3 20 2 19 2 18 L2 7 C2 6 3 5 4 5 Z")],
    "nova_mapa": [("p", "M4 5 L9 5 L11 8 L20 8 C21 8 22 9 22 10 L22 18 C22 19 21 20 20 20 L4 20 C3 20 2 19 2 18 L2 7 C2 6 3 5 4 5 Z"), ("p", "M12 11 L12 17"), ("p", "M9 14 L15 14")],
    "uvezi": [("p", "M21 15 L21 19 C21 20 20 21 19 21 L5 21 C4 21 3 20 3 19 L3 15"), ("p", "M7 10 L12 15 L17 10"), ("p", "M12 15 L12 3")],
    "izvezi": [("p", "M21 15 L21 19 C21 20 20 21 19 21 L5 21 C4 21 3 20 3 19 L3 15"), ("p", "M17 8 L12 3 L7 8"), ("p", "M12 3 L12 15")],
    "tocka": [("p", "M20 10 C20 16 12 22 12 22 C12 22 4 16 4 10 C4 5.6 7.6 2 12 2 C16.4 2 20 5.6 20 10 Z"), ("c", 12, 10, 3)],
    "linija": [("c", 5, 19, 2.5), ("c", 19, 5, 2.5), ("p", "M6.8 17.2 L17.2 6.8")],
    "poligon": [("p", "M12 3 L21 10 L17.5 21 L6.5 21 L3 10 Z")],
    "ravnalo": [("p", "M2.7 15.3 L15.3 2.7 L21.3 8.7 L8.7 21.3 Z"), ("p", "M7.5 10.5 L9.5 12.5"), ("p", "M10.5 7.5 L13.5 10.5"), ("p", "M13.5 4.5 L15.5 6.5"), ("p", "M4.5 13.5 L7.5 16.5")],
    "odaberi": [("p", "M4 4 L11 21 L13.5 13.5 L21 11 Z")],
    "zaslon": [("p", "M8 3 L5 3 C4 3 3 4 3 5 L3 8"), ("p", "M21 8 L21 5 C21 4 20 3 19 3 L16 3"), ("p", "M3 16 L3 19 C3 20 4 21 5 21 L8 21"), ("p", "M16 21 L19 21 C20 21 21 20 21 19 L21 16")],
    "ispis": [("p", "M6 9 L6 2 L18 2 L18 9"), ("p", "M6 18 L4 18 C3 18 2 17 2 16 L2 11 C2 10 3 9 4 9 L20 9 C21 9 22 10 22 11 L22 16 C22 17 21 18 20 18 L18 18"), ("r", 6, 14, 12, 8, 0)],
    "plus": [("p", "M12 5 L12 19"), ("p", "M5 12 L19 12")],
    "minus": [("p", "M5 12 L19 12")],
    "slojevi": [("p", "M12 2 L22 7 L12 12 L2 7 Z"), ("p", "M2 12 L12 17 L22 12"), ("p", "M2 17 L12 22 L22 17")],
    "sjever": [("p", "M12 2 L19 21 L12 17 L5 21 Z")],
    "ponisti": [("p", "M3 7 L3 13 L9 13"), ("p", "M3 13 C6 8 10 6.5 14 7 C18 7.5 20.5 11 21 17")],
    "ponovi": [("p", "M21 7 L21 13 L15 13"), ("p", "M21 13 C18 8 14 6.5 10 7 C6 7.5 3.5 11 3 17")],
    "mjesec": [("p", "M21 12.8 C20 17 16.5 21 12 21 C7 21 3 17 3 12 C3 7.5 7 4 11.2 3 C9.5 6 10 10 12 12 C14 14 18 14.5 21 12.8 Z")],
    "olovka": [("p", "M17 3 L21 7 L8 20 L3 21 L4 16 Z")],
    "kanta": [("p", "M3 6 L21 6"), ("p", "M19 6 L18 20 C18 21 17 22 16 22 L8 22 C7 22 6 21 6 20 L5 6"), ("p", "M8 6 L8 4 C8 3 9 2 10 2 L14 2 C15 2 16 3 16 4 L16 6")],
    "tablica": [("r", 3, 3, 18, 18, 2), ("p", "M3 9 L21 9"), ("p", "M3 15 L21 15"), ("p", "M12 3 L12 21")],
    "datoteka": [("p", "M14 2 L6 2 C5 2 4 3 4 4 L4 20 C4 21 5 22 6 22 L18 22 C19 22 20 21 20 20 L20 8 Z"), ("p", "M14 2 L14 8 L20 8")],
    "baza": [("e", 12, 5, 9, 3), ("p", "M3 5 L3 19 C3 20.7 7 22 12 22 C17 22 21 20.7 21 19 L21 5"), ("p", "M3 12 C3 13.7 7 15 12 15 C17 15 21 13.7 21 12")],
    "kvacica": [("p", "M20 6 L9 17 L4 12")],
    "upozorenje": [("p", "M12 3 L22 20 L2 20 Z"), ("p", "M12 9 L12 14"), ("p", "M12 17 L12 17.1")],
    "info": [("c", 12, 12, 10), ("p", "M12 16 L12 12"), ("p", "M12 8 L12 8.1")],
    "slika": [("r", 3, 3, 18, 18, 2), ("c", 8.5, 8.5, 1.5), ("p", "M21 15 L16 10 L5 21")],
    "tekst": [("p", "M4 7 L4 4 L20 4 L20 7"), ("p", "M9 20 L15 20"), ("p", "M12 4 L12 20")],
    "brava": [("r", 3, 11, 18, 11, 2), ("p", "M7 11 L7 7 C7 4 9 2 12 2 C15 2 17 4 17 7 L17 11")],
    "mreza": [("c", 12, 12, 10), ("p", "M2 12 L22 12"), ("p", "M12 2 C15 5 16 9 16 12 C16 15 15 19 12 22 C9 19 8 15 8 12 C8 9 9 5 12 2 Z")],
    "filtar": [("p", "M22 3 L2 3 L10 12.5 L10 19 L14 21 L14 12.5 Z")],
    "spremi": [("p", "M19 21 L5 21 C4 21 3 20 3 19 L3 5 C3 4 4 3 5 3 L16 3 L21 8 L21 19 C21 20 20 21 19 21 Z"), ("p", "M17 21 L17 13 L7 13 L7 21"), ("p", "M7 3 L7 8 L15 8")],
    "legenda": [("p", "M8 6 L21 6"), ("p", "M8 12 L21 12"), ("p", "M8 18 L21 18"), ("p", "M3 6 L3.1 6"), ("p", "M3 12 L3.1 12"), ("p", "M3 18 L3.1 18")],
    "mjerilo": [("p", "M2 17 L22 17"), ("p", "M2 12 L2 17"), ("p", "M12 12 L12 17"), ("p", "M22 12 L22 17")],
    "kalendar": [("r", 3, 4, 18, 18, 2), ("p", "M16 2 L16 6"), ("p", "M8 2 L8 6"), ("p", "M3 10 L21 10")],
    "okvir": [("r", 3, 3, 18, 18, 2)],
    "uspravno": [("r", 6, 3, 12, 18, 1.5)],
    "polozeno": [("r", 3, 6, 18, 12, 1.5)],
    "karta": [("p", "M9 3 L3 6 L3 21 L9 18 L15 21 L21 18 L21 3 L15 6 Z"), ("p", "M9 3 L9 18"), ("p", "M15 6 L15 21")],
    "crta": [("p", "M4 20 L20 4")],
    "pomoc": [("c", 12, 12, 10), ("p", "M9 9 C9 7 10.5 6 12 6 C13.5 6 15 7 15 9 C15 11 12 11.5 12 14"), ("p", "M12 17.5 L12 17.6")],
    "poravnaj": [("p", "M4 3 L4 21"), ("r", 8, 6, 12, 4, 1), ("r", 8, 14, 7, 4, 1)],
    "ispred": [("r", 3, 3, 13, 13, 1.5), ("p", "M21 8 L21 19 C21 20 20 21 19 21 L8 21")],
    "hvataljka": [("c", 9, 6, 1), ("c", 15, 6, 1), ("c", 9, 12, 1), ("c", 15, 12, 1), ("c", 9, 18, 1), ("c", 15, 18, 1)],
    "kap": [("p", "M12 2 C12 2 5 10 5 15 C5 19 8 22 12 22 C16 22 19 19 19 15 C19 10 12 2 12 2 Z")],
    "zastava": [("p", "M5 22 L5 3"), ("p", "M5 4 L19 4 L15 9 L19 14 L5 14")],
    "kuca": [("p", "M3 11 L12 3 L21 11"), ("p", "M5 9 L5 21 L19 21 L19 9")],
    "trokut": [("p", "M12 3 L22 20 L2 20 Z")],
    "krug": [("c", 12, 12, 9)],
    "kvadrat": [("r", 4, 4, 16, 16, 1)],
    "zvijezda": [("p", "M12 2 L15 9 L22 9.5 L16.5 14 L18.5 21.5 L12 17.5 L5.5 21.5 L7.5 14 L2 9.5 L9 9 Z")],
    "valovi": [("p", "M2 8 C5 5 8 5 12 8 C16 11 19 11 22 8"), ("p", "M2 15 C5 12 8 12 12 15 C16 18 19 18 22 15")],
    "kriz": [("c", 12, 12, 9), ("p", "M12 7 L12 17"), ("p", "M7 12 L17 12")],
    "premjesti": [("p", "M4 5 L9 5 L11 8 L20 8 C21 8 22 9 22 10 L22 18 C22 19 21 20 20 20 L4 20 C3 20 2 19 2 18 L2 7 C2 6 3 5 4 5 Z"), ("p", "M9 14 L15 14"), ("p", "M12 11 L15 14 L12 17")],
    "stil": [("c", 12, 12, 10), ("c", 8, 10, 1), ("c", 12, 7.5, 1), ("c", 16, 10, 1), ("p", "M12 22 C10 22 10 20 11 18.5 C12 17 14 17 16 17 C19 17 22 15.5 22 12")],
}


def _skaliraj(put, s):
    return re.sub(r"-?\d+(?:\.\d+)?", lambda m: f"{float(m.group()) * s:.2f}", put)


def ikona(r, ime, vel=20, boja=None, sir=1.75, x=0, y=0, **pr):
    b = boja or P["sivo"]
    f = okvir(r, "ikona · " + ime, x=x, y=y, w=vel, h=vel, **pr)
    s = vel / 24
    crta = {"fill": "none", "stroke": b, "strokeWeight": sir, "strokeAlign": "center"}
    for pr_ in IKONE[ime]:
        v = pr_[0]
        if v == "p":
            vektor(f, "crta", _skaliraj(pr_[1], s), stroke=b, sw=sir)
        elif v == "c":
            add("node.create", _roditelj({"type": "ellipse", "name": "krug", "x": (pr_[1] - pr_[3]) * s, "y": (pr_[2] - pr_[3]) * s, "width": 2 * pr_[3] * s, "height": 2 * pr_[3] * s, "props": dict(crta)}, f))
        elif v == "e":
            add("node.create", _roditelj({"type": "ellipse", "name": "elipsa", "x": (pr_[1] - pr_[3]) * s, "y": (pr_[2] - pr_[4]) * s, "width": 2 * pr_[3] * s, "height": 2 * pr_[4] * s, "props": dict(crta)}, f))
        elif v == "r":
            add("node.create", _roditelj({"type": "rectangle", "name": "pravokutnik", "x": pr_[1] * s, "y": pr_[2] * s, "width": pr_[3] * s, "height": pr_[4] * s, "props": dict(crta, cornerRadius=pr_[5] * s)}, f))
    return f


SJENA = lambda: [{"type": "dropShadow", "color": P["sjena"], "offset": [0, 2], "radius": 8}]


def gumb_ikona(r, ime, vel=36, ik=20, boja=None, ploha=None, naziv=None, **pr):
    d = dict(layout="horizontal", mainAlign="center", crossAlign="center", cornerRadius=8, fill=ploha or "none", sizingH="fixed", sizingV="fixed")
    d.update(pr)
    f = okvir(r, naziv or ("gumb · " + ime), w=vel, h=vel, **d)
    ikona(f, ime, vel=ik, boja=boja)
    return f


def prekidac(r, ukljucen=True):
    if ukljucen:
        f = okvir(r, "prekidač · uključen", w=36, h=20, fill=P["glavna"], cornerRadius=10, sizingH="fixed", sizingV="fixed")
        krug(f, "kvaka", 26, 10, 7, P["na_glavnoj"])
    else:
        f = okvir(r, "prekidač · isključen", w=36, h=20, fill=P["visoko"], cornerRadius=10, stroke=P["rub"], strokeWeight=1, sizingH="fixed", sizingV="fixed")
        krug(f, "kvaka", 10, 10, 6, P["blijedo"])
    return f


def cip(r, t, odabran=False, ikona_ime=None):
    d = dict(padding=[7, 12, 7, 12], cornerRadius=8, gap=6)
    if odabran:
        d.update(fill=P["tint"])
    else:
        d.update(stroke=P["rub"], strokeWeight=1)
    f = red(r, "čip · " + t, **d)
    if odabran:
        ikona(f, "kvacica", vel=16, boja=P["na_tintu"], sir=2)
    elif ikona_ime:
        ikona(f, ikona_ime, vel=16)
    tekst(f, t, vel=13, boja=P["na_tintu"] if odabran else P["tekst"], stil="Medium")
    return f


def cipovi(r, popis):
    f = red(r, "čipovi", gap=8, wrap=True, sizingH="fill", crossAlign="min")
    for c in popis:
        cip(f, *c) if isinstance(c, tuple) else cip(f, c)
    return f


def gumb(r, t, vrsta="glavni", ikona_ime=None, **pr):
    d = dict(padding=[10, 20, 10, 20], cornerRadius=999, gap=8, mainAlign="center")
    if vrsta == "glavni":
        d.update(fill=P["glavna"])
        boja = P["na_glavnoj"]
    elif vrsta == "obrub":
        d.update(stroke=P["rub"], strokeWeight=1)
        boja = P["glavna"]
    elif vrsta == "opasan":
        d.update(padding=[10, 12, 10, 12])
        boja = P["greska"]
    else:
        d.update(padding=[10, 12, 10, 12])
        boja = P["glavna"]
    d.update(pr)
    f = red(r, "gumb · " + t, **d)
    if ikona_ime:
        ikona(f, ikona_ime, vel=18, boja=boja)
    tekst(f, t, vel=14, boja=boja, stil="Medium")
    return f


def naslov_polja(r, t):
    return tekst(r, t, vel=13, boja=P["sivo"])


def polje(r, vrijednost, oznaka=None, ikona_ime=None, sivo=False):
    s = stup(r, "polje · " + (oznaka or vrijednost), gap=6, sizingH="fill")
    if oznaka:
        naslov_polja(s, oznaka)
    f = red(s, "unos", padding=[10, 12, 10, 12], cornerRadius=8, stroke=P["rub"], strokeWeight=1, fill=P["ploha"], sizingH="fill")
    tekst(f, vrijednost, vel=14, boja=P["blijedo"] if sivo else P["tekst"])
    if ikona_ime:
        razmak(f)
        ikona(f, ikona_ime, vel=18)
    return s


def razdjelnik(r, w=None):
    d = dict(sizingH="fill") if w is None else {}
    return okvir(r, "razdjelnik", w=w or 10, h=1, fill=P["rub"], **d)


def poruka(r, t, vrsta="info"):
    """Napomena: tekst s ikonom, a upozorenje još i žutom plohom. Nikad samo bojom."""
    ploha = P["upoz_ploha"] if vrsta == "upozorenje" else P["nisko"]
    b = P["upoz"] if vrsta == "upozorenje" else P["sivo"]
    f = red(r, "poruka", gap=10, padding=[10, 12, 10, 12], cornerRadius=8, fill=ploha, sizingH="fill", crossAlign="min")
    ikona(f, "upozorenje" if vrsta == "upozorenje" else "info", vel=18, boja=b)
    tekst(f, t, vel=13, boja=P["tekst"] if vrsta == "upozorenje" else P["sivo"], w=10, sizingH="fill", textAutoResize="height")
    return f


def novi_okvir(page_x, ime, w, h, y=60, natpis=True):
    """Okvir zaslona na stranici, s natpisom iznad sebe."""
    if natpis:
        tekst(None, ime, x=page_x, y=y - 40, vel=22, stil="Medium", boja="#44474F")
    f = okvir(None, ime, x=page_x, y=y, w=w, h=h, fill=P["vrh"], clip=True)
    return f
