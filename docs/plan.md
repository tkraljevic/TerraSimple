# Plan rada

Prijedlog od 6. 11. 2026., potvrđen (Odluke (7. 11. 2026.)). Nacrt sučelja je u
`dizajn/TerraSimple.figcraft`, a slova uz faze (A, B, C…) su njegovi zasloni.
Standardi kvalitete su u [kvaliteta.md](kvaliteta.md) i vrijede od prvog retka.

## Stanje

Stanje 9. 11. 2026.: faza 0 je gotova, osim glatkoće karte u prozoru, a faza 1 se privodi kraju za izdanje
0.0.1 alfa. Program ima sedam paketa: `koordinate`, `geometrija`, `slojevi`, `karta`, `prozor`, `geopaket` i
`qgis`. Crta, mjeri, sprema u GeoPackage i otvara projekt u QGIS-u. Što je ušlo u izdanje, piše u
[PROMJENE.md](../PROMJENE.md).

Tražilica i paketi za preuzimanje spojeni su 9. 11. 2026. Tijek paketa na GitHubu prošao je na sva tri sustava, s
provjerom pokretanja; rezultati su u [REZULTAT-izdanje.md](../REZULTAT-izdanje.md). Do
izdanja 0.0.1 ostaje proba od početka do kraja s pravim QGIS-om; prozor „O programu” gotov je 10. 11. 2026. Upozorenje
za lik koji siječe sam sebe gotovo je 9. 11. 2026. (`geometrija/REZULTAT.md`, `karta/REZULTAT.md`); traži
odluku smije li se takav lik spremiti, a sad se sprema uz upozorenje. Poruka nakon sukoba koja nudi Spremi kao i skriven gumb Ispis gotovi su 9. 11. 2026. Odluke su u
odjeljku „Gumb Otvori u QGIS-u” niže.

Rust 1.99.1 je instaliran, a git repozitorij je privatan na GitHubu, bez javne objave. Isprobani su `egui`,
`walkers`, `lyon`, `geographiclib-rs`, `rusqlite`, `geozero`, `proj4rs`, `krilla`, `tiff` i `quick-xml`. Ostale
biblioteke provjerene su samo po podacima s crates.io i njihovim uputama.

Rok koji dolazi izvana: Google najavljuje da se Earth Pro od 25. 6. 2027. više
neće moći preuzeti. Prvo upotrebljivo izdanje treba biti gotovo prije toga.

## Biblioteke

Stanje s crates.io 6. 11. 2026. Sve su dopuštajućih licenci, spojivih s EUPL-om,
što će stalno provjeravati `cargo-deny`.

| Posao | Biblioteka | Inačica | Licenca | Napomena |
|---|---|---|---|---|
| Sučelje | `egui`, `eframe` | 0.37 | MIT/Apache | |
| Prikaz karte | `walkers` | 0.65 | MIT | Pločice, vektorske pločice (MVT), PMTiles, vlastito crtanje preko dodatka. Često izlazi nova inačica, pa se inačica drži čvrsto. |
| Testovi sučelja | `egui_kittest` | 0.37 | MIT/Apache | |
| Geometrija | `geo` | 0.33 | MIT/Apache | |
| Mjere na elipsoidu | `geographiclib-rs` | 0.2 | MIT | |
| Koordinate | `proj4rs` | 0.3 | MIT/Apache | Proba: unutar 1 mm prema QGIS-u. Sustave daje naš popis, ne `proj4wkt`. |
| GeoPackage | `rusqlite`, `geozero` | 0.41, 0.15 | MIT, MIT/Apache | SQLite se ugrađuje u program. Iz `geozero` samo `with-wkb`, jer `with-gpkg` vuče `sqlx`. |
| KML | vlastiti čitač | | | Na `quick-xml` i `zip`. Biblioteka `kml` 0.15 tiho gubi podatke (Odluke (7. 11. 2026.)). |
| GeoJSON, SHP | `geojson`, `shapefile` | 1.0, 0.9 | MIT/Apache, MIT | |
| KMZ, XML | `zip`, `quick-xml` | 9.4, 0.42 | MIT | |
| Podloge iz datoteke | `tiff`, `pmtiles`, `mbtiles` | 0.11, 0.28, 0.22 | MIT, MIT/Apache, MIT/Apache | |
| PDF | `krilla` | 0.8 | MIT/Apache | Proba: unutar 0,04 mm prema računu. Zamjena: `pdf-writer`. |
| Excel, CSV | `calamine`, `rust_xlsxwriter`, `csv` | 0.38, 0.102, 1.4 | MIT, MIT/Apache, MIT | |
| Prijevodi | `fluent` | 0.17 | MIT/Apache | |

Ako `walkers` ne prođe probu, zamjena je `maplibre_native` (0.10, MIT/Apache). To je
gotov C++ prikaz karte, teži za isporuku.

## Što smo provjerili i što iz toga slijedi

- **Karta je vektorska.** OpenFreeMap daje samo vektorske pločice, pa specifikacija
  griješi kad ga stavlja među raster podloge. `walkers` ih zna crtati, ali uz
  `wgpu` i stil koji treba složiti.
- **`walkers` zna samo Web Mercator.** Ekran ostaje u Mercatoru, a u statusnoj traci
  i izračunima su HTRS96/TM i elipsoid. Mjerilo na ekranu računa se uz ispravak za
  geografsku širinu, jer je kod nas Mercator razvučen oko 1,4 puta.
- **DGU WMS radi.** `https://geoportal.dgu.hr/wms` nudi slojeve DOF, HOK i TK25. U
  popisu su sustavi EPSG:3765, 3907, 3908, 4326, 3785 i 900913. U opisu usluge piše da
  nema naknade ni ograničenja pristupa. Uvjeti su pročitani u [podloge.md](podloge.md).
- **Popisu sustava WMS poslužitelja ne smije se slijepo vjerovati.** EPSG:3785, koji DGU
  navodi, vraća krivo mjesto, a EPSG:3857, koji ne navodi, vraća pravi Osijek. 900913
  daje istu sliku kao 3857. Zato uz svaki dodani WMS izvor treba provjera položaja, a
  program prvo traži 3857.
- **HDKS:** 3907 i 3908 u DGU-ovu popisu jesu HDKS zone 5 i 6, ali su zastarjeli
  brojevi (vidi rezultat probe koordinata).
- **OpenStreetMap ne dopušta preuzimanje unaprijed.** Karta bez interneta zato ide
  samo iz korisnikove datoteke (PMTiles, MBTiles, GeoTIFF).
- **Okvir ispisa nije pravi pravokutnik na ekranu.** List je u HTRS96/TM, a ekran u
  Mercatoru. Daleko od srednjeg meridijana 16° 30′ okvir je malo zakrenut, u Osijeku
  oko 1,5°, pa ga tako treba i crtati.
- **`rust-code-analysis-cli`** zadnji put je izašao u siječnju 2023. Ako ne čita
  noviji Rust, složenost će brojati naš `xtask`.

## Faze

Trajanje je gruba procjena iz specifikacije, uz dodano što smo dogovorili, i to uz
neprekidan rad na ovom projektu. Rad uz goCOP traje dulje.

### Faza 0: proba (2–3 tjedna)

Mali programi koji odgovaraju na pitanja, a ne početak programa. Svaka proba ima
mjerilo i na kraju se zapiše odluka. Ako neka padne, plan se mijenja prije faze 1.

