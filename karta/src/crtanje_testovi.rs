use super::*;
use slojevi::{Geometrija, Mjere, StanjeSloja, Uredjivac};

/// 1000 piksela po stupnju, od ishodišta 18° E, 45° N, a y raste prema jugu.
struct Ravno;

impl Pretvorba for Ravno {
    fn u_ekran(&self, v: &Vrh) -> Pos2 {
        pos2(
            ((v.duzina - 18.0) * 1000.0) as f32,
            ((45.0 - v.sirina) * 1000.0) as f32,
        )
    }

    fn na_kartu(&self, m: Pos2) -> Vrh {
        Vrh::novi(
            18.0 + f64::from(m.x) / 1000.0,
            45.0 - f64::from(m.y) / 1000.0,
        )
    }
}

fn vidljiv(_: IdSloja) -> bool {
    true
}

fn naziv(v: VrstaGeometrije) -> String {
    format!("Novi {v:?}")
}

fn opis(_: &Mjere) -> String {
    String::new()
}

fn naziv_objekta(_: VrstaGeometrije, redni: usize) -> String {
    format!("Objekt {redni}")
}
fn duljina(_: f64) -> String {
    String::new()
}

const OK: Okolina<'static> = Okolina {
    vidljiv: &vidljiv,
    naziv_sloja: &naziv,
    opis_mjere: &opis,
    opis_duljine: &duljina,
    naziv_objekta: &naziv_objekta,
};

struct Stol {
    c: Crtanje,
    u: Uredjivac,
}

impl Stol {
    fn novi(alat: Alat) -> Self {
        let mut c = Crtanje::default();
        c.postavi_alat(alat);
        Self {
            c,
            u: Uredjivac::novi(),
        }
    }

    fn klik(&mut self, x: f32, y: f32) {
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
        self.c.klik(pos2(x, y), &mut self.u, &OK, &Ravno);
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
    }

    fn desni(&mut self, x: f32, y: f32) {
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
        self.c.desni_klik(pos2(x, y), &mut self.u);
    }

    fn vuci(&mut self, od: Pos2, do_: Pos2) {
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
        self.c.pocni_vuci(od, Ravno.na_kartu(do_), self.u.projekt());
        self.c.vuci(Ravno.na_kartu(do_));
        self.c.pusti(&mut self.u);
    }

    fn zavrsi(&mut self) {
        self.c.zavrsi(&mut self.u, &OK);
    }

    /// Crta poligon i vraća ga odabranog.
    fn poligon(&mut self) -> (IdSloja, IdObjekta) {
        self.c.postavi_alat(Alat::Poligon);
        for (x, y) in [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)] {
            self.klik(x, y);
        }
        self.zavrsi();
        self.c.odabran().expect("odabran")
    }

    fn geometrija(&self, cilj: (IdSloja, IdObjekta)) -> Geometrija {
        let o = self.u.projekt().objekt(cilj.0, cilj.1).expect("objekt");
        o.geometrija.clone()
    }
}

fn broj_vrhova(g: &Geometrija) -> usize {
    prstenovi(g).iter().map(|p| p.len()).sum()
}

#[test]
fn tocka_nastaje_jednim_klikom_a_sloj_sam() {
    let mut s = Stol::novi(Alat::Tocka);
    s.klik(50.0, 20.0);
    let (sloj, o) = s.c.odabran().expect("odabran");
    let sl = s.u.projekt().sloj(sloj).expect("sloj");
    assert_eq!(sl.naziv, "Novi Tocka");
    assert_eq!(sl.vrsta, VrstaGeometrije::Tocka);
    match sl.objekti[&o].geometrija {
        Geometrija::Tocka(v) => {
            assert!((v.duzina - 18.05).abs() < 1e-6 && (v.sirina - 44.98).abs() < 1e-6);
        }
        ref g => panic!("{g:?}"),
    }
    assert_eq!(s.c.alat(), Alat::Odaberi, "poslije crtanja je odabir");
    assert_eq!(s.u.ponisti(), Ok(true));
    assert!(
        s.u.projekt().korijen().is_empty(),
        "sloj i objekt su jedan korak"
    );
}

#[test]
fn drugi_objekt_ide_u_isti_sloj() {
    let mut s = Stol::novi(Alat::Tocka);
    s.klik(0.0, 0.0);
    let (prvi, _) = s.c.odabran().expect("odabran");
    s.c.postavi_alat(Alat::Tocka);
    s.klik(50.0, 0.0);
    assert_eq!(s.c.odabran().expect("odabran").0, prvi);
    assert_eq!(s.u.projekt().sloj(prvi).expect("sloj").objekti.len(), 2);
}

#[test]
fn zakljucan_sloj_se_preskace_a_aktivni_se_postuje() {
    let mut s = Stol::novi(Alat::Tocka);
    s.klik(0.0, 0.0);
    let (prvi, _) = s.c.odabran().expect("odabran");
    s.u.postavi_stanje_sloja(prvi, StanjeSloja::SamoCitanje)
        .expect("stanje");
    s.c.postavi_alat(Alat::Tocka);
    s.klik(50.0, 0.0);
    let (drugi, _) = s.c.odabran().expect("odabran");
    assert_ne!(drugi, prvi, "u zaključan sloj se ne crta");
    s.c.postavi_aktivni_sloj(VrstaGeometrije::Tocka, drugi);
    s.c.postavi_alat(Alat::Tocka);
    s.klik(70.0, 0.0);
    assert_eq!(s.c.odabran().expect("odabran").0, drugi);
}

