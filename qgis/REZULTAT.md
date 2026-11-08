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