| Pitanje | Prolazi ako |
|---|---|
| Može li `egui` izgledati kao nacrt? | Glavni prozor (A) izgleda kao nacrt, u obje teme, na sve tri platforme. |
| Vuče li `walkers` kartu? | OpenFreeMap sa stilom i PMTiles Sektora B glatko se pomiču. Običan laptop nije uvjet (Odluke (7. 11. 2026.)). |
| Radi li crtanje kako smo rekli? | Lijevi klik, povlačenje, desni klik i dvoklik rade i uz 10 000 vrhova, a povlačenje točke ne pomiče kartu. |
| Radi li DGU WMS kao podloga? | DOF se vidi u mjerilu 1 : 4 500. |
| Jesu li koordinate točne? | Kontrolne točke za 3765, 4326, 3857, 3907 i 3908 slažu se s QGIS-om unutar 1 mm. |
| Čita li QGIS naš GeoPackage? | Slojevi, stupci i stil sloja otvaraju se u QGIS-u, a izmjena u QGIS-u vidi se kod nas. |
| Može li stil jednog objekta u QGIS? | Objekt s vlastitim stilom i u QGIS-u izgleda isto. Ako ne, zapisujemo ograničenje. |
| Čita li se pravi `myplaces.kml`? | Duboke mape, nazivi, opisi i stilovi uvoze se bez gubitka. Datoteka ne ide u repozitorij. |
| Vuče li karta stvarne podatke? (dodano 7. 11. 2026.) | 3 milijuna vrhova iz `myplaces.kml` glatko se pomiču i približavaju. |
| Je li PDF u mjerilu? | A4 izmjeren na papiru odgovara mjerilu, a dijakritika je ispravna. A3 se provjerava bez papira, jer pisača za A3 nemamo. |
| Može li GeoTIFF od 2,4 GB? | Otvara se bez čekanja i bez punjenja memorije. |

Usput se postavlja `cargo xtask provjera` i prvo referentno mjerenje. Oboje je napravljeno
7. 11. 2026. (`xtask/IZVJESTAJ.md`). Mjeri se zasad samo sam `xtask`, a Windows, Linux, CI
i duboka provjera još nisu napravljeni.

**Rezultati:**

- **Izgled (6. 11. 2026., `probe/izgled`, Rust 1.99.1, egui 0.37, wgpu):** zaslon A
  nacrtan je u pravom prozoru, svijetlo, tamno i s pločom lijevo, i vrlo je blizu
  nacrtu. Probu je na macOS-u snimio sam program, na vanjskom monitoru bez Retine.
  Nije provjereno: Windows, Linux, Retina zaslon, klikovi i povlačenje ploče (kod
  postoji, ali nije isproban).
  - Tekst je na običnom monitoru malo deblji nego u FigCraftu.
  - egui puni samo izbočene likove, pa parcele traže `lyon`, kao i `walkers`.
  - Nema auto layouta kao u FigCraftu, nego se položaji računaju ručno. Za pravi
    program treba mali skup sastavnica (čip, prekidač, redak stabla).
- **Karta (6. 11. 2026., `probe/karta`, walkers 0.65):** OpenFreeMap, PMTiles (120 MB
  i šira karta) i DGU DOF rade u pravom prozoru, s istim Osijekom na svima. Program je
  sam pomicao i približavao kartu 16 sekundi.
  - Okvir ostaje na granici monitora (20 ms, jer je zaslon na 50 Hz). Medijan je
    20,0 ms u svim mjerenjima. Najgore je 99 % do 25,9 ms i jedan okvir do 29,4 ms.
  - Crta s 10 000 vrhova nije usporila okvire. Vrijeme našeg koda u okviru je 0,4 ms,
    bez iscrtavanja na grafičkoj kartici.
  - Memorija je najviše 212 MB.
  - Stil karte već postoji u svijetloj i tamnoj inačici. Natpisi su, kao i u goCOP-u,
    dio stila, pa su hrvatski nazivi tu samo ako ih pločice imaju.
  - Nije provjereno: običan laptop (mjereno je na M4 Pro s 48 GB, a slabija računala nisu
    uvjet, Odluke (7. 11. 2026.)), Windows, Linux, pomicanje mišem, tamna tema karte, ispod zuma 11
    i iznad 17. Vrijeme do prve slike nije izmjereno, jer je brojač pokazao samo
    donju granicu iz probe (0,5 s).
  - `walkers` ima i `with_projection`, a `walkers_extras` sloj za KML i GeoJSON. Oboje je
    pogledano prije paketa `karta` (vidi rezultat probe `walkers-dodaci` u fazi 1).
- **Crtanje mišem (6. 11. 2026., `probe/crtanje`):** lijevi klik, povlačenje točke,
  povlačenje karte, desni klik, dvoklik, srednja točka, poništavanje, Backspace i Esc
  rade na pravoj karti. Samoprovjera od 25 radnji prošla je 10 puta od 10, s istom
  površinom svaki put. Događaji miša ulaze u egui, pa operacijski sustav i pravi miš tu
  nisu sudjelovali.
  - Lik od 1 000 vrhova drži granicu monitora (20 ms). Lik od 10 000 vrhova, sa svih
    20 000 hvatišta na ekranu, treba oko 28 ms po okviru, što je oko 35 sličica u
    sekundi. Radi, ali u programu ne treba crtati sva hvatišta odjednom.
  - **Povlačenje:** `panning()` u `walkersu` vrijedi samo za kotačić. Povlačenje isključuje
    `drag_pan_buttons`, a odluka se donosi na mjestu gdje je tipka pritisnuta.
  - **Površina:** `geographiclib` za lik nacrtan u smjeru kazaljke na satu vraća površinu
    ostatka Zemlje ako se ne traži predznak. Proba je to otkrila tek na snimci, jer je
    prva provjera bila preslaba. Zato kritični računi moraju imati stroge testove i
    usporedbu s neovisnim izračunom.
  - **Inercija:** karta nakon brzog povlačenja još klizi, pa klik odmah nakon toga može
    promašiti. Karta i dalje klizi, a klik je samo zaustavi (Odluke (7. 11. 2026.), vidi rezultat
    probe `walkers-dodaci` u fazi 1).
  - **Lik koji siječe sam sebe:** QGIS takvu geometriju smatra neispravnom. Od 9. 11. 2026.
    program označi mjesto presjeka i upozori (`geometrija/REZULTAT.md`).
  - **Samoprovjera mora biti stabilna:** prve inačice padale su nasumično jer su čekale u
    okvirima, a monitori imaju 50 i 120 Hz, i jer je klizanje karte ovisilo o brzini.
    Testovi sučelja u programu moraju mjeriti vrijeme, a ne okvire.
