# Rezultat paketa `geometrija`

Mjere na elipsoidu opisane su u `docs/plan.md`, u rezultatima paketa `geometrija`.

## Lik koji siječe sam sebe (9. 11. 2026.)

Liku koji siječe sam sebe površina je razlika njegovih dijelova, kao u QGIS-u, pa osmica ima površinu
0. `presjek_lika` (`src/presjek.rs`) kaže siječe li lik sam sebe i daje jedno mjesto presjeka, da ga
karta može označiti.

**Što je presjek.** Računa se u ravnini dužine i širine, kao GEOS u QGIS-u za sloj u EPSG:4326.

- Unutar istog prstena presjek je svaki dodir rubova koji nisu susjedi: križanje, vrh na tuđem
  rubu, isti vrh dvaput i rubovi koji se preklapaju na istom pravcu. Presjek je i rub koji se vraća
  po sebi (šiljak).
- Između prstenova presjek je križanje i preklapanje. Dodir u jednoj točki je dopušten, kao u
  GEOS-u. Drugo mjesto dodira istih prstenova je presjek: prsten tada prelazi drugi kroz vrhove ili
  dijeli unutrašnjost lika na dva dijela.
- Ista točka ponovljena zaredom nije presjek, samo rub duljine 0, pa se izostavlja. Zatvarajuća
  točka i smjer crtanja ne mijenjaju rezultat. Manje od tri različite točke još nije lik.
- Lik preko 180. meridijana u ravnini izgleda kao da obilazi Zemlju i može dobiti lažan presjek.
  Takvih likova u našem radu nema.

**Usporedba s QGIS-om.** `provjera/presjek.py` daje GEOS-u iz QGIS-a 3.44.15, bez prozora, 415
izmišljenih likova, od toga 312 neispravnih. Tu su ručni rubni slučajevi i 400 nasumičnih, pola na
mreži cijelih brojeva da bude dodira i preklapanja. Test `tests/presjek_qgis.rs` slaže se s GEOS-om
na svima, osim devet „Hole lies outside shell”: GEOS takav lik smatra neispravnim, ali to nije
presjek ruba. Prvi krug je našao jednu razliku, tri lika u kojima rub prolazi kroz dva vrha rupe;
iz toga je pravilo o drugom mjestu dodira.

**Brza i spora inačica.** Rubovi se poredaju po najmanjoj dužini i uspoređuju samo s onima koji se
s njima preklapaju po dužini. Spora inačica u testovima uspoređuje svaki rub sa svakim; obje se
slažu na 10 000 nasumičnih likova.

**Brzina** (`examples/brzina_presjeka.rs`, izdanje, Apple M4 Pro), lik bez presjeka, dakle cijeli
pregledan:

| Vrhova | Krug | Nazubljen rub |
|---:|---:|---:|
| 10 000 | 0,2 ms | 0,1 ms |
| 100 000 | 2,3 ms | 1,4 ms |
| 1 000 000 | 23,7 ms | 14,4 ms |

Rast je linearan. Lik u kojem je mnogo dugih rubova iznad istog mjesta po dužini, npr. češalj s
visokim zupcima, bio bi sporiji; takav nije mjeren.

**Namjerne greške** na `src/presjek.rs`: 125 uhvaćeno, 0 preživjelo, 4 neizvedivo, 0 isteklo. Prvi
krug ostavio je jednu istovjetnu (`>` u `>=` u provjeri šiljka, gdje umnožak može biti 0 samo za rub
duljine 0) i tri istekle (raspon po dužini zamijenjen stalnim brojem: račun ostaje točan, ali spor,
pa je test s 20 000 vrhova bez optimizacije istekao). Uvjet šiljka prepisan je bez umnoška, a test
s krugom smanjen je na 4 000 vrhova; brzinu mjeri primjer.

**Traži odluku:** smije li se lik koji siječe sam sebe spremiti. Sad se sprema, uz upozorenje.

**Nije provjereno:** stvarni likovi iz rada, običan laptop, lik s vrlo bliskim vrhovima gdje račun s
brojevima s pomičnim zarezom može dati drukčiji odgovor od GEOS-a.
