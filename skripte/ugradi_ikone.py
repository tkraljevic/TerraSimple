"""Složi sve slike ikona iz manifest.json u jednu datoteku koju program ugradi.

Program ne smije čitati ikone s diska: staza s računala na kojem je izgrađen na drugom računalu ne
postoji (proba 0.0.1 na Windowsu, 10. 11. 2026.). Pokreni nakon svake promjene ikona; test
`sve_ikone_iz_popisa_su_ugradjene` pada ako je ugrađeni paket zastario.

Izlaz uz ikone: `ugradjene.bin` (slike jedna za drugom) i `ugradjene.txt` (redak po slici:
`staza početak duljina`, poredano po stazi).
"""
import json
from pathlib import Path

MAPA = Path(__file__).resolve().parent.parent / "resursi/ikone/sucelje/efekti"


def glavno():
    manifest = json.loads((MAPA / "manifest.json").read_text(encoding="utf-8"))
    staze = sorted({i[k] for i in manifest["izvozi"] for k in ("png1x", "png2x")})
    sadrzaj, popis = bytearray(), []
    for staza in staze:
        bajtovi = (MAPA / staza).read_bytes()
        popis.append(f"{staza} {len(sadrzaj)} {len(bajtovi)}")
        sadrzaj += bajtovi
    (MAPA / "ugradjene.bin").write_bytes(bytes(sadrzaj))
    (MAPA / "ugradjene.txt").write_text("\n".join(popis) + "\n", encoding="utf-8")
    print(f"{len(staze)} slika, {len(sadrzaj)} bajtova")


if __name__ == "__main__":
    glavno()
