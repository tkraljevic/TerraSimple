# Primjedbe na specifikaciju i veza s goCOP-om

Razgovor 27. 10. 2026. Ništa od ovoga nije odlučeno; to su prijedlozi uz
nacrt specifikacije.

## Podjela na tri dijela

1. **TerraSimple** je samostalan, otvoren, u Rustu, i ne zna ništa
   posebno o goCOP-u. Ima svoje korisnike i svoj ritam izdanja.
2. **Arhiva goCOP-a** je mali servis bez prozora: čita i stvara .cop
   pakete, sprema ih u hladno spremište (šifrirano, s pravilom čuvanja),
   pazi na rupe u nizu izdanja i povremeno radi probni povrat u odbačenu
   instancu goCOP-a. Prijedlog je da bude u Go-u, unutar goCOP-a, jer
   čitanje i potpis paketa već postoje (`CitajCop`, `IzveziCop`,
   `ProvjeriCopPotpis`). Dva programa koja moraju dati isti otisak i potpis
   najveći su rizik za backup. Paket neka izda čvor, a arhiva ga samo odnese
   i provjeri. Izmjene se ne vraćaju kao novi paket, nego kroz API kao
   obični upisi.
3. **Spoj** neka bude standardni: goCOP nudi **OGC API – Features** za
   prostorne slojeve (nasipi, dionice, letve, objekti, obuhvati). QGIS ga
   čita sam, a TerraSimple dobiva općeniti izvor „OGC API Features”, bez
   posebnog koda za goCOP. Vraćanje izmjena preko dijela 4 tog standarda
   ili preko API-ja goCOP-a; provjeriti koliko QGIS podržava upis preko
   dijela 4. Podaci goCOP-a ostaju iza prijave, a vodostaji nikad javno.

U goCOP-u bi u Administraciji mogla stajati pločica „Arhiva” (zadnji
backup, zadnji probni povrat, broj izdanja) i gumb „Otvori u TerraSimpleu”
za odabrano područje.

## Primjedbe na specifikaciju

1. **HDKS** (Gauss-Krüger, zone 5 i 6) od v1.0. Stari planovi, katastar i
   DXF-ovi geodeta često su u njemu. Transformacija službenom mrežom DGU-a
   ili barem 7 parametara, uz natpis o točnosti (~1 m).
2. **Bez GDAL-a u v1.0.** WGS84, Web Mercator i HTRS96/TM pokriva čisti
   Rust; HTRS96 i WGS84 se u mjerilu karte praktički poklapaju. GDAL/PROJ
   tek za HDKS sa službenom mrežom, ako ne ide drukčije, ili za GeoTIFF.
3. **Projekt kao jedan GeoPackage** umjesto `.tgis` uz podatke: stanje
   sučelja, stilovi i predlošci u vlastitim tablicama, slojevi kao obični
   slojevi, stil u `layer_styles` koji QGIS sam čita. Jedna datoteka.
4. **Slikovni prekrivač u v1.0** (KML GroundOverlay i LatLonQuad):
   skenirana karta razvučena na kutove. Jednostavno, a za stare planove
   neprocjenjivo.
5. **Pravi `myplaces.kml`** kao kriterij prihvaćanja: mape, NetworkLink,
   gx-proširenja, stilovi ikona.
6. **Upotrebljivo izdanje prije ispisa:** 0.x s podlogama, crtanjem, KML-om
   i GeoPackageom već zamjenjuje Google Earth Pro većini ljudi. Moguće za
   3–4 mjeseca; ispis i PDF poslije.
7. **Ekran i PDF iz istog popisa vektorskih naredbi**, da se boje, debljine
   i natpisi na karti i papiru ne raziđu.
8. **Vrijeme** (KML TimeSpan i TimeStamp, vremenski klizač) za kasnije;
   korisno i za goCOP (obilasci, prijave po danu).
9. **Licenca:** sve navedene ovisnosti su dopuštajuće; MIT/Apache-2.0 ili
   GPL tehnički su obje u redu.
10. **Ime:** prije objave provjeriti „TerraSimple” i domenu.

## Buduće mogućnosti (nakon v1.0)

Obje trebaju isto: učitavanje modela reljefa (GeoTIFF, kasnije i LiDAR
LAS/LAZ) i pločice visina.

- **Reljef** kao vrsta podloge: sjenčanje, boje po visini i izohipse koje se
  pale i gase. Izvori: Copernicus DEM i SRTM (30 m), gotove podloge poput
  OpenTopoMapa uz njihova pravila, DGU (provjeriti što nudi), LiDAR. Za
  ravnicu boje razvučene na lokalni raspon i pojačana visina; LiDAR od 1 m
  pokazuje nasipe, stara korita, mrtvaje i udubine. Uz to visina ispod
  pokazivača i visinski profil nacrtane linije, kao u Google Earth Prou.
