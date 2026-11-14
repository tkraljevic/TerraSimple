> Raniji izvještaj prve redovne provjere. Aktualna duboka provjera i CI opisani su u [IZVJESTAJ-DUBOKA.md](IZVJESTAJ-DUBOKA.md).

# Provjera kvalitete — 7. 11. 2026.

Naredba je napravljena i isprobana na macOS-u. Sam `xtask` ima 18 prolaznih
testova, 90,86 % pokrivenih produkcijskih područja, najveću složenost 9 i najveći
CRAP 9,014. Formatiranje, Clippy, testovi, mjerenje i provjera ovisnosti završili
su uspješno. Cijela `provjera` ipak vraća **1**: početna referenca još nije
potvrđena. To nije prikazano kao prolaz.

`quality/baseline.json` namjerno sadrži prazan popis mjerenja. Referencu
čovjek namjerno zapisuje posebnom naredbom. Nakon pregleda
izmjerenih podataka, početna referenca potvrđuje se ovako:

```text
cargo xtask zapamti
cargo xtask provjera
```

Prva naredba ponovno pokreće alate i zapisuje mjerenje samo ako oni uspiju.
Ispisuje i metričke nalaze: zapis reference sam po sebi ne znači da su granice
zadovoljene. Druga naredba ništa ne zapisuje u referencu. U pokusima je
`zapamti` pokrenut samo u privremenoj kopiji projekta.

## Što je dodano

- Rust radni prostor s jedinim članom `xtask`; probe ostaju zasebni paketi.
- Cargo alias, pa se naredba pokreće kao `cargo xtask provjera`.
- `--mapa <staza>` za provjeru drugog Cargo paketa ili radnog prostora.
- `quality/config.json`, `quality/baseline.json` i konzervativni popis
  dopuštenih licenci u `quality/deny.toml`. Nepoznata licenca ruši provjeru.
- Hrvatski katalog poruka u `xtask/prijevodi/hr.ftl`.
- Zasebna mapa `.quality/<proces>-<vrijeme>/` za svaki poziv, s izvornim LLVM
  JSON-om, spojenim mjerenjem i izlazima pojedinih provjera. `.quality/` je
  dodan u `.gitignore`.

Naredba pokreće `cargo fmt --all -- --check`, Clippy nad cijelim odabranim
radnim prostorom i svim ciljevima uz `-D warnings`, testove, `cargo llvm-cov`
s JSON izlazom i `cargo deny` s JSON dijagnostikom. Cargo čitanje metapodataka,
testovi, Clippy, pokrivenost i provjera ovisnosti koriste `--locked`.
Nedostupan alat ili komponenta `llvm-tools` znači pad s uputom za instalaciju.
Ne koristi se ljuska ni `make`.

`cargo xtask provjera-duboka` vraća **1** i kaže:

```text
Duboka provjera još nije napravljena. Mutacije, ponovljeni kod i fuzz nisu provjereni.
```

## Kako se mjeri i uspoređuje

Pokrivenost se računa iz LLVM-ovih **područja**, a ne iz broja pozvanih
funkcija. Područja se povezuju sa sintaksnim rasponom funkcije i izvornom
datotekom. Ista područja različitih instanciranja generičke funkcije spajaju
se: dovoljno je da je područje izvršeno u jednom instanciranju. Testne
funkcije, moduli `#[cfg(test)]` i datoteke `#![cfg(test)]` ne ulaze u mjeru.
Funkcija bez LLVM područja dobiva 0 % i zasebnu poruku, nikad izmišljeni prolaz.

Ukupni udio ponderiran je brojem područja. Funkcija bez područja doprinosi
jedno nepokriveno područje. Ugniježđene imenovane funkcije mjere se zasebno;
njihova područja ne pripisuju se vanjskoj funkciji. Zatvaranja ostaju dio
funkcije koja ih sadrži, i za složenost i za pokrivenost.

Identitet je relativna putanja i ime funkcije, uključujući modul, tip i trait
kod metoda. Brojevi redaka nisu dio identiteta. Referenca odvojeno čuva
mjerenja različitih Cargo paketa/radnih prostora. Putanje se spremaju s `/`.

Za nove funkcije vrijedi CC/CRAP najviše 10, odnosno 7 za kritični kod.
Postojećoj nekritičnoj funkciji CC ne smije narasti, a CRAP smije narasti
najviše 0,5. Kritični kod uvijek ima granice 7 i najmanje 95 % po funkciji.
Prepoznaju se nazivi iz specifikacije u paketima, putanjama i unutarnjim
modulima; `geopaket` se također smatra kritičnim uvozom/izvozom.

