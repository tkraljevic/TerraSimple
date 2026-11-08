//! Pravila crtanja: lijevi klik dodaje točku, povlačenje je pomiče, desni klik briše, a dvoklik
//! završava. Ne zna za grafičku karticu ni za `walkers`, a sve izmjene idu kroz naredbe.

use eframe::egui::{Pos2, pos2};
use geometrija::mjeri_crtu;
use slojevi::{
    Geometrija, Greska, IdObjekta, IdSloja, IzmjenaObjekta, Mjere, NaredbaObjekta, NaredbaStabla,
    Novo, Objekt, Projekt, Stavka, Uredjivac, Vrh, VrstaGeometrije,
};

const POLUMJER_HVATA: f32 = 9.0;
const PREDNOST_PRAVE_TOCKE: f32 = 2.0;
const DOSEG_LINIJE: f32 = 6.0;
const DOSEG_TOCKE: f32 = 12.0;

/// Pretvorba između karte i ekrana. U programu je daje `walkers`.
pub trait Pretvorba {
    fn u_ekran(&self, v: &Vrh) -> Pos2;
    fn na_kartu(&self, m: Pos2) -> Vrh;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alat {
    #[default]
    Odaberi,
    Tocka,
    Linija,
    Poligon,
    /// Mjeri bez spremanja, a klik na prvu točku zatvara lik i daje površinu.
    Mjeri,
}

/// Što treba reći korisniku. Tekst daje `prozor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Poruka {
    /// Lik još nema dovoljno točaka da se završi.
    Premalo {
        najmanje: usize,
    },
    /// Točka se ne može obrisati, jer objekt bez nje ne bi bio valjan.
    NeMozeBrisati {
        najmanje: usize,
    },
    Naredba(Greska),
}

/// Ono što `prozor` daje karti.
pub struct Okolina<'a> {
    pub vidljiv: &'a dyn Fn(IdSloja) -> bool,
    /// Naziv sloja koji nastaje kad korisnik crta, a sloja te vrste još nema.
    pub naziv_sloja: &'a dyn Fn(VrstaGeometrije) -> String,
    pub opis_mjere: &'a dyn Fn(&Mjere) -> String,
    /// Duljina jednog dijela mjerenja, u metrima.
    pub opis_duljine: &'a dyn Fn(f64) -> String,
    /// Naziv novog objekta, prema vrsti i rednom broju u sloju, npr. „Točka 3”.
    pub naziv_objekta: &'a dyn Fn(VrstaGeometrije, usize) -> String,
}

/// Što ploča za mjerenje prikazuje dok mjerenje postoji.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mjerenje {
    /// Duljina ili opseg i površina, s metodom.
    pub mjere: Mjere,
    /// Broj dijelova, bez onoga koji još ide do miša.
    pub dijelova: usize,
    pub gotovo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cilj {
    Crtez,
    Objekt(IdSloja, IdObjekta),
}

/// Mjesto na ekranu koje se može uhvatiti mišem. Srednja točka dodaje novi vrh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hvatiste {
    pub cilj: Cilj,
    pub prsten: usize,
    pub i: usize,
    pub srednje: bool,
    pub mjesto: Pos2,
}

/// Linija koja se produljuje. Vrhovi su u crtežu, a tek završetak mijenja objekt, pa je cijelo
/// produljenje jedan korak poništavanja.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Nastavak {
    sloj: IdSloja,
    objekt: IdObjekta,
    /// Crtež uvijek raste na kraju, pa se linija nastavljena od početka okreće i pri spremanju vraća.
    od_pocetka: bool,
    /// Koliko je vrhova imala, da Backspace ne briše ono što je već bilo.
    pocetnih: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Crtez {
    pub alat: Alat,
    pub vrhovi: Vec<Vrh>,
    /// Završeno mjerenje ostaje na karti dok se ne počne novo.
    pub gotov: bool,
    pub zatvoren: bool,
    pub nastavak: Option<Nastavak>,
}