- **GeoPackage i stil objekta (7. 11. 2026., `probe/geopaket`, QGIS 3.44.15, GDAL 3.13):**
  oba pitanja zasad prolaze. Provjera ide kroz QGIS-ov Python, bez njegova prozora:
  85 provjera, 5 krugova od 5, a GDAL-ov službeni validator nema primjedbi. Slojevi,
  stupci, dijakritika, sustavi 4326 i 3765, prostorni indeks i stil otvaraju se sami.
  Izmjene iz QGIS-a (vrijednost, vrh, novi i obrisani objekt, novi stupac, novi stil)
  vide se kod nas, a naši upisi u QGIS-ovu datoteku vide se u QGIS-u.
  Nije provjereno: pravi prozor QGIS-a, Windows, Linux, starije inačice QGIS-a, mrežni
  disk, i što program prikaže kad je stil u QGIS-u po kategorijama ili pravilima.
  - **Bez `ST_*` funkcija nema pisanja:** okidači prostornog indeksa pozivaju `ST_IsEmpty`,
    `ST_MinX` i slične, a obični SQLite ih nema. Program ih mora sam dodati.
  - **Stil objekta:** drži se u stupcima `stil_*`, koji su u tablici QGIS-a skriveni.
    Prazan stupac znači stil sloja. QGIS za prazan stupac širine ruba lika i oblika znaka
    zadrži vrijednost prethodnog objekta, pa ta dva svojstva imaju zamjenu u izrazu. Zato
    promjena tih dvaju svojstava u QGIS-ovu stilu sloja vjerojatno neće djelovati, što
    još treba provjeriti. Debljine su u milimetrima (0,8 mm je oko 3 piksela na 96 dpi),
    pa i program mora računati u milimetrima.
  - **Mape i stanje sučelja:** tablice koje počinju s `ts_` QGIS nudi korisniku kao
    slojeve. Tablice s predmetkom `gpkg_ts_`, upisane kao proširenje, ne nudi, validator
    ih prihvaća, a izmjene u QGIS-u ih ne diraju.
  - **Visina se tiho gubi:** `geozero` bez posebnog traženja ne čita Z, a `geo_types` ga
    nema. Neizmijenjeni objekt zato se nikad ne smije prepisivati. Model programa mora
    nositi Z ili izvorni zapis, inače bi uređivanje 3D linije bacilo visine.
  - **Krivulje:** `CompoundCurve` se ne može pretvoriti u `geo_types`. Treba odlučiti: samo
    prikaz, pretvorba u crtu uz upozorenje, ili odbijanje.
  - **Objekt bez geometrije:** QGIS ga piše kao `POINT EMPTY`, a nakon brisanja geometrije
    kao NULL. Čitanje mora preživjeti oba oblika.
  - **Čitanje stila:** QGIS-ov QML sadrži više simbola (i za odabir i legendu), pa vrijedi
    samo onaj unutar `renderer-v2`.
  - **Istodobni rad:** obje strane mogu biti otvorene, a program primijeti spremanje iz
    QGIS-a za oko 0,05 s (`PRAGMA data_version`). Ali:
    - ako QGIS sprema dok program piše, spremanje u QGIS-u pada s „database is locked”,
      i uz samo 0,2 s preklapanja. Izmjene mu ostaju pa se može ponoviti. Program zato
      mora pisati u što kraćim transakcijama. Trajanje naših upisa nije izmjereno.
    - ako oba promijene isti objekt, ostaje zadnje spremljeno i nitko ne dobije
      upozorenje. Program bi to morao primijetiti i javiti.
    - dok je sloj u QGIS-u u uređivanju, dio podataka je u datoteci `-wal`. Kopija samo
      `.gpkg` tada nema zadnje spremljene izmjene: u probi 4 objekta umjesto 5. Program
      to mora reći prije kopiranja ili slanja.
  - **Provjera je opet prošla krivo:** prvi prolaz od 77 provjera bio je čist, a tek je slika
    pokazala preuzeti oblik i rub i natpise kojih nema na linijama. Provjere dodane
    nakon toga pale su na starom stilu i prolaze na novom.
- **Koordinate (7. 11. 2026., `probe/koordinate`, QGIS 3.44.15, PROJ 9.8):** prolazi.
  Zapisana granica je **1 mm prema QGIS-u**. Na 386 točaka preko Hrvatske, u 21 smjeru,
  `proj4rs` i naš račun odstupaju najviše 0,105 mm (3908 > 3765), a svi ostali smjerovi
  manje od 0,00001 mm. Provjera pada kad se jedna točka pomakne za 2 mm. Naš račun prolazi
  i tri primjera iz EPSG-ovih uputa (Guidance Note 7-2), koji ne ovise o QGIS-u.
  Nije provjereno: stvarne kontrolne točke DGU-a, Windows, Linux i službena mreža DGU-a.
  - **Biblioteka:** `proj4rs` je za program, jer je oko 3,7 puta brži (milijun točaka iz
    HDKS-a u Mercator za 190 ms, naš račun 720 ms) i zna više projekcija. Naš račun od
    oko 150 redaka ostaje u testovima kao neovisna provjera.
  - **HDKS je potvrđen:** 3907 je zona 5 (15°), a 3908 zona 6 (18°). **Oba broja su
    zastarjela**, zamjena su 8677 i 8678. Račun je isti, a mijenja se samo službeni
    redoslijed osi. Program mora čitati stare i pisati nove brojeve.
  - **Sustav se ne smije prepoznavati samo po broju:** QGIS sloj u 3907 ili 3908 sprema u
    GeoPackage pod brojem 100000 i organizacijom `NONE`. Pravi broj piše samo na kraju
    definicije.
  - **Redoslijed osi:** u GeoPackageu je prva koordinata uvijek istok, i za 3908, iako u
    njegovoj definiciji prvo piše sjever. Program tu oznaku mora zanemariti.
  - **Definicija iz datoteke nije dovoljna:** `proj4wkt` iz definicije tiho daje krivo
    mjesto: **za HDKS 410 m**, jer u njoj nema pomaka prema WGS-u, a **za 3857 oko 21 km**,
    jer od kugle napravi elipsoid. Točan je samo 3765. Zato program ima vlastiti popis
    sustava koje pozna, a za nepoznat sustav pita korisnika.
  - **QGIS za HDKS ne bira hrvatski prijenos.** Bez izbora koristi operaciju MGI 1901 to
    WGS 84 (1), s pomakom od tri broja i točnošću 5 m, koja vrijedi za šire područje. U
    bazi postoji i (4), samo za Hrvatsku, sa 7 parametara i točnošću 1 m. Razlika je 2,4 do
    4,9 m u deset gradova, a na rubu mreže u Sloveniji 7 m. Polazište „7 parametara” zato
    ne odgovara onome što QGIS radi. **Zasad ide kao QGIS, operacija (1), uz natpis o
    točnosti oko 5 m** (7. 11. 2026.), da se isti sloj u oba programa crta na istom mjestu.
    Koja je operacija bliža stvarnom mjestu nije provjereno, jer nemamo s čim usporediti.
    To bi pokazao stari plan položen na ortofoto, pa se izbor tada može promijeniti.
  - **31275 i 31276 nisu isto što i 3907 i 3908:** isti brojevi, ali QGIS za njih koristi
    drugi prijenos, pa je razlika 304 do 320 m. Pri uvozu treba upozoriti.
  - **Krug nije savršen:** iz WGS-a u HDKS i natrag točka se i u QGIS-u vrati pored, 3,9 mm
    s operacijom (1) i 7,3 mm s (4). To je još jedan razlog da se neizmijenjeno ne
    preračunava. Kroz 3765 i 3857 pomaka nema.
  - **Okvir ispisa:** u Osijeku je zakrenut 1,559°, točno kao u QGIS-u.
  - **Mjerilo ekrana:** `1/cos(širina)` griješi do 0,18 % istok-zapad i 0,22 % sjever-jug.
    Za prikaz je to dovoljno, a za ispis mjerilo ionako ne ide iz Mercatora.
