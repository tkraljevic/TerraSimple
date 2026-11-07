# Popravci ikona

Odluke (7. 11. 2026.). Satin glass stil ostaje.

## Nacrt i granice rada

Izmijenjen je samo `TerraSimple-ikone-efekti.figcraft` iz mape `dizajn`.
Korišten je zaseban FigCraft na portu 7980. Prethodni nacrt
`TerraSimple.figcraft` ostao je nepromijenjen. Rust kod i probe nisu mijenjani.
Nije napravljen commit ni push. Putanje u spremljenim popisima su relativne.

## Što je popravljeno

1. **Tablica:** obod ima središnje postavljen potez. Unutarnje crte imaju
   ravne završetke točno na obodu, bez izbočenih zaobljenih krajeva. To vrijedi
   za obje teme i sve stvarne veličine. Geometrija je i u skripti, ne samo u izvozu.
2. **Veličine:** umjesto smanjivanja gotovog platna od 32 px postoje zasebni
   SVG izvori i PNG izvozi za okvire od 14, 16, 18, 20 i 22 px pri 1× i 2×.
   Njihova platna pri 1× imaju 18, 20, 24, 26 i 28 px. Sve su dimenzije cijele.
   Program treba poštovati rub iz manifesta i nacrtati platno oko okvira,
   bez naknadnog smanjivanja. Podroban prijedlog upotrebe je u README-u kompleta.
3. **Mali crteži:** na 14/16 px izostavljen je tanki odsjaj, a potez i sjena
   su blaži. Tablica ima manje razdjelnika, slojevi dvije razine, mreža
   jednostavniji meridijan. Ravnalo ima jednu crticu na 14 px i dvije na 16 px.
   To su stvarno različiti vektorski izvori, ne samo tvrdnja u dokumentaciji.
4. **Stanja:** manifest pokriva obično stanje, tri stvarno odabrana alata,
   kantu za opasnu radnju, negativnu oznaku, prigušeni prikaz, potvrdu i boje
   crteža. Zatečeni slojevi popisani su u `stanja-iz-nacrta.json`.
   Sve te inačice imaju obje teme. Nisu izmišljena stanja prelaska mišem ili fokusa.
5. **T:** svi završeci bijelih crta punog logotipa su zaobljeni, uključujući
   srednju nožicu i lijeve krajeve dviju gornjih crta. Tri nožice završavaju
   na istoj visini. Za mali prikaz postoji `terrasimple-mala.svg`, s većim
   razmacima među crtama. PNG od 32 i 64 px koristi tu inačicu. Puni logotip
   i dalje je pregust za preporučenu upotrebu na 32 px; zato se bira mali.
   U nacrtu su zamijenjene 23 stare oznake s peterokutom. Uređivač predloška
   prije nije imao oznaku, pa je T dodan i tamo: svih 24 zaslona sada ga ima.
6. **Dodatna greška izvoza:** SVG sjena više se ne računa iz omeđujućeg okvira
   crte bez visine. Izričito područje filtra sprječava nestajanje minusa u resvg-u.

U nacrtu je zadržano 990 instanci ikona. Raspodjela pet veličina potvrđena
je čitanjem nacrta: 506 × 18 px, 192 × 20 px, 136 × 22 px, 108 × 14 px i
45 × 16 px. Još su tri sjeverne oznake od 26, 28 i 30 px u prikazima ispisa;
one nisu dio tražene matrice pet veličina i ostale su vektorske u nacrtu.

## Lucide i obavijesti o izvorima

[Popis svih 60 usporedbi](../resursi/ikone/sucelje/LUCIDE-PODRIJETLO.md)
navodi izvorne datoteke i razlike. Uspoređeno je 2.029 izvornih SVG-ova iz
jasno označene revizije. Trinaest osnovica podudara se nakon normalizacije
na desetinku jedinice, prije optičkih dorada. Za 45 su navedeni prilagođeni
ili srodni crteži; za liniju i grafičko mjerilo nema podudarnog cijelog crteža.
To nije dokaz povijesnog autorstva svih prerada i ne isključuje obavijest o licenci.

Cijeli tekst preuzetog ISC i pripadajućeg MIT dijela za Feather sačuvan je
u `LICENCA-LUCIDE.txt`, u SVG datotekama i na stranici 9 odobrenog nacrta.
Opis u `temelj.py` više ne govori samo o sličnom stilu.
`probe/izgled/src/ikone.rs` nije diran. U popisu podrijetla piše kako pri
sljedećoj dopuštenoj izmjeni treba ispraviti njegovu napomenu „stil Lucide”.

## Okvir i kvadrat

Oba ostaju. `okvir` se koristi na zaslonu N, u alatima za dodavanje okvira
na papir u uređivaču predloška. `kvadrat` je izbor oznake točke na zaslonu
C3, u postavkama stila točaka. Sličan crtež ovdje ima različitu namjenu;
oba imena trebaju ostati radi jasne upotrebe. Nije rađeno brisanje ni spajanje.