#[derive(Clone, Debug)]
struct Vucenje {
    cilj: Cilj,
    prsten: usize,
    i: usize,
    /// Objekt mijenja kopija, a naredba ide tek kad se tipka pusti, pa je to jedan korak poništavanja.
    pregled: Option<Geometrija>,
}

#[derive(Debug, Default)]
pub struct Crtanje {
    alat: Alat,
    pub(crate) crtez: Option<Crtez>,
    odabran: Option<(IdSloja, IdObjekta)>,
    vuce: Option<Vucenje>,
    pub(crate) hvatista: Vec<Hvatiste>,
    poruka: Option<Poruka>,
    aktivni: Vec<(VrstaGeometrije, IdSloja)>,
    mjeri_povrsinu: bool,
}

fn vrsta_alata(alat: Alat) -> VrstaGeometrije {
    // Točka nikad ne prolazi kroz crtež, jer nastaje jednim klikom.
    if alat == Alat::Poligon {
        VrstaGeometrije::Poligon
    } else {
        VrstaGeometrije::Linija
    }
}

fn najmanje_vrhova(vrsta: VrstaGeometrije) -> usize {
    match vrsta {
        VrstaGeometrije::Tocka => 1,
        VrstaGeometrije::Linija => 2,
        VrstaGeometrije::Poligon => 3,
    }
}

/// GeoPackage i KML prsten zatvaraju ponovljenom prvom točkom. Za uređivanje je ta točka smetnja,
/// jer bi se pomaknula samo jedna od dvije.
fn bez_zatvaranja(prsten: &[Vrh]) -> &[Vrh] {
    match prsten.split_last() {
        Some((zadnji, ostali)) if prsten.first() == Some(zadnji) => ostali,
        _ => prsten,
    }
}

/// Isti objekt, ali s prstenovima bez ponovljene točke.
fn za_uredivanje(g: &Geometrija) -> Geometrija {
    match g {
        Geometrija::Poligon(p) => {
            Geometrija::Poligon(p.iter().map(|r| bez_zatvaranja(r).to_vec()).collect())
        }
        ostalo => ostalo.clone(),
    }
}

/// Nizovi vrhova koje korisnik može hvatati.
pub(crate) fn prstenovi(g: &Geometrija) -> Vec<&[Vrh]> {
    match g {
        Geometrija::Tocka(v) => vec![std::slice::from_ref(v)],
        Geometrija::Linija(v) => vec![v.as_slice()],
        Geometrija::Poligon(p) => p.iter().map(|r| bez_zatvaranja(r)).collect(),
    }
}

fn postavi_vrh(g: &mut Geometrija, prsten: usize, i: usize, novi: Vrh) {
    let vrh = match g {
        Geometrija::Tocka(v) => Some(v),
        Geometrija::Linija(v) => v.get_mut(i),
        Geometrija::Poligon(p) => p.get_mut(prsten).and_then(|r| r.get_mut(i)),
    };
    if let Some(v) = vrh {
        // Visina ostaje, jer pomak po karti ne govori ništa o njoj.
        *v = Vrh {
            visina: v.visina,
            ..novi
        };
    }
}

fn niz_mut(g: &mut Geometrija, prsten: usize) -> Option<&mut Vec<Vrh>> {
    match g {
        Geometrija::Tocka(_) => None,
        Geometrija::Linija(v) => Some(v),
        Geometrija::Poligon(p) => p.get_mut(prsten),
    }
}

fn u_tocke(vrhovi: &[Vrh], p: &dyn Pretvorba) -> Vec<Pos2> {
    vrhovi.iter().map(|v| p.u_ekran(v)).collect()
}

fn do_duzine(m: Pos2, a: Pos2, b: Pos2) -> f32 {
    let d = b - a;
    let duljina = d.length_sq();
    if duljina == 0.0 {
        return m.distance(a);
    }
    let u = ((m - a).dot(d) / duljina).clamp(0.0, 1.0);
    m.distance(a + d * u)
}

