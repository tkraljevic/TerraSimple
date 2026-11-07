"""Geometrija, veličine i izvoz satin glass ikona.

Odluke (7. 11. 2026.). SVG se iscrtava izravno u svakoj veličini.
Bez argumenata ispisuje naredbe za uređivu biblioteku u nacrtu.
"""

import argparse
from copy import deepcopy
from html import escape
import json
from pathlib import Path
import re

from izvezi_ikone import DORADE
from temelj import IKONE

LICENCA = (Path(__file__).resolve().parents[2] / "resursi/ikone/sucelje/LICENCA-LUCIDE.txt").read_text()
VELICINE = (14, 16, 18, 20, 22)
ISPUNE = {"tocka", "poligon", "odaberi", "mapa", "nova_mapa", "datoteka",
          "brava", "mjesec", "sjever", "ispis", "baza", "ravnalo"}
POPRAVCI = {
    "tablica": [("r", 3, 3, 18, 18, 2), ("p", "M3 9 L21 9"),
                ("p", "M3 15 L21 15"), ("p", "M12 3 L12 21")],
}
MALI = {
    "tablica": [("r", 3, 3, 18, 18, 2), ("p", "M3 9 L21 9"), ("p", "M12 3 L12 21")],
    "slojevi": [("p", "M12 3 L21 8 L12 13 L3 8 Z"), ("p", "M3 14 L12 19 L21 14")],
    "mreza": [("c", 12, 12, 9), ("p", "M4 12 L20 12"),
              ("p", "M12 3 C16 8 16 16 12 21 C8 16 8 8 12 3 Z")],
    "ravnalo": [("p", "M3 15 L15 3 L21 9 L9 21 Z"),
                ("p", "M8 10 L11 13"), ("p", "M13 5 L16 8")],
}


def dijelovi(ime, vel=22):
    if ime == "ravnalo" and vel == 14:
        return [("p", "M3 15 L15 3 L21 9 L9 21 Z"), ("p", "M10 8 L13 11")]
    if vel <= 16 and ime in MALI:
        return deepcopy(MALI[ime])
    return deepcopy(dict(IKONE, **DORADE, **POPRAVCI)[ime])


def profil(vel):
    return dict(crtez=vel, rub=2 if vel <= 16 else 3,
                potez=1 if vel <= 16 else 2, jednostavan=vel <= 16)


def paleta(ime, tema, stanje="obicno"):
    tamna = tema == "tamna"
    if stanje == "odabrani_alat":
        return "#FFFFFF", "#CFF3DB"
    if stanje in ("opasna_radnja", "negativna_oznaka", "crvena_tocka"):
        return ("#FFD0D5", "#F58B9B") if tamna else ("#FF8B91", "#B4293C")
    if stanje == "priguseno":
        return ("#9BB1A8", "#6D8D80") if tamna else ("#9BA5A1", "#69766F")
    if stanje == "plava_podloga" or stanje == "plava_linija":
        return ("#B6E5FF", "#6DBBF2") if tamna else ("#68BAFC", "#145AB5")
    if ime == "upozorenje" or stanje == "narancasti_poligon":
        return ("#FFE0A4", "#EDAF62") if tamna else ("#FFB970", "#C54D0C")
    if stanje == "potvrda" or ime == "kvacica":
        return ("#B4F5C2", "#70CF99") if tamna else ("#8ADB9C", "#236A3B")
    return ("#D7FFF0", "#64BFAE") if tamna else ("#3C9B8A", "#075448")


def gradijent(a, b):
    return dict(type="linearGradient", **{"from": [0.1, 0], "to": [0.8, 1]}, stops=[a, b])


def poravnaj(v, vel):
    faza = 0.5 if profil(vel)["potez"] == 1 else 0
    return round(v - faza) + faza


def geometrija(vrsta, p, vel):
    s = vel / 24
    if vrsta == "p":
        dij = re.findall(r"[MLCZ][^MLCZ]*", p[0])
        zapis = []
        prethodna = None
        for d in dij:
            brojevi = [float(x) * s for x in re.findall(r"-?\d+(?:\.\d+)?", d[1:])]
            if d[0] in "ML":
                brojevi = [poravnaj(x, vel) for x in brojevi]
            if d[0] == "L" and brojevi == prethodna:
                # Kratke crte s okruglim krajem predstavljaju točku.
                brojevi[0] += 0.01
            zapis.append(d[0] + " ".join(f"{x:.4f}".rstrip("0").rstrip(".") for x in brojevi))
            if brojevi:
                prethodna = brojevi[-2:]
        return dict(type="vector", x=0, y=0, path=" ".join(zapis))
    if vrsta in ("c", "e"):
        rx, ry = p[2] * s, (p[2] if vrsta == "c" else p[3]) * s
        return dict(type="ellipse", x=p[0]*s-rx, y=p[1]*s-ry, width=2*rx, height=2*ry)
    x, y = poravnaj(p[0]*s, vel), poravnaj(p[1]*s, vel)
    desno, dolje = poravnaj((p[0]+p[2])*s, vel), poravnaj((p[1]+p[3])*s, vel)
    return dict(type="rectangle", x=x, y=y, width=desno-x, height=dolje-y, radius=p[4]*s)


