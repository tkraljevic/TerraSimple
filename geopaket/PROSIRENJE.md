# Proširenje stabla, inačica 1

Naziv je `terrasimple_stablo`, definicija `urn:terrasimple:stablo:1`, opseg
`read-write`. Za obje tablice postoji redak u `gpkg_extensions`; stupac je NULL.
Definicija URN odnosi se na ovaj dokument. Prefiks `gpkg_ts_` skriva pomoćne tablice
iz QGIS-ova popisa slojeva. Nisu upisane u `gpkg_contents` kao slojevi.

`gpkg_ts_stablo`:

| Stupac | Značenje |
|---|---|
| id | Jedinstveni unutarnji broj mape ili sloja |
| vrsta | `mapa` ili `sloj` |
| roditelj | Broj nadređene mape, NULL za korijen |
| polozaj | Redoslijed među djecom, od nule |
| naziv | Naziv mape ili sloja |
| tablica | Tablica objekata sloja, NULL za mapu |
| stil | JSON svih svojstava našeg osnovnog stila; QML ima prednost pri čitanju |
| stanje | 0 uređivo, 1 samo čitanje, 2 čeka slanje, 3 u sukobu |

`gpkg_ts_objekti` povezuje `tablica`, SQLite `fid` i unutarnji `id` objekta.
Ključevi su `(tablica,fid)` i `(tablica,id)`. `otvoreni` je JSON niz logičkih
vrijednosti koji pamti jesu li poligonski prstenovi u modelu bili otvoreni;
GeoPackage zapis uvijek ima zatvorene prstenove. Nedostajući stupac može se dodati.
Veze ostaju nakon brisanja radi poništavanja i očuvanja brojeva. Pri čitanju se
provjerava jedinstvenost unutarnjih brojeva među stvarno prisutnim objektima.
Najveći povijesni broj sprječava ponovno dodjeljivanje starog unutarnjeg broja.

Program koji ne poznaje proširenje može otvarati i mijenjati obične feature tablice.
Novi QGIS sloj bez retka stabla dodaje se na kraj korijena; novi objekt dobiva novi
unutarnji broj. Folderi i redoslijed nisu QGIS projekt i ne postaju QGIS stablo.

Poznati atributi objekta su `naziv`, `opis`, `stil_boja`, `stil_debljina`,
`stil_ispuna`, `stil_znak` i `stil_velicina`. Prazan stil objekta nasljeđuje stil
sloja. Boje imaju četiri sastavnice 0–255, širine i veličine su u milimetrima,
a oblici su `circle`, `square`, `triangle`, `star`. Nepoznati stupci ostaju.
Premještanje koje bi nepoznati atribut prepisalo preko geometrije ili primarnog
ključa odredišnog sloja vraća grešku i ne sprema dio izmjene.
