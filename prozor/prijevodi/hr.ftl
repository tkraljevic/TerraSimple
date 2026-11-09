naslov-aplikacije = TerraSimple
izbornik-datoteka = Datoteka
izbornik-uredi = Uredi
izbornik-crtanje = Crtanje
izbornik-slojevi = Slojevi
izbornik-prikaz = Prikaz
izbornik-ispis = Ispis
izbornik-alati = Alati
izbornik-pomoc = Pomoć

gumb-qgis = Otvori u QGIS-u
primjer-datoteka-status = Teren 2026.gpkg · spremljeno 20:41

gumb-ponisti = Poništi
gumb-ponovi = Ponovi
gumb-tema = Promijeni temu
gumb-pomoc = Pomoć

trazi-mjesto = Traži koordinate ili naziv
trazi-nema-rezultata = Nema koordinata ni objekta s tim nazivom.
trazi-sloj = Sloj: { $naziv }
trazi-u-sloju = u sloju { $sloj }
trazi-zamijenjen-redoslijed = zamijenjena širina i dužina, Hrvatska
trazi-zamijenjen-htrs = zamijenjen istok i sjever, HTRS96/TM
trazi-greska-sirina = Zemljopisna širina mora biti između -90° i 90°.
trazi-greska-duzina = Zemljopisna dužina mora biti između -180° i 180°.
trazi-greska-minute = Minute moraju biti manje od 60.
trazi-greska-sekunde = Sekunde moraju biti manje od 60.
trazi-greska-previse-brojeva = Previše brojeva u koordinati.
trazi-greska-htrs = Koordinate HTRS96/TM su izvan raspona za Hrvatsku.
atribucija-osm = © OpenStreetMap suradnici
mjerilo-100m = 100 m

alat-odaberi = Odaberi
alat-tocka = Točka
alat-linija = Linija
alat-poligon = Poligon
alat-mjeri = Mjeri

lebdeci-zaslon = Cijeli zaslon
lebdeci-ispis = Ispis
lebdeci-sjever = Sjever
lebdeci-plus = Približi
lebdeci-minus = Udalji

ploca-slojevi = Slojevi
ploca-premjesti = Premjesti ploču Slojevi
ploca-zatvori = Zatvori ploču Slojevi
gumb-novi-sloj = Novi sloj
gumb-uvezi-sloj = Uvezi sloj
gumb-nova-mapa = Mapa
gumb-preimenuj = Preimenuj
gumb-obrisi = Obriši

podloga-naslov = Podloga
podloga-karta = Karta
podloga-dgu = DGU ortofoto
podloga-lokalna = ortofoto_2024.tif
gumb-ucitaj-datoteku = Učitaj datoteku
gumb-dodaj-wms = Dodaj WMS

status-sustav = HTRS96/TM ˅
status-koordinate = E {$istok}   N {$sjever}
status-primjer-mjerilo = 1 : 4 500 ˅
status-izvor = OpenFreeMap
status-mreza = Mreža radi

sloj-samo-citanje = samo za čitanje
oznaka-zakljucano = Zaključano
oznaka-vidljivost = Vidljivost
oznaka-prosiri = Proširi ili skupi

broj-slojeva =
    { $broj ->
        [one] { $broj } sloj
        [few] { $broj } sloja
       *[other] { $broj } slojeva
    }

broj-objekata =
    { $broj ->
        [one] { $broj } objekt
        [few] { $broj } objekta
       *[other] { $broj } objekata
    }

novi-sloj-naziv = Sloj { $redni }
nova-mapa-naziv = Mapa { $redni }

primjer-teren-2026 = Teren 2026
primjer-parcele = Parcele
primjer-parcela-1 = Parcela 1118/2
primjer-parcela-2 = Parcela 1124/3
primjer-parcela-3 = Parcela 1126
primjer-parcela-ostalo = Parcela { $redni }
primjer-trase = Trase
primjer-trasa-1 = Trasa 1
primjer-tocke = Točke
primjer-tocka = Točka { $redni }
primjer-parcele-2025 = Parcele 2025
primjer-stare-parcele = Stare parcele
primjer-granice = Granice
primjer-uvezeno = Uvezeno
primjer-obilazak = obilazak
primjer-obilazak-tocka = Obilazak { $redni }
primjer-nasipi = nasipi
primjer-oznaka-kml = KML
primjer-oznaka-gpkg = GPKG

# Karta u prozoru
sloj-tocke = Točke
sloj-linije = Linije
sloj-poligoni = Poligoni
mjera-lika = { $povrsina }, opseg { $opseg }
mjerenje-udaljenost = Udaljenost
mjerenje-povrsina = Površina
mjerenje-pocetak = Klikni na kartu za prvu točku.
mjerenje-dijelova =
    { $broj ->
        [one] Ukupno, { $broj } dio. Računato na elipsoidu WGS84.
        [few] Ukupno, { $broj } dijela. Računato na elipsoidu WGS84.
       *[other] Ukupno, { $broj } dijelova. Računato na elipsoidu WGS84.
    }
