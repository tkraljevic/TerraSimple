# Rezultat paketa geopaket

Redovna provjera i razmjena s QGIS-om prolaze. Puna duboka provjera završila je
s nalazima; naknadna ciljana mjerenja navedena su odvojeno. Nema dodanih iznimaka
ni promjene reference.

Paket sprema i čita projekt, mape, redoslijed, stanja slojeva, nazive, opise,
stilove i visine. Slojevi su obične GeoPackage tablice, a stablo je registrirano
proširenje opisano u `PROSIRENJE.md`. Novi slojevi su EPSG:4326; strani zadržavaju
sustav, uključujući QGIS-ov zapis 100000. Nepoznat sustav vraća grešku.

Neizmijenjene geometrije ostaju bajt za bajt iste. Nepoznati stupci i nepovezane
tablice ostaju sačuvani; `spremi_kao` koristi SQLite backup, uključujući WAL.
Sukob s drugim piscem prekida spremanje. Priprema je izvan transakcije, a upis
je atomski. Testovi stvarnih SQLite grešaka i izlaska podprocesa usred upisa
potvrdili su povrat starog projekta i `integrity_check = ok`.

## Dodano u slojevi

Novi `zapis.rs` sadrži `Zapisi`, `IzvorSloja`, `ZapisObjekta`, `Vrijednost` i
`IzmjeneSpremanja`: izvorni sustav, shemu, fid, nepoznate atribute i dijeljene
sirove geometrije. Dodani su `Projekt::iz_zapisa`, pristup izvornim zapisima,
`izmjene_od` i `Uredjivac::iz_projekta`. Uređivač nastavlja brojeve iznad najvećeg
poznatog broja, s praznom poviješću. Postojeće naredbe nisu mijenjane i njihovi
testovi prolaze. Spremanje nakon poništavanja pokriva izmjene, brisanje,
premještanje, obnovu sloja, ugniježđene mape i ručni redoslijed.

## Stvarno provjereno

- `cargo xtask provjera`: rustfmt, Clippy, 153 testa, LLVM pokrivenost, složenost,
  rizik i cargo-deny prolaze. Ukupno 359 funkcija i 96,90 % područja. GeoPaket ima
  186 funkcija, najveći CC i CRAP 7 te najnižu pokrivenost funkcije 95,24 %.
  Jedan pomoćni test prekida proces pa ga pokreće roditeljski test.
- QGIS 3.44.15 i GDAL 3.13.3, bez prozora: svih 50 provjera prolazi, službeni
  validator prošao je sedam poziva bez primjedbi. Krug uključuje naš zapis,
  QGIS izmjenu vrha, naziva, stila i tuđeg stupca, naš ponovni zapis i ponovno
  otvaranje u QGIS-u. Provjereni su strani EPSG:3765, zapis 100000 i CompoundCurve.
- Testovi bez QGIS-a čitaju izmišljenu snimku `tests/podaci/qgis.json`. Pokrivaju
  ST funkcije i okidače, Z, NULL, POINT EMPTY, više QML simbola, prednost
  `renderer-v2`, `coalesce` za oblik i rub te skrivene pomoćne tablice.
- `provjera/dokazi.py` uklonio je prijenos Z iz pravog čitača: tvrdnja testa pala
  je, kod je vraćen bajt za bajt i test ponovno prolazi. `dokaz-pada.json`
  sadrži rezultat; završna provjera potvrdila je isti sažetak izvornog koda.

Puna `cargo xtask provjera-duboka` stvarno je izmjerila sva tri kritična paketa:

| Paket | Mutacije | Uhvaćene | Neprevodive | Preživjele | Istek vremena |
|---|---:|---:|---:|---:|---:|
| geopaket | 531 | 420 | 104 | 5 | 2 |
| koordinate | 45 | 40 | 5 | 0 | 0 |
| geometrija | 37 | 35 | 2 | 0 | 0 |

