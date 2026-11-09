# Rezultat paketa `koordinate`

## Tražilica

U paketu `koordinate` implementirano je robustno prepoznavanje koordinata iz slobodnog teksta u modulu `citanje.rs` (`prepoznaj_koordinate`).

### Što prepoznaje i što ne
- **Prepoznaje:**
  - Decimalne stupnjeve s točkom (`45.5547, 18.6955` i `45.5547 18.6955`).
  - Decimalne stupnjeve s hrvatskim decimalnim zarezom odvojene razmakom ili točkom sa zarezom (`45,5547 18,6955`, `45,5547; 18,6955`).
  - Stupnjeve, minute i sekunde sa stranama svijeta ili bez njih (`45°33'17"N 18°41'44"E`, `45 33 17 S 18 41 44 I`, uz podršku za oznake S, J, I, Z i N, S, E, W).
  - Stupnjeve i decimalne minute (`45°33.28' N 18°41.73' E`).
  - HTRS96/TM koordinate u metrima (istok pa sjever, npr. `676543 5047321`, s točkom ili zarezom za decimale).
  - Negativne predznake (`-` i unicode minus `−`), uključujući predznak `-0`.
- **Što ne prepoznaje / odbija:**
  - Tekst koji ne sadrži koordinatni par vraća `IshodPrepoznavanja::NijeKoordinata` bez panike ili greške.
  - Vrijednosti izvan valjanih raspona (širina iznad 90°, minute ili sekunde >= 60, HTRS96/TM izvan raspona [100.000..900.000, 4.000.000..6.000.000]) vraćaju `IshodPrepoznavanja::IzvanRaspona(razlog)`.
  - HDKS se zasad ne prepoznaje. Odluke (9. 11. 2026.): prepoznaje se sam po rasponu, bez oznake ispred, kao zaseban zadatak nakon spajanja.

### Kako se rješava dvoznačan upis
- Zadani redoslijed za geografske koordinate je širina pa dužina (lat, lon).
- Ako bi zamjena redoslijeda (lon, lat) dala točku unutar granica Hrvatske (42°..47° N, 13°..19.6° E), a upisani redoslijed ne bi, funkcija nudi oba tumačenja u povratnom popisu (`Prepoznato(Vec<PrepoznataKoordinata>)`). Korisnik u padajućem popisu tražilice vidi obje opcije uz jasne oznake.

### Brojke mutacija i namjernih grešaka (krug nakon ispravaka 9. 11. 2026.)
- **cargo mutants na `koordinate/src/citanje.rs`:**
  - Ukupno mutanata: 180.
  - Uhvaćeno (caught): 166 (100 % izvedivih).
  - Preživjelo (missed): 0.
  - Neizvedivo (unviable): 14.
- **Namjerne greške (Fault Injection):**
  - **J bez minusa:** u `odredi_oznaku` uklonjen minus (`s` umjesto `-s` za južnu širinu). Test `citanje::testovi::stupnjevi_minute_sekunde_i_strane_svijeta` odmah je pao s `assertion failed: tocke_se[0].sirina < 0.0`. Kvar vraćen.
  - **Karta koja se ne pomakne:** u `prozor/src/trazilica.rs::primijeni_odabir` isključen poziv `karta.karta().idi_na(*duzina, *sirina, 16.0)`. Test `trazilica_testovi::pretraga_koordinata_pomice_kartu_i_postavlja_privremenu_oznaku` odmah je pao s `panicked at prozor/src/trazilica_testovi.rs: sredina duzina 16.450000000000014, ocekivano 18.6955`. Kvar vraćen.

### Što traži odluku
1. **Odabir HDKS sustava:** Predlaže se da korisnik može eksplicitno zadati sustav prefiksom ili sufiksom u upitu (npr. `HDKS: 5543210 5047321` ili `5543210 5047321 HDKS5`), ili putem izbornika u postavkama/statusnoj traci.
2. **Pretraživanje adresa i naselja preko interneta:** Trenutno isključeno radi privatnosti i rada bez interneta. Potrebno je definirati servis (npr. DGU geokoder ili Nominatim/OSM) i uvjete korištenja.

### Što nije provjereno
- Egzotični koordinatni sustavi izvan WGS84 i HTRS96/TM (npr. UTM zone izvan zone 33N).
- MGI / Gauss-Krüger bez navođenja zone.
