# Popis promjena

## 0.0.1 alfa (u pripremi)

Prvo izdanje. Program je još u ranoj fazi, pa podatke čuvaj i drugdje.

### Karta

- Podloga OpenFreeMap u svijetloj i tamnoj inačici te DGU ortofoto. Izvor podataka uvijek piše na karti.
- Karta se pomiče mišem i kliže kad je pustiš. Gumbi Približi, Udalji i Cijeli zaslon.
- Mjerilo i koordinate pod mišem u HTRS96/TM.

### Crtanje i slojevi

- Točke, linije i poligoni. Klik dodaje točku, povlačenje je pomiče, desni klik briše, a dvoklik završava.
- Klik na kraj gotove linije nastavlja je s te strane.
- Ploča Slojevi: mape i slojevi, uključivanje i isključivanje, preimenovanje i brisanje. Prije brisanja sloja
  sa zapisima koje program ne prikazuje, npr. krivuljama, javlja koliko ih ima.
- Novi sloj se otvara u prozorčiću s nazivom, vrstom i mapom. Sloj drži samo jednu vrstu: točke, linije ili
  poligone.
- Svaka izmjena može se poništiti i ponoviti.

### Tražilica

- Polje „Traži koordinate ili naziv” (⌘F na Macu, Ctrl+F drugdje) čita koordinate u stupnjevima s točkom ili
  zarezom, u stupnjevima, minutama i sekundama i u HTRS96/TM, te nalazi slojeve i objekte po nazivu, i bez
  kvačica.
- Odabrana koordinata dobije privremenu crvenu oznaku na karti, koja se ne sprema. Objekt se odabere, karta
  ga pokaže cijelog, a mape iznad njega se otvore.
- Slovo S uz I ili Z znači sjever, a uz E ili W jug. Za upis izvan raspona tražilica kaže što ne valja, npr.
  „Minute moraju biti manje od 60.”

### Mjerenje

- Udaljenost i površina računaju se na elipsoidu WGS84, kao u QGIS-u.
- Izmjereno se može spremiti kao linija ili poligon.

### Datoteke i QGIS

- Novi, Otvori, Spremi i Spremi kao. Prečaci su na Macu ⌘N, ⌘O, ⌘S i ⇧⌘S, a na Windowsu i Linuxu Ctrl+N,
  Ctrl+O, Ctrl+S i Ctrl+Shift+S.
- Projekt je jedna GeoPackage datoteka. Tuđi slojevi, stupci i tablice u njoj ostaju netaknuti.
- Prije zatvaranja, otvaranja drugog projekta ili novog projekta program pita za nespremljene izmjene.
- Gumb „Otvori u QGIS-u” sprema projekt i otvara ga u QGIS-u, s nazivima slojeva, mapama i stilovima.
- Ako je QGIS u međuvremenu promijenio datoteku, spremanje se odbije i program ponudi Spremi kao. Tvoj crtež
  ide u novu datoteku, a datoteka koju je mijenjao QGIS ostaje kakva jest.
- Stil, natpisi i prozirnost koje namjestiš u QGIS-u i spremiš u projekt ostaju nakon spremanja iz
  TerraSimplea. Zamijene se samo kad tom sloju promijeniš stil u TerraSimpleu.

### Poznata ograničenja

- Ispis, KML i ostali formati još ne rade. Tražilica ne zna HDKS ni adrese.
- Gumbi Sjever, Uvezi sloj, Učitaj datoteku i Dodaj WMS još nemaju radnju. Gumb Ispis je skriven dok ispis
  ne proradi.
- Na Macu svaki klik na „Otvori u QGIS-u” pokreće novi QGIS, i kad je jedan već otvoren.
- Ispisni predlošci i ostale postavke cijelog projekta napravljene u QGIS-u ne ostaju nakon spremanja iz
  TerraSimplea. Ostaje samo ono što pripada pojedinom sloju.
- Spremanje velikog projekta je sporo: oko 1 s na 87 000 objekata.
- Pravim mišem isproban je samo na macOS-u. Na Windowsu i Linuxu zasad prolaze samo automatski testovi.