Nakon tog mjerenja dodani su testovi, bez promjene produkcijskog koda.
Ciljano ponavljanje svih šest mutacija uvjeta stabla, uz cijelu zbirku testova
paketa, uhvatilo je svih šest.
Izravan test završetka XML-a uhvatio je obje mutacije koje su u cijeloj zbirci
uzrokovale petlju i istek vremena. To je mjerenje samo tog testa, a ne ponovljena
cijela zbirka. Ranije ciljno mjerenje također je uhvatilo mutaciju povijesti fid-a.
Dvije preživjele zamjene `|` u `^` nad disjunktnim SQLite zastavicama daju isti
rezultat; eventualna iznimka traži pregled. Puna naredba nije ponovno označena
prolaznom. Prvo mjerenje duplikacije bilo je 0 od 6930 redaka; ono prethodi zadnjim
dopunama testova. Sažeti nalazi i dopune su u `provjera/rezultati.json`.

## Mjerenje na izmišljenim podacima

87 105 linija, 3 048 675 vrhova sa Z i 208 slojeva; release, macOS arm64,
`taskpolicy -b`, jedno pokretanje uz druge poslove na računalu.

| Mjera | Rezultat |
|---|---:|
| Spremanje cijelog projekta | 10,704 s |
| Početna transakcija | 6,536 s |
| Čitanje | 4,588 s |
| Promjena opisa jednog objekta, cijeli poziv | 1,082 s |
| Transakcija jednog upisa | 4,068 ms |
| RSS nakon izrade / spremanja / čitanja | 159,60 / 554,58 / 1008,47 MB |
| Najveći RSS procesa | 1022,69 MB |

Transakcija obuhvaća SQL upise i COMMIT nakon BEGIN IMMEDIATE i provjere verzije;
ne uključuje čekanje zaključavanja. Početni atomski upis drži bazu sekundama.
MB znači milijun bajtova. Brojke nisu obećanje brzine na drugom računalu.

## Popravci, granice i odluke

Popravljeni su identitet i prvo zapisivanje stabla stranog sloja. Dodavanje 2D
objekta u Z sloj ili obratno mijenja deklaraciju Z u opcionalnu; izvorni bajtovi
ostaju isti i visina se ne izmišlja. Prije popravka taj je slučaj stvarno odbio
GDAL validator; nakon popravka oba smjera prolaze.

Krivulje se prijavljuju i čuvaju. Njihovo uređivanje traži odluku. Brisanje sloja
s nepodržanim ili praznim zapisima dopušteno je uz upozorenje u sučelju (Odluke
(8. 11. 2026.)), a upis u sloj s obveznom M dimenzijom odbija
se. Obnova obrisanog sloja vraća shemu, fid, sustav, korisničke indekse i okidače,
a izvedeni prostorni indeks uklanja. Podržana QML svojstva i ograničenje zamjenskih
vrijednosti `coalesce` opisani su u README-u. Proizvoljna QGIS simbologija nije
modelirana. QGIS je pri zasebnom pokusu POINT EMPTY zapisao neusklađenu zastavicu
praznine koju je validator odbio; ispravan prazni zapis pokriva Rust test.
Izvoz HDKS-a ispisao je upozorenje o zastarjelom EPSG:3908.

`xtask` mutiranim kopijama sada daje vlastiti `target` i preskače zanemarene
izlaze pri kopiranju; poziv ima test. XML ovisnost podignuta je na 0.42 zbog
upozorenja cargo-denyja, bez iznimaka.

Windows, Linux, stvarni CI, pravi QGIS prozor i fuzz nisu pokrenuti. Nema commita,
pusha, instalacije alata ni privatnih testnih podataka. Zabranjene mape i postojeće
naredbe nisu mijenjane. Puni lokalni dnevnici su u zanemarenoj mapi `.quality`.

## QGIS projekt

Pri svakom spremanju i spremanju kao (`spremi`, `spremi_kao`), `geopaket` atomski
unutar iste SQLite transakcije zapisuje valjani QGIS projekt pod fiksnim nazivom
`TerraSimple` u tablicu `qgis_projects`. Ako tablica ne postoji, stvara se s
definicijom `(name TEXT PRIMARY KEY, metadata BLOB, content BLOB)`. Postojeći tuđi
projekti u toj tablici ostaju netaknuti (provjereno u testovima očuvanjem projekta
`KorisnickiProjekt`). Tablica `qgis_projects` nije registrirana kao vektorski sloj u
`gpkg_contents`, pa je QGIS i GDAL validator ne vide kao sloj.