mjerenje-spremi-liniju = Spremi kao liniju
mjerenje-spremi-poligon = Spremi kao poligon
mjerenje-ocisti = Očisti
mjerenje-zatvori = Zatvori mjerenje
status-mjerilo = 1 : { $nazivnik }
status-izvor-dgu = DGU ortofoto
poruka-premalo =
    { $broj ->
        [one] Za završetak treba bar { $broj } točka.
        [few] Za završetak treba bar { $broj } točke.
       *[other] Za završetak treba bar { $broj } točaka.
    }
poruka-ne-moze-brisati = Točka nije obrisana: objekt bez nje ne bi bio valjan.
poruka-naredba = Izmjena nije napravljena: { $razlog }.

# Novi sloj (zaslon F) i nazivi novih objekata
novi-sloj-naslov = Novi sloj
novi-sloj-polje-naziv = Naziv
novi-sloj-vrsta = Što će sloj sadržavati
novi-sloj-tocke = Točke
novi-sloj-linije = Linije
novi-sloj-poligone = Poligone
novi-sloj-mapa = U mapu
novi-sloj-bez-mape = Na vrh, bez mape
novi-sloj-napomena = Sloj drži samo jednu vrstu, pa točke, linije i poligoni idu svaki u svoj sloj. Vrsta se poslije ne mijenja.
novi-sloj-prazan-naziv = Upiši naziv sloja.
gumb-odustani = Odustani
gumb-napravi-sloj = Napravi sloj
objekt-tocka = Točka { $redni }
objekt-linija = Linija { $redni }
objekt-poligon = Poligon { $redni }
vrsta-sloja-tocka = točke
vrsta-sloja-linija = linije
vrsta-sloja-poligon = poligone
poruka-drugi-sloj = Sloj „{ $sloj }” sadrži { $vrsta_sloja }, pa novi objekt ide u sloj za { $vrsta_alata }.

izbornik-datoteka-novi = Novi
izbornik-datoteka-otvori = Otvori…
izbornik-datoteka-spremi = Spremi
izbornik-datoteka-spremi-kao = Spremi kao…
# Prečaci: na Macu ⌘, drugdje Ctrl (Odluke (9. 11. 2026.)).
precac-novi = { $sustav ->
    [mac] ⌘N
   *[ostali] Ctrl+N
}
precac-otvori = { $sustav ->
    [mac] ⌘O
   *[ostali] Ctrl+O
}
precac-spremi = { $sustav ->
    [mac] ⌘S
   *[ostali] Ctrl+S
}
precac-spremi-kao = { $sustav ->
    [mac] ⇧⌘S
   *[ostali] Ctrl+Shift+S
}

potvrda-naslov = Nespremljene izmjene
potvrda-poruka = Želiš li spremiti izmjene u projektu prije nastavka?
potvrda-gumb-spremi = Spremi
potvrda-gumb-ne-spremaj = Ne spremaj
potvrda-gumb-odustani = Odustani

brisanje-upozorenje-sloj = Sloj „{ $naziv }” ima { $broj_zapisa } koje program ne prikazuje, npr. krivulje ili prazne geometrije. Kad spremiš projekt, nestat će i oni.
brisanje-upozorenje-mapa = Mapa „{ $naziv }” ima { $broj_zapisa } koje program ne prikazuje, npr. krivulje ili prazne geometrije. Kad spremiš projekt, nestat će i oni.
broj-nepodrzanih-zapisa =
    { $broj ->
        [one] { $broj } zapis
        [few] { $broj } zapisa
       *[other] { $broj } zapisa
    }
brisanje-gumb-obrisi-sloj = Obriši sloj
brisanje-gumb-obrisi-mapu = Obriši mapu
brisanje-gumb-odustani = Odustani

greska-otvaranja-naslov = Greška pri otvaranju
greska-spremanja-naslov = Greška pri spremanju
greska-gumb-u-redu = U redu
greska-sukob-promjena = Projekt nije spremljen: datoteku je u međuvremenu promijenio drugi program. Otvori je ponovno.
greska-spremanja-opcenito = Projekt nije spremljen: { $detalji }. Pokušaj ponovno ili spremi pod drugim nazivom.
greska-otvaranja-opcenito = Projekt nije otvoren: { $detalji }. Provjeri je li datoteka ispravan GeoPackage.

upozorenja-naslov = Upozorenja pri otvaranju
upozorenja-broj =
    { $broj ->
        [one] Pri otvaranju je zabilježeno { $broj } upozorenje.
        [few] Pri otvaranju su zabilježena { $broj } upozorenja.
       *[other] Pri otvaranju je zabilježeno { $broj } upozorenja.
    }
upozorenja-pokazi-popis = Prikaži popis ({ $broj })
upozorenja-sakrij-popis = Sakrij popis
upozorenja-gumb-u-redu = U redu

datoteka-status-spremljeno = { $naziv } · spremljeno { $vrijeme }
datoteka-status-izmijenjeno = { $naziv } · nespremljeno
datoteka-status-spremljeno-izmijenjeno = { $naziv } · spremljeno { $vrijeme } (nespremljeno)
datoteka-novi-nespremljeno = Novi projekt · nespremljeno

greska-qgis-naslov = QGIS se nije otvorio
