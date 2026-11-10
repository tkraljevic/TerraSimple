# TerraSimple

Jednostavan program za crtanje i mjerenje na karti, za Windows, macOS i Linux. Nacrtano se sprema u
GeoPackage i odmah otvara u QGIS-u. Napisan je u Rustu, na hrvatskom, pod licencom EUPL-1.2. Radimo ga za
svoje potrebe, a tko god želi, može ga koristiti i prenamijeniti (Odluke (6. 11. 2026.)). Dijelovi projekta
razvijeni su uz pomoć umjetne inteligencije.

## Zašto

QGIS je moćan, ali običnom korisniku kompliciran. Google Earth Pro je jednostavan i lijep za crtanje, ali mu
izlazi podrška, a Google gura web izdanje koje ne želimo. Fale mu i neke mogućnosti te veza s QGIS-om za
profesionalce.

TerraSimple je između njih: jednostavan kao Google Earth Pro, na računalu, a sve nacrtano odmah se otvara u
QGIS-u, pa profesionalac nastavlja tamo gdje je običan korisnik stao.

## Što zasad radi

Izdanja su zasad probna (alfa), za Windows, macOS i Linux, na stranici **Releases**. Upute su u
[izdanje.md](docs/izdanje.md).

- **Karta:** podloga OpenFreeMap ili DGU ortofoto, svijetla i tamna tema, mjerilo i koordinate pod mišem u
  HTRS96/TM.
- **Crtanje:** točke, linije i poligoni u slojevima i mapama. Klik dodaje točku, povlačenje je pomiče, Backspace
  briše zadnju, a desni klik završava crtu ili lik. Nakon crtanja upisuje se naziv. Objekti se biraju
  klikom ili okvirom, kao u AutoCAD-u, a briše ih Delete ili ⌘ + Backspace (Ctrl + Backspace drugdje). Svaka izmjena može se poništiti.
- **Mjerenje:** udaljenost i površina na elipsoidu WGS84, kao u QGIS-u. Izmjereno se može spremiti kao
  linija ili poligon.
- **Tražilica:** koordinate u stupnjevima i u HTRS96/TM te slojevi i objekti po nazivu.
- **Spremanje:** projekt je jedna GeoPackage datoteka. Tuđi slojevi i stupci u njoj ostaju netaknuti.
- **QGIS:** gumb „Otvori u QGIS-u” sprema projekt i otvara ga u QGIS-u, s nazivima slojeva, mapama i
  stilovima. Stil namješten u QGIS-u ostaje dok ga ne promijeniš u TerraSimpleu. Ako je QGIS u međuvremenu
  promijenio datoteku, TerraSimple je ne pregazi, a Spremi kao sprema tvoj crtež u novu datoteku.

Još ne radi: ispis, KML i ostali formati, polja u slojevima i podloge iz vlastitih datoteka. Dolaze
redom, po [planu rada](docs/plan.md).

## Pokretanje iz izvornog koda

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

- [Plan rada](docs/plan.md), sa stanjem i rezultatima svakog paketa
- [Popis promjena](PROMJENE.md)
- [Proba prije izdanja 0.0.1](docs/proba-0.0.1.md)
- [Specifikacija, nacrt v1.0 (25. 10. 2026.)](docs/specifikacija.md)
- [Kvaliteta koda (6. 11. 2026.)](docs/kvaliteta.md)
- [Pregled podloga i uvjeta (7. 11. 2026.)](docs/podloge.md)
- Nacrt sučelja: `dizajn/TerraSimple.figcraft`
- Nacrt s ikonama u stilu satin glass (stil: Odluke (7. 11. 2026.)): `dizajn/TerraSimple-ikone-efekti.figcraft`
- [Veličine i stanja ikona](resursi/ikone/sucelje/efekti/README.md) i [pregledi](dizajn/pregledi-ikona/README.md)

Odluke (27. 10. 2026.): radni naziv je TerraSimple, bez „GIS” u imenu. Prije javne objave treba provjeriti
žigove, domenu i GitHub.

## Licenca

EUPL-1.2, na engleskom u [LICENSE](LICENSE), a na hrvatskom u [LICENSE_hr.txt](LICENSE_hr.txt). Karta
OpenFreeMap sadrži podatke OpenStreetMapa i OpenMapTilesa, a ortofoto podatke Državne geodetske uprave. Izvor
se uvijek vidi na karti.