#[test]
fn linija_treba_dvije_tocke_a_dvoklik_je_zavrsava() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.zavrsi();
    assert_eq!(s.c.poruka(), Some(Poruka::Premalo { najmanje: 2 }));
    assert!(s.u.projekt().korijen().is_empty());
    s.klik(100.0, 0.0);
    assert_eq!(s.c.poruka(), None, "svaki klik briše staru poruku");
    s.zavrsi();
    let cilj = s.c.odabran().expect("odabran");
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 2);
    assert!(s.c.crtez.is_none());
}

#[test]
fn zavrsi_bez_crteza_ne_radi_nista() {
    let mut s = Stol::novi(Alat::Linija);
    s.zavrsi();
    assert!(s.c.poruka().is_none());
    assert!(s.u.projekt().korijen().is_empty());
}

#[test]
fn klik_na_prvu_tocku_zatvara_poligon_a_ostale_ne_dodaju_nista() {
    let mut s = Stol::novi(Alat::Poligon);
    for (x, y) in [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)] {
        s.klik(x, y);
    }
    s.klik(100.0, 0.0);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 3);
    s.klik(0.0, 0.0);
    let cilj = s.c.odabran().expect("odabran");
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 3);
}

#[test]
fn klik_na_prvu_tocku_linije_ne_zatvara_je() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.klik(0.0, 0.0);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 2);
    assert!(s.c.odabran().is_none());
}

#[test]
fn desni_klik_brise_tocku_crteza_a_zadnja_brise_crtez() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.desni(100.0, 0.0);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 1);
    s.desni(500.0, 500.0);
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        1,
        "promašaj"
    );
    s.desni(0.0, 0.0);
    assert!(s.c.crtez.is_none());
}

#[test]
fn backspace_i_esc() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.c.natrag();
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 1);
    s.c.natrag();
    assert!(s.c.crtez.is_none());
    s.klik(0.0, 0.0);
    s.c.odustani();
    assert!(s.c.crtez.is_none());
}

#[test]
fn povlacenje_tocke_crteza_je_mice_a_visinu_cuva() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.c.crtez.as_mut().expect("crtež").vrhovi[1].visina = Some(120.0);
    s.vuci(pos2(100.0, 0.0), pos2(100.0, 50.0));
    let v = s.c.crtez.as_ref().expect("crtež").vrhovi[1];
    assert!((v.sirina - 44.95).abs() < 1e-6);
    assert_eq!(v.visina, Some(120.0));
}

#[test]
fn povlacenje_tocke_objekta_je_jedan_korak_poništavanja() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    let prije = s.geometrija(cilj);
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    s.c.pocni_vuci(
        pos2(100.0, 100.0),
        Ravno.na_kartu(pos2(110.0, 110.0)),
        s.u.projekt(),
    );
    assert_eq!(
        s.c.skriveni(),
        Some(cilj),
        "dok se vuče, crta ga ova strana"
    );
    s.c.vuci(Ravno.na_kartu(pos2(150.0, 150.0)));
    s.c.vuci(Ravno.na_kartu(pos2(200.0, 200.0)));
    assert_eq!(s.geometrija(cilj), prije, "naredba ide tek kad se pusti");
    s.c.pusti(&mut s.u);
    assert_ne!(s.geometrija(cilj), prije);
    assert!(s.c.skriveni().is_none());
    assert_eq!(s.u.ponisti(), Ok(true));
    assert_eq!(s.geometrija(cilj), prije);
}

#[test]
fn srednja_tocka_dodaje_novi_vrh_na_pravo_mjesto() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    s.vuci(pos2(50.0, 0.0), pos2(50.0, -30.0));
    let Geometrija::Poligon(p) = s.geometrija(cilj) else {
        panic!("poligon");
    };
    assert_eq!(p[0].len(), 5);
    assert!((p[0][1].sirina - 45.03).abs() < 1e-6, "{:?}", p[0][1]);
}

#[test]
fn zatvorenici_iz_datoteke_su_otvoreni_pri_uredjivanju() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    let Geometrija::Poligon(mut p) = s.geometrija(cilj) else {
        panic!("poligon");
    };
    let prva = p[0][0];
    p[0].push(prva);
    let n = NaredbaObjekta::Izmijeni {
        sloj: cilj.0,
        objekt: cilj.1,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Poligon(p)),
    };
    s.u.izvrsi(n).expect("zatvoren");
    s.vuci(pos2(0.0, 0.0), pos2(-20.0, -20.0));
    let Geometrija::Poligon(p) = s.geometrija(cilj) else {
        panic!("poligon");
    };
    assert_eq!(p[0].len(), 4, "ponovljena točka je maknuta");
    assert!((p[0][0].duzina - 17.98).abs() < 1e-6);
}

#[test]
fn desni_klik_brise_vrh_objekta_ali_ne_ispod_najmanjeg_broja() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    s.desni(100.0, 100.0);
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 3);
    s.desni(100.0, 0.0);
    assert_eq!(s.c.poruka(), Some(Poruka::NeMozeBrisati { najmanje: 3 }));
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 3);
    assert_eq!(s.u.ponisti(), Ok(true));
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 4);
}

#[test]
fn desni_klik_na_srednju_tocku_ne_brise_nista() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    s.desni(50.0, 0.0);
    assert_eq!(broj_vrhova(&s.geometrija(cilj)), 4);
}

