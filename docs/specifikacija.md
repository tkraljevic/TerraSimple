# Projektna specifikacija — TerraSimple

**Status:** nacrt v1.0 · **Datum:** 25. listopada 2026.
**Radni naziv:** TerraSimple (Odluke (27. 10. 2026.); provjeriti dostupnost imena prije objave)
**Vrsta:** otvoreni kod pod licencom EUPL-1.2, izvorna desktop aplikacija za 2D GIS. Radimo je za svoje potrebe, a tko god želi, može je koristiti i prenamijeniti (Odluke (6. 11. 2026.)).
**Platforme:** Windows, macOS, Linux
**Primarni jezik:** Rust

## 1. Vizija

Izraditi jednostavnu desktop alternativu svakodnevnom radu u Google Earth Prou, s intuitivnim crtanjem i organizacijom objekata, ali s otvorenim GIS podacima, atributnim tablicama, kompatibilnošću s QGIS-om te kvalitetnim ispisom. Korisnik ne mora poznavati GIS da bi nacrtao, izmjerio, označio, spremio i ispisao kartu.

Temeljno načelo: jednostavnost na površini, ispravni GIS standardi u pozadini. Nema obveznog korisničkog računa, pretplate, preglednika ni cloud poslužitelja. Mreža je potrebna samo za udaljene podloge, pretraživanje ili ažuriranja; lokalni projekti rade offline.

## 2. Opseg i granice

### U prvoj stabilnoj verziji (v1.0)

- 2D karta s pomicanjem, zumiranjem, koordinatama kursora i pretraživanjem lokacija (kad je dostupan odgovarajući servis).
- Internetske raster podloge XYZ, WMS i osnovni WMTS; izbor i konfiguracija izvora; lokalni cache prema uvjetima pružatelja.
- Crtanje i uređivanje točaka, linija, poligona; mjerenje udaljenosti i površina; Undo/Redo.
- Organizacija objekata u mape i slojeve, promjena vidljivosti, redoslijeda, boje, širine i prozirnosti.
- Uvoz/izvoz KML, KMZ, GeoJSON i GeoPackage; čitanje SHP-a; očuvanje geometrija i atributa unutar mogućnosti formata.
- Atributne tablice s uređivanjem ćelija, sortiranjem, filtriranjem, višestrukim odabirom i povezivanjem s objektima na karti.
- Spremanje i ponovno otvaranje projekta, autosave i oporavak nakon prekida.
- Vizualni uređivač predložaka ispisa; A4–A0 i prilagođene dimenzije, točno mjerilo, legenda, naslov, slike, sjever, grafičko mjerilo, PDF izvoz.
- Jasna upozorenja kad neki format ne može očuvati stil, atribut, koordinatni sustav ili projektni element.

### Kasnije verzije

- Napredna obrada rasterskih podataka i GeoTIFF, opsežniji WMTS i vektorske pločice, offline MBTiles.
- Izvoz/uvoz QGIS projekta (.qgz) i pretvorba predložaka u QGIS Print Layout, uz definirani podskup podržanih elemenata.
- Napredne prostorne analize, više kartografskih okvira na stranici, serijski/atlas ispis, automatizacija kroz CLI.
- 2.5D/3D prikaz samo ako se kasnije pokaže potreba.

### Izvan opsega v1.0

- Google Street View, globalni 3D globus, 3D gradovi, povijesne Googleove snimke, puni QGIS processing ekosustav, višekorisnički cloud GIS, vlastita globalna satelitska infrastruktura.

## 3. Korisnici i ključni scenariji

Korisnici: građevinari, geodeti, službe zaštite i spašavanja, komunalne službe, poljoprivrednici, studenti i građani.

1. Korisnik otvori kartu, odabere ortofoto, nacrta liniju, vidi duljinu i spremi je bez otvaranja atributne sheme.
2. Korisnik učita postojeći KML/KMZ iz Google Earth Proa, uredi točke i ponovno izveze kompatibilnu datoteku.
3. Korisnik učita GeoPackage iz QGIS-a, pregleda i uređuje atribute te spremi promjene bez gubitka nepovezanih tablica ili polja.
4. Korisnik odabere vlastiti A3 predložak, postavi mjerilo 1:5000, izveze čitljiv PDF i ispiše ga na ploteru.
5. Korisnik bez interneta otvori lokalni projekt i lokalne podloge.