/// Pravilo parnosti, kao i ispuna, pa je rupa izvan lika.
fn unutar(prsten: &[Pos2], m: Pos2) -> bool {
    let mut unutra = false;
    for (i, a) in prsten.iter().enumerate() {
        let b = prsten[(i + 1) % prsten.len()];
        if (a.y > m.y) != (b.y > m.y) && m.x < a.x + (m.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            unutra = !unutra;
        }
    }
    unutra
}

fn pogada(g: &Geometrija, m: Pos2, p: &dyn Pretvorba) -> bool {
    match g {
        Geometrija::Tocka(v) => p.u_ekran(v).distance(m) < DOSEG_TOCKE,
        Geometrija::Linija(v) => u_tocke(v, p)
            .windows(2)
            .any(|d| do_duzine(m, d[0], d[1]) < DOSEG_LINIJE),
        Geometrija::Poligon(prs) => {
            let parno = prs.iter().filter(|r| unutar(&u_tocke(r, p), m)).count();
            parno % 2 == 1
        }
    }
}

/// Slojevi od vrha stabla prema dnu. Bez rekurzije, jer dubinu određuje datoteka.
pub(crate) fn slojevi_odozgo(projekt: &Projekt) -> Vec<IdSloja> {
    let mut izlaz = Vec::new();
    let mut stog: Vec<Stavka> = projekt.korijen().iter().rev().copied().collect();
    while let Some(s) = stog.pop() {
        match s {
            Stavka::Sloj(id) => izlaz.push(id),
            Stavka::Mapa(id) => {
                if let Some(m) = projekt.mapa(id) {
                    stog.extend(m.djeca.iter().rev());
                }
            }
        }
    }
    izlaz
}

fn najblizi_objekt(objekti: Vec<&Objekt>, m: Pos2, p: &dyn Pretvorba) -> Option<IdObjekta> {
    objekti
        .into_iter()
        .rev()
        .find(|o| pogada(&o.geometrija, m, p))
        .map(|o| o.id)
}

impl Crtanje {
    pub fn alat(&self) -> Alat {
        self.alat
    }

    pub fn postavi_alat(&mut self, alat: Alat) {
        self.alat = alat;
        self.crtez = None;
        self.vuce = None;
        self.poruka = None;
    }

    pub fn poruka(&self) -> Option<Poruka> {
        self.poruka
    }

    pub fn odabran(&self) -> Option<(IdSloja, IdObjekta)> {
        self.odabran
    }

    /// Odabir iz ploče Slojevi, da karta istakne isti objekt.
    pub fn postavi_odabrani(&mut self, objekt: Option<(IdSloja, IdObjekta)>) {
        self.odabran = objekt;
    }

    /// Crta li se ili mjeri upravo nešto. Tada Backspace i Escape pripadaju karti.
    pub fn u_tijeku(&self) -> bool {
        self.crtez.as_ref().is_some_and(|c| !c.gotov) || self.vuce.is_some()
    }

    /// U koji sloj ide novi objekt te vrste. Ako ga nema, nastaje novi.
    pub fn postavi_aktivni_sloj(&mut self, vrsta: VrstaGeometrije, sloj: IdSloja) {
        self.aktivni.retain(|(v, _)| *v != vrsta);
        self.aktivni.push((vrsta, sloj));
    }

    /// Objekt koji se upravo vuče crta ova strana, a ne priprema za karticu.
    pub(crate) fn skriveni(&self) -> Option<(IdSloja, IdObjekta)> {
        match self.vuce.as_ref()?.cilj {
            Cilj::Objekt(s, o) => Some((s, o)),
            Cilj::Crtez => None,
        }
    }

    pub(crate) fn pregled(&self) -> Option<&Geometrija> {
        self.vuce.as_ref()?.pregled.as_ref()
    }

    pub(crate) fn vuce(&self) -> bool {
        self.vuce.is_some()
    }

    /// Hoće li mjerenje računati površinu, kao izbor Udaljenost ili Površina u ploči.
    pub fn mjeri_povrsinu(&mut self, povrsina: bool) {
        self.mjeri_povrsinu = povrsina;
        if let Some(c) = self.crtez.as_mut().filter(|c| c.alat == Alat::Mjeri) {
            c.zatvoren = povrsina;
        }
    }

