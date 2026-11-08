"""Namjerno uklanja prijenos Z, provjerava pad testa i uvijek vraća izvorne bajtove."""
import hashlib
import json
import os
import pathlib
import subprocess

korijen = pathlib.Path(__file__).resolve().parents[2]
izvor = korijen / 'geopaket/src/geometrija.rs'
zapis = korijen / '.quality/namjerno-pokvaren-kod.log'
prije = izvor.read_bytes()
uzorak = b'visina: z.filter(|v| v.is_finite()),'
assert prije.count(uzorak) == 1
naredba = ['/usr/sbin/taskpolicy', '-b', 'cargo', 'test', '-p', 'geopaket', '--lib', '--locked', '--', '--exact', 'geometrija::testovi::svi_oblici_i_visine_prolaze_obje_dimezije']
okolje = dict(os.environ, CARGO_TARGET_DIR=str(korijen / 'target'))
try:
    izvor.write_bytes(prije.replace(uzorak, b'visina: None,'))
    pad = subprocess.run(naredba, cwd=korijen, env=okolje, capture_output=True, text=True)
    zapis.write_text(pad.stdout + pad.stderr)
finally:
    izvor.write_bytes(prije)
obnovljen = izvor.read_bytes() == prije
assert pad.returncode == 101 and '1 failed' in pad.stdout and 'assertion' in pad.stdout, 'Nije dokazan pad tvrdnje testa.'
prolaz = subprocess.run(naredba, cwd=korijen, env=okolje, capture_output=True, text=True)
(korijen / '.quality/obnovljen-kod.log').write_text(prolaz.stdout + prolaz.stderr)
assert prolaz.returncode == 0 and '1 passed' in prolaz.stdout
rezultat = {
    'provjera': 'Gubitak visine u stvarnom čitaču geometrije',
    'datoteka': 'geopaket/src/geometrija.rs',
    'promjena': 'Prijenos visine zamijenjen je s None.',
    'izlazni_kod': pad.returncode,
    'test_je_pao': True,
    'izvorni_kod_vracen': obnovljen,
    'obnovljeni_test_prosao': True,
    'sha256_obnovljenog_koda': hashlib.sha256(prije).hexdigest(),
}
(pathlib.Path(__file__).parent / 'dokaz-pada.json').write_text(json.dumps(rezultat, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(rezultat, ensure_ascii=False))