- **PDF u mjerilu (7. 11. 2026., `probe/ispis`, krilla 0.8.2):** prolazi po dužoj strani
  lista. **Na papiru A4 crvena crta od 1 km izmjerena je ravnalom i točno je 20 cm**, što
  je 1 : 5 000. Dijakritika je na papiru ispravna, a pisač nije ništa odrezao. Nije
  izmjerena kraća strana lista (zelena crta od 100 mm). Korisnik ima pisač samo za A4, pa
  se A3 provjerava samo bez papira. Program crta list u mjerilu 1 : 5 000, QGIS crta isti
  list, a Poppler oba pretvara u sliku od 600 dpi koja se mjeri.
  - **Položaj:** naš list je najviše 0,037 mm od računa, na oba formata, uz granicu 0,1 mm.
    Mreža je svakih 19,999 mm na A4 i 20,000 mm na A3. Krivo mjerilo 1 : 5 010 ruši provjeru.
  - **QGIS nije savršeno mjerilo.** Njegov list je do 0,181 mm od računa na A4 i 0,076 mm na
    A3, jer list zaokružuje na cijele točke (A4 je 297,039 × 209,903 mm, A3 je 420,158 ×
    297,039 mm), a crtež mu je oko 0,05 % drukčijeg mjerila. Zato je granica prema QGIS-u
    0,2 mm. Naš list od QGIS-ova je najviše 0,169 mm na A4 i 0,085 mm na A3. Prva granica
    od 0,1 mm pala je na A4, a strogo mjerilo ostaje račun.
  - **List i tekst:** list je točno 297,000 × 210,000 mm i 420,000 × 297,000 mm. Dijakritika
    se vidi i kopira, a font je ugrađen kao podskup. Prva stranica nema ni jedne rasterske
    slike. Debljine, crtice 4 + 2 mm, rupa u liku i prozirnost odgovaraju QGIS-u.
  - **Podloga:** DOF iz DGU WMS-a na 200 dpi leži u okviru unutar jednog piksela (0,13 mm).
    PDF s njom ima 0,9 MB na A4 i 1,9 MB na A3. Da je WMS u 3765 dao pravo mjesto provjereno
    je samo po izgledu Osijeka.
  - **Mjerilo vrijedi na karti, ne na tlu.** 1 000 m u HTRS96/TM je na tlu 999,47 m kod
    Iloka, 999,74 m kod Osijeka i 1 000,10 m kod Splita. Na širini od 400 mm to je najviše
    0,21 mm. QGIS radi isto, pa program tako ostaje.
  - **QGIS tiho javlja besmisleno mjerilo** (1 : 522 247 330) ako karta nema sustav, a crtež
    izgleda isto. Prvi prolaz probe je tako bio krivo postavljen.
  - **Provjera opet nije dovoljna bez slike:** sve je prolazilo, a natpis trake mjerila
    izlazio je preko ruba lista, a drugi natpisi preko crta. Provjera rubova pala je na toj
    inačici i prolazi nakon popravka. Preklapanje natpisa nijedna provjera ne vidi, pa
    program za to treba raspored natpisa ili barem upozorenje.
  - **Natpisi na ortofotu:** tamni natpis bez obruba gotovo se ne vidi. Bijeli obrub od
    0,7 mm rješava to, a crta se kao put, pa se tekst pri kopiranju iz PDF-a ne ponavlja.
    Program ga treba imati kao zadano za natpise na podlozi.
  - **Rub lista:** prvi raspored na A4 stavio je tekst 3,5 mm od ruba, što pisač obično ne
    može otisnuti. Uhvatio ga je test, pa program mora poznavati najmanji rub pisača.
  - Nije provjereno: kraća strana na papiru, A3 na papiru, drugi pisači, drugi pregledači
    PDF-a osim Popplera i onoga kojim je list ispisan, Windows i Linux, kosi okvir, natpisi
    uz crtu, veće podloge od ove.

- **GeoTIFF (7. 11. 2026., `probe/geotiff`, tiff 0.11.3, GDAL 3.13):** prolazi, uz jedan
  slučaj koji traži izradu piramida u programu. Na disku nema prave datoteke od 2,4 GB, pa
  je 14 slika izmišljeno, a uz njih je čitano i 6 stvarnih (najveća 0,40 GB). Svi isječci
  jednaki su GDAL-ovima. JPEG se razlikuje najviše 5 od 255, jer dva dekodera nisu ista.
  Cijeli prikaz najveće stvarne slike jednak je GDAL-ovu u svih 1 280 × 800 piksela.

  | Slika od 2,4 GB, unutarnji SSD | Otvaranje | Prva slika | Pomak (medijan) | Najviše memorije |
  |---|---:|---:|---:|---:|
  | Pločice i piramide, Deflate | 1 ms | 62 ms | 40 ms | 265 MB |
  | JPEG s piramidama (125 MB) | 1 ms | 29 ms | 21 ms | 208 MB |
  | Nesažete trake | 4 ms | 107 ms | 114 ms | 13 MB |
  | Jedna traka od 2,4 GB | 1 ms | 132 ms | 121 ms | 12 MB |
  | LZW trake bez piramida | 2 ms | 424 ms | 374 ms | 328 MB |
  | **Pločice bez piramida** | **1 ms** | **12,9 s** | **343 ms** | **304 MB** |

  - **Stvarne datoteke nemaju piramide.** Ni jedna od 7 pregledanih nema piramide ni
    pločice, pa je sporiji put uobičajen, a ne iznimka. Program mora ponuditi izradu
    piramida. Pročitati cijelu sliku od 2,4 GB traje 3,7 do 13,3 s.
  - **Spori disk:** sa SD kartice stvarna slika od 0,40 GB daje prvu sliku za 2,7 s, a pomak
    za oko 1 s. Memorije je najviše 161 MB. I to je razlog za piramide.
  - **Izgled bez piramida:** program uzima svaki n-ti piksel, pa tanke crte nestaju ili se
    pojavljuju. To je prihvatljivo samo kao prvi prikaz dok se piramide rade.
  - **Što stvarne datoteke traže:** 3 od 7 imaju paletu boja, 3 imaju prozirnost, 3 nemaju
    koordinatni sustav, a dvije imaju položaj samo u `.tfw`. Program za takve mora pitati
    sustav, a brojevi poput 6 500 004 upućuju na HDKS zonu 6, pa to može predložiti.
  - **`tiff` ne zna paletu.** Ne vraća je ni kao vrstu boje ni kao podatke, pa je čitanje
    palete s PackBits i LZW naš kod od oko 60 redaka. JPEG vraća kao YCbCr, pa pretvorbu
    u RGB radimo sami.
  - **Oštećena datoteka** javlja urednu grešku bez pada. Ni GDAL je ne otvara.
  - **Položaj:** `PixelIsPoint` pomiče sliku za pola piksela. Provjera pada kad se to
    namjerno pokvari.
  - **Sigurno čitanje:** proba koristi `libc` i `unsafe` samo za mjerenje memorije i traženje
    da sustav ne čuva pročitano. To ne ide u program.
  - Nije provjereno: **običan laptop**, stvarno hladan disk (bez administratorskih ovlasti
    ne može se isprazniti međuspremnik sustava, pa su brojke s unutarnjeg diska moguće
    bolje nego u praksi), slika stvarnih 2,4 GB, iscrtavanje u `egui`, preračun u Mercator
    ekrana, visinski modeli (16 bita, decimalni brojevi), Windows i Linux.

- **KML i KMZ (7. 11. 2026., `probe/kml`, `quick-xml` 0.42, cijeli izvještaj u
  `probe/kml/REZULTAT.md`):** prolazi po broju elemenata. Pravi `myplaces.kml` (147 MB, 87 105
  objekata, 3 032 552 vrha, 251 spremnik, dubina 8) i još 8 stvarnih datoteka čitaju se
  cijeli, a isto vrijedi za KMZ. Vrijeme je oko 0,4 s, a memorija 203 MB. Brojke su neovisno
  potvrđene, a 2 033 stvarna naziva ne pojavljuju se ni u probi ni u izlazu.
  Nije provjereno: **vrijednosti znak po znak na stvarnoj datoteci** (samo na izmišljenim
  primjerima), povratak u Google Earth, Windows i Linux.
  - **Vlastiti čitač (Odluke (7. 11. 2026.)):** `kml` 0.15 tiho gubi 2 069 vrhova, 52 stila i
    jedan opis mape, a na praznoj točki ruši program. Takve točke u stvarnim datotekama nema,
    nego je to izmišljeni primjer. Čitač ima 2 444 retka koje održavamo sami. Što ne razumije,
    broji i prijavljuje.
  - **Rupe:** 332 lika imaju više prstenova u istom `innerBoundaryIs`, ukupno 4 287 rupa.
    Koliko znam, to je izvan službene sheme. Google Earth to čuva pri spremanju, a GDAL iz
    svakog uzima samo prvi prsten, pa bi QGIS iz tog KML-a izgubio 3 475 rupa. Program ih
    mora čitati sve, a u GeoPackage i KML pisati tako da ih QGIS vidi.
  - **Količina:** kartu smo probali s 10 000 vrhova, a stvarnih je 3 milijuna, 300 puta više.
    Zato je dodana proba `probe/mnogo`.
  - **Isključene stavke:** 85 972 stavke imaju `visibility=0`, pa ih uvoz ne smije crtati.
  - **Slojevi:** podjela po vrsti geometrije daje 208 slojeva. Miješani `MultiGeometry` (točka
    i linija, 1 279 objekata) dijeli se na dva sloja. Stablo slojeva mora podnijeti toliko.
  - **Stil:** `StyleMap` često stoji prije `Style`, a ista oznaka ponavlja se u ugniježđenim
    dokumentima, pa se stilovi rješavaju tek nakon čitanja. Boje su zapisane kao `aabbggrr`.
  - **Google Earth sam sprema:** i bez korisnikova spremanja zapiše stanje prikaza (`open`,
    `visibility`). Podaci su 7. 11. ostali znak po znak isti. Program stanje prikaza drži
    odvojeno, da otvaranje karte ne izgleda kao izmjena projekta.