#[test]
fn tocka_se_vuce_ali_desni_klik_je_ne_brise() {
    let mut s = Stol::novi(Alat::Tocka);
    s.klik(0.0, 0.0);
    let cilj = s.c.odabran().expect("odabran");
    s.vuci(pos2(0.0, 0.0), pos2(30.0, 0.0));
    let Geometrija::Tocka(v) = s.geometrija(cilj) else {
        panic!("točka");
    };
    assert!((v.duzina - 18.03).abs() < 1e-6);
    s.desni(30.0, 0.0);
    assert_eq!(s.c.poruka(), Some(Poruka::NeMozeBrisati { najmanje: 1 }));
}

#[test]
fn zakljucan_objekt_se_odabire_ali_nema_hvatista() {
    let mut s = Stol::novi(Alat::Poligon);
    let cilj = s.poligon();
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    assert_eq!(s.c.hvatista.len(), 8, "četiri vrha i četiri srednje točke");
    s.u.postavi_stanje_sloja(cilj.0, StanjeSloja::USukobu)
        .expect("stanje");
    s.klik(500.0, 500.0);
    assert!(s.c.odabran().is_none());
    s.klik(50.0, 50.0);
    assert_eq!(s.c.odabran(), Some(cilj));
    assert!(s.c.hvatista.is_empty());
}

#[test]
fn odabir_gleda_gornji_sloj_rupu_i_vidljivost() {
    let mut s = Stol::novi(Alat::Poligon);
    let donji = s.poligon();
    // Novi sloj ide na kraj korijena, pa je ispod, a isti sloj bira zadnji nacrtani objekt.
    let gornji = s.poligon();
    assert_eq!(donji.0, gornji.0);
    s.klik(50.0, 50.0);
    assert_eq!(s.c.odabran(), Some(gornji));
    let rupa: Vec<Vrh> = [(25.0, 25.0), (75.0, 25.0), (75.0, 75.0), (25.0, 75.0)]
        .iter()
        .map(|(x, y)| Ravno.na_kartu(pos2(*x, *y)))
        .collect();
    let Geometrija::Poligon(mut p) = s.geometrija(gornji) else {
        panic!("poligon");
    };
    p.push(rupa);
    s.u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: gornji.0,
        objekt: gornji.1,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Poligon(p)),
    })
    .expect("rupa");
    s.klik(50.0, 50.0);
    assert_eq!(s.c.odabran(), Some(donji), "kroz rupu se vidi donji");
    let nista = |_: IdSloja| false;
    let skriveno = Okolina {
        vidljiv: &nista,
        ..OK
    };
    s.c.klik(pos2(50.0, 50.0), &mut s.u, &skriveno, &Ravno);
    assert!(s.c.odabran().is_none(), "skriveni sloj se ne može odabrati");
}

#[test]
fn linija_i_tocka_imaju_doseg_u_pikselima() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.zavrsi();
    let cilj = s.c.odabran().expect("odabran");
    s.klik(50.0, 5.0);
    assert_eq!(s.c.odabran(), Some(cilj));
    s.klik(50.0, 7.0);
    assert!(s.c.odabran().is_none());
    assert_eq!(
        do_duzine(pos2(3.0, 4.0), pos2(0.0, 0.0), pos2(0.0, 0.0)),
        5.0
    );
}

#[test]
fn mjerenje_se_ne_sprema_a_klik_na_prvu_tocku_daje_povrsinu() {
    let mut s = Stol::novi(Alat::Mjeri);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    assert!(matches!(s.c.mjere_crteza(None), Some(Mjere::Linija(_))));
    s.klik(100.0, 100.0);
    assert!(matches!(s.c.mjere_crteza(None), Some(Mjere::Linija(_))));
    s.klik(0.0, 0.0);
    let c = s.c.crtez.as_ref().expect("crtež");
    assert!(c.gotov && c.zatvoren);
    assert!(matches!(s.c.mjere_crteza(None), Some(Mjere::Poligon(_))));
    assert!(s.u.projekt().korijen().is_empty(), "ništa nije spremljeno");
    s.klik(300.0, 300.0);
    let novo = s.c.crtez.as_ref().expect("novi crtež");
    assert_eq!(novo.vrhovi.len(), 1, "novi klik počinje novo mjerenje");
}

#[test]
fn mjerenje_osmice_javlja_mjesto_presjeka() {
    let mut s = Stol::novi(Alat::Mjeri);
    for (x, y) in [(0.0, 0.0), (100.0, 100.0), (100.0, 0.0), (0.0, 100.0)] {
        s.klik(x, y);
    }
    assert_eq!(
        s.c.mjerenje().expect("mjerenje").presjek,
        None,
        "još nije lik"
    );
    s.klik(0.0, 0.0);
    let m = s.c.mjerenje().expect("mjerenje");
    assert!(m.gotovo);
    let t = m.presjek.expect("osmica siječe samu sebe");
    assert!(
        (t.duzina - 18.05).abs() < 1e-9 && (t.sirina - 44.95).abs() < 1e-9,
        "{t:?}"
    );
    // Isti lik dok se crta: presjek nastaje tek s točkom pod mišem.
    let mut s = Stol::novi(Alat::Poligon);
    for (x, y) in [(0.0, 0.0), (100.0, 100.0), (100.0, 0.0)] {
        s.klik(x, y);
    }
    assert_eq!(s.c.presjek_crteza(None), None);
    let t =
        s.c.presjek_crteza(Some(Ravno.na_kartu(pos2(0.0, 100.0))))
            .expect("presjek");
    assert!((t.duzina - 18.05).abs() < 1e-9, "{t:?}");
}

