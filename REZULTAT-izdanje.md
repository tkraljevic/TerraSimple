# Priprema izdanja 0.0.1 alfa

Radna grana: `paket/izdanje`. Bez commita, pusha i oznake. Glavna radna mapa,
`aplikacija.rs`, zabranjeni paketi i referenca kvalitete nisu mijenjani.

## Što je pripremljeno

- `.github/workflows/izdanje.yml`: ručno pokretanje ili oznaka `v*`; macOS
  univerzalni `.app` preko `lipo` u ZIP-u, Windows x86_64 EXE u ZIP-u i Linux
  x86_64 program s ikonom i `.desktop` datotekom u tar.gz-u.
- Artefakti ostaju u privatnom repozitoriju. Samo oznaka koja odgovara inačici
  programa smije stvoriti nacrt izdanja. Skripta odbija mijenjati već objavljeno
  izdanje; ništa ne objavljuje i ne mijenja privatnost repozitorija.
- `skripte/izdanje.py` obavlja istu gradnju, licence, pakiranje i provjeru lokalno
  i u CI-ju. Gotovi program mora potvrditi nacrtani okvir i vratiti kod 0.
- `prozor/src/pokretanje.rs`: ikona prozora, inačica iz Cargo.toml, početak
  programa i skrivena provjera bez mreže. Windows program nema prozor konzole.
  Windows resurs i ICO dodani su preko `winresource 0.1.34`; `cargo-deny` prolazi.
- `resursi/ikone/izdanje/`: ICNS i ICO izvedeni iz postojećih PNG ikona.
  Ponovljivu pretvorbu obavlja `skripte/ikone_izdanja.py`.
- Svaki paket dobiva EUPL, hrvatski tekst, Inter OFL, Lucide ISC, izvore karte
  i HTML popis licenci iz Cargo.lock. `cargo-about --locked --fail` i
  `cargo-deny` koriste isti popis dopuštenih licenci iz `quality/deny.toml`.
- `docs/izdanje.md`: preuzimanje, pokretanje u korisničkoj mapi i upozorenja
  za nepotpisan program, bez isključivanja zaštite cijelog računala.

## Profil izdanja i mjerenja

Zadržano je samo `strip = "symbols"`. LTO i broj jedinica prevođenja ostaju zadani.

| Mjerenje | Bajtova programa | Trajanje |
|---|---:|---:|
| Početna puna gradnja, prije dovršetka provjere pokretanja | 24 901 424 | 1 184,532 s |
| Dovršeni kod, zadani profil, već prevedene ovisnosti | 24 901 824 | 58,248 s |
| Isti kod, završno povezivanje sa `-C strip=symbols` | 20 371 600 | 65,800 s |

Posljednja dva mjerenja pokazuju smanjenje od 4 530 224 bajta (18,19 %).
To nije usporedba dviju čistih punih gradnji: koriste već prevedene ovisnosti,
a trajanja uključuju čekanje na Cargo i usporedni rad na računalu. Ne pokazuju
pouzdano ubrzanje ili usporenje. Mjerenja su u `.quality/profil-*-velicina.json`.

## Izvedene provjere

- Izvršna ARM datoteka prije konačnog pakiranja: potvrda prvog okvira `true`,
  izlaz 0, 0,985 s. Namjerno pokvaren prvi okvir: potvrda `false`, izlaz 1,
  0,345 s. Oba pokretanja sama su završila; vanjska granica nije ih gasila.
- Prvi pokušaj otkrio je da nespremljeni izmišljeni primjer sprečava zatvaranje.
  Program je sam izašao kodom 1 na vlastitoj granici od 12 sekundi. Popravak
  označava spremljenim samo privremeni primjer provjere i ništa ne zapisuje.
- Provjera slike čita GPU snimku, traži prikazanu kartu, veličinu slike i više
  od 16 boja. To nije ručni vizualni pregled niti usporedba sa zlatnom slikom.
- `cargo-about 0.9.3` stvarno je izradio popis licenci od 393 874 bajta;
  provjereno je da sadrži i `winresource 0.1.34` te grafičke biblioteke.