    pub fn mjerenje_povrsine(&self) -> bool {
        self.mjeri_povrsinu
    }

    /// Mjerenje za ploču. Ništa se ne sprema dok korisnik to ne zatraži.
    pub fn mjerenje(&self) -> Option<Mjerenje> {
        let c = self.crtez.as_ref().filter(|c| c.alat == Alat::Mjeri)?;
        let n = c.vrhovi.len();
        Some(Mjerenje {
            mjere: self.mjere_crteza(None)?,
            dijelova: if je_lik(c) && n >= 3 {
                n
            } else {
                n.saturating_sub(1)
            },
            gotovo: c.gotov,
        })
    }

    pub fn ocisti_mjerenje(&mut self) {
        if self.crtez.as_ref().is_some_and(|c| c.alat == Alat::Mjeri) {
            self.crtez = None;
        }
    }

    /// Gumb „Spremi kao liniju”, a zatvoreno mjerenje postaje poligon.
    pub fn spremi_mjerenje(&mut self, u: &mut Uredjivac, ok: &Okolina<'_>) {
        let Some(c) = self.crtez.as_ref().filter(|c| c.alat == Alat::Mjeri) else {
            return;
        };
        let (vrsta, najmanje) = if je_lik(c) {
            (VrstaGeometrije::Poligon, 3)
        } else {
            (VrstaGeometrije::Linija, 2)
        };
        if c.vrhovi.len() < najmanje {
            self.poruka = Some(Poruka::Premalo { najmanje });
            return;
        }
        let g = if vrsta == VrstaGeometrije::Poligon {
            Geometrija::Poligon(vec![c.vrhovi.clone()])
        } else {
            Geometrija::Linija(c.vrhovi.clone())
        };
        self.spremi(vrsta, g, u, ok);
    }

    /// Duljina svakog dijela mjerenja i sredina tog dijela, za natpis na karti. Dok mjerenje traje,
    /// zadnji dio ide do miša.
    pub(crate) fn dijelovi_mjerenja(
        &self,
        mis: Option<Vrh>,
        p: &dyn Pretvorba,
    ) -> Vec<(Pos2, f64)> {
        let Some(c) = self.crtez.as_ref().filter(|c| c.alat == Alat::Mjeri) else {
            return Vec::new();
        };
        let mut v = c.vrhovi.clone();
        v.extend(mis.filter(|_| !c.gotov));
        if je_lik(c) && v.len() >= 3 {
            v.extend(v.first().copied());
        }
        v.windows(2).filter_map(|d| dio(d[0], d[1], p)).collect()
    }

    /// Mjesta za hvatanje, za ono što se trenutno vidi na ekranu.
    pub(crate) fn osvjezi_hvatista(&mut self, projekt: &Projekt, p: &dyn Pretvorba) {
        let mut h = Vec::new();
        if let Some(c) = self.crtez.as_ref() {
            dodaj_hvatista(&mut h, Cilj::Crtez, &[&c.vrhovi], false, false, p);
        } else if let Some((s, o)) = self.odabran.filter(|_| self.crtez.is_none()) {
            let sloj = projekt.sloj(s).filter(|s| s.stanje.smije_mijenjati());
            let objekt = sloj.and_then(|x| x.objekti.get(&o));
            if let Some(g) = self.pregled().or(objekt.map(|x| &x.geometrija)) {
                let zatvoren = matches!(g, Geometrija::Poligon(_));
                let srednje = !matches!(g, Geometrija::Tocka(_));
                dodaj_hvatista(
                    &mut h,
                    Cilj::Objekt(s, o),
                    &prstenovi(g),
                    srednje,
                    zatvoren,
                    p,
                );
            }
        }
        self.hvatista = h;
    }

