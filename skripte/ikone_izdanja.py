"""Na Macu pretvara postojeće PNG ikone u ICNS i ICO; ne mijenja crtež."""
from pathlib import Path
import struct
import tempfile
from izdanje import KORIJEN, izvedi

def glavno():
    rad = KORIJEN / "target/ikone"; rad.mkdir(parents=True, exist_ok=True)
    izlaz = KORIJEN / "resursi/ikone/izdanje"; izlaz.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=rad) as d:
        skup = Path(d) / "TerraSimple.iconset"; skup.mkdir()
        for n in [16, 32, 128, 256, 512]:
            for skala in [1, 2]:
                velicina = n * skala
                ime = "terrasimple-32.png" if velicina <= 32 else "terrasimple-64.png" if velicina <= 64 else "terrasimple.png"
                izvor = KORIJEN / "resursi/ikone" / ime
                cilj = skup / (f"icon_{n}x{n}" + ("@2x" if skala == 2 else "") + ".png")
                izvedi(["sips", "-z", str(velicina), str(velicina), str(izvor), "--out", str(cilj)], 30)
        izvedi(["iconutil", "-c", "icns", str(skup), "-o", str(izlaz / "TerraSimple.icns")], 30)
        slike = [(32, (KORIJEN / "resursi/ikone/terrasimple-32.png").read_bytes()),
                 (64, (KORIJEN / "resursi/ikone/terrasimple-64.png").read_bytes()),
                 (256, (skup / "icon_256x256.png").read_bytes())]
        zaglavlje = struct.pack("<HHH", 0, 1, len(slike)); pomak = 6 + 16 * len(slike)
        for n, b in slike:
            zaglavlje += struct.pack("<BBBBHHII", n % 256, n % 256, 0, 0, 1, 32, len(b), pomak)
            pomak += len(b)
        (izlaz / "TerraSimple.ico").write_bytes(zaglavlje + b"".join(b for n, b in slike))

if __name__ == "__main__": glavno()
