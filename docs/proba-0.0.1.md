# Proba prije izdanja 0.0.1 alfa

Proba se radi pravim mišem, na programu iz paketa za preuzimanje, a ne iz `cargo run`. Prvo na macOS-u, a
zatim, ako ih budemo imali, na Windowsu i Linuxu. Podaci su izmišljeni: crta se bilo gdje na karti, a
projekt se sprema u novu praznu mapu.

Uz svaki korak piše što se mora vidjeti. Kad nešto ne odgovara, zapiši broj koraka i što se vidjelo, pa
nastavi dalje ako može.

## 1. Pokretanje

1. Pokreni program dvoklikom. Mora se otvoriti bez crnog prozora naredbenog retka. U naslovu piše
   inačica 0.0.1.
2. Prazan projekt: karta OpenFreeMap, ploča Slojevi prazna, ispod naziva piše „Novi projekt · nespremljeno”.
3. Pomakni kartu, pusti je dok se miče, zumiraj kotačićem i gumbima Približi i Udalji. Karta kliže i staje
   bez skoka.
4. Promijeni temu pa podlogu na DGU ortofoto i natrag. Izvor podataka na karti mijenja se s podlogom.
5. Mjerilo i koordinate u statusnoj traci mijenjaju se dok pomičeš miš.

## 2. Crtanje

1. Novi sloj „Točke” za točke. Nacrtaj tri točke. Pojavljuju se u ploči kao „Točka 1”, „Točka 2”, „Točka 3”.
2. Novi sloj „Linije” za linije. Nacrtaj liniju od pet točaka i završi dvoklikom.
3. Klikni kraj linije i dodaj još dvije točke. Poništi jednom: nestanu obje nove točke odjednom.
4. Novi sloj „Poligoni” za poligone. Nacrtaj poligon, pomakni mu jednu točku povlačenjem i obriši drugu
   desnim klikom.
5. Dok je u ploči odabran sloj „Linije”, nacrtaj točku. Točka ide u sloj „Točke”, a ispod karte piše zašto.
6. Poništi i ponovi nekoliko puta (⌘Z i ⌘Y na Macu, Ctrl+Z i Ctrl+Y drugdje). Crtež se vraća točno.
7. Napravi mapu, preimenuj je i isključi joj vidljivost. Slojevi u njoj nestaju s karte i vraćaju se.

## 3. Mjerenje

1. Izmjeri udaljenost između dva poznata mjesta i usporedi s mjerenjem u QGIS-u ili Google Earthu. Razlika
   ne smije biti veća od nekoliko centimetara.
2. Izmjeri površinu i spremi je kao poligon. Poligon se pojavi u sloju za poligone.
3. Očisti mjerenje. Na karti ne ostaje ništa osim spremljenog poligona.
4. Izmjeri površinu kao osmicu: četiri točke u križ. Već s četvrtom točkom pod mišem na križanju se pojavi
   crveni krug, a ispod karte piše da rub siječe sam sebe. Nacrtaj isti lik alatom Poligon i odaberi ga:
   krug i poruka su tu. Odaberi ispravan lik: krug i poruka nestanu.

## 4. Spremanje i otvaranje

1. Spremi kao, u novu mapu, pod nazivom „Proba 0.0.1”. Ispod naziva piše „spremljeno” i vrijeme.
2. Promijeni nešto. Uz naziv piše „nespremljeno”. Poništi promjenu: opet piše „spremljeno”.
3. Promijeni nešto pa odaberi Novi. Program pita za spremanje; odaberi Odustani. Ništa se ne gubi.
4. Zatvori program bez spremanja. Program pita za spremanje.
5. Otvori program i Otvori „Proba 0.0.1”. Sve je kao prije: slojevi, mape, nazivi, vidljivost i crtež.

## 5. QGIS

1. Klikni „Otvori u QGIS-u”. QGIS se otvara s projektom „TerraSimple”: nazivi slojeva, mape kao skupine,
   isti redoslijed i podloga OpenStreetMap. Objekti su na istom mjestu kao u TerraSimpleu.
2. U QGIS-u izmjeri isti poligon. Površina na elipsoidu ista je kao u TerraSimpleu.
3. U QGIS-u dodaj točku u sloj „Točke” i spremi izmjene sloja.
4. U TerraSimpleu ponovno otvori projekt. Nova točka iz QGIS-a je na karti.

## 6. Sukob

1. Projekt je otvoren u oba programa. U QGIS-u pomakni jednu točku i spremi izmjene sloja.
2. U TerraSimpleu nacrtaj nešto i spremi. Iskoči prozorčić „Datoteku je promijenio drugi program” s gumbima
   Odustani i Spremi kao…. Odustani: crtež je i dalje na karti, uz naziv piše „nespremljeno”.
3. Spremi opet pa odaberi Spremi kao…, u novu datoteku. Ispod naziva je nova datoteka i „spremljeno”.
4. Otvori novu datoteku: tvoj crtež je tu, a izmjena iz QGIS-a nije. Otvori izvornu: izmjena iz QGIS-a je tu,
   tvog crteža nema, a QGIS je i dalje otvara.

## 7. Tražilica

1. ⌘F (Ctrl+F drugdje) stavlja kursor u polje „Traži koordinate ili naziv”.
2. Upiši redom `45.5547, 18.6955`, `45,5547 18,6955`, `45°33'17"N 18°41'44"E` i HTRS96/TM istog mjesta
   (očitaj ga u statusnoj traci). Svaki put Enter: karta ide na isto mjesto s crvenom oznakom.
3. Upiši `33 51 36 J 151 12 00 I`. Karta ide u Sydney, na južnu polutku. Tekst u popisu čita se natrag u
   istu točku: kopiraj ga, upiši i provjeri.
4. Upiši `45 65 00 N 18 00 00 E`. Popis kaže da minute moraju biti manje od 60.
5. Nazovi neki objekt „Kuća” i upiši `kuca`. Objekt se nađe, odabere u ploči i karta ga pokaže cijelog.
6. Dok je objekt odabran, u polju obriši slova Backspaceom. Objekt ostaje.
7. Escape briše tekst i crvenu oznaku.

## 8. Na kraju

1. Klikni Pomoć u izborniku, pa gumb ? gore desno. Oboje otvara „O programu”: TerraSimple 0.0.1 alfa,
   licenca EUPL-1.2, izvori karte, slova i ikone. Zatvori ga gumbom Zatvori.
2. Zapiši na kojem je sustavu i računalu proba rađena, i što nije prošlo.
