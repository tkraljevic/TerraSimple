# Plan rada

Prijedlog od 6. 11. 2026., potvrđen (Odluke (7. 11. 2026.)). Nacrt sučelja je u `dizajn/TerraSimple.figcraft`,
a slova uz faze (A, B, C…) su njegovi zasloni. Standardi kvalitete su u [kvaliteta.md](kvaliteta.md) i vrijede
od prvog retka. Što smo isprobali i izmjerili, s odlukama koje su iz toga slijedile, je u
[rezultati.md](rezultati.md).

## Stanje

Stanje 10. 11. 2026.:

- **Izdanja:** 0.0.1 do 0.0.5 alfa izašla su 10. 11. 2026. kao probna, za Windows, macOS i Linux. Što je
  ušlo u koje, piše u [PROMJENE.md](../PROMJENE.md). 0.0.5 donosi uvoz iz Google Eartha, ploču Stil,
  brisanje s ⌘ + delete i popis prečaca.
- **Faze:** 0 i 1 su gotove. Faza 2 je gotova, osim sloja koji se čita iz izvorne datoteke. Iz
  faze 3 radi uvoz i izvoz KML-a, KMZ-a i GeoJSON-a, a iz faze 4 stil sloja i objekta i natpisi, bez ikona.
- **Kvaliteta:** provjera `cargo xtask provjera` prolazi, s pokrivenošću oko 96 %.
- **Repozitorij** je javan na GitHubu od 10. 11. 2026. Razvijamo na Rustu 1.99.1.

Rok koji dolazi izvana: Google najavljuje da se Earth Pro od 25. 6. 2027. više neće moći preuzeti. Prvo
upotrebljivo izdanje treba biti gotovo prije toga.

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


## Faze

Trajanje je gruba procjena iz specifikacije, uz dodano što smo dogovorili, i to uz neprekidan rad na ovom
projektu.

### Faza 0: proba (2–3 tjedna) — gotova

Mali programi koji odgovaraju na pitanja, a ne početak programa. Svaka proba ima mjerilo i na kraju se zapiše
odluka. Ako neka padne, plan se mijenja prije faze 1. Rezultati su u [rezultati.md](rezultati.md).

| Pitanje | Prolazi ako |
|---|---|
| Može li `egui` izgledati kao nacrt? | Glavni prozor (A) izgleda kao nacrt, u obje teme, na sve tri platforme. |
| Vuče li `walkers` kartu? | OpenFreeMap sa stilom i PMTiles Sektora B glatko se pomiču. Običan laptop nije uvjet (Odluke (7. 11. 2026.)). |
| Radi li crtanje kako smo rekli? | Lijevi klik, povlačenje i desni klik (do 10. 11. 2026. dvoklik) rade i uz 10 000 vrhova, a povlačenje točke ne pomiče kartu. |
| Radi li DGU WMS kao podloga? | DOF se vidi u mjerilu 1 : 4 500. |
| Jesu li koordinate točne? | Kontrolne točke za 3765, 4326, 3857, 3907 i 3908 slažu se s QGIS-om unutar 1 mm. |
| Čita li QGIS naš GeoPackage? | Slojevi, stupci i stil sloja otvaraju se u QGIS-u, a izmjena u QGIS-u vidi se kod nas. |
| Može li stil jednog objekta u QGIS? | Objekt s vlastitim stilom i u QGIS-u izgleda isto. Ako ne, zapisujemo ograničenje. |
| Čita li se pravi `myplaces.kml`? | Duboke mape, nazivi, opisi i stilovi uvoze se bez gubitka. Datoteka ne ide u repozitorij. |
| Vuče li karta stvarne podatke? (dodano 7. 11. 2026.) | 3 milijuna vrhova iz `myplaces.kml` glatko se pomiču i približavaju. |
| Je li PDF u mjerilu? | A4 izmjeren na papiru odgovara mjerilu, a dijakritika je ispravna. A3 se provjerava bez papira, jer pisača za A3 nemamo. |
| Može li GeoTIFF od 2,4 GB? | Otvara se bez čekanja i bez punjenja memorije. |

Otvoreno: glatkoća karte u prozoru s upaljenim ekranom i pravim mišem.

### Faza 1: karta i crtanje (4–6 tjedana) — gotova

Zasloni A, B, C, D. Podloge OpenFreeMap i DGU, tražilica, crtanje točaka, linija i poligona, mjerenje,
poništavanje, tamna tema. Model sloja od početka zna za vanjske izvore (samo za čitanje, čeka slanje, u
sukobu). Svaka radnja od početka je naredba, a sučelje poziva iste naredbe, jer je to temelj za MCP (vidi
„Upravljanje preko MCP-a”).