Ukupna pokrivenost ima donju granicu 85 %. Pad prema referenci veći od
0,1 postotnog boda ima i vlastiti nalaz `pad-pokrivenosti:ukupno`.
CRAP se računa točno prema zadanoj formuli. Metoda mjerenja zapisana je u
JSON-u; neusklađena metoda ili nevaljana referenca ruši usporedbu.
Duplikacija se zasad ne mjeri: pripada dubokoj provjeri koja uvijek pada.

Iznimke imaju točan ključ nalaza, razlog, odgovornu osobu i datum roka.
Ne postoji opća iznimka za pad testa, sigurnosne provjere ili nedostajući alat.
Primjer oblika, koji nije dodan u stvarne postavke:

```json
{
  "iznimke": [{
    "kljuc": "slozenost:src/primjer.rs::ucitaj",
    "razlog": "Ovdje treba stvarni pregledani razlog.",
    "odgovorna_osoba": "Ime odgovorne osobe",
    "rok": "2026-12-01"
  }]
}
```

Rok vrijedi zaključno s navedenim lokalnim datumom. Provjeravaju se i iznimke
koje se trenutačno ne koriste. Prazni obvezni podaci, nevaljan datum ili
ponovljeni ključ također ruše provjeru. Stvarni popis iznimaka ostao je prazan.

## Odluke (7. 11. 2026.): brojenje složenosti

Instalirani `rust-code-analysis-cli` 0.0.25 ne čita sav podržani Rust. Na
ovom kodu Rust 1.99.1 uspješno je izgradio biblioteku:

```rust
pub async fn novi() {
    let f = async || 3;
    let _ = f().await;
}
```

Ali `rust-code-analysis-cli -p async.rs -f ERROR` ispisao je `ERROR` na
`async` ispred zatvaranja, uz izlazni kod **0**. Zato njegova uspješna naredba
ne dokazuje da je mjerenje valjano. Zato se koristi
`syn` i vlastiti mali brojač, uz testove novije sintakse.

Početna složenost je 1. `if`, `for`, `while`, `loop`, `?`, `&&`, `||`,
`let ... else`, čuvar grane i svaka dodatna alternativa uzorka dodaju 1.
`match` dodaje broj grana umanjen za jednu. Komentari i tekstualni literali
nisu grananje. Broje se i izrazi u poznatim makroima, uključujući `vec!`,
`matches!`, formatiranje, `params!` i `json!`. Sintaksni makroi `syn`-a za
čitanje graničnika dodaju po jednu odluku. `quote!` sadrži podatke o kodu,
pa se njegov predložak ne broji kao izvršivi kod.

Nepoznat funkcijski makro ili nečitljiva sintaksa ruše analizu. Makroi se ne
proširuju kao u prevoditelju. Kod proizveden proceduralnim makroima nema
potpuno zasebno mjerenje. Pregledavaju se Rust datoteke pod `src` i izričito
navedeni produkcijski Cargo ciljevi; vanjski moduli izvan tih mjesta i
izvedenice uvjetnog prevođenja zahtijevaju dodatnu provjeru prije proširenja
strukture programa. Neaktivni izvorni kod ne dobiva automatski pokrivenost.

## Stvarne probe

| Odabrana mapa | Funkcije | Pokrivenost područja | Izlazni kod | Stvarni nalazi |
|---|---:|---:|---:|---|
| `probe/koordinate` | 28 | 37,82 % | 1 | Rustfmt traži drukčiji oblik. Pet testova prolazi; Clippy i cargo-deny prolaze. |
| `probe/geopaket` | 41 | 31,22 % | 1 | Rustfmt, dva Clippy nalaza i dva sigurnosna nalaza ovisnosti. Četiri Rust testa prolaze. |
| `xtask`, zadana mapa | 64 | 90,86 % | 1 | Svi alati i metričke granice prolaze; nedostaje početna potvrđena referenca. |

GeoPackage ima `clippy::useless_format` u `src/paket.rs:127` i
`clippy::type_complexity` u `src/paket.rs:202`. Cargo-deny prijavljuje
`RUSTSEC-2026-0194` i `RUSTSEC-2026-0195` za `quick-xml`. To su nalazi alata
nad ovisnostima, a ne dokaz iskorištivosti baš u ovoj probi.
Probe nisu popravljane. Njihova pokrivenost također ne zadovoljava ciljeve;
brojke nisu proglašene dovoljnima zato što postojeći testovi prolaze.

Cijeli izlazi i mjerenja čuvaju se lokalno u `xtask/dokazi/`, izvan repozitorija, jer sadrže
lokalne staze: koordinate, GeoPackage,
xtask i 18 testova xtask-a.
Testovi same naredbe koriste i probni izvršivi alat za provjeru obrade padova;
sljedeći pokusi zasebno koriste stvarne Cargo alate.

## Šest namjernih padova