    /// Prava točka ima prednost pred srednjom kad su jako blizu.
    pub(crate) fn najblize(&self, m: Pos2) -> Option<Hvatiste> {
        let udaljenost = |h: &Hvatiste| {
            h.mjesto.distance(m) + if h.srednje { PREDNOST_PRAVE_TOCKE } else { 0.0 }
        };
        self.hvatista
            .iter()
            .filter(|h| udaljenost(h) <= POLUMJER_HVATA)
            .min_by(|a, b| udaljenost(a).total_cmp(&udaljenost(b)))
            .copied()
    }

    /// Točka se gleda tamo gdje je tipka pritisnuta. Inače bi brzi potez promašio točku i umjesto
    /// nje pomaknuo kartu.
    pub(crate) fn smije_pomicati_kartu(&self, pritisak: Option<Pos2>) -> bool {
        !self.vuce() && pritisak.and_then(|m| self.najblize(m)).is_none()
    }

    pub(crate) fn klik(&mut self, m: Pos2, u: &mut Uredjivac, ok: &Okolina<'_>, p: &dyn Pretvorba) {
        self.poruka = None;
        match self.alat {
            Alat::Odaberi => {
                if !self.pocni_nastavak(m, u.projekt()) {
                    self.odaberi(m, u.projekt(), ok, p);
                }
            }
            Alat::Tocka => {
                let g = Geometrija::Tocka(p.na_kartu(m));
                self.spremi(VrstaGeometrije::Tocka, g, u, ok);
            }
            Alat::Linija | Alat::Poligon | Alat::Mjeri => self.klik_crteza(m, u, ok, p),
        }
    }

    /// Klik na prvu ili zadnju točku odabrane linije nastavlja crtanje od te točke.
    fn pocni_nastavak(&mut self, m: Pos2, projekt: &Projekt) -> bool {
        let Some(h) = self.najblize(m).filter(|h| !h.srednje) else {
            return false;
        };
        let Cilj::Objekt(sloj, objekt) = h.cilj else {
            return false;
        };
        let Some(Geometrija::Linija(v)) = projekt.objekt(sloj, objekt).map(|o| &o.geometrija)
        else {
            return false;
        };
        let od_pocetka = h.i == 0;
        if !od_pocetka && h.i + 1 != v.len() {
            return false;
        }
        let mut vrhovi = v.clone();
        if od_pocetka {
            vrhovi.reverse();
        }
        let pocetnih = vrhovi.len();
        self.alat = Alat::Linija;
        self.crtez = Some(Crtez {
            alat: Alat::Linija,
            vrhovi,
            gotov: false,
            zatvoren: false,
            nastavak: Some(Nastavak {
                sloj,
                objekt,
                od_pocetka,
                pocetnih,
            }),
        });
        true
    }

    fn zavrsi_nastavak(&mut self, n: Nastavak, u: &mut Uredjivac) {
        let Some(c) = self.crtez.take() else {
            return;
        };
        let mut vrhovi = c.vrhovi;
        if n.od_pocetka {
            vrhovi.reverse();
        }
        self.alat = Alat::Odaberi;
        self.odabran = Some((n.sloj, n.objekt));
        let nova = Geometrija::Linija(vrhovi);
        // Dvoklik bez ijedne nove točke ne smije ostaviti prazan korak u povijesti.
        let ista = u
            .projekt()
            .objekt(n.sloj, n.objekt)
            .is_some_and(|o| o.geometrija == nova);
        if !ista {
            self.posalji_geometriju(n.sloj, n.objekt, nova, u);
        }
    }

    fn klik_crteza(&mut self, m: Pos2, u: &mut Uredjivac, ok: &Okolina<'_>, p: &dyn Pretvorba) {
        let alat = self.alat;
        if self.crtez.as_ref().is_some_and(|c| c.gotov) {
            self.crtez = None;
        }
        if let Some(h) = self.najblize(m).filter(|h| h.cilj == Cilj::Crtez) {
            // Klik na prvu točku lika zatvara ga, a ostale točke ne dodaju ništa.
            if h.i == 0 && alat != Alat::Linija {
                self.zatvori(u, ok);
            }
            return;
        }
        let zatvoren = alat == Alat::Mjeri && self.mjeri_povrsinu;
        let c = self.crtez.get_or_insert_with(|| Crtez {
            alat,
            vrhovi: Vec::new(),
            gotov: false,
            zatvoren,
            nastavak: None,
        });
        c.vrhovi.push(p.na_kartu(m));
    }

