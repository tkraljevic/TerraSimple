# Kvaliteta koda

TerraSimple preuzima standarde kvalitete iz goCOP-a (`docs/CODE_QUALITY.md` i
`quality/config.json` u goCOP-u) i globalni standard (EMM #2845). Mijenjaju se
samo alati, jer je goCOP-ov `dev/quality` napisan za Go.

Stanje 7. 11. 2026.: redovna i duboka provjera postoje. Duboka mjeri mutacije
kritičnih paketa i prvi udio ponovljenog koda. Stvarni rezultati i ograničenja su u
`xtask/IZVJESTAJ-DUBOKA.md`. Windows, Linux i stvarni CI još nisu isprobani.

## Cilj

Svaka izmjena provjerava se prije nego što uđe u program, i nijedna ne smije
pogoršati izmjereno stanje. Uspjeh nisu same brojke, nego da sljedeći čovjek
ili agent lakše razumije kod i mijenja ga s manje rizika. Pravila vrijede
jednako za ljudski i AI kod.

## Dijelovi o kojima sve ovisi

To su dijelovi u kojima greška tiho kvari korisnikove podatke ili kartu:

- pretvorba koordinata (`koordinate`);
- geometrija i mjere (`geometrija`);
- uvoz i izvoz (`formati`), posebno čitanje nepouzdanih datoteka;
- spremanje projekta, automatsko spremanje i oporavak (`projekt`);
- atributi: uređivanje i uvoz izmjena iz CSV-a i Excela (`atributi`);
- ispis u mjerilu (`predlozak`, `ispis`).

Sučelje (`prozor`, `karta`) ostaje tanko, a sva pravila o podacima su u jezgri
koja ne ovisi o sučelju.

## Granice

Iste su kao u goCOP-u:

| Što se mjeri | Dijelovi o kojima sve ovisi | Ostali kod |
|---|---:|---:|
| Pokrivenost testovima | 95 % | 85 % |
| Složenost nove funkcije | najviše 7 | najviše 10 |
| Rizik izmjene (CRAP) nove funkcije | najviše 7 | najviše 10 |

Rizik izmjene računa se ovako: `CRAP = CC² × (1 − pokrivenost)³ + CC`, gdje je
pokrivenost udio od 0 do 1.

Dopuštena odstupanja od referentnog mjerenja:

- ukupna pokrivenost smije pasti najviše 0,1 postotni bod;
- rizik izmjene postojeće funkcije smije narasti najviše 0,5, a složenost joj
  ne smije narasti;
- udio ponovljenog koda smije narasti najviše 0,1 postotni bod.

Brojke iz goCOP-a mjerene su drugim alatom i na drugom jeziku:

- Rust mjeri pokrivenost po retcima i područjima, a Go po naredbama.
- `match` i `?` u Rustu mogu složenost brojati drukčije nego grananje u Go-u.

Zato se granice potvrđuju na prvom mjerenju. Ako se pokažu krive za Rust,
promjena se zapisuje s razlogom, a ne radi se potiho.

Kako mjeri `xtask` (Odluke (7. 11. 2026.)):

- **Pokrivenost** se računa iz LLVM-ovih područja, a ne iz redaka. Testni kod ne ulazi u
  mjeru, a funkcija bez područja vrijedi kao nepokrivena.
- **Složenost** broji naš brojač na `syn`, jer `rust-code-analysis-cli` ne čita sav noviji
  Rust, a ipak vraća uspjeh. Osnova je 1. Po 1 dodaju `if`, `for`, `while`, `loop`, `?`, `&&`,
  `||`, `let ... else`, čuvar grane i dodatna alternativa uzorka, a `match` dodaje broj grana
  manje jedan. Zatvaranje se broji u funkciju koja ga sadrži. Nepoznat makro ruši analizu.
- **Kritični kod** nema dopušten stari dug: granice 7 i 95 % vrijede uvijek, i s referencom.
- **Ukupnih 85 %** je apsolutna donja granica, i kad je referenca niža.

## Pravila pisanja koda

1. **Prvo test.** Ponašanje koje se mijenja najprije se zaključa testom.
2. **Male funkcije s imenom koje kaže što rade.** Funkcija se ne dijeli radi
   same brojke kad to otežava čitanje. Tada ide zapisana iznimka.
3. **Greška se ne guta.** Vraća se s opisom gdje je nastala. `unwrap` i
   `expect` ne idu u kod izvan testova, a `let _ =` nad rezultatom traži razlog
   u istom retku.
4. **Nema globalnih tablica postavki.** Što se razlikuje predaje se kao
   parametar.
5. **Komentar kaže zašto, a ne što.** Odluka, izvor ili slučaj iz prakse, s
   datumom.
6. **Imena na hrvatskom**, osim ustaljenih izraza Rusta i GIS-a (Odluke
   (6. 11. 2026.)).
7. **Procjena nikad kao mjerenje.** Uz duljinu i površinu piše metoda, a
   izračunati stupci razlikuju se od upisanih.
8. **Tekst koji korisnik čita piše se cijelim rečenicama i stoji u katalogu, a ne
   u kodu** (Fluent, Odluke (6. 11. 2026.)). Rečenica se ne slaže od dijelova, a
   riječ uz broj ima sve oblike (1 objekt, 2 objekta, 5 objekata). Zasad postoji
   samo hrvatski katalog, a drugi jezik dodaje se novom datotekom.
9. **Podaci ne ovise o jeziku sučelja.** Interne tablice i stupci u GeoPackageu
   imaju stalna imena, pa se projekt otvara isto na svakom jeziku i u QGIS-u.
10. **Brojevi i datumi prikazuju se preko jedne funkcije.** Zadan je decimalni
    zarez. Izvoz u CSV zadano koristi decimalni zarez i točku-zarez kao
    razdjelnik, kako očekuje hrvatski Excel.
11. **Bez `unsafe`.** Naš kod ima `#![forbid(unsafe_code)]`. Iznimka je samo
    nužan spoj s tuđom bibliotekom, u zasebnom modulu, uz razlog.

Složena funkcija dijeli se kao u goCOP-u: prvo testovi na neizmijenjenom kodu,
stara kopija ostaje dok test uspoređuje izlaze, a novi dijelovi odmah moraju biti
unutar granica. Test koji uspoređuje izlaz zaokružuje decimale, jer procesori
zadnje decimale računaju malo drukčije.

## Testovi ponašanja

Pokrivenost kaže da se redak izvršio, a ne da je test provjerio pravu stvar. Zato
kritični dijelovi moraju imati i ove testove:

- **Koordinate:** kontrolne točke za svaki koordinatni sustav, u oba smjera, uz
  zapisanu dopuštenu grešku.
- **Razmjena s QGIS-om:** program → GeoPackage → QGIS → program, bez gubitka
  stupaca i nepovezanih tablica. QGIS i GDAL su samo alat za provjeru.
- **Razmjena s Google Earthom:** program → KML/KMZ → Google Earth Pro → program,
  i pravi `myplaces.kml` s dubokim mapama. Taj se dio radi ručno dok se Earth Pro
  može nabaviti.
- **Uvoz izmjena iz tablice:** krivi `fid`, krivi datum, obrisan redak, novi
  stupac, decimalni zarez i točka, CSV s točkom-zarezom i sa zarezom. Crteži i
  izračunati stupci ne smiju se promijeniti.
- **Nepouzdane datoteke:** oštećen KML, ZIP bomba, neočekivane putanje u KMZ-u.
- **Spremanje:** prekid usred spremanja ne smije izgubiti projekt.
- **Ispis:** fizička veličina stranice i mjerilo na testnoj karti, u milimetrima.

## Alati

Prednost imaju standardni alati Rusta. Projekt je pod licencom EUPL-1.2, pa
`cargo-deny` provjerava i jesu li licence ovisnosti spojive s njom. Repozitorij
je privatan, a kasnije može postati javan. U testne podatke ne ide ništa iz goCOP-a.


| Provjera | Alat | Kada |
|---|---|---|
| Oblik koda | `rustfmt` | uvijek |
| Statička analiza | `clippy`, upozorenja su greške | uvijek |
| Testovi i pokrivenost | `cargo test`, `cargo-llvm-cov` | uvijek |
| Složenost po funkciji | `xtask`, jer `rust-code-analysis` ne čita sav noviji Rust | uvijek |
| Ranjivosti i licence ovisnosti | `cargo-deny` | uvijek |
| Mutacije | `cargo-mutants` | sporo, zasebno |
| Ponovljeni kod | mali brojač u `xtask` (`retci-10-v1`) | sporo, zasebno |
| Čitanje nepouzdanih datoteka | `cargo-fuzz`, još nije uveden | nakon prvog čitača nepouzdanih datoteka |

Go treba zasebnu provjeru istodobnog pristupa. Rust utrke podataka u sigurnom
kodu odbija već pri prevođenju, pa je dovoljan `forbid(unsafe_code)`. Ako se
`unsafe` ipak pojavi, taj modul ide pod `miri`. Zastoje (deadlock) Rust ne
sprječava, pa ih pokrivaju testovi.

**Vlastiti alat piše se samo za ono što standardni ne može:** spojiti pokrivenost
i složenost u rizik izmjene po funkciji i usporediti ih s referentnim mjerenjem.
To je mali paket `xtask` u repozitoriju, koji ostale alate samo pokreće i čita
njihov JSON. Radi jednako na Windowsu, macOS-u i Linuxu, a ne treba `make`.

## Naredbe

```bash
cargo xtask provjera
```

Pokreće sve iz retka „uvijek”, uspoređuje s `quality/baseline.json` i pada kad
padne test, kad statička analiza nađe grešku ili kad izmjena nešto pogorša. Uz
`--mapa <staza>` provjerava drugi Cargo paket.

```bash
cargo xtask zapamti
```

Namjerno zapisuje novo referentno mjerenje, a to radi čovjek nakon pregleda.

```bash
cargo xtask provjera-duboka
```

Mutacije pokreće `cargo-mutants` 27.1.0 nad članovima radnog prostora koje isti
središnji popis u `mjerenje::kriticna` označuje kritičnima. Trenutačno su to
`koordinate` i `geometrija`. Po paketu prvo prolaze neizmijenjeni testovi; zatim se
svaka mutacija prevodi najviše 180 sekundi i testira najviše 60 sekundi. Koriste se
dva radnika i `--locked`, bez dodatne konfiguracije ili filtara mutacija.
Na macOS-u dugo mjerenje pokrenuti kao `/usr/sbin/taskpolicy -b cargo xtask provjera-duboka`.
Na drugim sustavima pokreće se obična naredba.

Preživjela mutacija ruši provjeru i ispisuje datoteku, funkciju i promjenu.
Istek vremena, nepotpun rezultat, pad osnovnih testova ili kvar alata također ruše
provjeru. Mutacija koja se ne može prevesti zasebno se ispisuje; nije uhvaćena
testom, ali sama ne ruši provjeru. Izvorni izlazi ostaju u `.quality/`.

Prvi udio ponovljenog koda samo izvještava i ne mijenja referencu. Broje se
blokovi od najmanje deset jednakih uzastopnih nepraznih redaka, nakon uklanjanja
rubnih razmaka i redaka koji počinju s `//`. Udio je broj označenih redaka kroz
broj svih takvih redaka; preklapanja se broje jednom, a oba pojavljivanja ulaze
u brojnik. Izvori su Cargo produkcijski ciljevi i njihova mapa `src`, uključujući
ugrađene testove. Zasebna mapa `tests` ne ulazi. To nije prepoznavanje semantički
istog koda: drukčija imena i formatiranje mogu sakriti ponavljanje, a blok-komentari
i literali mogu ući u mjeru. O usporedbi budućih mjerenja treba odlučiti nakon pregleda
prvog udjela; granica i referenca sada nisu promijenjene.

Fuzz nije pokrenut niti uveden: još nema paketa koji čita nepouzdane datoteke.

`.github/workflows/kvaliteta.yml` priprema redovnu provjeru za svaki PR i slanje
u `master`, na Windowsu, macOS-u i Linuxu. `duboka.yml` priprema tjedno i ručno
pokretanje na Linuxu. Oba koriste predmemoriju i ograničeno trajanje. Lokalno je
provjerena sintaksa YAML-a; stvarni CI i druge platforme nisu pokrenuti.

## Referentno mjerenje i iznimke

- Referentno mjerenje pomiče se samo namjerno, nakon pregleda, u zasebnoj izmjeni
  koja riječima kaže što se promijenilo i zašto. Nikad automatski nakon pada
  provjere.
- Iznimke su u `quality/config.json`, svaka s razlogom, odgovornom osobom i
  rokom. Istekla iznimka ruši provjeru.
- Ekvivalentna ili neopažljiva mutacija može se odobriti samo pregledanom iznimkom
  `mutacija:<točno ime iz cargo-mutants>`, s istim razlogom, osobom i rokom.
  Rok i potpunost provjeravaju se prije pokretanja. Ne koriste se skrivene oznake
  preskakanja u izvornom kodu ili zasebne postavke cargo-mutants.
- Iznimka se ne dodaje samo da bi provjera prošla, a posebno ne za grešku u
  podacima ili uvozu.
- Izvještaj nikad ne označava neizvršenu provjeru kao prošlu.

## Otvoreno

- **Testovi sučelja:** provjeriti može li `egui_kittest` pokriti crtanje mišem
  (lijevi klik, povlačenje, desni klik, dvoklik).
- **Granice za Rust:** potvrditi na prvom mjerenju programa.
- **Proturječje u granicama kritičnog koda:** uz složenost 7 i pokrivenost 95 % rizik je
  7,006, pa granica rizika 7 u tom slučaju traži 100 % pokrivenosti. Odlučiti na prvom
  mjerenju programa, a dotad granice ostaju kako pišu.