Pokus je napravljen u `/tmp/terrasimple-pokus-kvalitete`, s kopijom paketa
`xtask`, postavki i cijelog izvornog koda probe koordinata. Kopija je najprije
formatirana i u njoj je izričito pokrenut `zapamti --mapa proba`.
Izvorna proba ima stare metričke nalaze čak i uz referencu: kritične granice
ostaju stroge. Zato je za svaki pokus provjeren i **njegov konkretni novi
nalaz**, uz nenulti izlaz, a između pokusa izvorni tekst kopije je vraćen.
Nijedan pokus pada nije promijenio referentnu datoteku; uspoređeni su bajtovi.

| Pokus | Jedina namjerna promjena kopije | Izlaz |
|---|---|---:|
| Pokvaren test | Dodan `#[test]` s `assert_eq!(1, 2)`. | 1 |
| Clippy | Dodana funkcija koja vraća `a == true`. | 1 |
| Složena funkcija | Dodana funkcija s 13 grananja po mjerenju. | 1 |
| Pad pokrivenosti | Pet postojećih testova označeno je s `#[ignore]`. | 1 |
| Istekla iznimka | U kopiju postavki upisana iznimka s rokom `2000-01-01`. | 1 |
| Nedostajući alat | Zaseban `CARGO_HOME` i putanja bez cargo-deny, uz dostupne ostale alate. | 1 |

Sačuvani stvarni izdvojeni izlazi:

```text
test namjeran_pad ... FAILED
```

Clippy zapis sadrži dijagnostiku `bool_comparison`. Ostali stvarni retci:

```text
Provjera slozenost nije prošla na mjestu src/main.rs::namjerno_slozena: izmjereno je 13,0000, a granica je 7,0000. [slozenost:src/main.rs::namjerno_slozena]
Provjera pad-pokrivenosti nije prošla na mjestu proba_koordinate: izmjereno je 37,7315, a granica je 0,1000. [pad-pokrivenosti:ukupno]
Iznimka rizik:primjer istekla je 2000-01-01.
```

Puni dijagnostički retci za složenost, pad pokrivenosti i istek iznimke nalaze
se u pripadajućim lokalnim zapisima:
test, Clippy,
složenost,
pokrivenost,
istek, nedostajući alat.

Odvojena mala kontrolna proba s jednom funkcijom i testom obiju grana,
bez ovisnosti, nakon izričitog `zapamti` prošla je **iste stvarne alate**,
uz izlazni kod **0**:

```text
Redovna provjera je prošla. Duboka provjera nije izvršena.
```

Time je provjeren i prolaz, a ne samo odbijanje. Lokalno su sačuvani
kontrolni prolaz i
pad duboke provjere.

## Okruženje i ograničenja

Mjereno je na macOS-u 26.6.2, Apple Silicon, uz Rust/Cargo 1.99.1.
Instalirani su `cargo-llvm-cov` 0.9.1, `cargo-deny` 0.20.2 i
`rust-code-analysis-cli` 0.0.25, naredbama `cargo install ... --locked`.
`llvm-tools`, rustfmt i Clippy već su bili instalirani. Nije instaliran
drugi alat. Cargo je preuzeo ovisnosti novog paketa i cargo-deny bazu.

Windows i Linux **nisu isprobani**. Nisu isprobani mutacije, fuzz,
duplikacija, svi oblici Rust sintakse, prekid procesa usred zapisivanja
reference ni istodobna dva poziva `zapamti`. Naredba čeka završetak vanjskih
alata; ne uvodi vlastiti rok za test koji se zaglavi.


## Što predlažem dopisati u docs/kvaliteta.md

1. Aktualizirati početno stanje: Rust sada postoji i prva mjerenja su
   provedena, uz jasno odvajanje prolaznih testova od pada cijele provjere.
2. Izričito odabrati područja kao mjeru za CRAP i ukupni prag. Ne uspoređivati
   bez objašnjenja te postotke s Go naredbama ili s LLVM retcima.
3. Zapisati gornja pravila brojenja `match`, `?`, uzoraka, zatvaranja i
   makroa te ograničenja izvornog sintaksnog stabla.
4. Razjasniti da strogi CRAP 7 uz CC 7 matematički traži 100 % pokrivenosti;
   pokrivenost 95 % daje CRAP 7,006125. Granice nisu potiho promijenjene.
5. Razjasniti da je ukupnih 85 % apsolutna donja granica i kad je stara
   referenca niža, te da kritični kod nema dopušten stari dug preko granica.
6. Navesti da duboka provjera, uključujući toleranciju duplikacije od
   0,1 postotnog boda, još ne postoji i uvijek vraća grešku.

Na ovoj maloj količini koda nema dovoljno dokaza za promjenu granica 7/10.
`xtask` ih zadovoljava bez iznimaka. Kod proba treba pregledati u njihovu
opsegu, a ne prilagođavati pragove da bi postojeće probe prošle.