## 4. UX pravila

- **Nacrtaj odmah:** alati Točka, Linija, Poligon i Mjeri uvijek su dostupni. Nema obveznog prebacivanja sloja u način uređivanja za osobne objekte.
- **Klik–klik–desni klik:** klik dodaje vrh, desni klik završava geometriju; Escape odustaje; Backspace uklanja posljednji vrh (Odluke (10. 11. 2026.), prije dvoklik).
- **Uredi izravno:** odabir objekta otvara jednostavan panel; vrhovi se povlače, dodaju i brišu bez posebnog dijaloga.
- **Sigurno uređivanje:** Undo/Redo za geometriju, atribute, stilove i raspored ispisa; upozorenje prije destruktivnih promjena.
- **Poznata organizacija:** ploča **Slojevi** s mapama, slojevima i objektima, zadano desno (Odluke (6. 11. 2026.)); desni panel svojstava po potrebi; tablica se otvara na dnu ili u zasebnom prozoru.
- **Napredne opcije po potrebi:** CRS, sheme i izvori podataka dostupni su, ali ne blokiraju jednostavne radnje.
- **Pristupačnost:** tipkovnički prečaci, skaliranje sučelja, čitljive kontrole. Sučelje je zasad na hrvatskom, a tekstovi su u katalogu, pa se drugi jezik može dodati novom datotekom (Odluke (6. 11. 2026.)).

## 5. Sučelje

Glavni prozor: gornji izbornik i mala traka alata; sredina karta; desno ploča Slojevi sa stablom mapa i slojeva (može se povući lijevo); svojstva po potrebi; opcionalno dolje atributna tablica; statusna traka s koordinatama, CRS-om, mjerilom i stanjem mreže.

Predloženi izbornici: Datoteka, Uredi, Crtanje, Slojevi, Prikaz, Ispis, Alati, Pomoć.

Osnovni tijek crtanja poligona: odabir alata → klikovi po karti → desni klik → automatska površina → naziv u prozorčiću → alat se vraća na Odaberi → autosave. Zadani osobni sloj automatski se kreira ako ne postoji.

## 6. GIS podaci i kompatibilnost

### 6.1 Kanonski podaci

- GeoPackage (.gpkg) preferirani otvoreni spremnik za geometrije i atribute; ne mora biti jedini format projekta.
- KML/KMZ format za razmjenu s Google Earthom: točke, linije, poligoni, nazivi, opisi, mape, osnovni stilovi i ExtendedData gdje je moguće.
- GeoJSON za jednostavnu razmjenu; SHP kao kompatibilni uvozni format uz poznata ograničenja naziva polja i tipova.
- Projektna datoteka (radna oznaka .tgis) pohranjuje stanje sučelja, izvore, stilove, mape, raspored i reference na podatke; ne zamjenjuje GeoPackage.

### 6.2 Koordinatni sustavi

- Podrška za WGS84 (EPSG:4326), Web Mercator (EPSG:3857) i HTRS96/TM (EPSG:3765) od početka.
- Transformacije moraju biti eksplicitne i testirane; KML izvoz u WGS84 koordinatama.
- Ne pretpostavljati CRS kad nije poznat: tražiti potvrdu korisnika.
- Duljine i površine računati odgovarajućom metodom i jasno označiti jedinice/metodu.

### 6.3 Pravila razmjene

- Očuvati identifikatore, geometrije i atribute kad odredišni format to podržava.
- Nepoznate stupce i metapodatke ne brisati tiho.
- Izvoz s mogućim gubitkom informacija mora prikazati pregled ograničenja.
- Round-trip testovi: aplikacija → GeoPackage → QGIS → aplikacija; aplikacija → KML/KMZ → Google Earth Pro → aplikacija.
- Kompatibilnost s QGIS projektom i QGIS Print Layoutom nije isto što i kompatibilnost GeoPackage podataka; nije obećana za v1.0.

## 7. Kartografske podloge