**Izdanje 0.0.1 alfa (Odluke (7. 11. 2026.)):** karta, crtanje i mjerenje iz ove faze, uz spremanje u
GeoPackage i otvaranje u QGIS-u iz faze 2. Osnovno spremanje dolazi odmah, jer program bez njega ne služi
ničemu osim isprobavanja.

Napravljeno do 0.0.4 (10. 11. 2026.): sve iz faze, uz to desni klik završava crtež (Odluke (10. 11. 2026.),
prije dvoklik), naziv se upisuje odmah nakon crtanja, a odabir je kao u AutoCAD-u. Rezultati paketa i odluke
o gumbu „Otvori u QGIS-u” su u [rezultati.md](rezultati.md).

Otvoreno: Windows i Linux pravim mišem, HDKS u tražilici (odluka postoji, 9. 11. 2026.) i smije li se
spremiti lik koji siječe sam sebe (sad se sprema uz upozorenje).

### Faza 2: projekt i slojevi (3–4 tjedna) — djelomično

Zasloni A4, F, F2, G, J. Projekt je jedan GeoPackage, s mapama, novim slojem, uvozom GeoPackagea, automatskim
spremanjem, oporavkom i gumbom „Otvori u QGIS-u”.

Napravljeno: mape, novi sloj, premještanje mapa, slojeva i objekata (0.0.4), spremanje u GeoPackage i
„Otvori u QGIS-u”.

**Automatsko spremanje i oporavak (14. 11. 2026.):** dok ima nespremljenih izmjena, program najviše
svakih 30 s sprema kopiju u mapu programa, u pozadini. Projekt se mijenja tek na Spremi, pa se kopija ne
sudara s QGIS-om (Odluke (14. 11. 2026.)). Nakon pada program pri pokretanju nudi Vrati ili Odbaci, a
vraćeni rad Spremi sprema na novo mjesto. Uredno zatvaranje i spremanje brišu kopiju. Svaki pokrenuti
program drži svoju bravu, pa se kopija prozora koji još radi ne nudi.

**Uvoz GeoPackagea (14. 11. 2026.):** „Uvezi sloj” prima i GeoPackage. Prozorčić (zaslon G) nudi slojeve
iz datoteke s kućicama, a odabrani se kopiraju u novu mapu, s mapama, nazivima, opisima i stilom
(Odluke (14. 11. 2026.): zasad samo kopija). Izvorna datoteka se otvara samo za čitanje. Objekt bez
stupca `naziv` uzima naziv iz stupca kao `name` ili `ime`. Ostali stupci se ne kopiraju, jer projekt zasad
pamti samo naziv i opis; prozorčić ih navodi.

Nedostaje: čitanje sloja iz izvorne datoteke, bez kopiranja (zaseban posao).

### Faza 3: Google Earth (3–5 tjedana) — u radu

Zasloni A3, K, L. KML, KMZ, GeoJSON i SHP, pregled gubitaka pri izvozu, pitanje o koordinatnom sustavu,
razmjena s Earth Proom.

- **Uvoz KML-a i KMZ-a (10. 11. 2026., paket `formati`):** vlastiti čitač na `quick-xml`, po probi.
  Odluke (10. 11. 2026.): svaka mapa Google Eartha ostaje mapa, a u njoj su slojevi Točke, Linije i
  Poligoni; točka natpisa uz liniju uvozi se kao zaseban objekt. Stil u ugniježđenom `Document`-u vrijedi
  samo u njemu. Pravi `myplaces.kml` (147 MB): čitanje 0,42 s, uvoz 0,06 s, 340 MB memorije, 251 mapa,
  207 slojeva, 88 901 objekt, ništa odbijeno; spremanje u GeoPackage 1,0 s.
- **Izvoz u KML i KMZ (14. 11. 2026.):** cijeli projekt iz izbornika Datoteka, a mapa ili sloj desnim
  klikom. Prije izvoza prozorčić (zaslon K) kaže što ide u datoteku i što se mijenja: isprekidane crte,
  šrafura, natpisi uz linije, kurziv, drugi koordinatni sustav, vrijednosti koje nisu tekst ni broj.
  Sloj s nazivom iz uvoza (Točke, Linije, Poligoni) ne dobiva svoju mapu, pa se uvezeni KML vraća u
  istom obliku. Stupci iz GeoPackagea idu u `ExtendedData`.