- Šest Rust testova provjere pokretanja prolazi. Provjeravaju zastavice,
  isključenu mrežu, sliku, ishod, rok i slijed četiri prolaza bez pravog prozora.
  Test slijeda koristi kontrolirano vrijeme; program koristi stvarni sat.
- Šest Python testova prolazi: rok i pad procesa, sadržaj arhiva, obvezna
  potvrda pokretanja te zaštita oznake, objavljenog izdanja i stvaranja nacrta.
  Test isteka namjerno prekida izmišljeni proces spavanja nakon 0,1 s.

## Ograničenja i odluke

- Odluke (9. 11. 2026.): korisnik je odbio dodavanje Intel Rust cilja.
  Lokalni paket stoga je samo ARM i nosi oznaku `arm64-proba`. Univerzalni
  paket ostaje u CI-ju; lokalno ga nije moguće potvrditi s instaliranim ciljevima.
- Predloženi identifikator je `org.terrasimple.TerraSimple`; čeka potvrdu. Domena
  `terrasimple.org` nije provjerena, a README traži da se domena i ime provjere
  prije javne objave.
  Može se zadati preko `TERRASIMPLE_BUNDLE_ID`.
- Pripremljeni oblici su ZIP za Mac/Windows i tar.gz za Linux. Izbor DMG-a ili
  AppImagea ostaje za potvrdu. Potpisivanje i Appleova ovjera nisu uvedeni.
- `actionlint` nije instaliran. YAML je pročitan parserom i pregledana je matrica
  triju sustava, ali to ne zamjenjuje actionlint ni stvarno pokretanje tijeka.
- GitHub tijek, univerzalna gradnja, stvarni Windows/WARP i Linux/Mesa prozor,
  Windows ikona i odsutnost konzole te upozorenja pri preuzimanju nisu provjereni.
  CI ima te provjere pokretanja i ne preskače njihov pad.
- Lokalno je instaliran samo `cargo-about 0.9.3 --features cli`, u `.quality/alati`.
  Prvi pokušaj bez značajke `cli` nije dao izvršnu datoteku. Nije mijenjana zadana
  inačica Rusta i nije dodan nijedan cilj ni sistemski alat.

## Završna provjera (9. 11. 2026., dovršeno pri pregledu)

- `cargo xtask provjera` prolazi: 868 funkcija, pokrivenost 95,15 %. Za
  `prozor/build.rs::main` alat nema područja pa je računa kao 0 %, ali to ne ruši
  provjeru.
- ARM paket složen je u 13:46, nakon zadnje izmjene koda u 13:35. Gradnja
  programa trajala je 1 701 s uz usporedni rad na računalu. Program ima
  20 371 440 bajtova, a ZIP 9 809 596 bajtova. Zapis paketa kaže da je provjera
  pokretanja prošla i prije i poslije raspakiravanja; skripta ga piše tek kad obje
  prođu. Pri pregledu prozor nije ponovno otvaran.
- Raspakirani `.app` pregledan je bez pokretanja: `lipo -info` daje `arm64`,
  `Info.plist` ima ikonu `TerraSimple.icns`, inačicu 0.0.1 i predloženi
  identifikator, a `Dozvole` ima svih šest datoteka. Popis licenci ima 227 stavki.
- **Potpis na Macu.** U tom paketu potpis je imala samo izvršna datoteka, koju je
  potpisao povezivač. `codesign --verify --strict` na cijelom `.app` padao je
  porukom „code has no resources but signature indicates they must be present”.
  Takav preuzeti program macOS može proglasiti oštećenim. Skripta sad potpisuje
  cijeli `.app` bez izdavača i provjerava ga. Na kopiji ovog paketa ta dva koraka
  daju „valid on disk” i ispravan identifikator. Paket nakon te izmjene nije
  ponovno složen, jer bi skripta otvorila prozor programa.
- Python testovi skripte prolaze (6).
- `actionlint` i dalje nije pokrenut: instalaciju u privremenu mapu zaustavila je
  zaštita računala.