- Početni izvori: konfigurabilni OpenFreeMap (vektorske pločice, vidi [plan.md](plan.md)) ili drugi dopušteni globalni izvor; DGU ortofoto i topografski WMS za Hrvatsku, nakon provjere aktualnih URL-ova, licenci i uvjeta pristupa.
- Korisnik može dodati vlastiti XYZ/WMS/WMTS izvor i odabrati podlogu jednim klikom.
- Cache mora poštovati uvjete pojedinog izvora; nema automatskog masovnog preuzimanja protivno pravilima.
- Jasno prikazivati atribuciju i izvore na ekranu i u ispisu kad licenca to zahtijeva.
- Mrežni neuspjeh ne smije onemogućiti uređivanje lokalnih podataka.

## 8. Atributne tablice

- Jedan objekt = jedan zapis povezan s geometrijom; podrška za različite tipove polja.
- Dvoklik za uređivanje; kopiranje/lijepljenje raspona ćelija; sortiranje, filtriranje, pretraživanje i višestruki odabir.
- Klik u tablici ističe objekt na karti; klik na objekt označava redak.
- Promjene se validiraju prema tipu polja i ograničenjima baze; greške su razumljive korisniku.
- Izračunata geometrijska polja jasno razlikovati od ručno unesenih vrijednosti.

## 9. Uređivač predložaka ispisa

### 9.1 Korisničko iskustvo

- „Ispis” odmah otvara pregled s trenutačnim pogledom karte i posljednjim predloškom.
- „Uredi predložak” otvara vizualni uređivač nalik jednostavnom alatu za prezentacije: povuci, promijeni veličinu, poravnaj, zaključaj.
- Predlošci su korisnički, prenosivi i ponovno upotrebljivi; mogu uključivati ugrađene logotipe i slike.

### 9.2 Elementi

- Okvir karte, tekst, naslov, legenda, grafičko mjerilo, sjever, slika, linija, pravokutnik, datum i izvor podataka.
- Dinamička polja: naziv projekta, datum, mjerilo, CRS i autor (ako je definiran).
- A4–A0 i proizvoljne dimenzije, portret/pejzaž, margine i vodilice.
- Precizno mjerilo i pregled ispisa s provjerom obuhvata karte.

### 9.3 Izvoz

- PDF s vektorskim geometrijama i tekstom gdje je moguće, raster podlogama u kontroliranoj rezoluciji i ugrađenim ili pravilno zamijenjenim fontovima.
- Izravni ispis preko OS-a i postojećih upravljačkih programa pisača/plotera.
- PNG/SVG kao kasnije proširenje; za v1.0 PDF je prioritet.
- Prikaz upozorenja kada udaljena podloga nema dovoljnu rezoluciju ili licenca ograničava izvoz.

### 9.4 Format

- Verzijski format predloška (radna oznaka .tgislayout), strukturirani manifest i resursi; migracije pri promjeni verzije.
- QGIS Print Layout izvoz planirati kao zaseban adapter u kasnijoj fazi, bez tvrdnje o potpunoj identičnosti izgleda.

## 10. Tehnička arhitektura

Predloženi moduli/crateovi (važeći popis, s hrvatskim imenima, je u [plan.md](plan.md)):

```
app/                 # eframe/egui desktop sučelje
map_view/            # 2D prikaz, interakcije, podloge
geometry/            # geometrija, mjerenja, odabir, editiranje
crs/                 # koordinatni sustavi i transformacije
formats/             # KML/KMZ, GeoJSON, GeoPackage, SHP
project/             # spremanje projekta, migracije, autosave
attributes/          # tablice i validacija
layout/              # model predložaka i uređivač
export/              # PDF i sustav ispisa
providers/           # XYZ/WMS/WMTS i pravila cachea
tests/               # integracijski i kompatibilnosni testovi
```

Predložene tehnologije (potvrditi prototipom): Rust, egui/eframe, wgpu ili postojeći renderer karte, geo/GeoRust, SQLite, GDAL/PROJ ako je potreban pouzdan širok skup formata i CRS transformacija, PDF biblioteka s vektorskim izlazom. Izbor Walkers biblioteke ovisi o provjeri WMS/WMTS, slojeva i preciznog ispisa; ne pretpostaviti da sve podržava bez dodatnog rada.

