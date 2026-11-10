# Popis promjena

## 0.0.5 alfa (u radu)

- Novo: ploča Stil. Desni klik na sloj ili objekt u ploči Slojevi, pa „Stil”. Bira se jedna od osam boja ili
  bilo koja druga, debljina crte, prozirnost, a za točke veličina. Iz retka objekta mijenja se samo taj
  objekt ili, uz „Cijeli sloj”, cijeli sloj. Karta se mijenja odmah, a cijelo povlačenje klizača poništava
  se jednim korakom. Stil se sprema u GeoPackage, pa ga QGIS otvara s njim.
- Objekte, slojeve i mape sad briše i ⌘ + delete na Macu (Ctrl + Backspace drugdje), kao u Finderu. Delete
  i fn + delete rade kao i dosad.
- Sam Backspace (na Macu tipka delete) briše samo točku: zadnju dok crtaš, a odabranu na gotovom objektu.
  Dosad je bez odabrane točke brisao cijeli objekt, pa se lako moglo obrisati više nego što si htio.

## 0.0.4 alfa (probno, 10. 11. 2026.)

- Mapa ili sloj premješta se u drugu mapu desnim klikom i stavkom „Premjesti u”, koja nudi sve mape i
  „Izvan mapa”. Mapa ne nudi samu sebe ni svoje podmape.
- Isto se može povlačenjem mišem: ispušteno na mapu ide na kraj te mape, a ispušteno na sloj ide ispred
  njega. Premještanje se poništava kao i svaka druga izmjena.
- Objekt se na isti način prebacuje u drugi sloj iste vrste: poligon u sloj poligona, linija u sloj linija.
  Podizbornik pokazuje mape kao natpise, a nudi samo slojeve u koje objekt smije.
- Desni klik sad završava crtu, lik ili mjerenje, umjesto dvoklika. Dok crtaš, Backspace briše zadnju točku.
- Čim je objekt nacrtan, otvara se prozorčić s nazivom. Predloženi naziv, npr. „Poligon 3”, sav je označen,
  pa ga upisivanje zamijeni. Enter sprema naziv, a Esc ostavlja predloženi.
- Nakon crtanja alat se vraća na Odaberi. Dosad je ostajao uključen, iako je karta već prešla na odabir.
- Odabir kao u AutoCAD-u: klik na objekt dodaje ga u odabir, a Shift + klik ga izbacuje. Klik na prazno
  mjesto počinje okvir, a drugi klik ga zatvara. Okvir slijeva nadesno bira samo ono što je cijelo u njemu,
  a zdesna nalijevo sve što dotakne. Esc poništava odabir.
- Delete briše sve odabrane objekte odjednom, a poništava se jednim korakom.
- Klik na točku odabranog objekta odabire tu točku, a Backspace je briše. Desni klik više ne briše točku.

## 0.0.3 alfa (probno, 10. 11. 2026.)

- Pri zatvaranju programa „Ne spremaj” nije zatvaralo prozor, nego je pitanje stalno dolazilo ponovno, pa se
  izaći moglo samo spremanjem. Sad se prozor zatvori, a izmjene se ne spreme.
- Desni klik na mapu, sloj ili objekt u ploči Slojevi otvara izbornik s „Preimenuj” i „Obriši”. Preimenovati se
  moglo i prije, ali samo dvoklikom ili tipkom F2, pa se to teško nalazilo.
- Naziv koji mijenjaš sad je u polju i sav je označen, pa ga upisivanje zamijeni. Strelica skida oznaku i
  upisano ide na kraj. Enter sprema novi naziv, a Esc ga odbacuje.

## 0.0.2 alfa (probno, 10. 11. 2026.)

- Program se na Windowsu nije pokretao: tražio je slike ikona na disku, po stazi s računala na kojem je
  izgrađen. Slike su sad ugrađene u program, pa mu ne treba ništa osim njega samoga.
- Kad se program ne može pokrenuti, sad to kaže u prozorčiću i opis greške zapiše u
  `TerraSimple-greska.txt` u privremenoj mapi. Prije se na Windowsu samo ništa nije dogodilo.
- Paketi se na stroju za izgradnju sad provjeravaju bez izvornog koda pri ruci, pa bi se ovakva greška
  vidjela prije izdanja.

## 0.0.1 alfa (probno, 10. 11. 2026.)

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
- Lik koji siječe sam sebe, npr. osmica, dobiva crveni krug na mjestu presjeka, i dok ga crtaš ili mjeriš
  i kad ga odabereš. Ispod karte piše da površina zato nije točna.

### Program

- U naslovu prozora piše inačica. Pomoć u izborniku i gumb ? otvaraju „O programu”, s inačicom, licencom i
  izvorima karte.
- Paketi za Windows, macOS i Linux raspakiraju se u korisničku mapu i ne traže administratora.

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