#[test]
fn presjek_za_upozorenje_gleda_crtez_pa_odabrani_lik() {
    let mut s = Stol::novi(Alat::Poligon);
    assert_eq!(
        s.c.presjek(s.u.projekt()),
        None,
        "nema ni crteža ni odabira"
    );
    for (x, y) in [(0.0, 0.0), (100.0, 100.0), (100.0, 0.0), (0.0, 100.0)] {
        s.klik(x, y);
    }
    assert!(s.c.presjek(s.u.projekt()).is_some(), "crtež koji traje");
    s.zavrsi();
    assert!(s.c.odabran().is_some() && s.c.crtez.is_none());
    let t = s.c.presjek(s.u.projekt()).expect("odabrani lik");
    assert!(
        (t.duzina - 18.05).abs() < 1e-9 && (t.sirina - 44.95).abs() < 1e-9,
        "{t:?}"
    );
    let ispravan = s.poligon();
    assert_eq!(s.c.odabran(), Some(ispravan));
    assert_eq!(s.c.presjek(s.u.projekt()), None);
    // Novi crtež bez presjeka ima prednost pred odabranom osmicom.
    s.c.postavi_odabrani(None);
    s.c.postavi_alat(Alat::Poligon);
    s.klik(0.0, 0.0);
    assert_eq!(s.c.presjek(s.u.projekt()), None);
}

#[test]
fn presjek_imaju_samo_likovi() {
    let kvadrat = Geometrija::Poligon(vec![vec![
        Vrh::novi(0.0, 0.0),
        Vrh::novi(1.0, 0.0),
        Vrh::novi(1.0, 1.0),
        Vrh::novi(0.0, 1.0),
    ]]);
    assert_eq!(presjek_geometrije(&kvadrat), None);
    let osmica = Geometrija::Poligon(vec![vec![
        Vrh::novi(0.0, 0.0),
        Vrh::novi(1.0, 1.0),
        Vrh::novi(1.0, 0.0),
        Vrh::novi(0.0, 1.0),
    ]]);
    assert_eq!(presjek_geometrije(&osmica), Some(Vrh::novi(0.5, 0.5)));
    let crta = Geometrija::Linija(vec![
        Vrh::novi(0.0, 0.0),
        Vrh::novi(1.0, 1.0),
        Vrh::novi(1.0, 0.0),
        Vrh::novi(0.0, 1.0),
    ]);
    assert_eq!(
        presjek_geometrije(&crta),
        None,
        "linija koja se križa nije lik"
    );
    assert_eq!(
        presjek_geometrije(&Geometrija::Tocka(Vrh::novi(0.0, 0.0))),
        None
    );
}

#[test]
fn zatvaranje_mjerenja_s_premalo_tocaka_ne_radi_nista() {
    let mut s = Stol::novi(Alat::Mjeri);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.klik(0.0, 0.0);
    assert!(!s.c.crtez.as_ref().expect("crtež").gotov);
}

#[test]
fn dvoklik_zavrsava_mjerenje_crte_i_ne_sprema() {
    let mut s = Stol::novi(Alat::Mjeri);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.zavrsi();
    assert!(s.c.crtez.as_ref().expect("crtež").gotov);
    assert_eq!(s.c.alat(), Alat::Mjeri);
    assert!(s.u.projekt().korijen().is_empty());
    s.c.natrag();
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        2,
        "gotovo se ne dira"
    );
}

#[test]
fn mjere_ukljucuju_tocku_pod_misem_dok_crtanje_traje() {
    let mut s = Stol::novi(Alat::Poligon);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    assert!(matches!(s.c.mjere_crteza(None), Some(Mjere::Linija(_))));
    let mis = Ravno.na_kartu(pos2(100.0, 100.0));
    assert!(matches!(
        s.c.mjere_crteza(Some(mis)),
        Some(Mjere::Poligon(_))
    ));
    s.c.postavi_alat(Alat::Odaberi);
    assert_eq!(s.c.mjere_crteza(None), None);
}

#[test]
fn prava_tocka_ima_prednost_a_karta_se_ne_pomice_kad_je_tocka_pod_misem() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(20.0, 0.0);
    s.zavrsi();
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    // Na 5,5 je srednja točka bliža, ali prava ima prednost od 2 piksela.
    let h = s.c.najblize(pos2(5.5, 0.0)).expect("hvatište");
    assert!(!h.srednje && h.i == 0, "{h:?}");
    assert!(s.c.najblize(pos2(8.0, 0.0)).expect("hvatište").srednje);
    assert!(s.c.najblize(pos2(40.0, 40.0)).is_none());
    assert!(!s.c.smije_pomicati_kartu(Some(pos2(0.0, 0.0))));
    assert!(s.c.smije_pomicati_kartu(Some(pos2(40.0, 40.0))));
    assert!(s.c.smije_pomicati_kartu(None));
    s.c.pocni_vuci(
        pos2(0.0, 0.0),
        Ravno.na_kartu(pos2(1.0, 1.0)),
        s.u.projekt(),
    );
    assert!(
        !s.c.smije_pomicati_kartu(Some(pos2(40.0, 40.0))),
        "dok se vuče"
    );
    s.c.odustani();
    assert!(s.c.smije_pomicati_kartu(None), "Esc prekida vučenje");
}