Pravila: GIS jezgra ne ovisi o GUI-ju; ispis koristi isti model geometrija/stilova, ali odvojen renderer za fizičke dimenzije; svi mrežni zahtjevi i veliki uvozi rade asinkrono bez blokiranja sučelja.

## 11. Pouzdanost, sigurnost i kvaliteta

- Autosave s oporavkom, atomarne promjene gdje je moguće, sigurnosne kopije prije migracija.
- Ne mijenjati originalne uvezene datoteke bez potvrde; opcija „Spremi kao”.
- Sigurno parsiranje nepouzdanih KML/KMZ/GeoPackage datoteka; ograničenja veličine, zaštita od ZIP bombi i neočekivanih putanja.
- Testovi preciznosti CRS-a, geometrija, izvoza i ispisa u mjerilu.
- Testovi na Windowsu, macOS-u i Linuxu; automatska izgradnja i regresijski testovi.
- Osnovna telemetrija nije obvezna i zadano je isključena; nema skrivenog slanja korisničkih projekata.

## 12. Kriteriji prihvaćanja v1.0

1. Novi korisnik može nacrtati poligon, imenovati ga i izvesti KML bez kreiranja GIS sheme.
2. KML/KMZ otvoren iz Google Earth Proa prikazuje osnovne geometrije i nazive; izvoz se ponovno otvara u Google Earth Prou.
3. GeoPackage s točkama, linijama, poligonima i atributima otvara se u QGIS-u nakon uređivanja u aplikaciji.
4. EPSG:3765 podaci pravilno se preklapaju s EPSG:3857 podlogom, uz testove kontrolnih točaka.
5. Atributi se mogu uređivati u tablici; odabir na karti i u tablici ostaje sinkroniziran.
6. Korisnik izrađuje i sprema vlastiti A3 predložak te izvozi PDF s naslovom, legendom i mjerilom.
7. PDF ispis u zadanom mjerilu prolazi provjeru fizičke dimenzije na testnoj karti, uz jasno navedene pretpostavke projekcije.
8. Lokalni projekt otvara se bez interneta; mrežni izvori ne blokiraju rad.
9. Nema tihog gubitka podataka pri nepodržanim pretvorbama; prikazuju se upozorenja.
10. Program se instalira i pokreće na sve tri ciljane platforme bez obveznog preglednika ili korisničkog računa.

## 13. Plan razvoja i procjena

| Faza | Sadržaj | Okvirno trajanje |
|---|---|---|
| 0 | PoC: prikaz karte, CRT/CRS test, odabir biblioteka | 2–3 tjedna |
| 1 | Podloge, kamera, jednostavno crtanje, Undo/Redo | 4–6 tjedana |
| 2 | KML/KMZ, GeoJSON, projekt i organizacija objekata | 4–6 tjedana |
| 3 | GeoPackage, CRS, atributne tablice, SHP uvoz | 5–8 tjedana |
| 4 | Predlošci ispisa, PDF, precizno mjerilo | 5–8 tjedana |
| 5 | Integracija, interoperabilnost, UX i beta testiranje | 4–8 tjedana |

Ukupno: približno 6–10 mjeseci kontinuiranog razvoja za ambicioznu v1.0 uz AI pomoć; dulje uz povremeni rad. Prvi upotrebljivi prototip moguć je ranije. Procjene imaju veliku neizvjesnost, osobito za GIS interoperabilnost i ispis.

## 14. Rizici i odluke

- **Licenciranje podloga:** prije isporuke provjeriti uvjete svakog pružatelja, uključujući cache, atribuciju i tisak.
- **Točnost ispisa:** implementirati u fizičkim jedinicama, ne kao snimku ekrana.
- **Kompatibilnost:** razdvojiti „podaci se otvaraju u QGIS-u” od „QGIS projekt i layout identični su”.
- **GDAL/PROJ distribucija:** odlučiti između pakiranja biblioteka i čistog Rusta prema testu pokrivenosti formata i CRS-a.
- **Scope creep:** napredne analize, 3D i puni QGZ odgoditi dok osnovni tijek crtanje → atributi → izvoz → ispis ne bude stabilan.
- **Licenca projekta:** EUPL-1.2 (Odluke (6. 11. 2026.)). Svaka ovisnost mora biti spojiva s njom, a to provjerava `cargo-deny`.

