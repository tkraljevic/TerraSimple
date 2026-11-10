use super::*;
use crate::crtanje::Alat;
use eframe::egui::pos2;
use slojevi::{Mjere, NaredbaStabla, Novo, StanjeSloja, Vrh, VrstaGeometrije};

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
fn naziv(_: VrstaGeometrije) -> String {
    String::new()
}
fn opis(_: &Mjere) -> String {
    String::new()
}
fn naziv_objekta(_: VrstaGeometrije, _: usize) -> String {
    String::new()
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

fn kvadrat(x: f32, y: f32, a: f32) -> Geometrija {
    let v = |x, y| Ravno.na_kartu(pos2(x, y));
    Geometrija::Poligon(vec![vec![
        v(x, y),
        v(x + a, y),
        v(x + a, y + a),
        v(x, y + a),
    ]])
}

struct Stol {
    c: Crtanje,
    u: Uredjivac,
    sloj: IdSloja,
    /// Kvadrat 0–100 i kvadrat 150–250, oba od y = 0.
    lijevi: (IdSloja, IdObjekta),
    desni: (IdSloja, IdObjekta),
}

fn dodaj(u: &mut Uredjivac, sloj: IdSloja, g: Geometrija) -> (IdSloja, IdObjekta) {
    let Ok(Some(Novo::Objekt(id))) = u.izvrsi(NaredbaObjekta::Dodaj {
        sloj,
        naziv: "Kvadrat".into(),
        geometrija: g,
    }) else {
        panic!("objekt")
    };
    (sloj, id)
}

fn stol() -> Stol {
    let mut u = Uredjivac::novi();
    let Ok(Some(Novo::Sloj(sloj))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Parcele".into(),
        vrsta: VrstaGeometrije::Poligon,
    }) else {
        panic!("sloj")
    };
    let lijevi = dodaj(&mut u, sloj, kvadrat(0.0, 0.0, 100.0));
    let desni = dodaj(&mut u, sloj, kvadrat(150.0, 0.0, 100.0));
    let mut c = Crtanje::default();
    c.postavi_alat(Alat::Odaberi);
    Stol {
        c,
        u,
        sloj,
        lijevi,
        desni,
    }
}

impl Stol {
    fn klik(&mut self, x: f32, y: f32, shift: bool) {
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
        self.c.klik_s(pos2(x, y), shift, &mut self.u, &OK, &Ravno);
        self.c.osvjezi_hvatista(self.u.projekt(), &Ravno);
    }
}

#[test]
fn okvir_slijeva_nadesno_bira_samo_cijele_objekte() {
    let mut s = stol();
    s.klik(-10.0, -10.0, false);
    assert!(
        s.c.okvir_odabira().is_some(),
        "klik na prazno počinje okvir"
    );
    assert!(s.c.odabrani().is_empty());
    s.klik(200.0, 120.0, false);
    assert!(s.c.okvir_odabira().is_none());
    assert_eq!(s.c.odabrani(), [s.lijevi]);
    assert_eq!(s.c.odabran(), Some(s.lijevi));
}

#[test]
fn okvir_zdesna_nalijevo_bira_sve_sto_dotakne() {
    let mut s = stol();
    s.klik(200.0, 120.0, false);
    s.klik(-10.0, -10.0, false);
    assert_eq!(s.c.odabrani(), [s.lijevi, s.desni]);
    assert_eq!(s.c.odabran(), None, "više odabranih nema točke za hvatanje");
    assert!(s.c.hvatista.is_empty());
}

#[test]
fn klik_dodaje_u_odabir_a_shift_klik_izbacuje() {
    let mut s = stol();
    s.klik(50.0, 50.0, false);
    s.klik(200.0, 50.0, false);
    assert_eq!(s.c.odabrani(), [s.lijevi, s.desni]);
    s.klik(200.0, 50.0, false);
    assert_eq!(
        s.c.odabrani(),
        [s.lijevi, s.desni],
        "drugi klik ne izbacuje"
    );
    s.klik(50.0, 50.0, true);
    assert_eq!(s.c.odabrani(), [s.desni]);

    // Shift i okvir izbacuju sve što okvir bira.
    s.klik(300.0, 120.0, true);
    s.klik(-10.0, -10.0, true);
    assert!(s.c.odabrani().is_empty());
}

#[test]
fn esc_odbacuje_okvir_pa_odabir() {
    let mut s = stol();
    s.klik(50.0, 50.0, false);
    s.klik(500.0, 500.0, false);
    assert!(s.c.okvir_odabira().is_some());
    s.c.odustani();
    assert!(s.c.okvir_odabira().is_none());
    assert_eq!(s.c.odabrani(), [s.lijevi]);
    s.c.odustani();
    assert!(s.c.odabrani().is_empty());
}

#[test]
fn brisanje_odabranih_je_jedan_korak_a_zakljucan_sloj_ostaje() {
    let mut s = stol();
    s.klik(300.0, 120.0, false);
    s.klik(-10.0, -10.0, false);
    assert!(s.c.obrisi_odabrane(&mut s.u));
    assert!(s.u.projekt().sloj(s.sloj).expect("sloj").objekti.is_empty());
    assert!(s.c.odabrani().is_empty());
    assert_eq!(s.u.ponisti(), Ok(true));
    assert_eq!(s.u.projekt().sloj(s.sloj).expect("sloj").objekti.len(), 2);

    assert!(!s.c.obrisi_odabrane(&mut s.u), "ništa nije odabrano");
    s.klik(50.0, 50.0, false);
    s.u.postavi_stanje_sloja(s.sloj, StanjeSloja::SamoCitanje)
        .expect("zaključavanje");
    assert!(!s.c.obrisi_odabrane(&mut s.u));
    assert_eq!(s.u.projekt().sloj(s.sloj).expect("sloj").objekti.len(), 2);
}

#[test]
fn dodir_vidi_crtu_kroz_okvir_i_okvir_unutar_lika() {
    let okvir = Rect::from_min_max(pos2(40.0, 40.0), pos2(60.0, 60.0));
    let v = |x, y| Ravno.na_kartu(pos2(x, y));
    let kroz = Geometrija::Linija(vec![v(0.0, 50.0), v(100.0, 50.0)]);
    assert!(u_okviru(&kroz, okvir, true, &Ravno));
    assert!(!u_okviru(&kroz, okvir, false, &Ravno));
    let oko = kvadrat(0.0, 0.0, 100.0);
    assert!(u_okviru(&oko, okvir, true, &Ravno), "okvir je unutar lika");
    let pokraj = Geometrija::Linija(vec![v(0.0, 0.0), v(100.0, 0.0)]);
    assert!(!u_okviru(&pokraj, okvir, true, &Ravno));
    let tocka = Geometrija::Tocka(v(50.0, 50.0));
    assert!(u_okviru(&tocka, okvir, false, &Ravno));
    assert!(!u_okviru(
        &Geometrija::Linija(Vec::new()),
        okvir,
        true,
        &Ravno
    ));
}

#[test]
fn klik_na_tocku_odabranog_lika_odabire_tocku_a_ne_pocinje_okvir() {
    let mut s = stol();
    s.klik(50.0, 50.0, false);
    s.klik(100.0, 100.0, false);
    assert_eq!(s.c.odabrana_tocka(), Some((0, 2)));
    assert!(s.c.okvir_odabira().is_none());
}
