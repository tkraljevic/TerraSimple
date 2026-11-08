# Otvaranje u QGIS-u

Paket nema ovisnost o sučelju. `pronadji` daje prednost korisnikovoj stazi;
inače traži macOS pakete QGIS, Windows instalacije u Program Files i OSGeo4W
te program `qgis` u svim mapama PATH-a na Linuxu. Brojevi inačica uspoređuju se
brojčano. Duplikati iste datoteke se uklanjaju.

macOS inačica čita se iz XML ili binarnog `Info.plist`. Windows inačica čita se
iz OSGeo4W popisa instaliranih paketa ili naziva instalacijske mape. Kad nema
metapodataka, odvojeni upit `--version` može dohvatiti inačicu. Ako više programa
nije moguće usporediti, korisnik treba zadati stazu; ne nagađa se koji je noviji.
Zadana staza može biti macOS `.app` ili izravna izvršna datoteka. Pamćenje
postavke i gumb u prozoru nisu dio ovog paketa.

```rust,no_run
# fn primjer() -> Result<(), qgis::Greska> {
let qgis = qgis::pronadji(None)?;
let naredba = qgis.naredba(std::path::Path::new("projekt.gpkg"))?;
// Do ovog mjesta datoteka još nije otvorena.
let mut proces = naredba.pokreni()?;
// Pozivatelj poslije može provjeriti proces bez blokiranja.
let stanje = proces.try_wait()?;
# Ok(()) }
```

`Qgis::otvori` spaja pripremu i pokretanje. Vraća `Child` odmah nakon pokretanja,
bez čekanja zatvaranja QGIS-a. Pozivatelj treba kasnije preuzeti stanje procesa.
`naredba` provjerava datoteku i pretvara je u apsolutnu stazu, čuvajući razmake
i posebne znakove u jednom argumentu. macOS koristi otvaranje dokumenta kroz
`open -a`, a ostali sustavi izravni izvršni program s odvajanjem argumenata `--`.
Na Windowsu se koristi `qgis-bin.exe` ili `qgis-ltr-bin.exe`, kao i instalirani
prečaci; ne sastavlja se naredba za ljusku.

`pronadji_u` prima mape pretraživanja i zamjenski upit inačice. Testovi tako
koriste izmišljene instalacije i unaprijed zadane odgovore, bez stvarnog QGIS-a.
Primjer `cargo run -p qgis --example pregled` čita samo mape i metapodatke;
ne izvodi ni upit `--version` ni otvaranje datoteke.

Osnova za naredbu je [QGIS-ovo čitanje argumenata](https://github.com/qgis/QGIS/blob/master/src/app/main.cpp),
a Windows izvršni program prati [OSGeo4W instalacijske prečace](https://github.com/jef-n/OSGeo4W/blob/master/src/qgis/osgeo4w/postinstall-desktop.bat).
Stvarno provjereno i ograničenja navedeni su u [REZULTAT.md](REZULTAT.md).
