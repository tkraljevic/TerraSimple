# Rezultat fuzza čitača

Tri cilja u zasebnom `fuzz/` radnom prostoru provjeravaju GP/WKB kroz svih pet
koordinatnih sustava, QML i cijelu `.gpkg` datoteku. Početni ulazi su izmišljeni:
krivulje, POINT EMPTY, NULL u bazi, odrezani podaci i QML s više simbola.
Značajka `geopaket/fuzz` otvara samo postojeće čitače geometrije i stila.

## Izmjereno

Svaki cilj radio je 1801 sekundu kroz `/usr/sbin/taskpolicy -b`, s vlastitim
`target/fuzz`, AddressSanitizerom i granicama: 5 sekundi po ulazu, 1024 MB RSS-a,
262 144 bajta po ulazu. Svi su završili kodom 0.

| Cilj | Izvršeni ulazi | Sekunde | Najveći RSS (MB) |
| --- | ---: | ---: | ---: |
| geometrija | 1 620 513 | 1801 | 527 |
| qml | 2 318 973 | 1801 | 528 |
| datoteka | 132 640 | 1801 | 409 |

Ukupno: 4 072 126 ulaza. Nije prijavljen pad, zastoj, prekoračenje memorije ni
nalaz sanitizera. Zato nema popravka čitača ni novog testa za pronađenu grešku.
To ne dokazuje odsutnost grešaka izvan ovih pokusa i granica.

Brojke i SHA-256 sažeci dnevnika su u `fuzz/dugi-pokusi.json`; dnevnici su
`.quality/fuzz-geometrija-1800.log`, `.quality/fuzz-qml-1800.log` i
`.quality/fuzz-datoteka-1800.log`. Nastali korpus, artefakti i radne datoteke
zanemareni su u gitu; ostaju samo mali početni ulazi.

## Pokretanje i dokaz kvara

Iz korijena radne kopije, sva tri cilja redom:

```sh
TERRASIMPLE_FUZZ_SEKUNDE=1800 CARGO_TARGET_DIR=target /usr/sbin/taskpolicy -b cargo xtask fuzz
```

Zadana naredba, bez varijable trajanja, također je stvarno prošla: po 60 sekundi
za svaki cilj, 35 627 / 46 696 / 4 538 ulaza. Stvarni kratki poziv za duboku
provjeru prošao je s 7 776 / 10 043 / 728 ulaza. Zapisi su u
`.quality/55264-1794141643215452000/fuzz.json` i `.quality/kratki-fuzz/fuzz.json`.
Testovi zamjenskih naredbi provjerili su pad i nedostajući nightly; redovna
provjera ga ne traži, a duboka bez njega izričito bilježi da fuzz nije pokrenut.

`python3 fuzz/dokaz.py` u odvojenoj kopiji pravog QML čitača mijenja kraj XML-a
u petlju. Fuzz ju je uhvatio na praznom ulazu nakon 2 sekunde: cilj je završio
kodom 70, naredba cargo-fuzz kodom 1. Nakon vraćanja izvornih bajtova obnovljeni
čitač prošao je 111 616 ulaza u 11 sekundi, kodom 0. SHA-256 potvrđuje jednake
bajtove. Sažetak je `fuzz/dokaz.json`, zapisi `.quality/dokaz-fuzza-ixk27ry_`.
Prvi pokušaj istekao je tijekom hladne gradnje; nije stigao do fuzza i nije
računat kao dokaz. Radni izvor čitača nije mijenjan tim pokusima.

## Završna provjera i ograničenja

`cargo xtask provjera` prošla je: rustfmt, Clippy, testovi, pokrivenost, složenost
i cargo-deny. Prošlo je 290 testova; jedan pomoćni test označen je kao preskočen
jer ga roditeljski test pokreće zasebno. Izmjereno je 695 funkcija i 96,11 %
pokrivenosti. GeoPaket ima 99,41 % ukupno, najmanje 95,24 % po funkciji i najveću
složenost 7. Završni zapis je `.quality/provjera-zavrsna.log`, mjerenje u
`.quality/61773-1794142432322351000/mjerenje.json`.

Zasebni fuzz radni prostor prošao je rustfmt i cargo-deny. NCSA je dopuštena
samo za `libfuzzer-sys`; pragovi i referenca nisu mijenjani. Zadani Rust ostao
je 1.99.1; fuzz je koristio postojeći nightly 1.102.0 (f5cedb83a, 7. 11. 2026.)
i cargo-fuzz 0.13.2. Ništa nije instalirano na sustav.

Windows, Linux i puna duboka provjera svih mutacija nisu pokrenuti. Nema nove
odluke potrebne za ove ciljeve. Upute i pojedinačne naredbe su u [README.md](README.md).