#[test]
fn slojevi_odozgo_ide_redom_stabla_kroz_mape() {
    let mut u = Uredjivac::novi();
    let d = |u: &mut Uredjivac, g| {
        u.izvrsi(NaredbaStabla::DodajSloj {
            u: g,
            naziv: "S".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
    };
    let Ok(Some(Novo::Sloj(a))) = d(&mut u, None) else {
        panic!("a");
    };
    let m = u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "M".into(),
    });
    let Ok(Some(Novo::Mapa(m))) = m else {
        panic!("m");
    };
    let Ok(Some(Novo::Sloj(b))) = d(&mut u, Some(m)) else {
        panic!("b");
    };
    let Ok(Some(Novo::Sloj(c))) = d(&mut u, None) else {
        panic!("c");
    };
    assert_eq!(slojevi_odozgo(u.projekt()), vec![a, b, c]);
}

/// Slovo L, nesimetrično, s ulegnutim dijelom gore desno.
fn slovo_l() -> Vec<Pos2> {
    [(0, 0), (40, 0), (40, 60), (100, 60), (100, 100), (0, 100)]
        .iter()
        .map(|(x, y)| pos2(*x as f32, *y as f32))
        .collect()
}

#[test]
fn unutar_lika_gleda_pravi_oblik_a_ne_okvir() {
    let l = slovo_l();
    for (x, y) in [
        (20.0, 30.0),
        (20.0, 80.0),
        (70.0, 80.0),
        (99.0, 61.0),
        (1.0, 1.0),
    ] {
        assert!(unutar(&l, pos2(x, y)), "({x}, {y}) je unutra");
    }
    for (x, y) in [
        (70.0, 30.0),
        (-1.0, 30.0),
        (41.0, 30.0),
        (101.0, 80.0),
        (-1.0, 80.0),
        (20.0, -1.0),
        (20.0, 101.0),
        (70.0, 59.0),
    ] {
        assert!(!unutar(&l, pos2(x, y)), "({x}, {y}) je vani");
    }
}

#[test]
fn unutar_racuna_kose_stranice_tocno() {
    // Trokut s kosom stranicom, gdje je na visini 10 granica na x = 75.
    let t = [pos2(0.0, 0.0), pos2(100.0, 40.0), pos2(0.0, 40.0)];
    assert!(unutar(&t, pos2(24.0, 10.0)));
    assert!(!unutar(&t, pos2(26.0, 10.0)));
    // Isti trokut pomaknut daleko od ishodišta, da se vidi i pomak u računu.
    let d: Vec<Pos2> = t.iter().map(|m| pos2(m.x + 300.0, m.y + 200.0)).collect();
    assert!(unutar(&d, pos2(324.0, 210.0)));
    assert!(!unutar(&d, pos2(326.0, 210.0)));
    assert!(!unutar(&d, pos2(299.0, 210.0)));
}

#[test]
fn zraka_kroz_vrh_lika_broji_se_jednom() {
    // Lijevi i desni vrh romba na visini su točke, pa zraka prolazi točno kroz vrh.
    let romb = [
        pos2(0.0, -10.0),
        pos2(10.0, 0.0),
        pos2(0.0, 10.0),
        pos2(-10.0, 0.0),
    ];
    assert!(unutar(&romb, pos2(0.0, 0.0)));
    assert!(!unutar(&romb, pos2(20.0, 0.0)));
    assert!(!unutar(&romb, pos2(-20.0, 0.0)));
}

#[test]
fn doseg_tocke_i_linije_je_tocno_u_pikselima() {
    let t = Geometrija::Tocka(Ravno.na_kartu(pos2(100.0, 100.0)));
    assert!(pogada(&t, pos2(111.5, 100.0), &Ravno));
    assert!(
        !pogada(&t, pos2(112.0, 100.0), &Ravno),
        "granica nije unutra"
    );
    assert!(!pogada(&t, pos2(150.0, 100.0), &Ravno));
    let v = vec![
        Ravno.na_kartu(pos2(0.0, 0.0)),
        Ravno.na_kartu(pos2(100.0, 0.0)),
    ];
    let c = Geometrija::Linija(v);
    assert!(pogada(&c, pos2(50.0, 5.5), &Ravno));
    assert!(!pogada(&c, pos2(50.0, 6.0), &Ravno), "granica nije unutra");
}

#[test]
fn odabir_koristi_oblik_lika() {
    let mut s = Stol::novi(Alat::Poligon);
    for m in slovo_l() {
        s.klik(m.x, m.y);
    }
    s.zavrsi();
    let cilj = s.c.odabran().expect("odabran");
    s.klik(70.0, 30.0);
    assert!(s.c.odabran().is_none(), "ulegnuti dio nije lik");
    s.klik(70.0, 80.0);
    assert_eq!(s.c.odabran(), Some(cilj));
}

