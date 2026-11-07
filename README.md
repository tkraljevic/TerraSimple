# TerraSimple (ideja)

Jednostavna desktop zamjena za svakodnevni rad u Google Earth Prou: crtanje,
mjerenje, „Moja mjesta”, KML/KMZ, a u pozadini ispravni GIS standardi
(GeoPackage, CRS, atributne tablice) i ispis karte u mjerilu. Rust, Windows,
macOS i Linux. Otvoreni kod pod licencom EUPL-1.2, kao i goCOP. Radimo ga za
svoje potrebe, na hrvatskom; tko god želi, može ga koristiti i prenamijeniti
(Odluke (6. 11. 2026.)). Dijelovi projekta razvijeni su uz pomoć umjetne
inteligencije.

## Zašto

QGIS je moćan, ali običnom korisniku kompliciran. Google Earth Pro je
jednostavan i lijep za crtanje, ali mu izlazi podrška, a Google gura web
izdanje koje ne želimo. Fale mu i neke mogućnosti te veza s QGIS-om za
profesionalce.

TerraSimple bi bio između njih: jednostavan kao Google Earth Pro, na
računalu i bez interneta, a sve nacrtano odmah se otvara u QGIS-u, pa
profesionalac nastavlja tamo gdje je običan korisnik stao.

Stanje: dokumenti, nacrt sučelja, prijedlog plana i sedam proba u `probe/`
(izgled, karta, crtanje, GeoPackage, koordinate, ispis, GeoTIFF). Program nije započet.

Odluke (27. 10. 2026.): radni naziv je TerraSimple, bez „GIS” u imenu.
Prije objave provjeriti žigove, domenu i GitHub.

- [Specifikacija, nacrt v1.0 (25. 10. 2026.)](docs/specifikacija.md)
- [Primjedbe i veza s goCOP-om (27. 10. 2026.)](docs/primjedbe.md)
- [Most prema goCOP-u preko plugina (27. 10. 2026.)](docs/gocop-plugin.md)
- [Kvaliteta koda (6. 11. 2026.)](docs/kvaliteta.md)
- [Plan rada, prijedlog (6. 11. 2026.)](docs/plan.md)
- Nacrt sučelja: `dizajn/TerraSimple.figcraft`