    /// Mjerenje s klikom na prvu točku postaje lik, a crtanje poligona ionako jest lik.
    fn zatvori(&mut self, u: &mut Uredjivac, ok: &Okolina<'_>) {
        let dovoljno = self.crtez.as_ref().is_some_and(|c| c.vrhovi.len() >= 3);
        if self.alat == Alat::Mjeri {
            if let Some(c) = self.crtez.as_mut().filter(|_| dovoljno) {
                c.zatvoren = true;
                c.gotov = true;
            }
            return;
        }
        self.zavrsi(u, ok);
    }

    /// Dvoklik. Prvi klik para već je dodao točku.
    pub(crate) fn zavrsi(&mut self, u: &mut Uredjivac, ok: &Okolina<'_>) {
        let Some(c) = self.crtez.as_ref() else {
            return;
        };
        let vrsta = vrsta_alata(c.alat);
        let najmanje = najmanje_vrhova(vrsta);
        if c.vrhovi.len() < najmanje {
            self.poruka = Some(Poruka::Premalo { najmanje });
            return;
        }
        if let Some(n) = c.nastavak {
            self.zavrsi_nastavak(n, u);
            return;
        }
        if c.alat == Alat::Mjeri {
            if let Some(c) = self.crtez.as_mut() {
                c.gotov = true;
            }
            return;
        }
        let g = match vrsta {
            VrstaGeometrije::Poligon => Geometrija::Poligon(vec![c.vrhovi.clone()]),
            _ => Geometrija::Linija(c.vrhovi.clone()),
        };
        self.spremi(vrsta, g, u, ok);
    }

    /// Nacrtaj odmah: ako prikladnog sloja nema, nastaje u istom koraku poništavanja.
    fn spremi(
        &mut self,
        vrsta: VrstaGeometrije,
        geometrija: Geometrija,
        u: &mut Uredjivac,
        ok: &Okolina<'_>,
    ) {
        let cilj = self.prikladan_sloj(vrsta, u.projekt());
        let naziv = (ok.naziv_sloja)(vrsta);
        let postojecih = cilj
            .and_then(|s| u.projekt().sloj(s))
            .map_or(0, |s| s.objekti.len());
        let naziv_objekta = (ok.naziv_objekta)(vrsta, postojecih + 1);
        let ishod = u.skupina(|g| {
            let sloj = match cilj {
                Some(s) => s,
                None => novi_sloj(g, naziv, vrsta)?,
            };
            let naredba = NaredbaObjekta::Dodaj {
                sloj,
                naziv: naziv_objekta,
                geometrija,
            };
            Ok((sloj, g.izvrsi(naredba)?))
        });
        match ishod {
            Ok((sloj, Some(Novo::Objekt(id)))) => {
                self.postavi_aktivni_sloj(vrsta, sloj);
                self.odabran = Some((sloj, id));
                self.crtez = None;
                self.alat = Alat::Odaberi;
            }
            Ok(_) => {}
            Err(g) => self.poruka = Some(Poruka::Naredba(g)),
        }
    }

    fn prikladan_sloj(&self, vrsta: VrstaGeometrije, projekt: &Projekt) -> Option<IdSloja> {
        let aktivan = self
            .aktivni
            .iter()
            .find(|(v, _)| *v == vrsta)
            .map(|(_, s)| *s);
        let dobar = |s: &IdSloja| {
            projekt
                .sloj(*s)
                .is_some_and(|x| x.vrsta == vrsta && x.stanje.smije_mijenjati())
        };
        aktivan
            .filter(dobar)
            .or_else(|| slojevi_odozgo(projekt).into_iter().find(dobar))
    }

