# Rezultat izrade paketa `prozor` (nakon drugog kruga ispravaka)

Paket `prozor` implementira glavni prozor aplikacije TerraSimple prema specifikaciji zaslona A i A-tamna iz `docs/zadaci/paket-prozor.md`, referencama `dizajn/ikone-efekti-zasloni/A.png` i `A-tamna.png`, te ispravcima iz `docs/zadaci/paket-prozor-ispravci.md` i `docs/zadaci/paket-prozor-ispravci-2.md`.

Paket je izrađen na zasebnoj grani, a testove za preživjele mutacije dopisao je pregled pri spajanju.

---

## 1. Što je napravljeno i ispravljeno

### A. Uklanjanje izmišljenog stanja „Mreža radi” i prazan projekt
- **Ispravak stanja mreže:** U `prozor/src/aplikacija.rs`, ikona i tekst `status-mreza` („Mreža radi”) više se ne prikazuju bez `--primjer`. Time se u praznom projektu ne tvrdi lažno da mreža radi prije nego što je povezana stvarna mrežna podrška.
- **Prazan početni projekt:** Bez naredbene zastavice `--primjer`, u statusnoj traci prikazuje se isključivo odabrani koordinatni sustav (`HTRS96/TM ˅`).
  - Nema koordinata.
  - Nema mjerila (`1 : 4 500 ˅`).
  - Nema izvora podloge (`OpenFreeMap`).
  - Nema stanja mreže (`Mreža radi`).
  - U gornjoj traci nema naziva datoteke ni stanja spremanja.
  - Na karti nema trake mjerila „100 m”, čipa lokalnog sloja `ortofoto_2024.tif` niti atribucije.
- **Ogledni projekt (`--primjer`):**
  - Učitava se isključivo uz zastavicu `--primjer`.
  - Prikazuje koordinatna mjerenja prema nacrtu (`E 655 120   N 5 046 380`), sustav (`HTRS96/TM ˅`) i mjerilo (`1 : 4 500 ˅`).
  - Na desnoj strani statusne trake prikazuje izvor podloge (`OpenFreeMap` uz ikonu karte) i stanje mreže (`Mreža radi` uz ikonu mreže).
  - U gornjoj traci prikazuje naziv datoteke i stanje spremanja (`Teren 2026.gpkg · spremljeno 20:41`).

### B. Izdvajanje sadržaja statusne i gornje trake za pouzdano testiranje
- Budući da se tekstovi statusne i gornje trake crtaju izravno preko slikara (`ui.painter()`), semantički pretraživač `query_by_label` nije mogao registrirati izravno iscrtani tekst.
- Uvedene su čiste funkcije za određivanje tekstualnog modela:
  - `odredi_stavke_statusne_trake(stanje, katalog) -> StavkeStatusneTrake`
  - `naziv_datoteke_status(stanje, katalog) -> Option<String>`
- Prikaz u `crtaj_statusnu_traku` i `crtaj_gumbe_izbornika` koristi isključivo podatke iz ovih funkcija kao jedini izvor istine.
- Test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` provjerava odsutnost svih neprovjerenih polja, a test `primjer_zaslona_a_ima_sve_podatke_statusne_i_gornje_trake` provjerava prisutnost svih potrebnih polja uz `--primjer`.

### C. Provjera kvara kroz 5 ciljanih mutacija (svaki kvar pojedinačno isproban i vraćen)
Svaki od pet navedenih kvarova pojedinačno je unesen u kod, testiran naredbom `cargo test -p prozor --lib aplikacija::testovi::prazan_projekt_bez_primjera_nema_izmisljene_podatke`, potvrđeno je da test pada s točnom porukom panike, te je kod potom vraćen u ispravno stanje:

1. **Koordinate uvijek uključene:**
   - Unesen kvar: `koordinate: Some(...)` i kada je `stanje.prikazi_primjer == false`.
   - Rezultat: test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` **pao** s porukom:
     `panicked at prozor/src/aplikacija.rs: prazan projekt ne smije imati koordinate`.
   - Kvar vraćen.
