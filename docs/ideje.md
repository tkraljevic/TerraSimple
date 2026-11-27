# Ideje

Ideje koje još nisu u planu. Uzimamo ih usput, kad se uklope u posao koji ionako radimo.
Uz svaku je upisano odakle je došla i što treba provjeriti prije nego što uđe u plan.

## Iz God's Eye Viewa (15. 11. 2026.)

[God's Eye View](https://github.com/bilawalsidhu/gods-eye-view) je web 3D globus u
JavaScriptu (CesiumJS) koji uživo prikazuje avione, brodove, satelite, potrese i javne
kamere. Kod je pod licencom MIT, a podaci su pod uvjetima svojih izvora. Neke priložene
zbirke smiju se koristiti samo nekomercijalno. Kod im je u drugom jeziku, pa od njih
uzimamo samo ideje.

### Traženje mjesta po imenu

Tražilica sad prepoznaje koordinate, slojeve i objekte, ali ne naselja i druga mjesta.
God's Eye View prvo pita Photon (komoot), a kad on ne odgovori, Nominatim. Oba rade bez
ključa.

- Uključuje se u postavkama, jer upit odlazi na tuđi poslužitelj.
- Nominatim dopušta najviše jedan upit u sekundi, bez traženja dok se tipka, i traži
  vlastiti `User-Agent`.
- Prije plana provjeriti uvjete javnog Photona.

### `geo:` adrese

Tražilica bi primila `geo:45.55,18.69` kao i obične koordinate. Uz objekt bi se njegova
`geo:` adresa mogla kopirati i poslati drugome. To je mala izmjena koja se može uklopiti
uz bilo koji posao na tražilici.

### Podloge za fazu 5

Dva izvora kojih još nema u [podloge.md](podloge.md):

- **NASA GIBS** (WMTS, bez ključa): svježe snimke Sentinel-2 i Landsat.
- **Panoramax**: otvorene fotografije s ulice, u duhu OpenStreetMapa.

Uvjete provjeriti istim redom kao za ostale podloge. Njihov
[DATA_SOURCES.md](https://github.com/bilawalsidhu/gods-eye-view/blob/main/DATA_SOURCES.md)
je dobar popis izvora s uvjetima.

### Što ne uzimamo

- Esri satelitsku podlogu bez ključa. Oni je drže kao zadanu, a prema podloge.md Esrijevi
  uvjeti to ne dopuštaju. Kod nas ostaje samo ako je korisnik doda sam.
- Praćenje uživo: letove, brodove, javne kamere i kamere za tablice. To nije ono čemu
  TerraSimple služi, a kamere otvaraju pitanje privatnosti.
- 3D globus. Plan kaže da ga nema.