## 15. Odluke i primjedbe uz nacrt

Primjedbe uz nacrt od 27. 10. 2026. Prijedlozi su, a ono što je usvojeno vodi se u [plan.md](plan.md).

1. **HDKS** (Gauss-Krüger, zone 5 i 6) od v1.0. Stari planovi, katastar i DXF-ovi geodeta često su u
   njemu. Transformacija službenom mrežom DGU-a ili barem 7 parametara, uz natpis o točnosti (~1 m).
2. **Bez GDAL-a u v1.0.** WGS84, Web Mercator i HTRS96/TM pokriva čisti Rust. GDAL/PROJ tek za HDKS
   sa službenom mrežom, ako ne ide drukčije, ili za GeoTIFF.
3. **Projekt kao jedan GeoPackage:** stanje sučelja, stilovi i predlošci u vlastitim tablicama, slojevi
   kao obični slojevi, stil u `layer_styles` koji QGIS sam čita.
4. **Slikovni prekrivač u v1.0** (KML GroundOverlay i LatLonQuad): skenirana karta razvučena na kutove.
5. **Pravi `myplaces.kml`** kao kriterij prihvaćanja: mape, NetworkLink, gx-proširenja, stilovi ikona.
   Datoteka ne ide u repozitorij.
6. **Upotrebljivo izdanje prije ispisa:** 0.x s podlogama, crtanjem, KML-om i GeoPackageom već
   zamjenjuje Google Earth Pro većini ljudi; ispis i PDF poslije.
7. **Ekran i PDF iz istog popisa vektorskih naredbi**, da se boje, debljine i natpisi ne raziđu.
8. **Vrijeme** (KML TimeSpan i TimeStamp, vremenski klizač) za kasnije.
9. **Reljef** kao vrsta podloge, nakon v1.0: sjenčanje, boje po visini i izohipse, visina ispod
   pokazivača i visinski profil linije. Izvori: Copernicus DEM, SRTM, DGU, LiDAR.
10. **Karta bez interneta** i iz PMTilesa, ne samo iz MBTilesa.

Odluke (6. 11. 2026.):

- Mjerilo je Google Earth Pro: jednako jednostavno, ali moderno i kompatibilno s QGIS-om. Google
  najavljuje da Earth Pro za računala od 25. 6. 2027. više neće biti dostupan za preuzimanje.
- Umjesto „Moja mjesta” ploča se zove **Slojevi**. Slojevi se slažu u mape, sloj se može napraviti ili
  uvesti postojeći. Ploča je zadano desno, a povlačenjem za glavu može se premjestiti lijevo.
- Podloga se može učitati iz datoteke ili dodati preko WMS-a.
- Atributna tablica može se izvesti u CSV ili Excel, urediti i ponovno uvesti. Retke spaja stupac
  `fid`, izmjene se pregledaju prije primjene, a obrisan redak ne briše objekt.
- Sloj ima jednu vrstu geometrije (točke, linije ili poligoni). KML s miješanim objektima pri uvozu se
  dijeli na više slojeva.
- Gumb „Otvori u QGIS-u” stoji u gornjoj traci. Glavna boja je tirkizna. Globusa nema i ne treba.
- TerraSimple radimo za svoje potrebe, a ne radi popularnosti: tko god želi, može ga koristiti i
  prenamijeniti, a na privlačenje korisnika i suradnika ne trošimo vrijeme. Kod i sučelje su na
  hrvatskom, a ako ga budemo htjeli izložiti široj publici, prepisat ćemo ga na engleski.
- Sučelje se od početka radi tako da se jezik može dodati: tekstovi su u katalogu, podaci ne ovise o
  jeziku, a brojevi i datumi idu preko jedne funkcije. Engleski prijevod se zasad ne radi.
- Stil (vrsta crte, boja, debljina, ikona, natpis) bira se za cijeli sloj ili za jedan objekt.
  Debljine su u milimetrima.