Format zapisa projekta:
- QGIS unutar stupca `content` očekuje heksadecimalno kodirani niz bajtova `.qgz`
  ZIP arhive, kao tekst. Sama arhiva
  sadrži XML datoteku projekta pod nazivom `TerraSimple.qgs`. Kompresija koristi
  paket `flate2` (`rust_backend`, bez C ovisnosti i bez ranjivosti prema cargo-deny).
- Stablo slojeva u projektu točno preslikava stablo iz `slojevi::Projekt`: skupine
  postaju `<layer-tree-group>`, a slojevi `<layer-tree-layer>`. Svi slojevi i skupine
  su uključeni i otvoreni, jer se vidljivost u TerraSimpleu ne sprema u datoteku.
- Redoslijed iscrtavanja (`<custom-order>`, `<layerorder>`) prati stablo od vrha prema
  dnu, i slojeve u mapama, s podlogom na samom dnu.
- Stilovi slojeva preuzimaju se izravno iz modula `stil`, koji generira QML
  odgovarajućih vektorskih pravila (točke, linije, poligoni) te se ugrađuju u
  `<maplayer>`.
- Koordinatni sustav projekta postavljen je na `EPSG:3765` (`HTRS96 / Croatia TM`).
  U svojstva projekta dodan je ključ `<SpatialRefSys><ProjectionsEnabled type="int">1</ProjectionsEnabled></SpatialRefSys>`
  kako bi QGIS automatski omogućio dinamičko preračunavanje (OTF projekciju) i
  ispravno očitao projicirani CRS.
- Na dno stabla i popisa slojeva dodan je rasterski XYZ sloj OpenStreetMap (`type="raster"`,
  `provider="wms"`, URL `type=xyz&url=https://tile.openstreetmap.org/{z}/{x}/{y}.png`).
- Početni obuhvat (extent) projekta računa se preko svih geometrijskih objekata u
  projektu uz 5 % margine. Ako u projektu nema objekata, koristi se standardni
  obuhvat Republike Hrvatske u HTRS96/TM.

Premještanje i preimenovanje datoteke:
- Poveznica slojeva u projektu (`<datasource>`) koristi relativnu stazu u formatu
  `./<datoteka>.gpkg|layername=<tablica>`.
- Ako se GeoPackage datoteka kopira ili premjesti u drugu mapu pod istim imenom,
  QGIS uspješno otvara sve slojeve jer relativna staza `./` ostaje točna u odnosu
  na spremnik projekta.
- Ako se datoteka preimenuje izvan TerraSimplea, QGIS ne nađe slojeve. Isto vrijedi
  za projekt koji u GeoPackage spremi sam QGIS: i on piše `./<ime>.gpkg`, a njegova
  preimenovana kopija ima nevaljane slojeve (isprobano bez prozora). Svako spremanje
  iz TerraSimplea prepiše projekt s trenutnim imenom, a gumb „Otvori u QGIS-u” uvijek
  najprije sprema. `krug.py` provjerava oboje: preimenovana kopija bez spremanja nema
  valjanih slojeva, a nakon spremanja ih ima.

Mjerenja kvalitete i mutacije:
- Sve funkcije modula `qgis_projekt` i prilagođenog `pisanje` zadovoljavaju stroge
  zahtjeve: ciklomatska složenost CC <= 7 (najveća je 7 na `xml_bijeg` i `pripremi_sloj`),
  CRAP rizik <= 7, a pokrivenost testovima prelazi 95 % (između 95,2 % i 100 %).
- U `cargo mutants` na `geopaket/src/qgis_projekt.rs` testirano je 135 mutacija:
  101 je uhvaćena, a 34 su preživjele.