- **Mnogo vrhova (7. 11. 2026., `probe/mnogo`, wgpu 30, walkers 0.65):** na kartici prolazi, a
  u prozoru je spoj potvrđen, ali **vrijeme okvira u prozoru nije izmjereno**. Stvarni
  `myplaces.kml` crta se bez prozora, u 1920 × 1080 s 4× MSAA, na M4 Pro:

  | Što se crta | Medijan | Najgore |
  |---|---:|---:|
  | Sve, s pojednostavljenjem, po mjerilima | 0,9–3,1 ms | 4,7 ms |
  | Sve, bez pojednostavljenja | 11,2 ms | 13,9 ms |
  | Samo uključeno u Google Earthu | 0,2 ms | 0,9 ms |

  - **Način crtanja:** geometrija se jednom pripremi i drži na kartici, a pomicanje mijenja samo
    jednu matricu. Kroz egui, kao u dosadašnjoj probi karte, ista bi količina samo za pripremu
    crta trajala 65 ms u svakom okviru, s 19 milijuna trokuta i 491 MB, bez slanja i crtanja.
    Karta u programu zato ne ide kroz `Plugin` s oblicima, nego preko `egui_wgpu::Callback`.
    Moja ranija procjena od nekoliko sekundi po okviru bila je kriva.
  - **Priprema:** čitanje 0,4 s i priprema 0,8 s na jednoj jakoj jezgri, a na štedljivim 2,3 i
    3,9 s, pa ide u pozadinu. Memorija doseže 1,4 GB dok se pripremaju sve razine, a na kartici
    je 321 MB. Samo uključeno traži 264 MB i 2 MB. `lyon` je ispunio sve likove.
  - **Štedljive jezgre:** crtanje gotovo ne usporavaju, a najgori slučaj bez pojednostavljenja
    je 16 ms. Kartica se ne može usporiti, pa to ne govori kako radi slabija kartica.
  - **Spoj s kartom:** crtanje preko `walkersa` radi s podlogom, a točka se s `walkersovom`
    poklapa unutar 0,4 piksela. Položaj bez prozora točan je do 1 piksela.
  - **Greške probe:** dvostruko mjerilo u sjenčaru izgledalo je na slici dobro, a uhvatila ga je
    tek provjera položaja u pikselima. Okviri u prozoru pokazali su 2 000 u sekundi jer je ekran
    bio ugašen, pa se vjerojatno ništa nije iscrtavalo. To mjerenje ne vrijedi.
  - Nije provjereno: **glatkoća u prozoru s upaljenim ekranom i pravim mišem**, slabija kartica,
    Windows, Linux, natpisi, odabir objekta na karti i uređivanje jednog objekta među milijunima.

### Faza 1: karta i crtanje (4–6 tjedana)

Zasloni A, B, C, D. Podloge OpenFreeMap i DGU, tražilica, crtanje točaka, linija i
poligona, mjerenje, poništavanje, tamna tema. Model sloja od početka zna za vanjske
izvore (samo za čitanje, čeka slanje, u sukobu). Svaka radnja od početka je naredba, a
sučelje poziva iste naredbe, jer je to temelj za MCP (vidi „Upravljanje preko MCP-a”).

**Izdanje 0.0.1 alfa (Odluke (7. 11. 2026.)):** karta, crtanje i mjerenje iz ove faze,
uz spremanje u GeoPackage i otvaranje u QGIS-u iz faze 2. Zato nacrtano ne živi samo u
memoriji: osnovno spremanje dolazi odmah, jer program bez njega ne služi ničemu osim
isprobavanja. Gruba procjena je 5–8 tjedana neprekidnog rada, izvedena iz trajanja faza,
a ne izmjerena. Rok nije važan, nego da program na kraju bude koristan.

Redoslijed: `koordinate`, `geometrija` (mjere), `slojevi` (model i naredbe s poništavanjem),
`karta` (crtanje na kartici iz `probe/mnogo`), `prozor`, a zatim spremanje u GeoPackage.

**Rezultat paketa `geometrija` (7. 11. 2026., `geographiclib-rs` 0.2.7, QGIS 3.44.15, PROJ 9.8):**
duljina crte te opseg i površina lika računaju se na elipsoidu WGS84, a uz svaku mjeru piše metoda.

- **Točnost:** na 150 crta i likova najveće je odstupanje od QGIS-a 0,0005 mm na duljini i
  0,004 m² na površini, i to na liku od 93 000 km². QGIS i PROJ daju iste brojeve jer računaju istom
  bibliotekom, pa je to jedna provjera, a ne dvije.
- **Neovisna provjera:** slažu se dva objavljena primjera iz GeographicLiba (JFK–LHR i Antarktika),
  Vincentyjeve formule unutar 1 mm i točna formula za pojas između paralela. Svih 20 namjerno
  ubačenih grešaka testovi su uhvatili, među njima i pogrešku od milijuntog dijela površine.
- **Kao u QGIS-u:** opseg uključuje rubove rupa, a lik koji siječe sam sebe dobiva razliku svojih
  dijelova, pa simetrična osmica ima površinu 0. Upozorenje je napravljeno 9. 11. 2026.
- **Krive koordinate:** točka u metrima predana kao stupnjevi vraća grešku, a ne besmislen broj.
- **Brzina:** 10 000 vrhova oko 5 ms, a 3 milijuna vrhova 0,3 do 0,5 s, pa se računa samo kad se lik
  promijeni, a velike likove treba računati u pozadini. Mjereno na M4 Pro, uz opterećeno računalo.
- **Površina u ravnini nije ista:** u HTRS96/TM na 1 ha ima 1,6 do 2 m² manje u Zagrebu i Splitu, a
  5,2 m² više u Osijeku, 8,7 m² u Puli i 10,6 m² u Iloku. QGIS bez elipsoida i računi u ravnini dat će
  te brojeve. Program zasad prikazuje samo elipsoid.
- Nije provjereno: Windows, Linux, usporedba sa stvarnim katastarskim površinama. Visina se ne računa.

**Rezultat probe `walkers-dodaci` (7. 11. 2026., `walkers` 0.65, cijeli izvještaj u
`probe/walkers-dodaci/REZULTAT.md`):** odgovorila je na ono što je trebalo znati prije paketa `karta`.

- **Ekran ostaje u Mercatoru.** `with_projection` s HTRS96/TM radi za točke i crtanje, ali pločice se ne
  preračunavaju, pa OpenFreeMap i PMTiles tada ne rade. HTRS96/TM računa paket `koordinate`.
- **`walkers_extras` se ne koristi za KML.** Gubi stilove, nazive i visine, prazna točka ruši program, a
  crta sve iznova na procesoru u svakom okviru. Ostaju naš čitač i crtanje iz `probe/mnogo`.
- **Inercija (Odluke (7. 11. 2026.)):** karta klizi i dok traje crtanje, a klik dok klizi samo je
  zaustavi i ne upisuje točku. Sljedeći klik upisuje točku. Napravljeno je u paketu `karta` i isprobano
  pravim mišem. Mjerenje je pokazalo da poziv `center_at(detached)` zaustavlja kartu, a da poziv
  poslije `Map::show` kod puštanja tipke ostavlja skok od 54 px, pa mora biti prije njega.
- **Zum:** dopušteno je 0 do 26. Iznad zadnje razine podataka (14 kod OpenFreeMapa, 15 kod PMTilesa
  Sektora B) rastegne se niža razina, a bez pločice ne prikazuje se ništa. Pada nije bilo na 9 ispitanih
  zumova.
