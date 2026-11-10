# Ikone sučelja: satin glass

Odluke (7. 11. 2026.). Stil ostaje satin glass. Uređivi nacrt je
`dizajn/TerraSimple-ikone-efekti.figcraft`; prethodni nacrt nije mijenjan.

## Ugrađeno u program

Program ne čita ikone s diska. `skripte/ugradi_ikone.py` slaže sve PNG-ove iz `manifest.json` u
`ugradjene.bin` i `ugradjene.txt`, a program ih ugradi pri izgradnji. Nakon svake promjene ikona
pokreni tu skriptu; test `sve_ikone_iz_popisa_su_ugradjene` pada ako je ugrađeno zastarjelo.

## Veličine i skale

`manifest.json` popisuje izvorni SVG i neposredno iscrtane PNG datoteke za
svaku temu, veličinu i zatečeno stanje. Veličina crteža znači njegov okvir
u rasporedu sučelja; pojedini simbol ne mora ispuniti cijeli taj okvir.

| Okvir crteža | Rub sa svake strane | Platno pri 1× | Platno pri 2× | Potez pri 1× |
|---:|---:|---:|---:|---:|
| 14 px | 2 px | 18 × 18 px | 36 × 36 px | 1 px |
| 16 px | 2 px | 20 × 20 px | 40 × 40 px | 1 px |
| 18 px | 3 px | 24 × 24 px | 48 × 48 px | 2 px |
| 20 px | 3 px | 26 × 26 px | 52 × 52 px | 2 px |
| 22 px | 3 px | 28 × 28 px | 56 × 56 px | 2 px |

Program treba odabrati veličinu okvira, temu i stanje, a zatim odgovarajući
PNG za 1× ili 2×. Platno se crta na položaju okvira umanjenom za `rub` iz
manifesta. Primjer: crtež od 18 px koristi platno od 24 px, koje počinje
3 px lijevo i iznad njegova okvira. Platno se **ne smanjuje** natrag na 18 px.
Položaj treba poravnati na cijeli fizički piksel. Ako program sam koristi
resvg, iscrtava pripadni SVG izravno pri skali zaslona, uz isti raspored.

Ravne crte poravnane su prema debljini poteza. Krivulje i dijagonale i dalje
imaju zaglađene rubove; efekt stakla također namjerno ima blage prijelaze.
Oštrina ne znači uklanjanje tog zaglađivanja ili jamstvo jednakih piksela
u različitim programima za iscrtavanje.

## Mali prikaz

Na 14 i 16 px nema tankog bijelog odsjaja, potez i sjena su blaži.
`tablica` ima jedan vodoravni razdjelnik, `slojevi` dvije razine,
`mreza` jednostavniju krivulju meridijana, a `ravnalo` jednu crticu na 14 px
i dvije na 16 px. Veći prikaz zadržava pune detalje. Unutarnje crte tablice
imaju ravne završetke točno na okviru, a obod ima središnje postavljen potez.

## Stanja iz nacrta

Uz obično stanje izvezena su samo zatečena stanja i značenja boja:

- `odabrani_alat`: odabir, poligon i mjerenje, sa svijetlim potezom;
- `opasna_radnja`: kanta za brisanje;
- `negativna_oznaka`: križić uz sadržaj koji se ne može izvesti;
- `priguseno`: ponavljanje i hvataljka; sama boja ne dokazuje zabranu klika;
- `narancasti_poligon`, `plava_linija`, `crvena_tocka`, `plava_podloga`;
- `potvrda`: pozitivna kvačica u sažetku promjena.

Obje teme imaju te inačice. Za odabrani alat SVG ima prozirnu podlogu;
tamnozeleni gumb pripada sastavnici sučelja. Nisu dodana pretpostavljena
stanja prelaska mišem, pritiska ili fokusa.

## Datoteke

- `velicine/<tema>/<stanje>/<veličina>/<ime>.svg`;
- uz SVG: `<ime>@1x.png` i `<ime>@2x.png`, iscrtani resvg-om;
- `stanja-iz-nacrta.json`: imena i primjeri slojeva iz nacrta;
- `manifest.json`: 720 SVG inačica i 1.440 pripadnih PNG datoteka;
- stariji raspored `<tema>/<ime>.svg`, `<ime>.png` i `32/<ime>.png`
  ostaje radi postojećih poveznica: crtež od 22 px na platnu od 32 px,
  uz PNG od 32 i 64 px. To nije zamjena za pet stvarnih veličina.

Izvoze izrađuju `izradi_satin_glass.py`, `izvezi_satin_glass.py` i
`iscrtaj_resvg.cjs` iz mape `dizajn/skripte`. Za tekst pregleda koristi se
odabrana datoteka slova. Putanje do programa, modula i slova predaju se pri
pokretanju, a ne upisuju se u resurse. Provjeru provodi `provjeri_ikone.py`.

Skripte trebaju Python paket Pillow, Node paket `@resvg/resvg-js` (korišten je 2.6.2) i
FigCraft CLI (korišten je 0.15.1). Paketi nisu dio projekta, pa ih treba instalirati posebno, a
`izvezi_satin_glass.py --help` ispisuje što se predaje pri pokretanju.

Rust kod i probe nisu prebačeni na ove datoteke.

## Pregledi i podrijetlo

[Pregledi svih veličina](../../../../dizajn/pregledi-ikona/README.md),
[izvještaj o popravcima](../../../../dizajn/ikone-popravci-izvjestaj.md),
[usporedba svih 60 osnovica s Lucideom](../LUCIDE-PODRIJETLO.md) i
[obavijesti o licencama](../LICENCA-LUCIDE.txt).
Cijeli tekst licence priložen je i u SVG datotekama.
