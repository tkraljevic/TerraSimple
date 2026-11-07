# Rezultat probe čitanja KML i KMZ datoteka (`probe/kml`)

**Status:** **PROŠLO** (uz obveznu zamjenu biblioteke `kml` 0.15 vlastitim čitačem nad `quick-xml` 0.42 i `zip` 9.4)

**Pitanje iz Faze 0:** *„Čita li se pravi `myplaces.kml`? Duboke mape, nazivi, opisi i stilovi uvoze se bez gubitka.“*

**Odgovor:** Pravi `myplaces.kml` (147,14 MB; 87.105 objekata; 3.032.552 vrha) i svih 8 dodatnih stvarnih KML datoteka čitaju se u potpunosti i bez ijednog tihog gubitka **našim čitačem nad `quick-xml` 0.42**, dok predložena biblioteka **`kml` 0.15 iz `docs/plan.md` NE ZADOVOLJAVA** jer na stvarnom `myplaces.kml` tiho gubi **2.069 vrhova**, **52 stila**, **1 CDATA opis mape**, **1 `NetworkLink`** i **1 `gx:Track`**, a na praznoj točki (`<Point><coordinates></coordinates></Point>`) ruši cijeli program u paniku (`unwrap`/`remove(0)`).

---

## 1. Preporuka za čitač (`kml` 0.15 naspram `quick-xml` 0.42 + `zip` 9.4)

Isprobali smo oba pristupa na istim izmišljenim i stvarnim datotekama:

| Kriterij | `kml` 0.15 (iz `docs/plan.md`) | Naš čitač (`quick-xml` 0.42 + `zip` 9.4) |
|---|---|---|
| Poligon s `<gx:drawOrder>` prije granice (`myplaces.kml`) | **Tiho gubi cijeli poligon (801 vrh)** jer `_ => break` u `read_polygon` prekida čitanje na prvom nepoznatom elementu | Čita sve vanjske i unutarnje prstenove bez gubitka |
| `<gx:Track>` i `<gx:MultiTrack>` (`myplaces.kml`) | **Tiho gubi geometriju (1.268 vrhova)** i vraća `geometry: None` | Čita sve `<gx:coord>`, `<when>` i `<gx:SimpleArrayData>` |
| Opis mape (`<Folder>` / `<Document>`) u `<![CDATA[...]]>` | **Tiho gubi cijeli opis** jer `read_element` ne obrađuje `Event::CData` | Čuva sav tekst, HTML i hrvatska slova (`čćđšž`) |
| `<NetworkLink>`, `<GroundOverlay>`, `<LookAt>` u `<Folder>` | **Tiho briše sve** jer `read_folder` zadržava samo `name`, `description` i `styleUrl` | Čita ili uredno bilježi svaki element u popisu gubitaka |
| `<Style>` unutar `<NetworkLink>` ili `<Placemark>` | **Ne parsira 52 stila** u `myplaces.kml` (ostaju neobrađeni `Element`) | Parsira svih 722 `<Style>` i 292 `<StyleMap>` |
| `<gx:altitudeMode>clampToSeaFloor</gx:altitudeMode>` | Puca s greškom `InvalidAltitudeMode` | Čita sve KML i `gx:` načine visine |
| Prazna točka `<Point><coordinates/></Point>` | **Ruši proces u paniku** (`props.coords.remove(0)`) | Vraća urednu grešku `PraznaGeometrija` bez ispisa sadržaja |
| KMZ zaštita od ZIP bombe i `../` putanja | Nema zaštitu ni čitanje slika iz KMZ-a | Odbija `..`, vodeći `/` ili `\`, prati omjer kompresije i veličinu te popisuje slike |
| Vrijeme i vršna memorija na `myplaces.kml` (147,14 MB) | **0,394 s** / **296,6 MB** | **0,361 s** / **203,2 MB** (31 % manje memorije) |

**Odluke (7. 11. 2026.):** program ne koristi `kml` 0.15, nego vlastiti čitač na `quick-xml` 0.42 i `zip` 9.4. Čitač iz ove probe (`src/citac.rs`) ima 2.444 retka, bez `unsafe` i bez `unwrap` izvan testova, i u paketu `formati` smije biti jednostavniji. Cijena je kod koji održavamo sami.

---

## 2. Tablica rezultata za sve stvarne i izmišljene datoteke

Svi brojevi našeg čitača u potpunosti se poklapaju s neovisnom Python skriptom (`provjera/referenca.py` nad `xml.parsers.expat`) i s GDAL-ovim `ogrinfo` (`/Applications/QGIS-LTR.app/Contents/MacOS/ogrinfo`). Nijedan naziv, opis ni koordinata iz stvarnih datoteka nisu ispisani.

### Stvarne datoteke (9 datoteka)

| Datoteka | Veličina (MB) | Dok. / Mape | Najveća dubina (mape / sve) | Objekti (`Placemark`) | S nazivom / S opisom | Točke / Linije / Poligoni / Tragovi | Rupe poligona (naš čitač / `ogrinfo`) | Ukupno vrhova (3D != 0) | Stilovi (`Style` + `StyleMap`) | Riješeni stilovi objekata |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `myplaces.kml` | 147,14 | 59 / 192 | 6 / 8 | 87.105 | 8.137 / 86.542 | 1.738 / 1.482 / 85.683 / 1 | **4.287** / **812** | 3.032.552 (190.430) | 722 + 292 | 87.105 (100 %) |
| `stvarna_1` | 0,01 | 1 / 1 | 1 / 2 | 11 | 11 / 11 | 11 / 0 / 0 / 0 | 0 / 0 | 11 (0) | 2 + 1 | 11 (100 %) |
| `stvarna_2` | 0,07 | 1 / 1 | 1 / 2 | 16 | 16 / 16 | 16 / 16 / 0 / 0 | 0 / 0 | 2.128 (0) | 2 + 1 | 16 (100 %) |
| `stvarna_3` | 0,12 | 1 / 1 | 1 / 2 | 44 | 44 / 44 | 44 / 44 / 0 / 0 | 0 / 0 | 2.794 (0) | 2 + 1 | 44 (100 %) |
| `stvarna_4` | 0,13 | 1 / 1 | 1 / 2 | 71 | 71 / 71 | 71 / 71 / 0 / 0 | 0 / 0 | 2.360 (0) | 2 + 1 | 71 (100 %) |
| `stvarna_5` | 0,60 | 1 / 1 | 1 / 2 | 500 | 500 / 500 | 500 / 500 / 0 / 0 | 0 / 0 | 7.449 (0) | 2 + 1 | 500 (100 %) |
| `stvarna_6` | 0,07 | 1 / 1 | 1 / 2 | 18 | 18 / 4 | 0 / 18 / 0 / 0 | 0 / 0 | 1.467 (0) | 36 + 18 | 18 (100 %) |
| `stvarna_7` | 0,01 | 1 / 1 | 1 / 2 | 11 | 11 / 11 | 11 / 0 / 0 / 0 | 0 / 0 | 11 (0) | 8 + 4 | 11 (100 %) |
| `stvarna_8` | 0,05 | 1 / 1 | 1 / 2 | 63 | 63 / 12 | 63 / 0 / 0 / 0 | 0 / 0 | 63 (0) | 32 + 16 | 63 (100 %) |

### Izmišljeni primjeri u `probe/kml/primjeri/` (8 datoteka)

| Datoteka | Namjena | Ishod |
|---|---|---|
| `sve_znacajke.kml` | Sve značajke Google Earth Pro-a: hrvatska slova (`čćđšž`), HTML u `CDATA` na mapi i objektu, `StyleMap` prije `Style`, ikone, boje `aabbggrr`, poligon s više `LinearRing` u istom `innerBoundaryIs`, miješani `MultiGeometry`, 3D visine, `gx:MultiTrack`/`gx:Track`, `NetworkLink`, `GroundOverlay` (`LatLonBox` i `gx:LatLonQuad`), `LookAt`, `Camera`, `TimeStamp`, `TimeSpan`, `gx:TimeSpan`, `ExtendedData` (`Data`, `SchemaData`, `gx:SimpleArrayData`), `gx:Tour`, ugniježđeni `Document` s ponovljenim `Style id` te nepoznata XML oznaka | **PROŠLO** (6 objekata, 32 vrha, 3 rupe, 6+1 stilova, 4 sloja) |
| `duboke_mape.kml` | 15 razina ugniježđenih `<Folder>` mapa (traženo >= 12) s hrvatskim slovima i stilovima | **PROŠLO** (dubina 15/16, 4 objekta, 4 sloja) |
| `mijesani_slojevi.kml` | Podjela mapa s miješanim vrstama geometrija i miješanim `MultiGeometry` na slojeve | **PROŠLO** (3 neprazne mape -> 6 slojeva: 3T, 2L, 1P) |
| `arhiva_sa_slikama.kmz` | KMZ arhiva s `doc.kml`, točkom s ikonom i `GroundOverlay` slikom (2 PNG slike u `files/`) | **PROŠLO** (1 objekt, 1 prekrivač, 2 slike u arhivi) |
| `ostecen.kml` | Neispravna koordinata i nezatvoren XML element | **Uredno odbijeno** (`NeispravnaKoordinata` na bajtu 263) |
| `prazna_geometrija.kml` | `<Point><coordinates></coordinates></Point>` (ruši `kml` 0.15 u paniku) | **Uredno odbijeno** (`PraznaGeometrija` na bajtu 231, bez panike) |
| `zip_bomba.kmz` | Mala KMZ arhiva (~18 KB) čiji se `doc.kml` širi na >16 MB uz omjer kompresije ~900:1 | **Uredno odbijeno** (`PrevelikaDatotekaUArhivi`) |
| `neocekivana_putanja.kmz` | KMZ arhiva s putanjom `../../podmetnuto.km` (izlazak iz mape) | **Uredno odbijeno** (`NeocekivanaPutanjaUArhivi`) |

---

## 3. Podjela na jednogeometrijske slojeve

Prema odluci da svaki sloj u TerraSimpleu ima jednu vrstu geometrije (točke, linije ili poligoni), svaka mapa s miješanim objektima ili miješanim `MultiGeometry` (`Point + LineString`) dijeli se na odgovarajuće slojeve:

| Datoteka | Ukupno spremnika (`Document` + `Folder`) | Nepraznih spremnika | Miješanih spremnika | Objekata s miješanim `MultiGeometry` | `ogrinfo` slojeva | **Ukupno TerraSimple slojeva** | Slojeva točaka | Slojeva linija | Slojeva poligona |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `myplaces.kml` | 251 | 169 | 37 | **1.279** | 250 | **208** | 53 | 44 | 111 |
| `stvarna_1` | 2 | 1 | 0 | 0 | 1 | **1** | 1 | 0 | 0 |
| `stvarna_2` | 2 | 1 | 1 | **16** | 1 | **2** | 1 | 1 | 0 |
| `stvarna_3` | 2 | 1 | 1 | **44** | 1 | **2** | 1 | 1 | 0 |
| `stvarna_4` | 2 | 1 | 1 | **71** | 1 | **2** | 1 | 1 | 0 |
| `stvarna_5` | 2 | 1 | 1 | **500** | 1 | **2** | 1 | 1 | 0 |
| `stvarna_6` | 2 | 1 | 0 | 0 | 1 | **1** | 0 | 1 | 0 |
| `stvarna_7` | 2 | 1 | 0 | 0 | 1 | **1** | 1 | 0 | 0 |
| `stvarna_8` | 2 | 1 | 0 | 0 | 1 | **1** | 1 | 0 | 0 |

---

## 4. Što se gubi za 2D vektorske slojeve u stvarnim datotekama

Naš čitač čuva sve podatke u memoriji (uključujući Z visine, `altitudeMode`, `ExtendedData` i stilove) i izričito broji sve elemente koji nisu dio ravninskih 2D vektorskih slojeva. U svih 9 stvarnih datoteka **nema nijedne nepoznate XML oznake (`nepoznate oznake: 0`)**.

Elementi koji ne ulaze u čist prikaz 2D vektorskih slojeva u stvarnim datotekama:

| Element / pojava | `myplaces.kml` | `stvarna_1`..`stvarna_8` ukupno | Objašnjenje za TerraSimple |
|---|---:|---:|---|
| Vrhovi s 3D visinom (`Z != 0`) | **190.430** (od 3.032.552) | **0** (svih 16.283 imaju `Z = 0`) | Čitač čuva `Z` koordinatu i `altitudeMode` (3.105 `altitudeMode`, 351 `gx:altitudeMode` u `myplaces.kml`) kako se ne bi izgubili pri izvozu ili prikazu visine. |
| `Snippet` | **6.174** | 0 | Kratki izvadak u stablu Google Eartha; čuvamo ga u modelu objekta uz puni `<description>`. |
| `LookAt` / `Camera` | **417** + **1** (te još 39 u `gx:Tour`) | **62** (`1` u `stvarna_1`, `1` u `stvarna_6`, `9` u `stvarna_7`, `51` u `stvarna_8`) | 3D položaj kamere; čitač ga čita u `PogledKamere`, a za 2D slojeve bilježi u izvještaju. |
| `ListStyle` / `BalloonStyle` | **69** / **26** | 0 / 0 | Izgled popisa u bočnoj traci i predložak skočnog balona; `BalloonStyle` čuvamo u modelu stila. |
| `NetworkLink` | **52** | 0 | Vanjske mrežne poveznice na udaljene KML datoteke; čitač ih sprema u `mrezne_veze` i prijavljuje korisniku. |
| `TimeStamp` / `TimeSpan` / `gx:TimeSpan` | **9** / **1** / **3** | 0 / 0 / 0 | Vremenske oznake; čitač ih čuva u `VrijemeKml` (i na objektima i unutar `LookAt`). |
| `atom:link` / `atom:author` | **11** / **10** | 0 / 0 | Metapodaci o izvoru dokumenta. |
| `gx:Tour` | **1** (39 `gx:FlyTo`, 20 `gx:Wait`, 10 `gx:TourControl`) | 0 | Snimljeni 3D prelet kamere u Google Earthu. |
| `prazan_LineString` | **2** | 0 | U `myplaces.kml` postoje 2 `<LineString>` elementa s praznim `<coordinates></coordinates>` koje je Google Earth Pro spremio pri nedovršenom crtanju linije. |

Uz to, `myplaces.kml` sadrži **50 `<ExtendedData>`** elemenata (`19 Data`, `1 SchemaData`, `2 gx:SimpleArrayData` s 2.536 vrijednosti u `gx:Track`), a `stvarna_1`, `stvarna_6`, `stvarna_7` i `stvarna_8` ukupno **29 `<ExtendedData>`** elemenata — sve njih naš čitač uvozi bez gubitka u atribute objekta.

---

## 5. Vrijeme čitanja i vršna memorija za 147 MB (`myplaces.kml`)

Mjerenje je izvedeno u `release` načinu rada na macOS-u (Apple Silicon), svako u svom zasebnom procesu (`ru_maxrss`):

| Čitač | Datoteka | Veličina | Vrijeme čitanja | Vršna memorija procesa (RSS) | Pročitano vrhova |
|---|---|---:|---:|---:|---:|
| **Naš čitač (`quick-xml` 0.42)** | `myplaces.kml` | 147,14 MB | **0,361 s (360,6 ms)** | **203,2 MB** | **3.032.552** (100 %) |
| `kml` 0.15 (iz plana) | `myplaces.kml` | 147,14 MB | 0,394 s (394,0 ms) | 296,6 MB | 3.030.483 (gubi 2.069 vrhova) |

Ostale stvarne datoteke (`stvarna_1` do `stvarna_8`, od 0,01 MB do 0,60 MB) naš čitač učitava za **0,2 ms do 1,7 ms** uz vršnu memoriju **2,3 MB do 3,6 MB**.

---

## 6. Zamke otkrivene na stvarnim datotekama i što znače za TerraSimple

1. **GDAL-ov `LIBKML` i `KML` tiho gube 3.475 rupa poligona u `myplaces.kml`:**
   U `myplaces.kml` nalazi se `85.683` poligona s ukupno `812` oznaka `<innerBoundaryIs>`, ali unutar tih `812` oznaka nalazi se ukupno **`4.287` `<LinearRing>` prstenova** (Google Earth Pro zapisuje više `<LinearRing>` rupa unutar jednog `<innerBoundaryIs>`). GDAL (`ogrinfo` / `osgeo.ogr`) uzima samo prvi `<LinearRing>` iz svakog `<innerBoundaryIs>` (`812` rupa) i **tiho odbacuje 3.475 rupa poligona**! Naš čitač u `procitaj_granicu_poligona` čita sve `<LinearRing>` elemente unutar `<innerBoundaryIs>` i čuva svih `4.287` rupa.
2. **Miješani `MultiGeometry` (`Point + LineString`) je pravilo, a ne iznimka:**
   U `myplaces.kml` postoji **1.279 objekata**, a u datotekama `stvarna_2` (16), `stvarna_3` (44), `stvarna_4` (71) i `stvarna_5` (500) **svi objekti (100 %)** imaju `<MultiGeometry>` koji istodobno sadrži `<Point>` (točku natpisa) i `<LineString>` (trasu ili granicu). Pri podjeli na slojeve s jednom vrstom geometrije takav se objekt mora razdvojiti na točkasti i linijski sloj iste mape uz zadržavanje zajedničkih atributa i stila.
3. **`<StyleMap>` stoji prije `<Style>` i `Style id` se ponavlja u ugniježđenim `<Document>`-ima:**
   U `myplaces.kml` u **396 od 584 `<Pair>` elementa** `<StyleMap>` se nalazi *prije* `<Style>` elementa na koji pokazuje, a u 59 ugniježđenih `<Document>` elemenata pojavljuje se i ponovljeni `Style id`. Zato se stilovi ne smiju rješavati u hodu tijekom čitanja, nego u drugom prolazu nakon čitanja spremnika, s nasljeđivanjem i lokalnim nadjačavanjem po spremniku.
4. **XML entiteti (`&amp;` i sl.) unutar `id` atributa stilova (`stvarna_8`):**
   U datoteci `stvarna_8` 21 `<Style id="...">` / `<StyleMap id="...">` i 23 `<styleUrl>` sadrže XML entitete (`&amp;`). Ako čitač odkodira XML entitete u tekstu `<styleUrl>`, a ne odkodira ih u atributu `id="..."`, stilovi ostaju neriješeni. Korištenje `quick_xml::escape::unescape` na vrijednostima atributa riješilo je 100 % stilova u svim datotekama.
5. **Prazan `<LineString><tessellate>1</tessellate><coordinates></coordinates></LineString>` u `myplaces.kml`:**
   Google Earth Pro u `myplaces.kml` zapisao je 2 `<LineString>` elementa bez vrhova. Čitač zbog toga ne smije srušiti uvoz cijele datoteke od 147 MB, nego takav prazni `LineString` preskočiti pri izradi 2D sloja i prijaviti ga u popisu gubitaka (`prazan_LineString: 2`).
6. **Nevezani XML prefiks u `myplaces.kml`:**
   Na retku 1.261.851 u `myplaces.kml` nalazi se XML prefiks bez `xmlns:` deklaracije, zbog čega strogi XML parseri s provjerom imenskih prostora (npr. Pythonov `ElementTree`) pucaju. `quick-xml` 0.42 to čita bez zastoja.
7. **KML redoslijed bajtova boje je `aabbggrr`, a ne `rrggbb`:**
   Boje u `<LineStyle>`, `<PolyStyle>`, `<IconStyle>` i `<LabelStyle>` zapisane su heksadecimalno kao `aabbggrr` (alfa, plava, zelena, crvena) i pri čitanju se odmah pretvaraju u standardni RGBA.

---

## 7. Provjera da svaka važna provjera pada kad se namjerno pokvari

U `src/primjeri.rs` (`provjeri_da_kvarovi_padaju`) i kroz `./provjera/pokreni.sh` automatski se izvodi 6 namjernih kvarova i potvrđuje da svaki od njih ruši provjeru:

1. **Gubitak druge rupe u istom `<innerBoundaryIs>` (zamka GDAL `LIBKML`-a)** -> pada s porukom: `krive geometrije: tocke=3, linije=2, poligoni=1, rupe=2, tragovi=1`.
2. **Gubitak `CDATA` opisa i hrvatskih slova na mapi (zamka `kml` 0.15)** -> pada s porukom: `izgubljen HTML ili hrvatska slova u CDATA opisu mape`.
3. **Krivo tumačenje KML boje kao `RRGGBB` umjesto `AABBGGRR`** -> pada s porukom: `kriva pretvorba KML boje aabbggrr u RGBA`.
4. **Tihi gubitak Z koordinate (visine) pri pretvorbi u 2D model** -> pada s porukom: `izgubljena visina (Z) ili altitudeMode točke`.
5. **Tihi gubitak nepoznatog XML elementa bez prijave u popisu gubitaka** -> pada s porukom: `nepoznati element NepoznatiDodatak nije prijavljen u gubicima`.
6. **Nerazdvojen miješani `MultiGeometry` (`Point + LineString`) pri podjeli na slojeve** -> pada s porukom: `očekivano mijesanih_visegeometrija == 1, dobiveno 0`.

---

## 8. Što ovom probom nije ispitano

1. **Windows i Linux:** Proba i mjerenje vršne memorije (`ru_maxrss`) izvedeni su samo na macOS-u (Apple Silicon). Na Linuxu `ru_maxrss` vraća kilobajte umjesto bajtova (u `src/memorija.rs` je pripremljen `#[cfg(target_os = "linux")]`, ali nije pokrenut na Linux stroju), a na Windowsu će za mjerenje trebati `GetProcessMemoryInfo`.
2. **Povratni zapis (izvoz) u Google Earth Pro:** Ova proba ispituje isključivo čitanje KML/KMZ datoteka i podjelu na slojeve; zapisivanje natrag u KML/KMZ i provjera otvaranja u Google Earth Pro-u nisu bili predmet ove probe.
3. **Dohvat udaljenih `NetworkLink` datoteka preko mreže:** `NetworkLink` elementi se čitaju i prijavljuju, ali se vanjski URL-ovi ne dohvaćaju preko mreže.
4. **Vrijednosti na stvarnim datotekama:** na stvarnim datotekama uspoređeni su brojevi elemenata i vrhova, a ne svaki naziv i svaka koordinata znak po znak. Vrijednosti su provjerene samo na izmišljenim primjerima, pa „bez gubitka” vrijedi uz tu ogradu.
5. **Tumačenje rupa u Google Earthu:** nije provjereno crta li Google Earth sve prstenove iz istog `<innerBoundaryIs>` kao rupe, nego samo da ih čuva pri spremanju.
6. **Prikaz na karti:** nije provjereno vuče li karta 3 milijuna vrhova. Proba karte rađena je s 10.000.