- **Tamna tema karte radi**, ali stil treba doraditi zbog niskog kontrasta, a program mora prikazivati
  oznaku izvora podataka.
- **Pregled izvještaja:** prva inačica imala je opis inercije koji ne odgovara kodu i preporuku sa skokom
  karte. Testovi su prolazili jer su klizanje mjerili tek nakon okvira u kojem nastaje. Dva namjerna
  kvara samo postavljaju zastavicu, a provjere stila i visine ne mogu pasti, pa te zaključke potvrđuje
  kod biblioteke.
- Nije provjereno: pravi miš, Windows, Linux, karta samo s DGU WMS-om u HTRS96/TM, brzina i oštrina
  rastegnute karte.

**Rezultat paketa `slojevi` (7. 11. 2026.):** model mapa, slojeva i objekata te naredbe s poništavanjem
i ponavljanjem. Sučelje i MCP šalju iste naredbe.

- **Model:** koordinate su u WGS84 i nose visinu, sloj ima jednu vrstu geometrije i stanje izvora, a
  zaključan sloj ne prima izmjene objekata. Poništavanje na zaključanom sloju javlja grešku i ne gubi
  povijest. Mapa ne može u samu sebe, a skupina naredbi je jedan korak koji prođe sav ili nikako.
- **Poništavanje ne kopira projekt.** Svaka promjena vraća promjenu koja je poništava.
- **Količina:** s izmišljenih 87 105 objekata, oko 3 milijuna vrhova i 208 slojeva, uvoz kao jedan korak
  traje 25 ms, poništavanje uvoza 12 ms, brisanje mape 0,02 ms, a memorija je 151 MB nakon uvoza i 235 MB
  na kraju mjerenja, s poviješću. Čitanje datoteke i priprema za crtanje nisu uključeni.
- **Provjera:** 40 nasumičnih nizova od po 500 pokušaja vraća se točno unatrag i unaprijed. Testovi su
  uhvatili 29 od 30 namjernih grešaka, a poništavanje preimenovanja promaklo im je pa je dodan test koji
  ga hvata.
- **Još nije tu:** vidljivost i otvorene mape (stanje prikaza, koje se ne poništava), nepoznati stupci,
  sustav sloja, popis izmijenjenog za spremanje i granica povijesti. Dolaze s `prozor` i GeoPackageom.

**Rezultat paketa `karta` (7. 11. 2026., `walkers` 0.65, `wgpu` 30):** karta s podlogom, crtanje i mjerenje.
Primjer se pokreće s `cargo run -p karta --example prozor`. Pravim mišem isprobano je na macOS-u, uz ocjenu
„sve radi što je potrebno dobro”.

- **Crtanje:** točka, linija i poligon. Lijevi klik dodaje točku, povlačenje je pomiče, desni klik briše, a
  dvoklik završava. Svaka promjena ide kroz naredbe iz paketa `slojevi`, pa se može poništiti.
- **Nastavljanje linije (Odluke (7. 11. 2026.)):** klik na krajnju točku gotove linije nastavlja je s te
  strane. Objekt se mijenja tek na završetku, a cijelo produljenje je jedan korak za poništavanje.
- **Mjerenje:** duljina i površina idu preko paketa `geometrija`, s pločom po zaslonu D. Ništa se ne sprema
  dok korisnik to ne zatraži.
- **Crtanje na kartici:** geometrija ostaje na kartici, u komadima i na šest razina detalja, a priprema se
  ponavlja samo kad se sloj promijeni. Za to je sloj dobio broj inačice.
- **Podloge:** OpenFreeMap i DGU, s navodom izvora.
- **Provjera:** 94 testa, od kojih 5 crta na pravoj kartici i čita piksele. Redovna provjera prolazi s 94,64 %
  na 326 funkcija u cijelom repozitoriju.
- **Namjerne greške:** prva tura našla je 58 preživjelih, a nakon novih testova ostalo ih je 6 od 406. Svih
  6 ostaju s razlogom jer ne mijenjaju rezultat: smjer zrake i rub u provjeri je li točka unutar lika (2), početak
  petlje i jednake udaljenosti u pojednostavljenju (3) te greška 0 kod premalog lika (1). Mjerene su
  `crtanje.rs`, `karta.rs` i `priprema.rs` te `promjena.rs` iz paketa `slojevi`. Crtač, kamera, Mercator i podloga
  nisu, ali crtač i kameru provjeravaju testovi s pravom karticom.
- **Ograničenje:** nema MSAA, jer `walkers` crta bez više uzoraka. Kose crte zato imaju malo nazubljene rubove.
- **Licence:** na popis dopuštenih dodani su `0BSD`, `BSL-1.0` i `CDLA-Permissive-2.0`, koje traže biblioteke
  ispod `walkers`.
- **Nije provjereno:** ručni rad na Windowsu i Linuxu (na njima se zasad vrte samo testovi na CI-ju), glatkoća u
  prozoru s upaljenim ekranom i rad na slabijem računalu.

**Rezultat paketa `prozor` (8. 11. 2026., `egui` 0.37):** glavni prozor po zaslonu A, u svijetloj i tamnoj
temi: izbornik, alati, ploča Slojevi sa stablom, izbor podloge i statusna traka. Cijeli izvještaj je u
`prozor/REZULTAT.md`.

- **Naredbe:** svaka izmjena stabla ide kroz naredbe iz paketa `slojevi`, pa se može poništiti.
- **Prazan projekt** ne prikazuje izmišljene koordinate, mjerilo ni stanje mreže. Primjer iz nacrta
  pokreće se zastavicom `--primjer`.
- **Provjera:** 33 testa, od kojih neki čitaju i tekst nacrtan na karti. Od 279 namjernih grešaka 61 je
  preživjela, i sve su u računu položaja i razmaka na zaslonu.
- **Još nije tu:** Novi, Otvori i Spremi te radnja gumba „Otvori u QGIS-u”.

**Karta u prozoru (8. 11. 2026.):** karta s podlogom ispunjava sredinu prozora, ispod ploča i gumba.

- **Usklađeno s pločom:** alat iz trake alata, odabrani objekt u oba smjera, nove točke i crte idu u
  odabrani sloj, a crtaju se samo slojevi koji su uključeni zajedno sa svim mapama iznad sebe. Podloga
  slijedi čipove Karta i DGU ortofoto, a karta OpenFreeMap slijedi tamnu temu.
- **Mjerenje:** ploča po zaslonu D, desno od alata: Udaljenost ili Površina, mjera, broj dijelova,
  Spremi kao liniju ili poligon, Očisti i zatvaranje.
- **Mjerilo i koordinate:** traka mjerila i statusna traka računaju se iz stvarnog zuma, a koordinate
  pod mišem pokazuju se u HTRS96/TM. Nazivnik mjerila uzima 96 točaka po inču, pa na zaslonu nije točan
  kao na papiru.
- **Tipke:** Backspace i Escape dok crtanje traje pripadaju karti, a inače Backspace briše odabrano u
  ploči. Gumbi Približi, Udalji i Cijeli zaslon rade, a Sjever još nema radnju. Ispis je skriven (Odluke (9. 11. 2026.)).
  Gumb Sjever ostaje (Odluke (9. 11. 2026.)).
- **Novi sloj (8. 11. 2026., nakon probe mišem):** gumb je uvijek stvarao sloj za poligone, pa su nacrtane
  točke i linije tiho završavale u drugom sloju, bez naziva. Sad otvara prozorčić po zaslonu F: naziv, vrsta
  (predlaže se prema alatu) i mapa. Polja i koordinatni sustav iz nacrta još nisu tu. Novi objekt dobiva naziv
  („Točka 3”), a kad odabrani sloj ne odgovara alatu, ispod karte piše u koji sloj ide novi objekt.