    fn odaberi(&mut self, m: Pos2, projekt: &Projekt, ok: &Okolina<'_>, p: &dyn Pretvorba) {
        self.odabran = slojevi_odozgo(projekt)
            .into_iter()
            .filter(|s| (ok.vidljiv)(*s))
            .find_map(|s| {
                let sloj = projekt.sloj(s)?;
                let id = najblizi_objekt(sloj.objekti.values().collect(), m, p)?;
                Some((s, id))
            });
    }

    pub(crate) fn desni_klik(&mut self, m: Pos2, u: &mut Uredjivac) {
        self.poruka = None;
        let Some(h) = self.najblize(m).filter(|h| !h.srednje) else {
            return;
        };
        match h.cilj {
            Cilj::Crtez => self.obrisi_iz_crteza(h.i),
            Cilj::Objekt(s, o) => self.obrisi_iz_objekta(s, o, h, u),
        }
    }

    fn obrisi_iz_crteza(&mut self, i: usize) {
        if let Some(c) = self.crtez.as_mut().filter(|c| i < c.vrhovi.len()) {
            c.vrhovi.remove(i);
            if let Some(n) = c.nastavak.as_mut().filter(|n| i < n.pocetnih) {
                n.pocetnih -= 1;
            }
        }
        self.makni_prazan_crtez();
    }

    fn makni_prazan_crtez(&mut self) {
        if self.crtez.as_ref().is_some_and(|c| c.vrhovi.is_empty()) {
            self.crtez = None;
        }
    }

    fn obrisi_iz_objekta(&mut self, s: IdSloja, o: IdObjekta, h: Hvatiste, u: &mut Uredjivac) {
        let Some(g) = u
            .projekt()
            .objekt(s, o)
            .map(|x| za_uredivanje(&x.geometrija))
        else {
            return;
        };
        let najmanje = najmanje_vrhova(g.vrsta());
        let mut nova = g;
        match niz_mut(&mut nova, h.prsten).filter(|n| n.len() > najmanje && h.i < n.len()) {
            Some(n) => {
                n.remove(h.i);
                self.posalji_geometriju(s, o, nova, u);
            }
            None => self.poruka = Some(Poruka::NeMozeBrisati { najmanje }),
        }
    }

    fn posalji_geometriju(&mut self, s: IdSloja, o: IdObjekta, g: Geometrija, u: &mut Uredjivac) {
        let naredba = NaredbaObjekta::Izmijeni {
            sloj: s,
            objekt: o,
            izmjena: IzmjenaObjekta::Geometrija(g),
        };
        if let Err(g) = u.izvrsi(naredba) {
            self.poruka = Some(Poruka::Naredba(g));
        }
    }

    /// Počinje vući točku koja je bila pod mišem kad je tipka pritisnuta.
    pub(crate) fn pocni_vuci(&mut self, pritisak: Pos2, m: Vrh, projekt: &Projekt) {
        let Some(h) = self.najblize(pritisak) else {
            return;
        };
        let pregled = match h.cilj {
            Cilj::Crtez => None,
            Cilj::Objekt(s, o) => projekt.objekt(s, o).map(|x| za_uredivanje(&x.geometrija)),
        };
        let mut v = Vucenje {
            cilj: h.cilj,
            prsten: h.prsten,
            i: h.i,
            pregled,
        };
        if h.srednje {
            v.i += 1;
            if let Some(n) = v.pregled.as_mut().and_then(|g| niz_mut(g, h.prsten)) {
                n.insert(v.i, m);
            }
        }
        self.vuce = Some(v);
    }

    pub(crate) fn vuci(&mut self, m: Vrh) {
        let Some(v) = self.vuce.as_mut() else {
            return;
        };
        if let Some(g) = v.pregled.as_mut() {
            postavi_vrh(g, v.prsten, v.i, m);
        } else if let Some(t) = self.crtez.as_mut().and_then(|c| c.vrhovi.get_mut(v.i)) {
            *t = Vrh {
                visina: t.visina,
                ..m
            };
        }
    }

