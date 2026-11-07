"""Izvoz prijedloga ikona sučelja iz nacrta, s ciljanim optičkim doradama.

SVG nastaje iz vektorske geometrije; PNG iscrtava VectorCraft CLI.
Pokretanje: python3 dizajn/skripte/izvezi_ikone.py --cli /put/do/vectorcraft-cli
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
from html import escape
from pathlib import Path
import subprocess

from temelj import IKONE


DORADE = {
    "ponisti": [("p", "M9 5 L4 10 L9 15"), ("p", "M4 10 L14 10 C18 10 21 13 21 18")],
    "ponovi": [("p", "M15 5 L20 10 L15 15"), ("p", "M20 10 L10 10 C6 10 3 13 3 18")],
    "ravnalo": [("p", "M3 15 L15 3 L21 9 L9 21 Z"), ("p", "M7 11 L9 13"), ("p", "M10 8 L13 11"), ("p", "M13 5 L15 7")],
    "spremi": [("p", "M5 3 L17 3 L21 7 L21 19 C21 20.1 20.1 21 19 21 L5 21 C3.9 21 3 20.1 3 19 L3 5 C3 3.9 3.9 3 5 3 Z"), ("p", "M7 3 L7 9 L16 9 L16 3"), ("r", 7, 14, 10, 7, 1)],
    "stil": [("p", "M21 12 C21 16 18 17 15 17 L13.5 17 C12 17 11.5 18 12.2 19 C13 20.2 12.4 21 11 21 C6 21 3 17 3 12 C3 7 7 3 12 3 C17 3 21 7 21 12 Z"), ("c", 7.5, 11, 0.8), ("c", 11, 7.5, 0.8), ("c", 15.5, 8.5, 0.8)],
    "olovka": [("p", "M16 3 L21 8 L8 21 L3 21 L3 16 Z"), ("p", "M13 6 L18 11"), ("p", "M3 16 L8 21")],
    "sjever": [("p", "M12 3 L19 21 L12 17 L5 21 Z"), ("p", "M12 3 L12 17")],
    "kuca": [("p", "M3 11 L12 3 L21 11"), ("p", "M5 9 L5 21 L19 21 L19 9"), ("p", "M10 21 L10 15 L14 15 L14 21")],
}


def likovi(dijelovi):
    izlaz = []
    for vrsta, *p in dijelovi:
        if vrsta == "p":
            izlaz.append(f'<path d="{escape(p[0])}"/>')
        elif vrsta == "c":
            izlaz.append(f'<circle cx="{p[0]}" cy="{p[1]}" r="{p[2]}"/>')
        elif vrsta == "e":
            izlaz.append(f'<ellipse cx="{p[0]}" cy="{p[1]}" rx="{p[2]}" ry="{p[3]}"/>')
        else:
            izlaz.append(f'<rect x="{p[0]}" y="{p[1]}" width="{p[2]}" height="{p[3]}" rx="{p[4]}"/>')
    return "".join(izlaz)


def grupa(dijelovi, pomak="", boja="currentColor"):
    return (f'<g transform="{pomak}" fill="none" stroke="{boja}" stroke-width="2.1" '
            f'stroke-linecap="round" stroke-linejoin="round">{likovi(dijelovi)}</g>')


def pregled(ikone):
    izlaz = ['<svg xmlns="http://www.w3.org/2000/svg" width="1100" height="1220" viewBox="0 0 1100 1220">',
             '<rect width="1100" height="1220" fill="#faf9fd"/>',
             '<text x="30" y="38" font-family="Arial" font-size="24" fill="#1a1c1e">TerraSimple · prijedlog ikona sučelja</text>',
             '<text x="30" y="65" font-family="Arial" font-size="14" fill="#44474f">60 ikona · mreža 24 × 24 · potez 2,1 · zaobljeni spojevi i završeci</text>']
    for i, (ime, dijelovi) in enumerate(ikone.items()):
        x, y = 30 + i % 6 * 180, 95 + i // 6 * 86
        oznaka = " *" if ime in DORADE else ""
        izlaz.extend([grupa(dijelovi, f"translate({x},{y}) scale(1.5)", "#0b5c56"),
                      grupa(dijelovi, f"translate({x+58},{y+8}) scale(.667)", "#44474f"),
                      f'<text x="{x}" y="{y+59}" font-family="Arial" font-size="12" fill="#44474f">{ime}{oznaka}</text>'])
    izlaz.append('<text x="30" y="982" font-family="Arial" font-size="15" fill="#1a1c1e">* Dorađene ikone: prije → poslije</text>')
    for i, (ime, dijelovi) in enumerate(DORADE.items()):
        x, y = 30 + i % 4 * 270, 1003 + i // 4 * 99
        izlaz.extend([grupa(IKONE[ime], f"translate({x},{y}) scale(1.5)", "#74777f"),
                      f'<text x="{x+51}" y="{y+27}" font-family="Arial" font-size="22" fill="#74777f">→</text>',
                      grupa(dijelovi, f"translate({x+87},{y}) scale(1.5)", "#0b5c56"),
                      f'<text x="{x}" y="{y+61}" font-family="Arial" font-size="13" fill="#44474f">{ime}</text>'])
    return "".join(izlaz) + "</svg>"


def pretvori(posao):
    cli, izvor, cilj, skala = posao
    subprocess.run([cli, "convert", str(izvor), str(cilj), "--scale", str(skala)],
                   check=True, capture_output=True, text=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, help="VectorCraft CLI")
    args = parser.parse_args()
    korijen = Path(__file__).resolve().parents[2]
    mapa = korijen / "resursi/ikone/sucelje"
    for podmapa in ("svg", "png/24", "png/48"):
        (mapa / podmapa).mkdir(parents=True, exist_ok=True)
    ikone = dict(IKONE, **DORADE)
    poslovi = []
    for ime, dijelovi in ikone.items():
        izvor = mapa / "svg" / f"{ime}.svg"
        izvor.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" color="#44474f">' + grupa(dijelovi) + '</svg>\n')
        for velicina in (24, 48):
            poslovi.append((args.cli, izvor, mapa / "png" / str(velicina) / f"{ime}.png", velicina / 24))
    izvor = korijen / "dizajn/ikone-sucelja-pregled.svg"
    izvor.write_text(pregled(ikone))
    poslovi.append((args.cli, izvor, izvor.with_suffix(".png"), 1))
    with ThreadPoolExecutor(max_workers=4) as izvrsitelj:
        list(izvrsitelj.map(pretvori, poslovi))
    print(f"Izvezeno {len(ikone)} SVG ikona, {len(poslovi)-1} PNG ikona i pregled.")


if __name__ == "__main__":
    main()
