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

trazi-mjesto = Traži mjesto ili koordinate
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