- **GeoJSON (14. 11. 2026.):** uvoz kroz „Uvezi sloj” istim putem kao KML, sa skupnim geometrijama,
  stilom `simplestyle` i starim `crs`-om (HTRS96/TM i HDKS se preračunaju, nepoznat sustav se odbije).
  Svojstva koja projekt ne pamti navedu se nakon uvoza. Izvoz u istom prozorčiću kao KML, s izborom formata
  KML, KMZ ili GeoJSON; mapa i sloj idu u svojstva `mapa` i `sloj`.
- **Uvoz SHP-a i pitanje o sustavu (14. 11. 2026.):** vlastiti čitač `.shp`, `.dbf` i `.prj` (točke,
  skupine točaka, linije i likovi, s visinom ili bez nje, rupe po smjeru prstena). Tekst u `.dbf` čita se kao
  UTF-8, a ako nije ispravan kao kodna stranica 1250, jer `.cpg` zna lagati. Sustav se prepoznaje iz `.prj`
  (HTRS96/TM, HDKS zone 5 i 6, WGS84, Web Mercator). Bez njega, i za GeoJSON s metrima bez `crs`-a,
  prozorčić (zaslon L) pita sustav i predlaže ga po koordinatama. „Drugi sustav (EPSG broj)” još nema.
- Nedostaje: izvoz u SHP.

**Na kraju izlazi 0.1, prvo upotrebljivo izdanje:** zamjenjuje Earth Pro za crtanje, mjerenje i KML, a sve se
otvara u QGIS-u. Do ovdje je 12–18 tjedana rada.

### Faza 4: stil i tablica (4–6 tjedana) — djelomično

Zasloni C2, C3, C4, E, E2, E3. Vrste crta, ikone, natpisi i stil objekta. Atributna tablica, izvoz u Excel i
CSV te uvoz izmjena s pregledom.

Napravljeno (0.0.5): ploča Stil za sloj ili jedan objekt, s bojom, debljinom, prozirnošću, vrstom crte,
šrafurom i veličinom točke; QGIS ih čita iste. Cijelo povlačenje klizača poništava se jednim korakom.
Natpisi (14. 11. 2026.): naziv linije ide uz liniju i prati zavoj, naziv točke uz znak, a naziv lika u
lik. Natpis koji ne stane ili bi prekrio drugi ne crta se, pa se karta kod udaljavanja sama prorijedi.
Kartica Natpis u ploči Stil bira veličinu, boju, kurziv i obrub za cijeli sloj, a QGIS ih čita iste.
Nedostaje: oblici i ikone točaka (karta ih zasad crta kao krug) i atributna tablica.

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

Ukupno 28–45 tjedana neprekidnog rada, odnosno oko 6,5–10,5 mjeseci. Proba je odgovorila na pitanja o karti,
ali ispis u mjerilu i dalje je najnesigurniji dio procjene.

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

Otvoreno: nacrt sklopke i postavki u FigCraftu, što je prije koda. Naredbe postoje od faze 1, a sklopka još
nije rađena.


## Paketi

Imena su na hrvatskom, a jezgra ne ovisi o sučelju. To zamjenjuje popis u specifikaciji.

```
prozor/       # sučelje: ploče, izbornici, dijalozi
karta/        # prikaz karte i crtanje na grafičkoj kartici
geometrija/   # geometrija i mjere na elipsoidu
koordinate/   # koordinatni sustavi
slojevi/      # model slojeva, mapa i stila, naredbe s poništavanjem
geopaket/     # spremanje u GeoPackage i QGIS projekt u njemu
qgis/         # pronalazi QGIS i otvara mu projekt
formati/      # KML i KMZ, a poslije GeoJSON i SHP
xtask/        # provjere kvalitete
```

Još nema, a dolaze po fazama:

```
podloge/      # WMS, WMTS, XYZ, GeoTIFF, PMTiles, MBTiles, cache
projekt/      # automatsko spremanje i oporavak
atributi/     # tablice, CSV, Excel, uvoz izmjena
predlozak/    # model predloška ispisa
ispis/        # PDF i ispis
```

## Što ne ulazi do 1.0

Plugin API dolazi poslije, a sada se samo pazi da mu model sloja ne smeta. Isto vrijedi
za vremenski klizač, reljef i poplavljeno područje, izvoz u QGIS projekt (`.qgz`) i 3D.
Globusa nema.

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

1. Provjera imena „TerraSimple” (žigovi i domena). Repozitorij je javan od 10. 11. 2026., a licenca fonta
   Inter (OFL) je u `resursi/fontovi`.
2. Glatkoća karte u prozoru, s upaljenim ekranom i pravim mišem.
3. Kasnije odluka o potpisivanju programa za Windows i macOS.
