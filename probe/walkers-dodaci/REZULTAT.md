# Rezultat probe: što još zna `walkers`

Proba prije paketa `karta` (7. 11. 2026., `walkers` i `walkers_extras` 0.65.0). Odgovori su iz
izvornog koda biblioteke i pokretanja. Prvu inačicu ovog izvještaja pregled je ispravio: opis
inercije nije odgovarao kodu, a preporučeni poziv ostavljao je skok karte.

Pokretanje, uz stazu lokalne PMTiles datoteke, koja ne ide u repozitorij:

```sh
TERRASIMPLE_PMTILES=/staza/do/karte.pmtiles ./provjera/pokreni.sh
```

Snimke se izrađuju samo ako ih nema ili uz `PONOVNO_SNIMI=1`.

## 1. `with_projection`: ekran ostaje u Mercatoru

`with_projection` prima vlastitu projekciju. Isprobana je HTRS96/TM (EPSG:3765):

- **Točka u piksel i natrag te crtanje u `Plugin`-u rade.** Povratna greška je 8,4 × 10⁻⁸°, a nagib
  meridijana kod Iloka 2,04°, što odgovara računu.
- **Pločice ne rade.** `walkers` iz projekcije računa koju pločicu traži i lijepi je kao kvadrat, bez
  preračuna. Za Zagreb na zumu 14 Mercator traži `x 8919, y 5840`, a HTRS96/TM `x 8175, y 8161`.
  OpenFreeMap i PMTiles izrezani su u Mercatoru, pa se ne mogu poklopiti.
- **DGU WMS:** adresa iz probe uz `CRS=EPSG:3765` vraća JPEG. Karta koja bi imala
  samo taj sloj nije isprobana, a provjera okvira u probi je samo vlastiti račun.

**Za paket `karta`:** ekran ostaje u Mercatoru, a HTRS96/TM računa paket `koordinate`.

## 2. `walkers_extras`: ne koristimo ga za KML

Nudi izbornik podloga, slojeve `KmlLayer` i `GeoJsonLayer` te grupiranje točaka. Licenca je MIT. Vuče
`kml` 0.14, `geojson`, `geo` i `rstar`, a `quick-xml` i `rstar` tada dolaze u po dvije inačice.

| Što je u KML-u | `KmlLayer` | Kako je utvrđeno |
|---|---|---|
| Rupe u odvojenim `innerBoundaryIs` | čuva 2 od 2, izrezane | test |
| Više prstenova u istom `innerBoundaryIs` | čita 2 od 2 | test, ali drugi prsten ima ponovljenu točku i nije provjereno je li izrezan |
| Stilovi, nazivi, opisi, mape | gubi | kod biblioteke |
| Visina | gubi | kod biblioteke |
| Prazna točka `<Point></Point>` | ruši program | test |
| `gx:Track` | tiho gubi trag, a ostali objekti ostaju | test, i uz poligone u istoj datoteci |
| Crtanje | u svakom okviru sve preračunava i ponovno sastavlja na procesoru | kod biblioteke |

**Za paket `karta`:** ostaju naš KML čitač iz `probe/kml` i crtanje iz `probe/mnogo`.

## 3. Inercija: poziv mora biti prije `Map::show`

Tijekom povlačenja karta pamti brzinu miša pomnoženu s 2. U okviru puštanja tipke `Map::show` prelazi u
klizanje i u istom okviru već pomakne kartu. Klizanje staje kad brzina padne ispod 10 px/s.

Poziv se izvodi u svakom okviru dok traje crtanje, **prije `Map::show`**:

```rust
if crtanje_aktivno && let Some(polozaj) = memorija.detached() {
    memorija.center_at(polozaj);
}
```

Simulirano je povlačenje stvarnim `egui` događajima, šest koraka po 25 i −15 px u 16 ms, a miš je prešao
174,93 px. Mjereno je ono što je `Map::show` stvarno nacrtao:

| Poziv | Skok u okviru puštanja | Klizanje nakon toga (0,64 s) | Kraj od zbroja miša |
|---|---:|---:|---:|
| Bez poziva | 54,15 px | 643,76 px | 697,91 px |
| Poslije `Map::show` | 54,15 px | 0,00 px | 54,15 px |
| **Prije `Map::show`** | **0,00 px** | **0,00 px** | **0,00 px** |

- Povlačenje uz poziv i dalje radi, a uključivanje crtanja zaustavlja i klizanje koje je već počelo.
- Kod sporog povlačenja karta na kraju kasni za mišem 6 px. To je vjerojatno `egui`-jev prag za početak
  povlačenja, a zasebno nije provjereno.

**Za paket `karta`:** prijedlog je da karta ne klizi dok traje crtanje, uz ovaj poziv.

## 4. Zum izvan raspona 11 do 17

- `walkers` dopušta zum od 0 do 26. Izvan toga `set_zoom` vraća grešku.
- OpenFreeMap ima pločice do razine 14, a PMTiles Sektora B na ispitanom mjestu od 0 do 15.
- **Iznad toga** prikazuje se rastegnuta niža razina: širina isječka na zumu 20 je 1/32 za OpenFreeMap i
  1/16 za PMTiles.
- **Bez pločice** (točka u New Yorku u datoteci Sektora B) ne prikazuje se ništa, a ako je u memoriji
  niža razina koja pokriva to mjesto, ona se rastegne.
- Pada nije bilo na zumovima 0, 2, 5, 8, 10, 18, 20, 22 i 26, na jednom mjestu i samo s PMTilesom.

Koliko je karta oštra kad je rastegnuta nije izmjereno.

## 5. Tamna tema karte

`openmaptiles_basemap_dark` i `protomaps_basemap_dark` rade, uz obavezni
`walkers::install_renderer(cc.wgpu_render_state.as_ref())` pri pokretanju. Snimke OpenFreeMapa su u
`snimke/`, a prosječna svjetlina je 218,4 i 8,5 od 255.

Na tamnoj snimci pozadina je crna, a kontrast cesta i manjih natpisa nizak, pa stil treba doraditi. Na
snimkama nema oznake izvora podataka, a program je mora prikazivati.

## Koliko vrijede provjere

- **Stvarni kvar:** projekcija pomaknuta za 25 px, poziv inercije poslije `Map::show` i zamijenjene
  teme ruše provjeru.
- **Samo zastavica:** namjerni kvarovi za `walkers_extras` i zum ručno postavljaju rezultat, pa ne
  dokazuju da provjera hvata pravu grešku.
- **Ne može pasti:** provjere stila i visine. Boju zadaje sama proba, a visina se izvodi iz veličine
  tipa. Te zaključke potvrđuje kod biblioteke.
- U `cargo test` rezultat za OpenFreeMap je upisan unaprijed. Stvarno se dohvaća samo u `--provjeri-sve`.

## Što nije provjereno

Pravi miš, Windows, Linux, karta samo s DGU WMS-om u HTRS96/TM, je li slika iz te adrese na pravom
mjestu, brzina i glatkoća (nisu ni mjerene, jer je računalo bilo opterećeno), zum izvan navedenih
razina i OpenFreeMap kroz `Map::show` na rubnim zumovima.