#[test]
fn povlacenje_tocke_objekta_cuva_visinu_a_pregled_je_vidljiv() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.zavrsi();
    let cilj = s.c.odabran().expect("odabran");
    let Geometrija::Linija(mut v) = s.geometrija(cilj) else {
        panic!("linija");
    };
    v[1].visina = Some(525.4);
    s.u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: cilj.0,
        objekt: cilj.1,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Linija(v)),
    })
    .expect("visina");
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    let cilj_mjesto = pos2(100.0, 70.0);
    s.c.pocni_vuci(pos2(100.0, 0.0), Ravno.na_kartu(cilj_mjesto), s.u.projekt());
    s.c.vuci(Ravno.na_kartu(cilj_mjesto));
    assert!(s.c.pregled().is_some(), "dok se vuče, ima pregled");
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    let h = s.c.najblize(cilj_mjesto).expect("hvatište prati miš");
    assert_eq!(h.i, 1);
    s.c.pusti(&mut s.u);
    assert!(s.c.pregled().is_none());
    let Geometrija::Linija(v) = s.geometrija(cilj) else {
        panic!("linija");
    };
    assert_eq!(v[1].visina, Some(525.4), "visina nije izgubljena");
    assert!((v[1].sirina - 44.93).abs() < 1e-6);
}

#[test]
fn aktivni_sloj_vrijedi_po_vrsti_a_novi_zamjenjuje_stari() {
    let mut s = Stol::novi(Alat::Tocka);
    let mut sloj = |naziv: &str, vrsta| match s.u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.into(),
        vrsta,
    }) {
        Ok(Some(Novo::Sloj(id))) => id,
        r => panic!("{r:?}"),
    };
    let t1 = sloj("T1", VrstaGeometrije::Tocka);
    let t2 = sloj("T2", VrstaGeometrije::Tocka);
    let t3 = sloj("T3", VrstaGeometrije::Tocka);
    let l = sloj("L", VrstaGeometrije::Linija);
    s.c.postavi_aktivni_sloj(VrstaGeometrije::Tocka, t2);
    s.c.postavi_aktivni_sloj(VrstaGeometrije::Linija, l);
    s.klik(0.0, 0.0);
    assert_eq!(s.c.odabran().expect("odabran").0, t2, "ne prvi u stablu");
    s.c.postavi_aktivni_sloj(VrstaGeometrije::Tocka, t3);
    s.c.postavi_alat(Alat::Tocka);
    s.klik(50.0, 0.0);
    assert_eq!(
        s.c.odabran().expect("odabran").0,
        t3,
        "novi zamjenjuje stari"
    );
    assert_ne!(t1, t3);
}

#[test]
fn srednje_tocke_su_tocno_na_sredini_stranica() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(10.0, 20.0);
    s.klik(50.0, 80.0);
    s.zavrsi();
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    let h = s.c.hvatista.iter().find(|h| h.srednje).expect("srednja");
    assert!(
        (h.mjesto.x - 30.0).abs() < 1e-3 && (h.mjesto.y - 50.0).abs() < 1e-3,
        "{h:?}"
    );
    assert_eq!(
        s.c.hvatista.iter().filter(|h| h.srednje).count(),
        1,
        "linija nije zatvorena"
    );
}

#[test]
fn zastarjelo_hvatiste_ne_rusi_program() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.c.obrisi_iz_crteza(2);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 2);
    s.zavrsi();
    let c = s.c.odabran().expect("odabran");
    let h = Hvatiste {
        cilj: Cilj::Objekt(c.0, c.1),
        prsten: 0,
        i: 2,
        srednje: false,
        mjesto: pos2(0.0, 0.0),
    };
    let dodaj = NaredbaObjekta::Izmijeni {
        sloj: c.0,
        objekt: c.1,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Linija(vec![
            Vrh::novi(18.0, 45.0),
            Vrh::novi(18.1, 45.0),
            Vrh::novi(18.2, 45.0),
        ])),
    };
    s.u.izvrsi(dodaj).expect("tri vrha");
    let h = Hvatiste { i: 3, ..h };
    s.c.obrisi_iz_objekta(c.0, c.1, h, &mut s.u);
    assert_eq!(broj_vrhova(&s.geometrija(c)), 3, "vrha 3 nema");
    assert_eq!(s.c.poruka(), Some(Poruka::NeMozeBrisati { najmanje: 2 }));
}

#[test]
fn mjerenje_daje_ukupno_dijelove_i_duljinu_svakog_dijela() {
    let mut s = Stol::novi(Alat::Mjeri);
    assert_eq!(s.c.mjerenje(), None);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.klik(100.0, 200.0);
    let m = s.c.mjerenje().expect("mjerenje");
    assert_eq!(m.dijelova, 2);
    assert!(!m.gotovo);
    let dijelovi = s.c.dijelovi_mjerenja(None, &Ravno);
    assert_eq!(dijelovi.len(), 2);
    assert!((dijelovi[0].0.x - 50.0).abs() < 1e-3 && dijelovi[0].0.y.abs() < 1e-3);
    assert!((dijelovi[1].0.x - 100.0).abs() < 1e-3 && (dijelovi[1].0.y - 100.0).abs() < 1e-3);
    // Isti račun kao ukupno: zbroj dijelova je ukupna duljina.
    let zbroj: f64 = dijelovi.iter().map(|(_, d)| d).sum();
    let Mjere::Linija(ukupno) = m.mjere else {
        panic!("linija");
    };
    assert!((zbroj - ukupno.duljina_m).abs() < 1e-6);
    assert!(ukupno.duljina_m > 1000.0);
    let do_misa =
        s.c.dijelovi_mjerenja(Some(Ravno.na_kartu(pos2(300.0, 200.0))), &Ravno);
    assert_eq!(do_misa.len(), 3, "zadnji dio ide do miša");
    s.zavrsi();
    assert!(s.c.mjerenje().expect("mjerenje").gotovo);
    let gotovo =
        s.c.dijelovi_mjerenja(Some(Ravno.na_kartu(pos2(300.0, 200.0))), &Ravno);
    assert_eq!(gotovo.len(), 2, "gotovo mjerenje ne prati miš");
}

