# GeoPackage

Paket sprema `slojevi::Projekt` u jednu datoteku. Ne ovisi o karti ni sučelju.
Novi slojevi imaju WGS84 (EPSG:4326), a pročitani zadržavaju izvorni sustav.

```rust,no_run
# fn primjer() -> geopaket::Rezultat<()> {
let mut datoteka = geopaket::Datoteka::otvori("projekt.gpkg")?;
let projekt = datoteka.ucitaj()?;
let mut uredjivac = slojevi::Uredjivac::iz_projekta(projekt);
// Naredbe uređivača mijenjaju projekt; poništavanje također ide u sljedeći zapis.
datoteka.spremi(uredjivac.projekt())?;
datoteka.spremi_kao("kopija.gpkg", uredjivac.projekt())?;
# Ok(()) }
```

Za novi projekt dovoljno je `otvori` pa `spremi`. Postojeća datoteka mora se
najprije učitati. Kopiju uvezenog projekta radi `spremi_kao` na otvorenom izvoru:
SQLite backup uključuje i nepoznate tablice te zapise koji nisu prikazivi u modelu.
Odredište mora biti novo. Ne povezujte projekt s drugom otvorenom datotekom.

Promjena datoteke drugim programom nakon čitanja vraća sukob; ponovno učitajte
podatke i razriješite izmjene. Čekanje zaključane baze ograničeno je na 250 ms.
Priprema geometrije i popisa izmjena prethodi neposrednoj transakciji. Neizmijenjena
geometrija ne prolazi novi preračun, a naziv, opis i stil mogu se mijenjati zasebno.

Točke, linije i poligoni podržavaju Z. Nepoznat sustav prekida čitanje.
Pri dodavanju 2D objekta u sloj koji zahtijeva Z, ili obratno, deklaracija Z
postaje opcionalna. Izvorni objekti ostaju isti i visina se ne izmišlja.
Upis u sloj s obveznom M dimenzijom ili nepodržanom vrstom geometrije odbija se.
NULL, prazne geometrije, krivulje i drugi nepodržani zapisi ostaju u bazi i navode
se u `upozorenja`. Sloj s takvim zapisima ne može se obrisati kroz model.
Uređivanje krivulja i višedijelnih geometrija nije uvedeno.

Stil sloja ide u `layer_styles`, stil objekta u `stil_*`. Podržani su jednostavni
QGIS simboli SimpleMarker, SimpleLine i SimpleFill. Čitaju se boje, širine, oblik
i veličina prvog sloja simbola u rendereru `singleSymbol`; druga QGIS svojstva
ne prelaze u model. Druga vrsta renderera daje upozorenje. Izvorni QML ostaje
sačuvan dok se stil sloja ne promijeni; tada se piše naš jednostavni simbol.
Ne koriste se simboli odabira ni legende.
Izrazi `coalesce` za oblik znaka i širinu ruba imaju ugrađenu zamjensku vrijednost,
kao u probi. Ako se osnovni simbol ručno mijenja u QGIS-u, treba uskladiti i tu
vrijednost u izrazu; paket ne tumači proizvoljne QGIS izraze.

Pri obnovi obrisanog sloja čuvaju se izvorna shema, nepoznati stupci, fid, sustav,
korisnički indeksi i okidači. Izvedeni prostorni indeks tada se uklanja; obnovljeni
sloj može biti bez njega. Na postojećim slojevima okidači prostornog indeksa rade
preko registriranih funkcija `ST_*`.

## Provjere

Iz korijena radne kopije:

```sh
cargo test -p geopaket -p slojevi --locked
cargo xtask provjera
cargo build -p geopaket --example krug --locked
geopaket/provjera/pokreni.sh
CARGO_TARGET_DIR=target TMPDIR="$PWD/.quality/mutacije-rad" /usr/sbin/taskpolicy -b cargo xtask provjera-duboka
CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo build --release -p geopaket --example mjerenje --locked
/usr/bin/time -l /usr/sbin/taskpolicy -b target/release/examples/mjerenje .quality/novo-mjerenje.gpkg
```

QGIS skripta pokreće samo Python bez prozora. Zadani omot je za instalirani QGIS
LTR na macOS-u; `QGIS_APP` mijenja korijen instalacije. Rust testovi ne trebaju QGIS:
`tests/podaci/qgis.json` snimljen je iz izmišljenog primjera. Mjerenje memorije u
primjeru koristi macOS `ps`; to nije prijenosna provjera memorije za druge sustave.
Rezultati i ograničenja su u [REZULTAT.md](REZULTAT.md), a proširenje u
[PROSIRENJE.md](PROSIRENJE.md).