- Iskrena klasifikacija preživjelih mutacija:
  - 17 mutacija odnosi se na računanje margine obuhvata (`* 0.05`, razlike `+`/`-`
    pri određivanju ruba platna); to utječe samo na početni zum u prozoru, a ne na logiku baze.
  - 4 mutacije odnose se na uvlačenje i razmake u XML strukturi stabla (`* 2` zamijenjeno s `+ 2`).
  - 4 mutacije odnose se na prazne ili zamjenske oznake tipa geometrije (`geometrija_oznake`).
  - 3 mutacije odnose se na zamjenu identifikacijskog teksta sloja (`id_sloja_tekst`,
    pri spajanju zamijenjen s `id_sloja`).
  - 2 mutacije odnose se na pomak zaglavlja unutar ZIP spremnika (`zapakiraj_qgz`).
  - Preostale mutacije pokrivaju rubne provjere praznog imena datoteke u memoriji i
    obuhvat nepoznatog sustava.
- Provjera stvarnim PyQGIS-om bez grafičkog sučelja (`geopaket/provjera/pokreni.sh`)
  proširena je s 50 na 75 provjera (pri spajanju 89) i sve prolaze (otvaranje projekta, provjera CRS-a,
  struktura stabla, valjanost slojeva, premještanje u drugu mapu, očuvanje tuđih projekata).
- Namjerno unesene greške u kod:
  1. Uklanjanje poziva spremanja projekta iz `pisanje.rs` uzrokovalo je pad PyQGIS testa
     uz `AssertionError: PyQGIS otvara projekt TerraSimple`.
  2. Obrtanje redoslijeda stabla slojeva u `qgis_projekt.rs` uzrokovalo je pad uz
     `AssertionError: prvi element stabla je Tocka`.

### Pregled pri spajanju (8. 11. 2026.)

Redoslijed crtanja išao je po redu stvaranja slojeva, a ne po stablu; QGIS ga zbog
`enabled="0"` nije koristio, ali je sad složen po stablu i ima test. Provjera
preimenovane datoteke sama je preusmjeravala slojeve na novu datoteku pa nije mogla
pasti; sad provjerava stvarno ponašanje. Ključne namjerne greške (preskočeno spremanje
projekta, prazan projekt, pokvareno stablo) ponovljene su i sve su uhvaćene.

Prva proba s pravim QGIS-om pokazala je praznu bijelu kartu, iako je stablo bilo dobro. Bile su tri
greške. Početni pogled bio je zapisan u `properties`, a QGIS ga čita iz `mapcanvas` s imenom
`theMapCanvas`. Podloga nije imala sustav. Slojevi su imali samo `<authid>EPSG:…</authid>`, a takav
zapis QGIS ne prihvaća, pa su ostali bez sustava i nisu se preračunavali. Sad se piše isti oblik kao
iz QGIS-a: puni zapis sustava za svaki sloj, prema sustavu u kojem je spremljen (`qgis_sustavi.rs`,
pet sustava iz QGIS-a 3.44, HDKS kao EPSG:8677 i 8678), te `mapcanvas` i `DefaultViewExtent`. Ključ
`ProjectionsEnabled` mora ostati, jer bez njega QGIS projekt otvori bez sustava. `krug.py` sad otvara
projekt s platnom karte bez prozora i provjerava da pogled obuhvaća objekte, da slojevi imaju valjan
sustav, da podloga ima EPSG:3857 i da se u početnom pogledu iscrtaju objekti. Na staroj inačici ove
provjere padaju.

Druga proba pokazala je podlogu bez nacrtanih objekata. QGIS oznaku sloja od 10 ili manje znakova
zanemari i sloju dade novu, pa stavke stabla (`sloj_2_2`) nisu pokazivale ni na jedan sloj i platno je
crtalo samo podlogu (`osm_podloga` ima 11 znakova). Oznake sad počinju s `TerraSimple_`. Provjera bez
prozora to nije uhvatila jer je crtala ručno nabrojene slojeve; sad platno dobiva slojeve iz stabla
kao u QGIS-u, a svaka stavka stabla mora pokazivati na sloj. Na staroj datoteci stavke ne pokazuju
na slojeve (isprobano bez prozora).

Traži odluku: sustav projekta EPSG:3765 i podloga OpenStreetMap. Stil promijenjen u
QGIS-u ostaje u `layer_styles`, ali sljedeće spremanje iz TerraSimplea u projekt opet
upiše stil iz TerraSimplea.