#[test]
fn povrsina_se_bira_prije_i_tijekom_mjerenja_i_zatvara_lik() {
    let mut s = Stol::novi(Alat::Mjeri);
    s.c.mjeri_povrsinu(true);
    assert!(s.c.mjerenje_povrsine());
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    assert!(
        matches!(s.c.mjerenje().expect("m").mjere, Mjere::Linija(_)),
        "dvije točke nisu lik"
    );
    assert_eq!(s.c.mjerenje().expect("m").dijelova, 1);
    s.klik(100.0, 100.0);
    let m = s.c.mjerenje().expect("m");
    assert!(matches!(m.mjere, Mjere::Poligon(_)));
    assert_eq!(m.dijelova, 3, "uključuje stranicu natrag na početak");
    assert_eq!(s.c.dijelovi_mjerenja(None, &Ravno).len(), 3);
    s.c.mjeri_povrsinu(false);
    assert!(!s.c.mjerenje_povrsine());
    assert!(matches!(s.c.mjerenje().expect("m").mjere, Mjere::Linija(_)));
    assert_eq!(s.c.mjerenje().expect("m").dijelova, 2);
}

#[test]
fn spremanje_mjerenja_stvara_objekt_tek_na_zahtjev() {
    let mut s = Stol::novi(Alat::Mjeri);
    s.c.spremi_mjerenje(&mut s.u, &OK);
    assert!(s.u.projekt().korijen().is_empty(), "nema što spremiti");
    s.klik(0.0, 0.0);
    s.c.spremi_mjerenje(&mut s.u, &OK);
    assert_eq!(s.c.poruka(), Some(Poruka::Premalo { najmanje: 2 }));
    s.klik(100.0, 0.0);
    assert!(s.u.projekt().korijen().is_empty());
    s.c.spremi_mjerenje(&mut s.u, &OK);
    let c = s.c.odabran().expect("odabran");
    assert!(matches!(s.geometrija(c), Geometrija::Linija(_)));
    assert!(s.c.crtez.is_none(), "mjerenje je postalo objekt");

    s.c.postavi_alat(Alat::Mjeri);
    s.c.mjeri_povrsinu(true);
    s.klik(200.0, 0.0);
    s.klik(300.0, 0.0);
    s.c.spremi_mjerenje(&mut s.u, &OK);
    assert_eq!(s.c.poruka(), Some(Poruka::Premalo { najmanje: 3 }));
    s.klik(300.0, 100.0);
    s.c.spremi_mjerenje(&mut s.u, &OK);
    let c = s.c.odabran().expect("odabran");
    assert!(matches!(s.geometrija(c), Geometrija::Poligon(_)));
}

#[test]
fn ocisti_brise_samo_mjerenje_a_gotovo_se_jos_moze_doraditi() {
    let mut s = Stol::novi(Alat::Linija);
    s.klik(0.0, 0.0);
    s.c.ocisti_mjerenje();
    assert!(s.c.crtez.is_some(), "crtež linije nije mjerenje");
    assert_eq!(s.c.mjerenje(), None);
    assert!(s.c.dijelovi_mjerenja(None, &Ravno).is_empty());
    let mut s = Stol::novi(Alat::Mjeri);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.zavrsi();
    s.vuci(pos2(100.0, 0.0), pos2(100.0, 50.0));
    let v = s.c.crtez.as_ref().expect("crtež").vrhovi[1];
    assert!(
        (v.sirina - 44.95).abs() < 1e-6,
        "gotovom mjerenju vrh se može pomaknuti"
    );
    s.c.ocisti_mjerenje();
    assert!(s.c.crtez.is_none());
}

/// Crta liniju od tri točke i vraća je odabranu, s prvom točkom koja ima visinu.
fn linija(s: &mut Stol) -> (IdSloja, IdObjekta) {
    s.c.postavi_alat(Alat::Linija);
    s.klik(0.0, 0.0);
    s.klik(100.0, 0.0);
    s.klik(200.0, 0.0);
    s.zavrsi();
    let c = s.c.odabran().expect("odabran");
    let Geometrija::Linija(mut v) = s.geometrija(c) else {
        panic!("linija");
    };
    v[0].visina = Some(100.0);
    s.u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: c.0,
        objekt: c.1,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Linija(v)),
    })
    .expect("visina");
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    c
}

fn vrhovi_linije(s: &Stol, c: (IdSloja, IdObjekta)) -> Vec<Vrh> {
    match s.geometrija(c) {
        Geometrija::Linija(v) => v,
        g => panic!("{g:?}"),
    }
}

