# Rezultat paketa QGIS

Paket `qgis` pronalazi program na macOS-u, Windowsu i Linuxu, brojčano uspoređuje
inačice i daje prednost korisnikovoj stazi. Priprema naredbu za jednu `.gpkg`
datoteku, a pokretanje vraća proces bez čekanja zatvaranja QGIS-a. Nedostajući
program ima poruku s uputom za instalaciju ili odabir staze. Gumb nije spojen.

Pokretanje provjera iz korijena radne kopije:

```sh
CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo test -p qgis --locked
CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo run -p qgis --example pregled --locked
```

Sedam testova prošlo je na macOS-u. Koriste izmišljene mape za sva tri sustava:
usporedbu inačica, prvenstvo zadane staze, XML i binarni plist, OSGeo4W popis,
nepotpune instalacije, pogreške i točne argumente s razmacima i posebnim znakovima.
Izlazi upita inačice su zadani u testu. Nijedan test ne otvara pravi QGIS.

Stvarni pregled samo čitanjem mapa i metapodataka pronašao je QGIS-LTR 3.44.15.
Zapis je u `.quality/qgis-pregled.log`. Upit `--version` na pravom programu nije
pokrenut. Windows, Linux, pravo otvaranje datoteke i ponašanje stvarnog procesa
nisu provjereni; to treba isprobati pri spajanju gumba. Testovi modeliraju te
sustave na macOS-u i nisu zamjena za izvođenje na njima.

Ako se nađe više programa s nečitljivim inačicama, traži se zadana staza. Nema
nove odluke potrebne za ovaj paket. Pamćenje staze ostaje izvan njegova opsega.

`cargo xtask provjera` prošla je: oblikovanje, Clippy, testovi, pokrivenost,
složenost i ovisnosti. Paket QGIS ima 26 funkcija, 97,49 % pokrivenosti i najveću
složenost 6. Nije označen kritičnim paketom. Referenca nije mijenjana.
Završni zapis je `.quality/provjera-zavrsna.log`, mjerenje u
`.quality/61773-1794142432322351000/mjerenje.json`.

## Pregled pri spajanju (8. 11. 2026.)

Naredba više ne stavlja `--` ispred datoteke, jer ga `open` na macOS-u ne navodi u uputama, a staza je
uvijek apsolutna pa se ne može zamijeniti s opcijom. Na Windowsu se skida prefiks `\\?\` koji daje
`canonicalize`. Dodani su testovi za nečitljivu mapu programa i za zadanu stazu na Macu i Linuxu, jer su
te namjerne greške prošle. Od 81 namjerne greške u paketu i naredbi za fuzz sve logičke sad su uhvaćene.

## QGIS projekt

Paket `qgis` proširen je funkcijama `naredba_projekt(&self, datoteka: &Path, projekt: &str) -> Result<Naredba, Greska>`
i `otvori_projekt(&self, datoteka: &Path, projekt: &str) -> Result<Child, Greska>`.
Postojeća funkcija `naredba` za otvaranje same `.gpkg` datoteke ostala je nepromijenjena
i u potpunosti podržana.

Ponašanje na operacijskim sustavima i URI identifikator:
- Za otvaranje projekta unutar GeoPackage spremnika generira se URI:
  `geopackage:<apsolutna_staza>?projectName=<projekt>`.
- Na macOS-u s paketom aplikacije (`.app` svežanj), koristi se `/usr/bin/open`:
  `/usr/bin/open -n -a <app> --args --project geopackage:<staza>?projectName=<projekt>`.
  Zastavica `-n` (`--new`) osigurava pokretanje novog procesa QGIS-a i prosljeđivanje
  argumenata čak i ako je QGIS već pokrenut. Bez zastavice `-n`, operacijski sustav bi
  samo aktivirao postojeći prozor i zanemario proslijeđene `--args` argumente.
- Na Linuxu i Windowsu (te na macOS-u ako se koristi izravna izvršna binarna datoteka
  izvan svežnja), pokreće se izvršna datoteka s opcijom:
  `<program> --project geopackage:<staza>?projectName=<projekt>`.
- Na Windowsu se uklanja prefiks `\\?\` kojeg vraća `canonicalize`, a obrnute kose
  crte (`\`) pretvaraju se u obične kose crte (`/`) radi valjanosti URI specifikacije.
- Ako je naziv projekta prazan ili ako datoteka na disku ne postoji, funkcija odmah
  vraća grešku (`Greska::NeispravanNazivProjekta` odnosno `Greska::DatotekaNePostoji`).

Testiranje i mutacije:
- Dodani su jedinični testovi u `qgis/src/testovi.rs`:
  - `naredba_projekt_argumenti`: provjera ispravnosti argumenata za Mac svežanj i običnu binarnu datoteku.
  - `naredba_projekt_prazan_naziv_i_nepostojeca_datoteka`: odbijanje nevaljanih parametara.
  - `naredba_projekt_windows_staza`: provjera čišćenja `\\?\` prefiksa i normalizacije kosih crta.
- `cargo mutants` pokrenut nad `qgis/src/lib.rs` testirao je 19 mutacija:
  10 mutacija je uhvaćeno testovima, a 9 je bilo neprevodivo. Niti jedna mutacija
  nije preživjela (0 missed mutants).

Pregled pri spajanju (8. 11. 2026.): gumb „Otvori u QGIS-u” sad poziva `otvori_projekt` s projektom
`TerraSimple`. Zbog `-n` svaki klik na Macu pokreće novi QGIS, i kad je jedan već otvoren; to traži odluku.
Staza sa znakom `?` ili `#` u imenu mape nije isprobana u URI-ju projekta. Pravo otvaranje s prozorom,
Windows i Linux nisu provjereni.