- **Navod izvora** crta prozor, lijevo od ploče, jer bi ga ploča inače pokrila.
- **Provjera:** 45 testova prozora, sa stvarnim klikovima na kartu. Od 145 namjernih grešaka preživjele
  su 33: 32 samo pomiču crtež na zaslonu, a jedna je granica nazivnika mjerila točno na 1. Snimke u svijetloj
  i tamnoj temi pregledane su s pravom podlogom.
- **Nije provjereno:** pravi miš, Windows i Linux. Tražilica radi od 9. 11. 2026.; rezultati su u
  `prozor/REZULTAT.md` i `koordinate/REZULTAT.md`.

**Rezultat paketa `geopaket` (8. 11. 2026., `rusqlite` 0.41, `geozero` 0.15, QGIS 3.44.15, GDAL 3.13):**
projekt se sprema u GeoPackage i čita natrag. Cijeli izvještaj je u `geopaket/REZULTAT.md`, a zapis stabla
u `geopaket/PROSIRENJE.md`.

- **Što se sprema:** mape, redoslijed, stanja slojeva, nazivi, opisi, stilovi i visine. Slojevi su obične
  tablice GeoPackagea, a stablo je registrirano proširenje. Novi slojevi su u EPSG:4326, a tuđi zadržavaju
  svoj sustav.
- **Tuđe se ne dira:** nepromijenjene geometrije ostaju bajt za bajt iste, a nepoznati stupci i tablice
  ostaju sačuvani. Sprema se samo ono što se promijenilo, i to sve ili ništa.
- **QGIS:** svih 50 provjera bez prozora prolazi, a GDAL-ov validator nema primjedbi. Krug je naše
  spremanje, izmjena u QGIS-u, naše ponovno spremanje i ponovno otvaranje u QGIS-u.
- **Namjerne greške:** od 609 ih je preživjelo 11, i nijedna nije rupa. Tri ne mijenjaju rezultat, a
  sedam u paketu `slojevi` hvataju testovi paketa `geopaket`. Dvije stvaraju beskonačnu petlju u čitanju
  stila, pa se računaju kao otkrivene. Provjera prolazi s 96,25 % na 660 funkcija.
- **Sporo na velikom:** s 87 105 objekata jedno spremanje traje oko 1 s, a memorija je oko 1 GB, jer se
  čuva cijela kopija projekta i uspoređuje objekt po objekt. Ubrzat će se preko brojeva inačica slojeva.
  Brojke je izmjerio izvođač paketa, a pri pregledu nisu ponovljene.
- **Brisanje sloja s neobičnim zapisima (Odluke (8. 11. 2026.)):** sloj koji iz tuđe datoteke donese
  krivulje, prazne ili NULL geometrije smije se obrisati, a s njim nestaju i ti zapisi. Prozor prije brisanja
  mora upozoriti koliko ih ima; to dolazi zajedno s naredbom Otvori.
- **Otvoreno:** fuzz čitača još nije uveden, iako ga `kvaliteta.md` traži od prvog čitača nepouzdanih
  datoteka. Nightly Rust i `cargo-fuzz` instalirani su 8. 11.

**Izbornik Datoteka (8. 11. 2026.):** Novi, Otvori, Spremi i Spremi kao, s prečacima i sustavskim prozorom za
izbor datoteke (`rfd`, na Linuxu preko XDG portala). Novi, Otvori i zatvaranje pitaju za nespremljene izmjene, a
uz naziv datoteke vidi se kad ih ima. Brisanje sloja ili mape s neobičnim zapisima najprije upozorava. Greške
pri otvaranju i spremanju javljaju se u dijalogu i ne gube projekt. Cijeli izvještaj je u `prozor/REZULTAT.md`.
Na Macu prečaci idu s ⌘ i tako pišu u izborniku (⌘S, ⇧⌘S), a drugdje s Ctrl (Odluke (9. 11. 2026.)).

- **Nespremljene izmjene** prati `Uredjivac`: svaki korak povijesti ima oznaku, pa poništavanje do spremljenog
  stanja opet znači „spremljeno”.
- **Pri spajanju ispravljeno:** klik i Delete prolazili su kroz otvoren dijalog na kartu i ploču, a nakon Otvori
  karta je mogla crtati slojeve starog projekta. Otvoreni projekt sad se prikazuje cijeli.
- **Nije provjereno:** pravi prozor za izbor datoteke na sva tri sustava, Windows i Linux.

**Fuzz čitača i paket `qgis` (8. 11. 2026.):** izvještaji su u `fuzz/REZULTAT.md` i `qgis/REZULTAT.md`.

- **Fuzz:** tri cilja, geometrija, QML stil i cijela datoteka, po 30 minuta: ukupno 4 072 126 ulaza bez pada,
  zastoja ili prekoračenja memorije. Namjerno pokvaren kraj čitanja stila fuzz uhvati za nekoliko sekundi.
  `cargo xtask fuzz` treba nightly, a duboka provjera bez njega bilježi da fuzz nije pokrenut. Licenca NCSA
  dopuštena je samo za libFuzzer.
- **QGIS:** paket pronalazi QGIS na tri sustava i otvara mu jednu datoteku, bez čekanja. Na ovom Macu našao je
  QGIS-LTR 3.44.15. Pravo otvaranje, Windows i Linux nisu isprobani, a gumb u prozoru još nije spojen.

**Gumb „Otvori u QGIS-u” i QGIS projekt u datoteci (8. 11. 2026.):** izvještaji su u `geopaket/REZULTAT.md` i
`qgis/REZULTAT.md`, odjeljak „QGIS projekt”.

- **Što radi:** gumb sprema projekt i otvara ga u QGIS-u. Svako spremanje u GeoPackage upiše i QGIS projekt
  „TerraSimple” s nazivima slojeva, mapama kao skupinama, redoslijedom iz stabla i stilovima. Projekti koje je
  korisnik u QGIS-u spremio pod drugim imenom ostaju.
- **Provjereno:** 89 provjera u QGIS-u bez prozora, s platnom karte, i GDAL-ov validator. Prve dvije probe s
  pravim QGIS-om dale su praznu kartu pa samo podlogu (pogled, sustavi i prekratke oznake slojeva); ispravljeno i
  pokriveno provjerom. Ključne namjerne greške su uhvaćene.
  Ako se datoteka preimenuje izvan TerraSimplea, QGIS ne nađe slojeve dok je TerraSimple opet ne spremi. Isto se
  događa s projektom koji sprema sam QGIS. Gumb uvijek najprije sprema.
- **Sukob s QGIS-om (9. 11. 2026.):** ako je QGIS u međuvremenu promijenio datoteku, spremanje se odbije i
  prozor javlja: „Projekt nije spremljen: datoteku je u međuvremenu promijenio drugi program. Otvori je
  ponovno.” Prije je pokazivao opću poruku, jer je razlog tražio po riječi u tekstu greške. Sad ga prepoznaje
  po vrsti greške iz paketa `geopaket`.
- **Odluke (9. 11. 2026.):**
  - podloga OpenStreetMap ostaje u QGIS projektu;
  - **sukob:** nakon sukoba s QGIS-om poruka nudi Spremi kao. Izmjene iz TerraSimplea idu u novu datoteku,
    a datoteka koju je mijenjao QGIS ostaje netaknuta. Gumba koji pregazi izmjene iz QGIS-a nema.
    Napravljeno: prozorčić „Datoteku je promijenio drugi program” ima gumbe Odustani i Spremi kao…;
  - **stil:** TerraSimple zamjenjuje stil sloja u QGIS projektu samo kad ga je korisnik u TerraSimpleu
    mijenjao. Inače ostaje sve što je za taj sloj namješteno u QGIS-u: stil, natpisi, prozirnost.
    Napravljeno i provjereno s pravim QGIS-om bez prozora. Postavke cijelog projekta iz QGIS-a, npr.
    ispisni predlošci, i dalje se gube; za njih odluke još nema;
  - **QGIS na Macu:** za 0.0.1 svaki klik i dalje pokreće novi QGIS, uz napomenu u popisu promjena. Poslije
    se provjeri prima li već otvoreni QGIS projekt kako treba;
  - **Ispis:** gumb i stavka izbornika skriveni su dok ispis ne proradi u fazi 6 (napravljeno);
  - **HDKS u tražilici:** program sam prepozna HDKS po rasponu brojeva, bez oznake ispred. U Hrvatskoj HDKS
    istok ima sedam znamenki i počinje brojem zone, a HTRS96/TM istok šest, pa se ne preklapaju. Uz rezultat
    piše da je točnost oko 5 m. Ide kao zaseban mali zadatak nakon spajanja tražilice.
