# Proba prije izdanja

Proba se radi pravim mišem, na programu iz paketa za preuzimanje, a ne iz `cargo run`. Prvo na macOS-u, a
zatim na Windowsu i Linuxu. Podaci su izmišljeni: crta se bilo gdje na karti, a projekt se sprema u novu
praznu mapu. Za uvoz iz Google Eartha treba nekoliko vlastitih mapa iz Earth Proa.

Uz svaki korak piše što se mora vidjeti. Kad nešto ne odgovara, zapiši broj koraka i što se vidjelo, pa
nastavi dalje ako može. Na Macu je ⌘ ono što je drugdje Ctrl, a tipka delete je Backspace.

## 1. Pokretanje

1. Pokreni program dvoklikom. Mora se otvoriti bez crnog prozora naredbenog retka. U naslovu piše inačica
   izdanja.
2. Prazan projekt: karta OpenFreeMap, ploča Slojevi prazna, ispod naziva piše „Novi projekt · nespremljeno”.
3. Pomakni kartu, pusti je dok se miče, zumiraj kotačićem (na Macu dva prsta ili štipanje) i gumbima
   Približi i Udalji. Karta kliže i staje bez skoka.
4. Promijeni temu pa podlogu na DGU ortofoto i natrag. Izvor podataka na karti mijenja se s podlogom.
5. Mjerilo i koordinate u statusnoj traci mijenjaju se dok pomičeš miš.

## 2. Crtanje

1. Novi sloj „Točke” za točke. Nacrtaj točku. Odmah se otvori prozorčić „Naziv objekta” s označenim
   „Točka 1”. Upiši „Bunar” i Enter. U ploči piše „Bunar”, a alat se vratio na Odaberi.
2. Nacrtaj još dvije točke i prozorčić zatvori s Esc. Ostaju „Točka 2” i „Točka 3”.
3. Novi sloj „Linije” za linije. Nacrtaj liniju od pet točaka. Dok crtaš, Backspace briše zadnju točku.
   Završi desnim klikom (na Macu i klik s dva prsta ili ⌃ + klik).
4. Klikni kraj linije i dodaj još dvije točke. Poništi jednom: nestanu obje nove točke odjednom.
5. Novi sloj „Poligoni” za poligone. Nacrtaj poligon i završi desnim klikom. Odaberi ga, pomakni mu jednu
   točku povlačenjem, a drugu klikni i obriši Backspaceom. Lik ostaje, s točkom manje.
6. Dok je u ploči odabran sloj „Linije”, nacrtaj točku. Točka ide u sloj „Točke”, a ispod karte piše zašto.
7. Poništi i ponovi nekoliko puta (⌘Z i ⇧⌘Z). Crtež se vraća točno.
8. Napravi mapu, preimenuj je i isključi joj vidljivost. Slojevi u njoj nestaju s karte i vraćaju se.

## 3. Odabir, brisanje i premještanje

1. Klikni prazno mjesto lijevo iznad dva objekta, pa desno ispod njih. Okvir slijeva nadesno odabere
   samo ono što je cijelo u njemu. Isto zdesna nalijevo odabere i ono što okvir samo dotakne.
2. ⇧ + klik na odabrani objekt izbaci ga iz odabira.
3. Sam Backspace (delete na Macu) bez odabrane točke ne briše objekt. ⌘ + delete, ili Delete, obriše sve
   odabrane objekte odjednom, a jedan ⌘Z ih vrati.
4. Desni klik na sloj u ploči, pa „Premjesti u” i druga mapa. Sloj je u njoj. Isto povlačenjem mišem.
5. Povuci objekt na drugi sloj iste vrste. Objekt je u tom sloju, a na sloj druge vrste ne ide.

## 4. Stil

1. Desni klik na sloj „Linije”, pa „Stil”. Odaberi plavu boju, isprekidanu crtu i povuci debljinu. Karta se
   mijenja dok povlačiš. Jedan ⌘Z vraća cijelo povlačenje.
2. Desni klik na poligon, pa „Stil”. Mijenja se samo taj objekt. Odaberi šrafuru: lik je iscrtan kosim
   crtama. „Cijeli sloj” pa „Bez ispune”: svi likovi u sloju imaju samo rub.
3. Krug „+” otvara birač s nijansom, zasićenošću i svjetlinom. Boja se mijenja dok povlačiš.
4. „Vrati zadano” vraća narančastu, a „Gotovo” vraća na ploču Slojevi.

## 5. Mjerenje

1. Izmjeri udaljenost između dva poznata mjesta i usporedi s mjerenjem u QGIS-u ili Google Earthu. Razlika
   ne smije biti veća od nekoliko centimetara.
