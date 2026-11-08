"""U zasebnoj kopiji pravog čitača vraća petlju, dokazuje zastoj i vraća bajtove."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import signal
import subprocess
import tempfile

korijen = Path(__file__).resolve().parent.parent
(korijen / '.quality').mkdir(exist_ok=True)
kopija = Path(tempfile.mkdtemp(prefix='dokaz-fuzza-', dir=korijen / '.quality'))
for paket in ['geopaket', 'slojevi', 'koordinate', 'geometrija']:
    shutil.copytree(korijen / paket, kopija / paket)
(kopija / 'Cargo.toml').write_text('[workspace]\nmembers=["geopaket","slojevi","koordinate","geometrija"]\nresolver="3"\n')
shutil.copy(korijen / 'Cargo.lock', kopija / 'Cargo.lock')
(kopija / 'fuzz').mkdir()
for ime in ['Cargo.toml', 'Cargo.lock']:
    shutil.copy(korijen / 'fuzz' / ime, kopija / 'fuzz' / ime)
shutil.copytree(korijen / 'fuzz/fuzz_targets', kopija / 'fuzz/fuzz_targets')
shutil.copytree(korijen / 'fuzz/pocetni/qml', kopija / 'fuzz/pocetni/qml')
(kopija / 'fuzz/corpus/qml').mkdir(parents=True)
(kopija / 'fuzz/artifacts/qml').mkdir(parents=True)
izvor = kopija / 'geopaket/src/stil.rs'
prije = izvor.read_bytes()
pravi = (korijen / 'geopaket/src/stil.rs').read_bytes()
assert prije == pravi
uzorak = b'Event::Eof => Ok(false),'
assert prije.count(uzorak) == 1
osnova = ['/usr/sbin/taskpolicy', '-b', 'cargo', '+nightly', 'fuzz']
naredba = osnova + ['run', 'qml', 'fuzz/corpus/qml', 'fuzz/pocetni/qml', '--', '-max_total_time=10', '-timeout=2', '-rss_limit_mb=1024', '-max_len=262144', '-print_final_stats=1']
okolje = dict(os.environ, CARGO_TARGET_DIR=str(korijen / 'target/fuzz-dokaz'), RUSTUP_AUTO_INSTALL='0')

def izvedi(args, ime, granica):
    with (kopija / ime).open('w') as log:
        p = subprocess.Popen(args, cwd=kopija, env=okolje, stdout=log,
                             stderr=subprocess.STDOUT, start_new_session=True)
        try:
            return p.wait(timeout=granica)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGTERM)
            try:
                p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL)
                p.wait()
            raise

# Hladna gradnja ovisnosti nije dio vremenskog ograničenja fuzz pokusa.
assert izvedi(osnova + ['build', 'qml'], 'gradnja.log', 3600) == 0
try:
    izvor.write_bytes(prije.replace(uzorak, b'Event::Eof => Ok(true),'))
    pad = izvedi(naredba, 'pad.log', 600)
finally:
    izvor.write_bytes(prije)
assert izvor.read_bytes() == prije and (korijen / 'geopaket/src/stil.rs').read_bytes() == pravi
assert pad != 0 and 'ERROR: libFuzzer: timeout' in (kopija / 'pad.log').read_text()
prolaz = izvedi(naredba, 'obnovljeno.log', 600)
assert prolaz == 0 and 'stat::number_of_executed_units:' in (kopija / 'obnovljeno.log').read_text()
rezultat = {
    'promjena': 'U kopiji pravog QML čitača Event::Eof vraća true umjesto false.',
    'uhvacen_zastoj': True, 'kod_pada': pad, 'kod_obnovljenog': prolaz,
    'kopija_vracena_bajt_za_bajt': True, 'radni_izvor_nije_mijenjan': True,
    'sha256_izvora': hashlib.sha256(pravi).hexdigest(),
    'zapisi': str(kopija.relative_to(korijen)),
}
(korijen / 'fuzz/dokaz.json').write_text(json.dumps(rezultat, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(rezultat, ensure_ascii=False))
