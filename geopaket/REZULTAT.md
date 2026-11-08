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