2. Izmjeri površinu i spremi je kao poligon. Poligon se pojavi u sloju za poligone.
3. Očisti mjerenje. Na karti ne ostaje ništa osim spremljenog poligona.
4. Izmjeri površinu kao osmicu: četiri točke u križ. Već s četvrtom točkom pod mišem na križanju se pojavi
   crveni krug, a ispod karte piše da rub siječe sam sebe. Nacrtaj isti lik alatom Poligon i odaberi ga:
   krug i poruka su tu. Odaberi ispravan lik: krug i poruka nestanu.

## 6. Uvoz iz Google Eartha

1. U Google Earth Prou spremi jednu svoju mapu kao KMZ. U TerraSimpleu „Uvezi sloj” i odaberi je. Na dnu
   ploče je nova mapa nazvana po datoteci, a karta se pomakne na uvezeno.
2. Prozorčić „Uvoz je gotov” kaže koliko je objekata, slojeva i mapa uvezeno, i što nije (npr. mrežne veze).
3. Mape su iste kao u Earthu, a u svakoj su slojevi Točke, Linije i Poligoni. Nazivi, opisi i boje su isti.
4. Jedan ⌘Z ukloni cijeli uvoz.
5. Uvezi i cijeli `myplaces.kml` iz mape podataka Google Eartha. Uvoz traje oko sekunde, a karta se i dalje
   glatko pomiče.

## 7. Spremanje i otvaranje

1. Spremi kao, u novu mapu, pod nazivom „Proba”. Ispod naziva piše „spremljeno” i vrijeme.
2. Promijeni nešto. Uz naziv piše „nespremljeno”. Poništi promjenu: opet piše „spremljeno”.
3. Promijeni nešto pa odaberi Novi. Program pita za spremanje; odaberi Odustani. Ništa se ne gubi.
4. Zatvori program bez spremanja. Program pita za spremanje.
5. Otvori program i Otvori „Proba”. Sve je kao prije: slojevi, mape, nazivi, stilovi, vidljivost i crtež.

## 8. QGIS

1. Klikni „Otvori u QGIS-u”. QGIS se otvara s projektom „TerraSimple”: nazivi slojeva, mape kao skupine,
   isti redoslijed i podloga OpenStreetMap. Objekti su na istom mjestu i istog stila kao u TerraSimpleu,
   s isprekidanom crtom i šrafurom.
2. U QGIS-u izmjeri isti poligon. Površina na elipsoidu ista je kao u TerraSimpleu.
3. U QGIS-u dodaj točku u sloj „Točke” i spremi izmjene sloja.
4. U TerraSimpleu ponovno otvori projekt. Nova točka iz QGIS-a je na karti.

## 9. Sukob

1. Projekt je otvoren u oba programa. U QGIS-u pomakni jednu točku i spremi izmjene sloja.
2. U TerraSimpleu nacrtaj nešto i spremi. Iskoči prozorčić „Datoteku je promijenio drugi program” s gumbima
   Odustani i Spremi kao…. Odustani: crtež je i dalje na karti, uz naziv piše „nespremljeno”.
3. Spremi opet pa odaberi Spremi kao…, u novu datoteku. Ispod naziva je nova datoteka i „spremljeno”.
4. Otvori novu datoteku: tvoj crtež je tu, a izmjena iz QGIS-a nije. Otvori izvornu: izmjena iz QGIS-a je tu,
   tvog crteža nema, a QGIS je i dalje otvara.

## 10. Tražilica

1. ⌘F stavlja kursor u polje „Traži koordinate ili naziv”.
2. Upiši redom `45.5547, 18.6955`, `45,5547 18,6955`, `45°33'17"N 18°41'44"E` i HTRS96/TM istog mjesta
   (očitaj ga u statusnoj traci). Svaki put Enter: karta ide na isto mjesto s crvenom oznakom.
3. Upiši `33 51 36 J 151 12 00 I`. Karta ide u Sydney, na južnu polutku. Tekst u popisu čita se natrag u
   istu točku: kopiraj ga, upiši i provjeri.
4. Upiši `45 65 00 N 18 00 00 E`. Popis kaže da minute moraju biti manje od 60.
5. Nazovi neki objekt „Kuća” i upiši `kuca`. Objekt se nađe, odabere u ploči i karta ga pokaže cijelog.
6. Dok je objekt odabran, u polju obriši slova Backspaceom. Objekt ostaje.
7. Escape briše tekst i crvenu oznaku.

## 11. Na kraju

1. Gumb ? gore desno, F1 ili ⌘/ otvaraju „Prečaci tipkovnice i miša”. Na Macu piše ⌘ i delete, na Windowsu
   i Linuxu Ctrl i Delete.
2. Pomoć u izborniku otvara „O programu”: inačica, licenca EUPL-1.2, izvori karte, slova i ikone.
3. Zapiši na kojem je sustavu i računalu proba rađena, i što nije prošlo.
