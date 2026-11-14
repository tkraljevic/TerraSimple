# Duboka provjera — 7. 11. 2026.

Dodana je `cargo xtask provjera-duboka`. Mutira kritične članove radnog prostora
prema istom popisu koji koristi redovno mjerenje (`mjerenje::kriticna`). Početni
paketi su `koordinate` i `geometrija`. Ispis preživjele mutacije sadrži datoteku,
funkciju i promjenu; svaka neodobrena preživjela mutacija ruši naredbu.
Istek vremena, kvar alata i nepotpuno mjerenje također znače pad.
Neprevodive mutacije zasebno se prijavljuju i ne smatraju uhvaćenima testom.

Instaliran je samo `cargo-mutants` **27.1.0**, u uobičajenu Cargo mapu, naredbom
`cargo install cargo-mutants --version 27.1.0 --locked`. Ostali alati već su bili
prisutni: Rust 1.99.1, cargo-llvm-cov 0.9.1 i cargo-deny 0.20.2. Nema novih
ovisnosti xtask-a. Pokusi mutacija na macOS-u pokrenuti su preko
`/usr/sbin/taskpolicy -b`, s dva radnika, rokom prevođenja 180 s i testa 60 s.

## Stvarno mjerenje

| Paket | Ukupno | Uhvaćeno testom | Preživjelo | Bez rezultata testa | Trajanje |
|---|---:|---:|---:|---:|---:|
| koordinate | 45 | 40 | 0 | 5 | 92,97 s |
| geometrija | 37 | 35 | 0 | 2 | 75,09 s |

Prva dva mjerenja radila su istodobno. Svih sedam slučajeva bez rezultata
nisu se mogla prevesti: E0277, nepostojanje `Default` za `Proj`, `Pretvornik`,
`Sustav`, `MjereCrte` ili `MjereLika`. Sve su to zamjene tijela s
`Ok(Default::default())` ili `Some(Default::default())`. Nije bilo isteka vremena.
Nema preživjelih mutacija u izvornim paketima, pa nema popisa preživjelih ni
prijedloga iznimaka. To vrijedi za mutacije koje ovaj alat stvara, ne za sve
moguće pogreške.

Zatim je stvarno pokrenuta cijela nova naredba: izlaz **0**, iste brojke po
paketima. Geometrija je trajala 65,42 s, koordinate 64,24 s, ukupno oko 130 s
za same mutacije. Sažeti strojno čitljivi dokazi, uključujući svaku mutaciju,
nalaze se u [duboka-prvo.json](duboka-prvo.json). Puni lokalni izlazi su u
zanemarenoj mapi `.quality/`.

Prvi udio ponovljenog koda je **0,00 % (0 od 1901 retka)**. Mali brojač ne traži
Node ni dodatnu instalaciju, za razliku od jscpd-a, i nema nove ovisnosti.
Metoda `retci-10-v1` traži deset jednakih uzastopnih nepraznih redaka, bez rubnih
razmaka i redaka koji počinju s `//`. Broji oba pojavljivanja, a preklapanja jednom.
Pregledava Cargo izvore i mape `src`, uključujući ugrađene testove, bez zasebnih
integracijskih testova. Ne prepoznaje preimenovan ili drukčije oblikovan isti kod;
blok-komentari i literali mogu ući u mjeru. Prvo mjerenje samo izvještava.

## Dokaz pada i redovna provjera

U privremenoj kopiji geometrije oslabljen je samo test
`poruke_postoje_za_svaku_gresku`: poziv ostaje, ali nema provjere teksta.
Preživjela je jedna mutacija u `src/mjere.rs:25:9`, funkciji
`<impl std::fmt::Display for Greska>::fmt`: zamjena tijela s
`Ok(Default::default())`. To pokazuje da test bez tvrdnje ne otkriva nestalu
poruku greške. Rezultat kopije: 37 ukupno, 34 uhvaćene, 1 preživjela i 2
neprevodive; `cargo xtask provjera-duboka --mapa <kopija>` vratio je **1**.
Privremeni paket potom je obrisan. Izvorni paketi i njihovi testovi nisu mijenjani.

Redovna provjera prošla je bez pomicanja reference: rustfmt, Clippy, testovi,
LLVM pokrivenost, složenost, rizik i cargo-deny. Izmjereno je 95 funkcija,
92,32 % područja; najveći CC i CRAP su 10. Testovi xtask-a obuhvaćaju prolaz,
preživjelu mutaciju, odobrenu iznimku, kvar alata, nepoznat ili nepotpun rezultat,
istek vremena i preklapanje ponovljenih blokova. Postavke iznimaka ostale su prazne.

## Platforme, CI i granice provjere

U xtask-u nema poziva macOS ljuske. Zamijenjena je apsolutna testna putanja
nepostojeće mape putanjom iz sustavske privremene mape. Prepoznavanje kritičnih
putanja sada prihvaća i `\`; dodan je test. Probni alat ostaje izvršna Rust
programska datoteka s nastavkom prema platformi, bez `sh` ili `chmod`.

CI priprema redovnu provjeru za PR i slanje u `master` na Windowsu, macOS-u i
Linuxu, a duboku ponedjeljkom u 03:23 UTC i ručno na Linuxu. Koristi predmemoriju
ovisnosti i alata, prekida zastarjele pozive te ograničava trajanje. Obje YAML
datoteke lokalno su pročitane Rubyjevim parserom i pregledani su okidači,
matrica platformi i naredbe. To nije provjera izvršavanja GitHub Actionsa.

Procjena po slanju u `master`: s toplom predmemorijom 3–6 minuta po platformi,
odnosno **9–18 minuta izvršavanja ukupno**; hladna predmemorija približno
24–45 minuta ukupno. To nije mjerenje CI-ja niti obračun naplate. Minuta na macOS-u prema GitHubovu cjeniku stoji oko 10 puta više od minute na Linuxu, a na Windowsu oko 1,7 puta, pa jedno slanje s toplom predmemorijom vrijedi oko 39–78 linuxskih minuta. Provjera na sve tri platforme pri svakom slanju ipak je izabrana (Odluke (7. 11. 2026.)). Repozitorij
je privatan; potrošnja kvote i naplata ovise o planu i platformi prema
[GitHubovim pravilima](https://docs.github.com/en/billing/concepts/product-billing/github-actions).
Duboka provjera ne dodaje minute svakom slanju.

**Nisu pokrenuti Windows, Linux ni stvarni CI. Fuzz nije uveden niti pokrenut**:
još nema paketa za čitanje nepouzdanih datoteka. Nije provjerena svaka moguća
pogreška ili svaka buduća struktura projekta.

Čovjek treba pregledati prvo mjerenje, odlučiti o budućem praćenju duplikacije
i nakon pregleda uključiti CI. Eventualna buduća iznimka za ekvivalentnu ili
neopažljivu mutaciju smije biti samo u `quality/config.json`, s točnim ključem
`mutacija:<ime mutacije>`, razlogom, odgovornom osobom i rokom. Nijedna nije dodana.