- **Poplavljeno područje po vodostaju:** razina vode po osi vodotoka (rkm),
  interpolirana između letvi; voda se širi samo kroz povezane ćelije, nasipi
  su granica, a može se prikazati i popuštanje nasipa. S prognozom iz
  goCOP-a animacija kroz vrijeme. Paziti: LiDAR ne vidi ispod vode (korito
  iz profila ili batimetrije), visine moraju biti u istom sustavu kao kote
  nula letvi (HVRS71), i to nije hidraulički model, pa na karti treba
  stajati da je procjena. Izračun u jezgri TerraSimplea, podaci (letve,
  kote, vodostaji, prognoza, nasipi) iz plugina za goCOP.

- **Karta bez interneta i iz PMTilesa, ne samo iz MBTilesa.** goCOP od
  0.0.46 drži vektorsku kartu OpenStreetMapa kao jednu `.pmtiles` datoteku
  (Sektor B oko 120 MB). Kad TerraSimple dobije vektorske pločice bez
  interneta, neka čita i PMTiles, pa ista datoteka služi i goCOP-u i
  TerraSimpleu, a MBTiles ostaje za karte iz drugih alata. Za plugin: goCOP
  zna gdje mu karta leži, pa je plugin može ponuditi kao podlogu.

## Izgled (korisnik, 6. 11. 2026.)

Dizajn Karte u goCOP-u je dobar polazak za izgled TerraSimplea: slojevi desno, ravnalo, ispis u mjerilu, uređivač crta. Uzima se ideja rasporeda, a ne kod.

Nacrt sučelja je u `dizajn/TerraSimple.figcraft`, a slaže ga skripta `dizajn/skripte/izradi.py`. Crta se prije plana i koda.

Odluke (6. 11. 2026.):

- Mjerilo je Google Earth Pro: jednako jednostavno, ali moderno i kompatibilno s QGIS-om. Google najavljuje da Earth Pro za računala od 25. 6. 2027. više neće biti dostupan za preuzimanje.
- Umjesto „Moja mjesta” ploča se zove **Slojevi**. Slojevi se slažu u mape, sloj se može napraviti ili uvesti postojeći.
- Podloga se može učitati iz datoteke ili dodati preko WMS-a.
- Atributna tablica može se izvesti u CSV ili Excel, urediti i ponovno uvesti. Retke spaja stupac `fid`, izmjene se pregledaju prije primjene, a obrisan redak ne briše objekt.
- Crtanje: lijevi klik dodaje točku, povlačenje je pomiče, desni klik je briše, a dvoklik završava crtu ili lik.
- Sloj ima jednu vrstu geometrije (točke, linije ili poligoni). KML s miješanim objektima pri uvozu se dijeli na više slojeva.
- Gumb „Otvori u QGIS-u” stoji u gornjoj traci. Glavna boja je tirkizna.
- Globusa nema i ne treba.
- TerraSimple je otvoreni kod i objavljuje se pod licencom EUPL-1.2, kao i goCOP. Radimo ga za svoje potrebe i ne radi se radi popularnosti: tko god želi, može ga koristiti i prenamijeniti, a na privlačenje korisnika i suradnika ne trošimo vrijeme. Kod i sučelje su na hrvatskom, a ako ga budemo htjeli izložiti široj publici, prepisat ćemo ga na engleski.
- Sučelje se od početka radi tako da se jezik može dodati: tekstovi su u katalogu, podaci ne ovise o jeziku, a brojevi i datumi idu preko jedne funkcije. Engleski prijevod se zasad ne radi.
- Ploča Slojevi zadano je desno, a korisnik je povlačenjem za glavu može premjestiti lijevo.
- Stil (vrsta crte, boja, debljina, ikona, natpis) bira se za cijeli sloj ili za jedan objekt. Debljine su u milimetrima.

## Kvaliteta koda (korisnik, 6. 11. 2026.)

Preuzimaju se smjernice kvalitete iz goCOP-a, uz globalni standard (EMM #2845): jedna naredba za sve provjere, ista i u CI-ju; zapamćeno polazno stanje, pa novi kod ne smije biti lošiji od njega (ne traži se da sve odmah bude savršeno); granica složenosti za nove funkcije; pokrivenost ukupno i posebno za kritični dio (podaci, pretvorba koordinata, uvoz i izvoz, ispis u mjerilu); testovi stvarnog ponašanja, a ne samo redaka; sporije provjere (mutacije, duplikati) kao zasebna naredba.

Alat `dev/quality` iz goCOP-a napisan je za Go, pa se ne prenosi. Prednost imaju standardni alati Rusta, npr. rustfmt, clippy, cargo-llvm-cov, cargo-audit ili cargo-deny, cargo-mutants. Vlastiti alat piše se samo ako standardni nešto stvarno ne može. Točan izbor predlaže plan i potvrđuje korisnik. Pogledati: `Makefile` i `dev/quality` u goCOP-u kao uzor što se mjeri.

Korisnik 6. 11. 2026.: zadržavaju se standardi iz goCOP-a, a imena u kodu su na hrvatskom, kao u goCOP-u. Prijedlog je u [kvaliteta.md](kvaliteta.md).