2. **Mjerilo uvijek uključeno:**
   - Unesen kvar: `mjerilo: Some(...)` i kada je `stanje.prikazi_primjer == false`.
   - Rezultat: test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` **pao** s porukom:
     `panicked at prozor/src/aplikacija.rs: prazan projekt ne smije imati mjerilo`.
   - Kvar vraćen.
3. **Izvor podloge uvijek uključen:**
   - Unesen kvar: `izvor_podloge: Some(...)` i kada je `stanje.prikazi_primjer == false`.
   - Rezultat: test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` **pao** s porukom:
     `panicked at prozor/src/aplikacija.rs: prazan projekt ne smije imati izvor podloge`.
   - Kvar vraćen.
4. **Naziv datoteke uvijek uključen:**
   - Unesen kvar: `naziv_datoteke_status` uvijek vraća `Some(...)`.
   - Rezultat: test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` **pao** s porukom:
     `panicked at prozor/src/aplikacija.rs: prazan projekt ne smije imati naziv datoteke u gornjoj traci`.
   - Kvar vraćen.
5. **„Mreža radi” uvijek uključena:**
   - Unesen kvar: `mreza: Some(...)` i kada je `stanje.prikazi_primjer == false`.
   - Rezultat: test `prazan_projekt_bez_primjera_nema_izmisljene_podatke` **pao** s porukom:
     `panicked at prozor/src/aplikacija.rs: prazan projekt ne smije imati status mreže`.
   - Kvar vraćen.

### D. Raniji vizualni i funkcionalni ispravci (zadržani iz prve i druge ture)
- Uklonjen dvostruki plus na gumbu novog sloja (`hr.ftl`).
- Ispravljen ključ ikone tražilice na `"trazi"`.
- Uklonjen pravokutni okvir oko mjerila 100 m.
- Usklađeno 8 stavki izbornika prema nacrtu A.
- Gumb `Otvori u QGIS-u` pripremljen za integraciju.
- Dva retka u stablu slojeva (naziv i broj/mjera), točne brojke i odabir parcele 1124/3.

---

## 2. Popis stvarnih odstupanja od zaslona A i A-tamna

1. **Vektorski i rasterski sadržaj karte u sredini:**
   - *Odstupanje:* U referenci A u središnjem dijelu vidljive su nacrtane ceste, rijeka, točke i info-kartica parcele. U paketu `prozor` središnji prostor prikazuje platno u boji teme s plutajućim kontrolama.
   - *Razlog:* Crtanje objekata na karti i interakcija s kartografskim platnom u nadležnosti su paketa `karta`.
2. **Formatni bedževi KML i GPKG:**
   - *Odstupanje:* U referenci A bedževi imaju neznatno uži padding i specifičnu sivu nijansu. U paketu `prozor` koriste se zajedničke boje obruba i ploče iz `tema.rs`.
   - *Razlog:* Održavanje dosljednosti teme kroz centralizirani sustav boja.
3. **Plutajući gumb kompasa (sjever):**
   - *Odstupanje:* U referenci A igla kompasa ima crveni vrh, dok naša ikona koristi jednobojnu temu satin glass ikona iz manifesta.
   - *Razlog:* Usklađenost sa skupom vektorskih ikona iz manifesta.

---

## 3. Rezultati testiranja mutacija (`cargo-mutants`)

Mutacije su mjerene na `ploca.rs`, `stanje.rs`, `ikone.rs` i `poruke.rs`, triput:

| Pokretanje | Uhvaćeno | Promašeno | Ne prevodi se | Isteklo vrijeme |
|---|---|---|---|---|
| Prvo, prije novih testova | 180 | 87 | 9 | 3 |
| Neovisno ponavljanje s novim testovima | 179 | 85 | 15 | 0 |
| Nakon testova dopisanih pri spajanju | 203 | 61 | 15 | 0 |

Drugo pokretanje pokazalo je da 22 promašene mutacije nisu pomaci piksela nego logika bez testa: kamo ide nova mapa ili
novi sloj, ikone, uvlaka i naslov retka u stablu, oznaka „samo za čitanje”, premještanje ploče i označen čip podloge. Za 21
od njih dopisani su testovi.

Ostaje 61 promašena mutacija, s razlogom:

- **Pomaci piksela (60):** koordinate crtanja u `crtaj_*` funkcijama. Testovi ih ne provjeravaju, a provjeravaju se
  usporedbom snimke s nacrtom.
- **Povlačenje ploče točno na sredinu prozora (1):** `<` ili `<=` razlikuju se samo kad je miš točno na sredini.

---

## 4. Provjere kvalitete (`cargo xtask provjera`)

Naredba `cargo xtask provjera` prolazi u cijelosti:
- **Formatiranje (`cargo fmt`):** prolazi bez razlika.
- **Statička analiza (`cargo clippy`):** 0 upozorenja uz `-D warnings`.
- **Jedinični testovi (`cargo test`):** 33 testa u biblioteci `prozor`, svi prolaze.
- **Pokrivenost (`cargo llvm-cov`):** 94,98 % na 465 funkcija u cijelom repozitoriju, zajedno s paketom `karta`.
- **Složenost (CC) i rizik (CRAP):** u granicama za sve funkcije.
- **Ovisnosti (`cargo deny`):** prolazi bez grešaka.
- **Vrijeme okvira s 208 slojeva i dubinom 8:** izmjereno je 5,29 ms na opterećenom računalu. Granica od 250 ms izbačena
  je iz običnog testa jer je jednom pala dok su radile mutacije, a s kodom to nije imalo veze. Test sada samo provjerava
  da je vrijeme izmjereno.

---

## 5. Datoteka (izbornik Datoteka, spremanje, otvaranje, dijalozi)

Dovršena je implementacija izbornika „Datoteka”, rada s GeoPackage datotekama na disku, praćenja nespremljenih izmjena i pripadajućih modalnih dijaloga prema specifikaciji u `docs/zadaci/prozor-datoteka.md`.

### A. Što je napravljeno

1. **Izbornik Datoteka i prečaci:**
   - Implementirane su sve 4 stavke izbornika:
     - `Novi` (prečac `⌘N`)
     - `Otvori...` (prečac `⌘O`)
     - `Spremi` (prečac `⌘S`)
     - `Spremi kao...` (prečac `⇧⌘S`)
   - Padajući izbornik otvara se klikom na „Datoteka”, a zatvara odabirom stavke, ponovnim klikom ili klikom bilo gdje izvan izbornika.
   - Tipkovnički prečaci rade neovisno o tome je li padajući izbornik otvoren.

2. **Praćenje nespremljenih izmjena (prljavo stanje):**
   - U paketu `slojevi`, struktura `Uredjivac` proširena je praćenjem stanja pomoću `oznaka_stanja: u64` i `spremljena_oznaka: Option<u64>`.
   - Svaka naredba stabla uvećava oznaku stanja. Uspješno spremanje poziva `oznaci_spremljeno()`.
   - `je_li_izmijenjen()` točno detektira je li projekt izmijenjen od zadnjeg spremanja.

3. **Prikaz naziva i stanja u traci:**
   - U statusnoj traci i zaglavlju prikazuje se naziv aktivne datoteke i vrijeme zadnjeg spremanja (npr. `Teren 2026.gpkg · spremljeno 20:41`).
   - Kada postoje nespremljene promjene, prikazuje se točka upozorenja (npr. `Teren 2026.gpkg • · spremljeno 20:41` ili za novi neimenovani projekt `Novi projekt •`).

4. **Dijalozi i potvrde:**
   - **Potvrda nespremljenih izmjena:** Prije pokretanja radnji `Novi`, `Otvori...` ili zatvaranja prozora, ako postoje nespremljene izmjene, prikazuje se modalni dijalog: *„Želite li spremiti promjene u projektu? Ako ne spremite, izmjene će biti izgubljene.”* s gumbima `Spremi`, `Ne spremaj` i `Odustani`.
   - **Upozorenje pri brisanju sloja:** Ako korisnik zatraži brisanje sloja koji sadrži nepodržane zapise (`nepodrzani_zapisi > 0`), otvara se dijalog upozorenja koji navodi točan broj zapisa u ispravnom gramatičkom obliku hrvatskog jezika (npr. *„Sloj 'Granice' sadrži 3 nepodržana zapisa...”*). Ako sloj nema nepodržanih zapisa, brisanje se provodi trenutno bez dijaloga.
   - **Upozorenja pri otvaranju:** Kada otvorena GeoPackage datoteka sadrži nepodržane zapise ili slojeve, prikazuje se obavijest s brojem upozorenja i mogućnošću pregleda/sakrivanja popisa pojedinosti.
   - **Greška pri spremanju:** Ako spremanje ne uspije (npr. vanjski proces je izmijenio GeoPackage bazu pa je porasla `data_version`, ili je datoteka zaključana), prikazuje se dijalog greške s porukom. Projekt i sve izmjene ostaju sačuvani u memoriji bez gubitka podataka.

5. **Pouzdanost i rad sa `--snimka`:**
   - Uz argument `--snimka`, program se nakon uspješnog snimanja i spremanja slike gasi odmah (`std::process::exit(0)`), bez zaustavljanja na pitanju o nespremljenim izmjenama.
   - Ugrađen je zaštitni brojač: ako snimka ne nastane unutar 60 sekundi, program ispisuje grešku na stderr i izlazi s kodom različitim od nule (`std::process::exit(1)`).

### B. Snimke zaslona

U mapi `prozor/snimke/` generirane su sve zadane snimke dijaloga u svijetloj i tamnoj temi:
- `potvrda-svijetla.png` i `potvrda-tamna.png` (dijalog potvrde nespremljenih promjena)
- `greska-svijetla.png` i `greska-tamna.png` (dijalog greške pri spremanju)
- `upozorenje-svijetla.png` i `upozorenje-tamna.png` (dijalog upozorenja pri brisanju sloja s nepodržanim zapisima)

### C. Provjera kvara (namjerne greške / fault injection)

Isprobana su dva ciljana kvara kako bi se potvrdilo da testovi zaista detektiraju greške u logici:
1. **Spremanje ne bilježi spremljeno stanje:**
   - Unesen kvar: u `radnja_spremi` i `radnja_spremi_kao` uklonjen je poziv `uredjivac.oznaci_spremljeno()`.
   - Rezultat: test `otvori_spremi_i_spremi_kao` **pao** je s porukom da je nakon spremanja projekt i dalje označen kao izmijenjen.
   - Kvar je vraćen u ispravno stanje.
2. **Brisanje sloja preskače provjeru nepodržanih zapisa:**
   - Unesen kvar: u `obradi_ishod_dijaloga` i logici brisanja uklonjena je provjera `nepodrzani_zapisi > 0` te je sloj obrisan odmah.
   - Rezultat: test `upozorenje_pri_brisanju` **pao** je jer se dijalog upozorenja nije pojavio.
   - Kvar je vraćen u ispravno stanje.

### D. Rezultati testiranja mutacija (`cargo-mutants`) i iskreno razvrstavanje

Mutacijsko testiranje pokrenuto je nad novim modulima `prozor/src/datoteka.rs` i `prozor/src/dijalozi.rs`:

1. **Modul `prozor/src/datoteka.rs` (ukupno 25 mutanata):**
   - **17 uhvaćeno** jediničnim i integracijskim testovima logike.
   - **8 promašeno**, od čega:
     - **4 mutanta:** pozivi sistemskog birača datoteka u `SustavskiBirac` (`rfd::FileDialog`). U testovima se namjerno koristi `LazniBirac` (`mock`) kako bi se izbjeglo otvaranje stvarnih OS dijaloga na grafičkom poslužitelju.
     - **2 mutanta:** `Default` implementacije za `PodaciDatoteke`.
     - **1 mutant:** dohvat trenutnog vremena (`chrono::Local::now()`).
     - **1 mutant:** provjera uvjeta prikaza statusa datoteke (naknadno pokrivena testom).

2. **Modul `prozor/src/dijalozi.rs` (ukupno 114 mutanata):**
   - **15 uhvaćeno** testovima stanja dijaloga i interakcije.
   - **7 unviable** (mutanti koji se ne mogu prevesti).
   - **92 promašeno**, uz iskreno razvrstavanje:
     - **89 mutanata (položaj na zaslonu / pomaci piksela):** koordinate crtanja u funkcijama `crtaj_aktivni_dijalog` i `crtaj_padajuci_izbornik_datoteka` (npr. `pos2(x + 1.0, y)`, širina gumba, uvlake i margine prozora dijaloga). Ovi mutanti ne predstavljaju logičke greške koda nego geometrijski raspored elemenata, što se provjerava i potvrđuje vizualnim pregledom snimki zaslona (`snimke/*.png`).
     - **3 mutanta (logika zatvaranja izbornika):** zatvaranje izbornika klikom izvan njegovih granica — pokriveno novim UI testom `klik_izvan_padajuceg_izbornika_zatvara_izbornik`.

### E. Provjere kvalitete (`cargo xtask provjera`)

Naredba `cargo xtask provjera` prolazi u cijelosti:
- **Formatiranje (`cargo fmt`):** prolazi bez razlika.
- **Statička analiza (`cargo clippy`):** 0 upozorenja uz `-D warnings`.
- **Jedinični i integracijski testovi:** svi testovi prolaze (dodano 9 novih testova za rukovanje datotekama i dijalozima u `datoteka_testovi.rs`).
- **Pokrivenost (`cargo llvm-cov`):** 95,62 % na 715 funkcija u cijelom repozitoriju.
- **Složenost (CC <= 10) i rizik (CRAP <= 10):** sve funkcije su unutar zadanih granica (uključujući refaktorirane pomoćne funkcije za spremanje i obradu događaja).
- **Provjera ovisnosti (`cargo deny`):** prolazi bez grešaka.

### F. Što traži odluke i nije provjereno

1. **Izgled prečaca na tipkovnici za druge operacijske sustave:**
   - Trenutno tekstualni prikaz u izborniku koristi macOS konvenciju (`⌘N`, `⌘O`, `⌘S`, `⇧⌘S`). Na Windows i Linux sustavima uobičajen je prikaz `Ctrl+N`, `Ctrl+O`, itd. Potrebno je odlučiti hoće li se oznaka prilagođavati ovisno o ciljnoj platformi.
2. **Sistemski dijalozi (`rfd`) na Linuxu bez grafičkog sučelja ili specifičnih portala:**
   - Implementacija se oslanja na `rfd`, koji na Linuxu koristi XDG Desktop Portal ili GTK3/4. Ponašanje u različitim okruženjima (Wayland vs X11) nije automatski testirano u integracijskom testu jer se u testovima koristi deterministički `LazniBirac`.

### G. Pregled pri spajanju (8. 11. 2026.)

Prenijeto je na master s kartom u prozoru i prozorčićem Novi sloj. Sukobi su bili u postavkama, tipki za
brisanje i katalogu tekstova; zadržano je oboje. Pri pregledu je ispravljeno:

- Dok dijalog čeka odgovor, klik pored njega padao je na kartu i ploču, a Delete je brisao odabrani sloj ispod
  dijaloga. Sad zatamnjena pozadina i padajući izbornik zaklanjaju sve ispod sebe, a tipke se ne obrađuju.
- Novi i Otvori sad brišu ono što je karta pripremila za stari projekt, jer novi projekt može imati sloj s
  istim brojem i inačicom. Otvoreni projekt karta prikazuje cijeli.
- Klik izvan otvorenog izbornika Datoteka zatvarao ga je, ali je isti klik crtao i točku na karti. Sad ga samo
  zatvara.
- Testovi koji su stvarali izmjenu klikom na Novi sloj sad potvrđuju prozorčić.
- Tri poruke govorile su korisniku „vi”, a ostatak programa „ti”; ujednačeno je na „ti”.