- **Nije provjereno:** pravo otvaranje projekta s prozorom, Windows i Linux.

### Faza 2: projekt i slojevi (3–4 tjedna)

Zasloni A4, F, F2, G, J. Projekt je jedan GeoPackage, s mapama, novim slojem,
uvozom GeoPackagea, automatskim spremanjem, oporavkom i gumbom „Otvori u QGIS-u”.
Osnovno spremanje i otvaranje u QGIS-u već su u izdanju 0.0.1.

### Faza 3: Google Earth (3–5 tjedana)

Zasloni A3, K, L. KML, KMZ, GeoJSON i SHP, pregled gubitaka pri izvozu, pitanje o
koordinatnom sustavu, razmjena s Earth Proom.

**Na kraju izlazi 0.1, prvo upotrebljivo izdanje:** zamjenjuje Earth Pro za crtanje,
mjerenje i KML, a sve se otvara u QGIS-u. Do ovdje je 12–18 tjedana rada.

### Faza 4: stil i tablica (4–6 tjedana)

Zasloni C2, C3, C4, E, E2, E3. Vrste crta, ikone, natpisi i stil objekta. Atributna
tablica, izvoz u Excel i CSV te uvoz izmjena s pregledom.

### Faza 5: podloge (3–5 tjedana)

Zasloni H, I. Dodavanje WMS, WMTS i XYZ izvora te podloge iz GeoTIFF-a, PMTilesa i
MBTilesa.

### Faza 6: ispis (5–8 tjedana)

Zasloni M, N. Ispis u mjerilu, PDF i uređivač predloška. Ovo je najneizvjesniji dio
nakon karte.

### Faza 7: dovršenje (4–8 tjedana)

Testovi na sve tri platforme, instalacijski paketi, priručnik, popravci i izdanje 1.0.

**Instalacija bez administratorskih ovlasti** (zahtjev, 7. 11. 2026.): program se instalira u
korisničku mapu (Windows `%LOCALAPPDATA%\Programs`, macOS `~/Applications`, Linux
`~/.local` ili AppImage), a isto vrijedi za nadogradnju i povezivanje vrste datoteke. Zato
nema usluga sustava ni upisa izvan korisnikova dijela. Nije isprobano. Dvije stvari ne ovise
o nama: tvrtka može pravilima zabraniti pokretanje iz korisničke mape, a nepotpisan program
dobiva upozorenje pri pokretanju. Dopuštenje vatrozida za MCP preko mreže može na Windowsu
tražiti administratora, a veza samo na ovo računalo ne traži.

Ukupno 28–45 tjedana neprekidnog rada, odnosno oko 6,5–10,5 mjeseci. Procjena je vrlo
nesigurna dok faza 0 ne odgovori na pitanja o karti i ispisu.

## Upravljanje preko MCP-a

Odluke (7. 11. 2026.): MCP je ugrađen u sučelje. **Sklopka u programu pokreće i gasi MCP
poslužitelj**, a na njega se može spojiti bilo koji AI, i javni i privatni na vlastitom
poslužitelju. Tako AI crta i upravlja programom.

Prijedlog izvedbe, još nije isproban:

- **Naredbe:** MCP je tanak sloj preko istih naredbi kojima se služi sučelje. Svaka izmjena
  ide kroz isti popis za poništavanje, pa se AI-jev rad vraća kao i ručni.
- **Veza:** HTTP, jer `stdin` i `stdout` ne mogu do drugog računala i jer se proces programa
  pokreće rukom, a ne iz AI klijenta.
- **Zadano:** sklopka je isključena pri svakom pokretanju, a uz nju se vidi adresa, je li
  poslužitelj upaljen i tko je spojen.
- **Doseg:** zadano samo ovo računalo. Mreža i drugi poslužitelji uključuju se izričito.
  Veza izvan računala traži pristupni ključ i šifriranu vezu, jer MCP upravlja podacima.
- **Prava:** samo čitanje ili i izmjene, po sloju. Brisanje i izvoz traže potvrdu u programu.
- **Nepouzdan sadržaj:** naziv ili opis iz uvezenog KML-a može sadržavati tekst koji navodi AI
  na radnju. Podatak se AI-ju predaje kao podatak, a prava vrijede bez obzira na to što traži.
- **Privatnost:** što AI pročita, odlazi poslužitelju na kojem taj AI radi. Privatni model ostaje
  kod tebe, a javni ne, pa program uz sklopku mora to jasno reći.

Razgovor s AI-em unutar programa zasad se ne radi, a možda dođe u nekom kasnijem izdanju (Odluke
(7. 11. 2026.)). To je zaseban posao: MCP poslužitelj samo prima, a razgovor bi morao sam zvati
model.

Otvoreno: nacrt sklopke i postavki u FigCraftu, što je prije koda. Vrijeme: naredbe su od faze 1,
a sklopka nakon izdanja 0.0.1.

## Paketi

Imena su na hrvatskom, a jezgra ne ovisi o sučelju. To zamjenjuje popis u
specifikaciji.

```
prozor/       # sučelje: ploče, izbornici, dijalozi
karta/        # prikaz karte i crtanje mišem
podloge/      # WMS, WMTS, XYZ, GeoTIFF, PMTiles, MBTiles, cache
geometrija/   # geometrija i mjere
koordinate/   # koordinatni sustavi
slojevi/      # model slojeva, mapa i stila
formati/      # KML, KMZ, GeoJSON, GeoPackage, SHP
projekt/      # spremanje, automatsko spremanje, oporavak
atributi/     # tablice, CSV, Excel, uvoz izmjena
predlozak/    # model predloška ispisa
ispis/        # PDF i ispis
xtask/        # provjere kvalitete
```

## Što ne ulazi do 1.0

Veza s goCOP-om i plugin API dolaze poslije, a sada se samo pazi da im model sloja ne
smeta. Isto vrijedi za vremenski klizač, reljef i poplavljeno područje, izvoz u QGIS
projekt (`.qgz`) i 3D. Globusa nema.

## Za kasnije, bez žurbe

Pregled besplatnih podloga i njihovih uvjeta je u [podloge.md](podloge.md) (7. 11. 2026.). To je
pregled uvjeta, a ne pravno mišljenje. OpenFreeMap i DGU ostaju ugrađeni, Esri i OpenTopoMap samo
ako ih korisnik doda sam, a Google se ne koristi. Program mora slati svoj `User-Agent` i prikazivati
propisani navod izvora na karti i na ispisu, što `walkers` ne radi sam.

## Polazišta koja proba može srušiti

- Nema GDAL-a ni PROJ-a u programu. Proba je to potvrdila za naših pet sustava. HDKS ide
  kao u QGIS-u, uz natpis o točnosti oko 5 m (vidi probu koordinata). Službena mreža
  DGU-a dolazi samo ako ovo ne bude dovoljno.
- Program radi s `wgpu`, što traži ispravan grafički upravljački program. Na starim
  uredskim računalima treba provjeriti radi li.
- Ekran i PDF crtaju se iz istog popisa naredbi, da se boje i debljine ne raziđu.

## Što treba od korisnika

1. Javna objava tek kad je ime provjereno i kad uz font Inter iz `probe/izgled/fonti`
   stoji njegova licenca (OFL). Repozitorij je zasad privatan na GitHubu (7. 11. 2026.).
2. Glatkoća karte u prozoru, s upaljenim ekranom i pravim mišem.
3. Kasnije odluka o potpisivanju programa za Windows i macOS.