    pub(crate) fn pusti(&mut self, u: &mut Uredjivac) {
        let Some(v) = self.vuce.take() else {
            return;
        };
        if let (Cilj::Objekt(s, o), Some(g)) = (v.cilj, v.pregled) {
            self.posalji_geometriju(s, o, g, u);
        }
    }

    /// Esc odustaje od onoga što je u tijeku.
    pub(crate) fn odustani(&mut self) {
        if self.vuce.take().is_none() {
            // Odustajanje od produljenja vraća na odabir, inače bi sljedeći klik počeo novu liniju.
            if self.crtez.take().is_some_and(|c| c.nastavak.is_some()) {
                self.alat = Alat::Odaberi;
            }
        }
        self.poruka = None;
    }

    /// Backspace miče zadnju točku crteža.
    pub(crate) fn natrag(&mut self) {
        if let Some(c) = self.crtez.as_mut().filter(|c| !c.gotov) {
            let najmanje = c.nastavak.map_or(0, |n| n.pocetnih);
            if c.vrhovi.len() > najmanje {
                c.vrhovi.pop();
            }
        }
        self.makni_prazan_crtez();
    }

    /// Mjere crteža, s točkom pod mišem dok crtanje traje.
    pub(crate) fn mjere_crteza(&self, mis: Option<Vrh>) -> Option<Mjere> {
        let c = self.crtez.as_ref()?;
        let mut v = c.vrhovi.clone();
        v.extend(mis.filter(|_| !c.gotov));
        let g = if je_lik(c) && v.len() >= 3 {
            Geometrija::Poligon(vec![v])
        } else {
            Geometrija::Linija(v)
        };
        g.mjere().ok()
    }
}

pub(crate) fn je_lik(c: &Crtez) -> bool {
    c.zatvoren || c.alat == Alat::Poligon
}

fn dio(a: Vrh, b: Vrh, p: &dyn Pretvorba) -> Option<(Pos2, f64)> {
    let duljina = mjeri_crtu(&[[a.duzina, a.sirina], [b.duzina, b.sirina]]).ok()?;
    let (ma, mb) = (p.u_ekran(&a), p.u_ekran(&b));
    Some((
        pos2((ma.x + mb.x) / 2.0, (ma.y + mb.y) / 2.0),
        duljina.duljina_m,
    ))
}

fn novi_sloj(
    g: &mut slojevi::Skupina<'_>,
    naziv: String,
    vrsta: VrstaGeometrije,
) -> Result<IdSloja, Greska> {
    let naredba = NaredbaStabla::DodajSloj {
        u: None,
        naziv,
        vrsta,
    };
    match g.izvrsi(naredba)? {
        Some(Novo::Sloj(s)) => Ok(s),
        _ => Err(Greska::NepoznatSloj),
    }
}

fn dodaj_hvatista(
    h: &mut Vec<Hvatiste>,
    cilj: Cilj,
    nizovi: &[&[Vrh]],
    srednje: bool,
    zatvoren: bool,
    p: &dyn Pretvorba,
) {
    for (prsten, niz) in nizovi.iter().enumerate() {
        let t = u_tocke(niz, p);
        h.extend(t.iter().enumerate().map(|(i, mjesto)| Hvatiste {
            cilj,
            prsten,
            i,
            srednje: false,
            mjesto: *mjesto,
        }));
        if srednje {
            dodaj_srednje(h, cilj, prsten, &t, zatvoren);
        }
    }
}

fn dodaj_srednje(h: &mut Vec<Hvatiste>, cilj: Cilj, prsten: usize, t: &[Pos2], zatvoren: bool) {
    let n = t.len();
    let koliko = if zatvoren { n } else { n.saturating_sub(1) };
    for i in 0..koliko {
        let (a, b) = (t[i], t[(i + 1) % n]);
        h.push(Hvatiste {
            cilj,
            prsten,
            i,
            srednje: true,
            mjesto: pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0),
        });
    }
}

#[cfg(test)]
#[path = "crtanje_testovi.rs"]
mod testovi;
