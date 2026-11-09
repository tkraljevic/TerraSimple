"""Samo nacrt za već postojeću oznaku; nikad ne objavljuje izdanje."""
import json
import os
from pathlib import Path
from izdanje import izvedi, inacica

def glavno():
    oznaka = os.environ["TERRASIMPLE_OZNAKA"]
    if oznaka != "v" + inacica():
        raise RuntimeError("Oznaka ne odgovara inačici u prozor/Cargo.toml.")
    # Popis razlikuje nepostojeći nacrt od greške mreže/autorizacije.
    izdanja = json.loads(izvedi(["gh", "api", "--paginate", "--slurp", "repos/{owner}/{repo}/releases"], 60))
    postojece = next((r for stranica in izdanja for r in stranica if r["tag_name"] == oznaka), None)
    if postojece and not postojece["draft"]:
        raise RuntimeError("Izdanje je već objavljeno; ovaj tijek ga ne smije mijenjati.")
    if not postojece:
        izvedi(["gh", "release", "create", oznaka, "--verify-tag", "--draft", "--prerelease",
                "--title", "TerraSimple " + inacica() + " alfa", "--notes", "Probno izdanje. Upute su u docs/izdanje.md."], 60)
    datoteke = sorted(p for p in Path("target/artefakti").iterdir() if p.is_file())
    if not datoteke:raise RuntimeError("Nema paketa za nacrt izdanja.")
    izvedi(["gh", "release", "upload", oznaka, *map(str, datoteke), "--clobber"], 180)
    stanje = json.loads(izvedi(["gh", "release", "view", oznaka, "--json", "isDraft"], 60))
    if not stanje["isDraft"]:raise RuntimeError("Izdanje nije nacrt.")

if __name__ == "__main__":glavno()
