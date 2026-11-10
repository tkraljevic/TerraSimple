# TerraSimple

Jednostavan program za crtanje i mjerenje na karti, za Windows, macOS i Linux. Nacrtano se sprema u
GeoPackage i odmah otvara u QGIS-u, a mape iz Google Eartha uvoze se jednim klikom. Napisan je u Rustu, na
hrvatskom, pod licencom EUPL-1.2. Radimo ga za svoje potrebe, a tko god želi, može ga koristiti i
prenamijeniti (Odluke (6. 11. 2026.)). Dijelovi projekta razvijeni su uz pomoć umjetne inteligencije.

## Zašto

QGIS je moćan, ali običnom korisniku kompliciran. Google Earth Pro je jednostavan i lijep za crtanje, ali mu
izlazi podrška: Google najavljuje da se od 25. 6. 2027. više neće moći preuzeti. Fale mu i neke mogućnosti te
veza s QGIS-om.

TerraSimple je između njih: jednostavan kao Google Earth Pro, na računalu, a sve nacrtano odmah se otvara u
QGIS-u, pa profesionalac nastavlja tamo gdje je običan korisnik stao.

## Preuzimanje

Izdanja su zasad probna (alfa), za Windows, macOS i Linux, na stranici
[Releases](https://github.com/tkraljevic/TerraSimple/releases). Program ne traži administratora: paket se
raspakira u svoju mapu i pokrene. Upute za svaki sustav su u [izdanje.md](docs/izdanje.md).

## Što radi

Stanje 0.0.5 (u radu). Što je ušlo u koje izdanje, piše u [PROMJENE.md](PROMJENE.md).

- **Karta:** podloga OpenFreeMap ili DGU ortofoto, svijetla i tamna tema, mjerilo i koordinate pod mišem u
  HTRS96/TM.
- **Crtanje:** točke, linije i poligoni u slojevima i mapama. Klik dodaje točku, Backspace briše zadnju, a
  desni klik završava crtu ili lik. Odmah nakon crtanja upisuje se naziv. Točka se pomiče povlačenjem.
- **Odabir:** klikom ili okvirom, kao u AutoCAD-u. Delete ili ⌘ + delete (Ctrl + Backspace drugdje) briše
  sve odabrano. Mape, slojevi i objekti premještaju se povlačenjem ili desnim klikom i „Premjesti u”.
- **Stil:** boja, debljina, prozirnost, vrsta crte i šrafura za cijeli sloj ili jedan objekt, a za točke
  veličina. Otvara se desnim klikom na sloj ili objekt i „Stil”.
- **Mjerenje:** udaljenost i površina na elipsoidu WGS84, kao u QGIS-u. Izmjereno se može spremiti kao
  linija ili poligon.
- **Tražilica:** koordinate u stupnjevima i u HTRS96/TM te slojevi i objekti po nazivu.
- **Spremanje:** projekt je jedna GeoPackage datoteka. Tuđi slojevi i stupci u njoj ostaju netaknuti.
- **QGIS:** gumb „Otvori u QGIS-u” sprema projekt i otvara ga u QGIS-u, s nazivima slojeva, mapama i
  stilovima. Ako je QGIS u međuvremenu promijenio datoteku, TerraSimple je ne pregazi, nego nudi Spremi kao.
- **Uvoz iz Google Eartha:** KML i KMZ, gumbom „Uvezi sloj” ili iz izbornika Datoteka. Mape ostaju mape, a
  objekti idu u slojeve Točke, Linije i Poligoni, s nazivima, opisima i bojama.
- **Prečaci:** gumb „?” ili F1. Program sam prepoznaje sustav, pa na Macu piše ⌘ i dodirnu plohu, a
  drugdje Ctrl i kotačić.
- **Poništavanje:** svaka izmjena, i cijeli uvoz, vraća se s ⌘Z (Ctrl+Z).

Još ne radi: izvoz u KML, GeoJSON i SHP, ikone i natpisi na karti, atributna tablica, podloge iz vlastitih
datoteka i ispis. Dolaze redom, po [planu rada](docs/plan.md).

## Iz izvornog koda

Treba Rust; razvijamo na inačici 1.99.1.

```bash
cargo run -p prozor --release
```

Zastavica `--tamna` otvara program u tamnoj temi, a `--primjer` s izmišljenim projektom iz nacrta sučelja:

```bash
cargo run -p prozor --release -- --primjer
```

Prije svake izmjene prolazi provjera kvalitete, opisana u [kvaliteta.md](docs/kvaliteta.md):

```bash
cargo xtask provjera
```

## Dokumenti

Za korisnike:

- [Preuzimanje i pokretanje](docs/izdanje.md)
- [Popis promjena](PROMJENE.md)
- [Proba prije izdanja](docs/proba-izdanja.md), korak po korak

Za razvoj:

- [Plan rada](docs/plan.md), sa stanjem svake faze
- [Rezultati proba i paketa](docs/rezultati.md), s odlukama koje su iz njih slijedile
- [Specifikacija, nacrt v1.0](docs/specifikacija.md)
- [Kvaliteta koda](docs/kvaliteta.md)
- [Pregled podloga i uvjeta korištenja](docs/podloge.md)
- [Priprema izdanja 0.0.1](docs/priprema-izdanja-0.0.1.md)
- Nacrt sučelja: `dizajn/TerraSimple.figcraft`, a s ikonama u stilu satin glass
  `dizajn/TerraSimple-ikone-efekti.figcraft` (Odluke (7. 11. 2026.))
- [Veličine i stanja ikona](resursi/ikone/sucelje/efekti/README.md) i [pregledi](dizajn/pregledi-ikona/README.md)

## Licenca

EUPL-1.2, na engleskom u [LICENSE](LICENSE), a na hrvatskom u [LICENSE_hr.txt](LICENSE_hr.txt). Karta
OpenFreeMap sadrži podatke OpenStreetMapa i OpenMapTilesa, a ortofoto podatke Državne geodetske uprave. Izvor
se uvijek vidi na karti. Font Inter je pod licencom OFL, u [resursi/fontovi](resursi/fontovi/LICENCA-OFL.txt).
