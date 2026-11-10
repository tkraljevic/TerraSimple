"""Gradnja, pakiranje i provjera gotovog programa; svi procesi imaju rok."""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import zipfile

KORIJEN = Path(__file__).resolve().parent.parent

def izvedi(naredba, sekunde=120, pozadina=False, okolina=None):
    if pozadina and platform.system() == "Darwin":
        naredba = ["/usr/sbin/taskpolicy", "-b"] + naredba
    okolina = dict(os.environ if okolina is None else okolina)
    okolina["CARGO_TARGET_DIR"] = str(KORIJEN / "target")
    p = subprocess.Popen(naredba, cwd=KORIJEN, env=okolina, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT, start_new_session=(os.name != "nt"))
    try:
        izlaz, _ = p.communicate(timeout=sekunde)
    except subprocess.TimeoutExpired:
        if os.name == "nt":
            subprocess.run(["taskkill", "/PID", str(p.pid), "/T", "/F"], timeout=15, check=False)
        else:
            os.killpg(p.pid, signal.SIGKILL)
        izlaz, _ = p.communicate(timeout=15)
        raise RuntimeError("Isteklo vrijeme procesa: " + " ".join(naredba))
    tekst = izlaz.decode("utf-8", errors="replace")
    print(tekst, end="", flush=True)
    if p.returncode:
        raise RuntimeError(f"Proces je završio kodom {p.returncode}: {naredba[0]}")
    return tekst

def inacica():
    tekst = (KORIJEN / "prozor/Cargo.toml").read_text(encoding="utf-8")
    return re.search(r'^version = "([^"]+)"', tekst, re.M).group(1)

def licence(mapa):
    mapa.mkdir(parents=True, exist_ok=True)
    for izvor, ime in [("LICENSE", "LICENSE"), ("LICENSE_hr.txt", "LICENSE_hr.txt"),
                        ("resursi/fontovi/LICENCA-OFL.txt", "LICENCA-INTER-OFL.txt"),
                        ("resursi/ikone/sucelje/LICENCA-LUCIDE.txt", "LICENCA-LUCIDE.txt"),
                        ("skripte/IZVORI-KARTE.txt", "IZVORI-KARTE.txt")]:
        shutil.copy2(KORIJEN / izvor, mapa / ime)
    # Jedan popis dopuštenih licenci: ista pravila kao u redovnoj provjeri.
    pravila = (KORIJEN / "quality/deny.toml").read_text(encoding="utf-8")
    dopustene = re.search(r'^allow = (\[[^\]]+\])', pravila, re.M).group(1)
    postavke = mapa / "about.toml"
    postavke.write_text("accepted = " + dopustene + "\nignore-dev-dependencies = true\n", encoding="utf-8")
    izvedi(["cargo", "deny", "--manifest-path", "prozor/Cargo.toml", "--config", "quality/deny.toml", "--locked", "check"], 600)
    izvedi(["cargo", "about", "generate", "--locked", "--fail", "--manifest-path", "prozor/Cargo.toml",
            "--config", str(postavke), "--output-file", str(mapa / "LICENCE-BIBLIOTEKA.html"),
            "skripte/licence.hbs"], 900)
    postavke.unlink()

def gradi(cilj):
    pocetak = time.monotonic()
    okolina = dict(os.environ)
    if cilj == "x86_64-pc-windows-msvc":
        # Prijenosni EXE ne smije tražiti zasebnu instalaciju MSVC okruženja.
        okolina["RUSTFLAGS"] = (okolina.get("RUSTFLAGS", "") + " -C target-feature=+crt-static").strip()
    izvedi(["cargo", "build", "--release", "--locked", "-p", "prozor", "--target", cilj], 7200, True, okolina)
    nastavak = ".exe" if "windows" in cilj else ""
    p = KORIJEN / "target" / cilj / "release" / ("prozor" + nastavak)
    return p, {"cilj": cilj, "sekunde": round(time.monotonic() - pocetak, 3), "bajtova": p.stat().st_size}

def mac(mapa, ulazi, verzija):
    app = mapa / "TerraSimple.app/Contents"
    (app / "MacOS").mkdir(parents=True)
    (app / "Resources").mkdir()
    exe = app / "MacOS/TerraSimple"
    izvedi(["lipo", "-create", *map(str, ulazi), "-output", str(exe)])
    izvedi(["lipo", "-info", str(exe)])
    if len(ulazi) == 2:
        # Datoteka ide prije -verify_arch; obrnuto lipo javlja grešku na svakom Macu.
        izvedi(["lipo", str(exe), "-verify_arch", "arm64", "x86_64"])
    shutil.copy2(KORIJEN / "resursi/ikone/izdanje/TerraSimple.icns", app / "Resources/TerraSimple.icns")
    podaci = {"CFBundleName": "TerraSimple", "CFBundleDisplayName": "TerraSimple",
              "CFBundleExecutable": "TerraSimple", "CFBundlePackageType": "APPL",
              "CFBundleIdentifier": os.environ.get("TERRASIMPLE_BUNDLE_ID", "org.terrasimple.TerraSimple"),
              "CFBundleShortVersionString": verzija, "CFBundleVersion": verzija,
              "CFBundleIconFile": "TerraSimple.icns", "NSHighResolutionCapable": True}
    (app / "Info.plist").write_bytes(plistlib.dumps(podaci))
    licence(app / "Resources/Dozvole")
    # Potpis samo izvršne datoteke ne zapečati ostatak paketa: `codesign --verify --strict` tada
    # javlja grešku, a macOS takav preuzeti program može proglasiti oštećenim. Potpis bez
    # izdavača (ad hoc) to ispravlja, ali ne zamjenjuje potvrdu izdavača ni Appleovu ovjeru.
    izvedi(["codesign", "--force", "--sign", "-", str(app.parent)], 120)
    izvedi(["codesign", "--verify", "--strict", str(app.parent)], 60)
    return exe