def slojevi(ime, vel, tema, stanje="obicno"):
    pr = profil(vel)
    a, b = paleta(ime, tema, stanje)
    ispuna = ("#8DE4C534", "#15776108") if tema == "tamna" else ("#E2F8ED", "#80CEB233")
    izlaz = []
    for i, (vrsta, *p) in enumerate(dijelovi(ime, vel)):
        g = geometrija(vrsta, p, vel)
        naglasak = ime in ("tocka", "linija") and vrsta == "c" and stanje != "odabrani_alat"
        fill = ("#FFF0B7", "#EAAE47") if naglasak else ispuna if i == 0 and ime in ISPUNE else None
        potez = ("#FFDC86", "#C58928") if naglasak else (a,b)
        cap = "butt" if ime == "tablica" and i > 0 else "round"
        izlaz.append(dict(g=g, fill=fill, stroke=potez, sw=pr["potez"], cap=cap, highlight=False))
        if not pr["jednostavan"]:
            sjaj = deepcopy(g);sjaj["y"] -= 0.35
            izlaz.append(dict(g=sjaj,fill=None,stroke=("#FFFFFFA0", "#FFFFFF00"),sw=0.4,cap=cap,highlight=True))
    return izlaz


def oblik_svg(g):
    if g["type"] == "vector":
        return "path", dict(d=g["path"], transform=f'translate({g["x"]} {g["y"]})')
    if g["type"] == "ellipse":
        return "ellipse",dict(cx=g["x"]+g["width"]/2,cy=g["y"]+g["height"]/2,rx=g["width"]/2,ry=g["height"]/2)
    return "rect",dict(x=g["x"],y=g["y"],width=g["width"],height=g["height"],rx=g["radius"])


def svg(ime, vel, tema, stanje="obicno", predznak="i"):
    pr = profil(vel);rub=pr["rub"];platno=vel+2*rub;defs=[];oblici=[]
    def paint(par, oznaka):
        if not par:return "none"
        stops=[]
        for i,c in enumerate(par):
            alpha=int(c[7:9],16)/255 if len(c)==9 else 1
            stops.append(f'<stop offset="{i}" stop-color="{c[:7]}" stop-opacity="{alpha:.5f}"/>')
        defs.append(f'<linearGradient id="{oznaka}" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="{vel}" y2="{vel}">'+''.join(stops)+'</linearGradient>')
        return f'url(#{oznaka})'
    for i,sl in enumerate(slojevi(ime,vel,tema,stanje)):
        tag,at=oblik_svg(sl["g"])
        at.update(fill=paint(sl["fill"],f'{predznak}f{i}'),stroke=paint(sl["stroke"],f'{predznak}s{i}'))
        at.update({"stroke-width":sl["sw"],"stroke-linecap":sl["cap"],"stroke-linejoin":"round"})
        oblici.append('<'+tag+' '+ ' '.join(f'{k}="{escape(str(v))}"' for k,v in at.items())+'/>')
    blur=0.3 if vel<=16 else 0.625;dy=0.5 if vel<=16 else 0.9
    defs.append(f'<filter id="{predznak}shadow" filterUnits="userSpaceOnUse" x="{-rub}" y="{-rub}" width="{platno}" height="{platno}" color-interpolation-filters="sRGB"><feDropShadow dx="0" dy="{dy}" stdDeviation="{blur}" flood-color="#001C15" flood-opacity="0.25"/></filter>')
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{platno}" height="{platno}" viewBox="0 0 {platno} {platno}"><metadata>'+escape(LICENCA)+'</metadata><defs>'+''.join(defs)+f'</defs><g transform="translate({rub} {rub})" filter="url(#{predznak}shadow)">'+''.join(oblici)+'</g></svg>'


def naredbe_lika(ime, vel, tema, stanje, parent):
    izlaz=[]
    for sl in slojevi(ime,vel,tema,stanje):
        g=deepcopy(sl["g"]);rad=g.pop("radius",None)
        props=dict(fill="none", strokes=[gradijent(*sl["stroke"])],strokeWeight=sl["sw"],
                   strokeAlign="center",strokeCap="none" if sl["cap"] == "butt" else sl["cap"],strokeJoin="round")
        if sl["fill"]:props["fills"]=[gradijent(*sl["fill"])]
        if rad is not None:props["cornerRadius"]=rad
        izlaz.append(dict(command="node.create",params=dict(**g,parent=parent,select=False,
            name=ime+(" · odsjaj" if sl["highlight"] else " · staklo"),props=props)))
    return izlaz


def naredbe_biblioteke():
    izlaz=[]
    for t,tema in enumerate(("svijetla","tamna")):
        ref="biblioteka_"+tema
        izlaz.append(dict(command="node.create",**{"as":ref},params=dict(type="frame",
            name="Satin glass · svih 60 ikona · "+tema,x=t*1160,y=0,width=1100,height=1120,
            props=dict(fill="#1D2422" if tema=="tamna" else "#F3F7F4"))))
        for i,ime in enumerate(IKONE):
            r="izvoz_"+tema+"_"+ime
            izlaz.append(dict(command="node.create",**{"as":r},params=dict(type="frame",parent={"$ref":ref},
                name="SVG · "+ime+" · "+tema,x=42+i%6*178,y=100+i//6*98,width=22,height=22,
                props=dict(fill="none",clip=False))))
            izlaz.extend(naredbe_lika(ime,22,tema,"obicno",{"$ref":r}))
    return izlaz


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--svg",nargs=4,metavar=("IME","VELICINA","TEMA","STANJE"))
    args=parser.parse_args()
    if args.svg:print(svg(args.svg[0],int(args.svg[1]),args.svg[2],args.svg[3]))
    else:print(json.dumps(naredbe_biblioteke(),ensure_ascii=False))
