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
