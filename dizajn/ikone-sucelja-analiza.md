> Raniji prijedlog. Odabran je satin glass stil — Odluke (7. 11. 2026.).
> Preporuke linijskog stila u ovom tekstu ostaju samo zapis prethodne usporedbe.

# Analiza ikona sučelja (7. 11. 2026.)

Pregledani su `dizajn/skripte/temelj.py`, mjesta upotrebe u skriptama nacrta
i `probe/izgled/src/ikone.rs`. Nacrt definira 60 ikona; Rust proba ima podskup
od 28. Manji broj u probi sam po sebi nije greška: ona ne prikazuje sve
planirane zaslone.

## Smjer

Za alate zadržati jednostavne linijske ikone. Sjajna ikona aplikacije ostaje
oznaka proizvoda; gumbi i oznake na karti moraju se brzo razlikovati na
14–24 px i u obje teme. Ovo je prvi prijedlog, bez zamjene ikona u postojećem
FigCraft dokumentu ili Rust probi.

## Ciljane dorade

| Ikona | Opažanje | Dorada u prijedlogu |
|---|---|---|
| Poništi / ponovi | Kratki povratni luk i pravokutan vrh slabo razdvajaju smjer. | Zrcalan par s jasnom strelicom ulijevo / udesno. |
| Ravnalo | Četiri crtice zgušnjavaju unutrašnjost pri maloj veličini. | Tri crtice različitih duljina i jednostavnija geometrija. |
| Spremi | Unutarnja gornja linija ostaje otvorena. | Zatvoren gornji pretinac i jasan donji dio diskete. |
| Stil | Puni kružni obris zajedno s dodatnim lukom daje dojam lica. | Jedan obris slikarske palete s urezom i tri mjesta za boju. |
| Olovka | Jednostavan obris ne naglašava vrh i završetak. | Odvojeni vrh i završetak olovke. |
| Sjever | Silueta je slična pokazivaču odabira. | Simetrična igla s unutarnjom osi. |
| Kuća | Prazan obris kuće gubi detalj pri maloj veličini. | Dodana vrata radi prepoznatljivosti simbola. |

Ostalih 52 ikone zadržavaju geometriju. Promjene su optičke procjene,
bez provedenog korisničkog testiranja prepoznatljivosti.

## Dosljednost prikaza

Nacrt zadano drži potez na 1,75 px bez obzira na veličinu ikone. Rust proba
računa `1.75 * velicina / 20.0`. Podudaraju se na 20 px, a razlikuju na
drugim veličinama. Novi SVG komplet koristi isti omjer kao Rust proba:
2,1 jedinicu na mreži 24 × 24. Izričito podebljane kvačice u nacrtu imaju
posebnu ulogu i ne treba ih automatski izjednačiti s ostalim ikonama.

Za integraciju treba primijeniti odabranu geometriju u oba postojeća izvora,
uskladiti skaliranje poteza u nacrtu i pregledati stvarne gumbe. Sam izvoz
resursa ne mijenja već nacrtane slojeve u FigCraftu.

## Provjera

VectorCraft CLI iscrtava SVG izvore u PNG. Usporedni pregled prikazuje sve
ikone u povećanom prikazu i na približno 16 px; osam dorada ima usporedbu
prije i poslije. Zasebni pregled malih veličina prikazuje dorade na stvarnih
16, 20, 24 i 32 px, na svijetloj i tamnoj podlozi.

Datoteke za upotrebu nalaze se u `resursi/ikone/sucelje/`: SVG i PNG 24/48 px.
Nema dodanih ovisnosti ni promjena izvršnog koda aplikacije.