#[test]
fn klik_na_zadnju_tocku_nastavlja_liniju_kao_jedan_korak() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    let prije = s.geometrija(c);
    s.klik(200.0, 0.0);
    assert_eq!(s.c.alat(), Alat::Linija);
    s.klik(300.0, 50.0);
    s.klik(400.0, 50.0);
    assert_eq!(s.geometrija(c), prije, "objekt se mijenja tek na završetku");
    s.zavrsi();
    let v = vrhovi_linije(&s, c);
    assert_eq!(v.len(), 5);
    assert!((v[4].duzina - 18.4).abs() < 1e-6);
    assert_eq!(v[0].visina, Some(100.0), "visina postojećih vrhova ostaje");
    assert_eq!(s.c.alat(), Alat::Odaberi);
    assert_eq!(s.c.odabran(), Some(c));
    assert!(s.c.crtez.is_none());
    assert_eq!(s.u.ponisti(), Ok(true));
    assert_eq!(s.geometrija(c), prije, "cijelo produljenje je jedan korak");
}

#[test]
fn klik_na_prvu_tocku_nastavlja_od_pocetka_a_smjer_ostaje() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    s.klik(0.0, 0.0);
    s.klik(-100.0, 0.0);
    s.zavrsi();
    let v = vrhovi_linije(&s, c);
    assert_eq!(v.len(), 4);
    assert!(
        (v[0].duzina - 17.9).abs() < 1e-6,
        "nova je točka na početku"
    );
    assert!((v[3].duzina - 18.2).abs() < 1e-6, "kraj je i dalje kraj");
    assert_eq!(v[1].visina, Some(100.0));
}

#[test]
fn srednji_vrh_i_srednja_tocka_ne_nastavljaju() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    s.klik(100.0, 0.0);
    assert_eq!(s.c.alat(), Alat::Odaberi, "srednji vrh");
    assert!(s.c.crtez.is_none());
    s.c.osvjezi_hvatista(s.u.projekt(), &Ravno);
    s.klik(50.0, 0.0);
    assert_eq!(s.c.alat(), Alat::Odaberi, "srednja točka");
    assert_eq!(s.c.odabran(), Some(c), "klik na liniju je i dalje odabire");
}

#[test]
fn poligon_se_ne_nastavlja() {
    let mut s = Stol::novi(Alat::Poligon);
    let c = s.poligon();
    s.klik(0.0, 0.0);
    assert_eq!(s.c.alat(), Alat::Odaberi);
    assert!(s.c.crtez.is_none());
    assert_eq!(broj_vrhova(&s.geometrija(c)), 4);
}

#[test]
fn zavrsetak_bez_novih_tocaka_ne_ostavlja_korak_u_povijesti() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    let prije = s.u.inacica();
    s.klik(200.0, 0.0);
    s.zavrsi();
    assert_eq!(s.c.alat(), Alat::Odaberi);
    assert_eq!(s.u.inacica(), prije);
    assert_eq!(vrhovi_linije(&s, c).len(), 3);
}

#[test]
fn esc_i_backspace_cuvaju_postojeci_dio_linije() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    let prije = s.geometrija(c);
    s.klik(200.0, 0.0);
    s.klik(300.0, 0.0);
    s.c.natrag();
    s.c.natrag();
    s.c.natrag();
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        3,
        "postojeći vrhovi ostaju"
    );
    s.c.odustani();
    assert_eq!(s.geometrija(c), prije);
    assert_eq!(
        s.c.alat(),
        Alat::Odaberi,
        "sljedeći klik ne počinje novu liniju"
    );
    s.klik(500.0, 500.0);
    assert!(s.c.crtez.is_none());
}

#[test]
fn desni_klik_u_nastavku_mijenja_i_granicu_za_backspace() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    s.klik(200.0, 0.0);
    s.klik(300.0, 0.0);
    s.desni(100.0, 0.0);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 3);
    s.c.natrag();
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        2,
        "nova točka je maknuta"
    );
    s.c.natrag();
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        2,
        "postojeće ostaju"
    );
    s.desni(300.0, 0.0);
    s.c.obrisi_iz_crteza(9);
    s.zavrsi();
    assert_eq!(vrhovi_linije(&s, c).len(), 2);
}

#[test]
fn desni_klik_na_novu_tocku_ne_otvara_brisanje_postojecih() {
    let mut s = Stol::novi(Alat::Linija);
    linija(&mut s);
    s.klik(200.0, 0.0);
    s.klik(300.0, 0.0);
    s.desni(300.0, 0.0);
    assert_eq!(s.c.crtez.as_ref().expect("crtež").vrhovi.len(), 3);
    s.c.natrag();
    assert_eq!(
        s.c.crtez.as_ref().expect("crtež").vrhovi.len(),
        3,
        "Backspace ne briše točke koje su već bile u liniji"
    );
}

#[test]
fn zakljucan_sloj_se_ne_nastavlja() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    s.u.postavi_stanje_sloja(c.0, StanjeSloja::SamoCitanje)
        .expect("stanje");
    s.klik(200.0, 0.0);
    assert_eq!(s.c.alat(), Alat::Odaberi);
    assert!(s.c.crtez.is_none());
}

#[test]
fn sloj_zakljucan_tijekom_nastavka_javlja_gresku_a_ne_gubi_objekt() {
    let mut s = Stol::novi(Alat::Linija);
    let c = linija(&mut s);
    let prije = s.geometrija(c);
    s.klik(200.0, 0.0);
    s.klik(300.0, 0.0);
    s.u.postavi_stanje_sloja(c.0, StanjeSloja::USukobu)
        .expect("stanje");
    s.zavrsi();
    assert_eq!(s.c.poruka(), Some(Poruka::Naredba(Greska::SlojZakljucan)));
    assert_eq!(s.geometrija(c), prije);
}
