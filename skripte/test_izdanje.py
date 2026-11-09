"""Provjere roka procesa i sadržaja prijenosnog paketa, bez prozora."""
import sys
import json
import os
import tempfile
import unittest
from pathlib import Path
import tarfile
import zipfile
from unittest.mock import patch
import izdanje
import nacrt_izdanja

class Provjere(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (izdanje.KORIJEN / ".quality").mkdir(exist_ok=True)

    def test_proces_ne_moze_prikriti_pad_ili_istek(self):
        self.assertIn("uspjeh", izdanje.izvedi([sys.executable, "-c", "print('uspjeh')"], 10))
        with self.assertRaisesRegex(RuntimeError, "kodom 7"):
            izdanje.izvedi([sys.executable, "-c", "raise SystemExit(7)"], 10)
        with self.assertRaisesRegex(RuntimeError, "Isteklo vrijeme"):
            izdanje.izvedi([sys.executable, "-c", "import time; time.sleep(60)"], 0.1)

    def test_arhive_cuvaju_program_ikonu_i_dozvole(self):
        with tempfile.TemporaryDirectory(dir=izdanje.KORIJEN / ".quality") as d:
            korijen=Path(d)
            mapa=korijen / "TerraSimple-proba";mapa.mkdir()
            program=korijen / "program";program.write_bytes(b"izmisljeno");program.chmod(0o755)
            def dozvole(p):
                p.mkdir();(p/"LICENSE").write_text("izmisljeno")
            with patch.object(izdanje,"licence",dozvole):
                exe=izdanje.prijenosni(mapa,program,"linux")
            self.assertEqual(exe.read_bytes(),b"izmisljeno")
            self.assertTrue((mapa/"terrasimple.png").is_file())
            self.assertTrue((mapa/"TerraSimple.desktop").is_file())
            tar=korijen/"program.tar.gz";izdanje.zapakiraj(mapa,tar,"linux")
            with tarfile.open(tar) as a:
                if os.name != "nt":
                    self.assertEqual(a.getmember(mapa.name+"/TerraSimple").mode & 0o111,0o111)
                self.assertIn(mapa.name+"/Dozvole/LICENSE",a.getnames())
            win = korijen / "TerraSimple-windows"; win.mkdir()
            with patch.object(izdanje, "licence", dozvole):
                izdanje.prijenosni(win, program, "windows")
            zip=korijen/"program.zip";izdanje.zapakiraj(win,zip,"windows")
            with zipfile.ZipFile(zip) as a:
                self.assertIn(win.name+"/Dozvole/LICENSE",a.namelist())
                self.assertEqual(a.read(win.name+"/TerraSimple.exe"), b"izmisljeno")

    def test_nema_lazne_potvrde_pokretanja(self):
        with patch.object(izdanje,"izvedi",return_value=""):
            with self.assertRaisesRegex(RuntimeError,"Nema potvrde"):
                izdanje.provjeri(Path("program"),"windows")
        with patch.object(izdanje,"izvedi",return_value="Provjera prvog okvira karte: true"):
            izdanje.provjeri(Path("program"),"windows")

class Nacrt(unittest.TestCase):
    def test_pogresna_oznaka_ne_dira_izdanja(self):
        with patch.dict(os.environ, {"TERRASIMPLE_OZNAKA": "v999"}), patch.object(nacrt_izdanja, "izvedi") as poziv:
            with self.assertRaisesRegex(RuntimeError, "Oznaka"):
                nacrt_izdanja.glavno()
            poziv.assert_not_called()

    def test_objavljeno_izdanje_ne_smije_se_mijenjati(self):
        oznaka = "v" + izdanje.inacica()
        odgovor = json.dumps([[{"tag_name": oznaka, "draft": False}]])
        with patch.dict(os.environ, {"TERRASIMPLE_OZNAKA": oznaka}), patch.object(nacrt_izdanja, "izvedi", return_value=odgovor) as poziv:
            with self.assertRaisesRegex(RuntimeError, "već objavljeno"):
                nacrt_izdanja.glavno()
            self.assertEqual(poziv.call_count, 1)

    def test_stvara_samo_nacrt_i_provjerava_ishod(self):
        oznaka = "v" + izdanje.inacica()
        for vec_postoji in [False, True]:
            odgovor = [[{"tag_name": oznaka, "draft": True}]] if vec_postoji else [[]]
            pozivi = []
            def gh(naredba, rok):
                pozivi.append(naredba)
                self.assertGreater(rok, 0)
                if naredba[1] == "api": return json.dumps(odgovor)
                if naredba[2] == "view": return '{"isDraft":true}'
                return ""
            with patch.dict(os.environ, {"TERRASIMPLE_OZNAKA": oznaka}), patch.object(nacrt_izdanja, "izvedi", side_effect=gh), patch.object(nacrt_izdanja.Path, "iterdir", return_value=iter([Path(__file__)])):
                nacrt_izdanja.glavno()
            stvaranje = [p for p in pozivi if p[1:3] == ["release", "create"]]
            self.assertEqual(len(stvaranje), 0 if vec_postoji else 1)
            if stvaranje:
                self.assertIn("--draft", stvaranje[0])
                self.assertIn("--verify-tag", stvaranje[0])
            self.assertFalse(any("publish" in p or "edit" in p for p in pozivi))

if __name__ == "__main__":unittest.main()
