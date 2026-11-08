# Fuzz čitača

`fuzz/` je zaseban Cargo radni prostor, izvan članova glavnog radnog prostora.
Koristi postojeći nightly i cargo-fuzz 0.13.2. Zadani Rust ostaje stabilan.

Iz korijena projekta:

```sh
CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo xtask fuzz
TERRASIMPLE_FUZZ_SEKUNDE=1800 CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo xtask fuzz
```

Druga naredba izvodi svaki cilj najmanje 30 minuta, redom. Zadano trajanje je
60 sekundi po cilju. Granice su 5 sekundi po ulazu, 1024 MB RSS-a i 262 144 bajta
po ulazu. Završetak zbog granice vremena pokusa je normalan; prekoračenje granice
jednog ulaza ili memorije znači pad. `provjera-duboka` izvodi kratke pokuse od
10 sekundi kad su alati dostupni. Ako nisu, bilježi `nije-pokrenuto`; to nije
potvrda prolaza fuzza. Redovna provjera ne pokreće nightly.

Ciljevi ne trebaju QGIS:

- `geometrija`: GP zaglavlje i WKB kroz svih pet poznatih koordinatnih sustava;
- `qml`: UTF-8 XML stila za točke, linije i poligone; ostali bajtovi se odbijaju;
- `datoteka`: svaki ulaz zapisuje u vlastitu `.gpkg` datoteku pa zove `otvori` i
  `ucitaj`. Datoteke su odvojene po procesu i ne prepisuju korisničke podatke.

Značajka `geopaket/fuzz` otvara samo dva postojeća čitača kroz skriveni modul
`geopaket::fuzz`: `geometrija` i `stil`. Nije uključena u redovnoj gradnji i nije
namijenjena sučelju aplikacije. Dopušten povrat greške se ne smatra padom;
panika, zastoj, prekoračenje memorije i nalaz sanitizera jesu padovi.

`pocetni/` sadrži male izmišljene ulaze, uključujući krivulju, POINT EMPTY, NULL
u cijeloj bazi i QML s više simbola. `python3 fuzz/pripremi.py` obnavlja ih.
Rastući `corpus/`, `artifacts/`, `rad/` i izlazi gradnje zanemareni su u gitu.
Dnevnici svakog pokusa i broj izvršenih ulaza spremaju se u `.quality`.

Pojedinačni cilj ili sačuvani pad pokreće se ovako:

```sh
CARGO_TARGET_DIR=target/fuzz /usr/sbin/taskpolicy -b cargo +nightly fuzz run qml fuzz/corpus/qml fuzz/pocetni/qml -- -max_total_time=60 -timeout=5 -rss_limit_mb=1024 -max_len=262144 -print_final_stats=1
CARGO_TARGET_DIR=target/fuzz /usr/sbin/taskpolicy -b cargo +nightly fuzz run qml fuzz/artifacts/qml/ime-ulaza -- -timeout=5 -rss_limit_mb=1024
```

`python3 fuzz/dokaz.py` u zasebnoj kopiji pravog čitača vraća beskonačnu petlju
na kraju XML-a. Traži da je fuzz otkrije i zatim vraća izvorne bajtove. Kopija
ostaje u vlastitoj mapi `.quality/dokaz-fuzza-*` radi pregleda. Svako pokretanje
stvara novu kopiju. Prvo prevodi ovisnosti, zatim odvojeno izvodi pokus.
Sažetak tog pokusa je u `dokaz.json`, a stvarni rezultati u [REZULTAT.md](REZULTAT.md).

Ovisnosti zasebnog radnog prostora provjeravaju se zasebno:

```sh
CARGO_TARGET_DIR=target/fuzz cargo deny --manifest-path fuzz/Cargo.toml --config quality/deny.toml --locked check
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
```

Licenca NCSA dopuštena je samo za `libfuzzer-sys` kroz imenovanu iznimku u
postavkama cargo-deny. Ostala pravila provjere ovisnosti vrijede i ovdje.