def prijenosni(mapa, program, sustav):
    exe = mapa / ("TerraSimple.exe" if sustav == "windows" else "TerraSimple")
    shutil.copy2(program, exe)
    licence(mapa / "Dozvole")
    if sustav == "linux":
        shutil.copy2(KORIJEN / "resursi/ikone/terrasimple.png", mapa / "terrasimple.png")
        shutil.copy2(KORIJEN / "skripte/TerraSimple.desktop", mapa / "TerraSimple.desktop")
        shutil.copy2(KORIJEN / "skripte/pokreni.sh", mapa / "pokreni.sh")
        (mapa / "pokreni.sh").chmod(0o755)
    return exe

@contextlib.contextmanager
def bez_izvora(korijen=KORIJEN):
    """Za vrijeme provjere mapa resursi iz izvornog koda nije na svom mjestu. Program koji bi
    tražio datoteke po stazi s računala na kojem je izgrađen tako pada već ovdje, a ne tek kod
    korisnika (proba 0.0.1 na Windowsu, 10. 11. 2026.)."""
    izvor, skriveno = korijen / "resursi", korijen / "resursi-skriveno-za-provjeru"
    izvor.rename(skriveno)
    try:
        yield
    finally:
        skriveno.rename(izvor)

def provjeri(exe, sustav):
    naredba = [str(exe), "--provjeri-pokretanje"]
    if sustav == "linux":
        naredba = ["xvfb-run", "-a"] + naredba
    tekst = izvedi(naredba, 25)
    if "Provjera prvog okvira karte: true" not in tekst:
        raise RuntimeError("Nema potvrde iscrtanog okvira u gotovom programu.")

def zapakiraj(mapa, cilj, sustav):
    if sustav == "linux":
        with tarfile.open(cilj, "w:gz") as arhiva:
            arhiva.add(mapa, arcname=mapa.name)
    elif sustav == "macos":
        izvedi(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(mapa / "TerraSimple.app"), str(cilj)], 180)
    else:
        with zipfile.ZipFile(cilj, "w", zipfile.ZIP_DEFLATED) as arhiva:
            for p in sorted(mapa.rglob("*")):
                if p.is_file():arhiva.write(p, p.relative_to(mapa.parent))

def glavno():
    parser = argparse.ArgumentParser()
    parser.add_argument("sustav", choices=["macos", "windows", "linux"])
    parser.add_argument("--bez-gradnje", action="store_true")
    parser.add_argument("--samo-arm", action="store_true", help="Samo lokalna ograničena provjera; nije univerzalni paket.")
    a = parser.parse_args()
    if a.samo_arm and a.sustav != "macos":parser.error("Samo ARM vrijedi za macOS.")
    ciljevi = {"macos": ["aarch64-apple-darwin", "x86_64-apple-darwin"],
               "windows": ["x86_64-pc-windows-msvc"], "linux": ["x86_64-unknown-linux-gnu"]}[a.sustav]
    if a.samo_arm:ciljevi = ciljevi[:1]
    ulazi, mjerenja = [], []
    for cilj in ciljevi:
        if a.bez_gradnje:
            nastavak = ".exe" if a.sustav == "windows" else ""
            p = KORIJEN / "target" / cilj / "release" / ("prozor" + nastavak)
        else:
            p, m = gradi(cilj); mjerenja.append(m)
        ulazi.append(p)
    oznaka = "arm64-proba" if a.samo_arm else "universal" if a.sustav == "macos" else "x86_64"
    naziv = f"TerraSimple-{inacica()}-{a.sustav}-{oznaka}"
    rad = KORIJEN / "target/izdanje"
    mapa = rad / naziv
    if mapa.exists():raise RuntimeError("Mapa paketa već postoji; odaberite novu praznu radnu kopiju izlaza: " + naziv)
    mapa.mkdir(parents=True)
    exe = mac(mapa, ulazi, inacica()) if a.sustav == "macos" else prijenosni(mapa, ulazi[0], a.sustav)
    with bez_izvora():
        provjeri(exe, a.sustav)
    nastavak = ".tar.gz" if a.sustav == "linux" else ".zip"
    arhiva = rad / (naziv + nastavak)
    zapakiraj(mapa, arhiva, a.sustav)
    if a.sustav == "macos":
        raspakirano = rad / (naziv + "-raspakirano")
        izvedi(["ditto", "-x", "-k", str(arhiva), str(raspakirano)], 180)
        with bez_izvora():
            provjeri(raspakirano / "TerraSimple.app/Contents/MacOS/TerraSimple", a.sustav)
    rezultat = {"inacica": inacica(), "sustav": a.sustav, "gradnje": mjerenja,
                "paket": arhiva.name, "bajtova": arhiva.stat().st_size,
                "sha256": hashlib.sha256(arhiva.read_bytes()).hexdigest(), "provjera_pokretanja": True}
    (rad / (naziv + ".json")).write_text(json.dumps(rezultat, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(rezultat, ensure_ascii=False))

if __name__ == "__main__":
    # Konzola Windowsa piše u cp1252 i ne zna okvirne crte iz ispisa cargo-denyja.
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    glavno()