## Raniji i nepovezani izvori

[Potpuni popis 188 datoteka](popis-nepovezanih-datoteka.csv) koristi relativna
imena i ne znači da ih treba obrisati. Nije pronađeno njihovo učitavanje u
Rust kodu. Pretraživanje koda obuhvatilo je nazive resursa i ugrađivanje datoteka.

- `ikonica.svg`: raniji ravni logotip;
- `ikonica-glass.vectorcraft`: raniji radni izvor logotipa, izvan sadašnjeg izvoza;
- `ikone-kontekst-glavni.png` i `ikone-kontekst-tamna.png`: stare snimke konteksta;
- `ikone-sucelja-pregled.svg/png` i `ikone-sucelja-velicine.svg/png`:
  raniji linijski pregled;
- 60 SVG i 120 PNG datoteka ranijeg linijskog kompleta u mapama
  `resursi/ikone/sucelje/svg` i `resursi/ikone/sucelje/png/24` te `png/48`.

I novi komplet zasad je pripremljen resurs: Rust proba i dalje sama crta
stare putanje. Nije napravljena integracija koju uputa zabranjuje.
Na vrhu ranije analize jasno je označeno da je to prethodni prijedlog.
Glavni README sada upućuje na odobreni nacrt i nove preglede.

## Novi pregledi i izvozi

[Indeks svih pregleda](pregledi-ikona/README.md) povezuje:

- 40 ploča svih 60 ikona: pet veličina, dvije teme, 1×/2× i dva programa
  za iscrtavanje, FigCraft CLI 0.15.1 i resvg preko paketa `@resvg/resvg-js` 2.6.2;
- 40 povećanja najbližim susjedom, uz izvorne ploče;
- četiri posebna pregleda tablice, slojeva, mreže i ravnala;
- preglede stvarnih stanja u oba programa pri 1× i 2×;
- usporedbu punog i malog logotipa pri 32 i 64 px;
- dvije ploče usporedbe s Lucideom.

[Manifest](../resursi/ikone/sucelje/efekti/manifest.json) ima 720 SVG inačica:
600 običnih i 120 zatečenih stanja. Svaka ima PNG za 1× i za 2×, ukupno 1.440.
Uz to je sačuvan i osvježen stariji raspored 120 SVG i 240 PNG datoteka
za platno od 32 px. Svih 24 ranijih PNG pregleda zaslona također je osvježeno.

## Što je stvarno provjereno

`provjeri_ikone.py` je na spremljenim datotekama provjerio:

- XML, dimenzije, izvore bez ugrađenih slika i obavijest o licenci;
- svih 1.440 novih PNG datoteka te 240 kompatibilnih PNG-ova: dimenzije,
  prozirnost, vidljiv sadržaj i vanjski rub bez odrezane sjene; najveća alfa
  na vanjskom rubu bila je 0;
- završetke unutarnjih crta tablice na okviru u svih pet veličina;
- svih 40 rasterskih ploča i 40 povećanja, piksel po piksel, za najbližeg susjeda;
- 24 zaslona i 24 logotipa T u spremljenom nacrtu;
- sažetke izvorne zaštićene FigCraft datoteke i 60 Rust datoteka: bez promjene.

Vizualno su pregledane usporedne ploče svih 60 ikona pri 1× u svih pet
veličina i obje teme, posebni povećani prikazi važnih ikona pri 1×/2×,
logotip 32/64 px, stanja te reprezentativni zasloni nacrta. Nije svaki
pojedini PNG pri 2× zasebno gledan; svi su iscrtani i dimenzije su izmjerene.

Pikseli dvaju programa nisu jednaki. Razlike su mjerene na pločicama ikona,
bez teksta: prosjeci RGB razlika po pločama bili su od 0,6203 do 5,1575 na
skali 0–255. Najveća prosječna razlika jedne ikone bila je 9,1708 za `slika`
u svijetloj temi na 14 px pri 1×. Za planirani resvg prikaz pripremljeni su
upravo njegovi PNG izvozi. Brojke su u [provjera.json](pregledi-ikona/provjera.json).
Zasebna provjera dokumentacije potvrdila je 86 lokalnih poveznica, prisutnost
licence u nacrtu i izostanak strojnih putanja u 944 pregledane tekstualne datoteke.

## Što nije provjereno ili se ne može zaključiti

Nisu mijenjani ni pokretani Rust prikaz i probe s novim resursima, pa nije
potvrđen njihov stvarni prikaz u programu. Nisu provjereni Windows, Linux,
različiti grafički pogoni ni skale poput 1,25×. Ova provjera obuhvaća
spremljene SVG/PNG datoteke, nacrt i izravno iscrtavanje pri 1× i 2×.

Nijedan traženi izvoz nije ostao neizrađen. Povijesno podrijetlo svake
pojedine prilagodbe ne može se dokazati samo današnjim crtežom; zato su
razlike popisane, a obavijest o izvornoj licenci nije uskraćena nijednoj ikoni.
