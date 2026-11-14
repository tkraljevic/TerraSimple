//! Odabir objekata kao u AutoCAD-u: klik na objekt dodaje ga u odabir, Shift + klik ga izbacuje, a
//! klik na prazno mjesto počinje okvir koji drugi klik zatvara. Okvir povučen slijeva nadesno bira
//! samo ono što je cijelo u njemu, a zdesna nalijevo sve što dotakne.

use crate::crtanje::{
    Cilj, Crtanje, Okolina, Poruka, Pretvorba, najblizi_objekt, pogada, prstenovi, slojevi_odozgo,
    u_tocke,
};
use eframe::egui::{Pos2, Rect};
use slojevi::{Geometrija, IdObjekta, IdSloja, NaredbaObjekta, Projekt, Uredjivac};

impl Crtanje {
    pub(crate) fn klik_odabira(
        &mut self,
        m: Pos2,
        shift: bool,
        projekt: &Projekt,
        ok: &Okolina<'_>,
        p: &dyn Pretvorba,
    ) {
        if let Some(pocetak) = self.okvir.take() {
            self.zatvori_okvir(p.u_ekran(&pocetak), m, shift, projekt, ok, p);
            return;
        }
        if self.pocni_nastavak(m, projekt) || self.odaberi_tocku(m, projekt) {
            return;
        }
        match objekt_pod(m, projekt, ok, p) {
            Some(objekt) => self.promijeni_odabir(objekt, shift),
            None => self.okvir = Some(p.na_kartu(m)),
        }
    }

    /// Klik na točku odabranog objekta odabire tu točku. Objekt koji je samo točka nema što
    /// odabrati, pa ga Backspace briše cijelog.
    fn odaberi_tocku(&mut self, m: Pos2, projekt: &Projekt) -> bool {
        let Some(h) = self.najblize(m).filter(|h| !h.srednje) else {
            return false;
        };
        let Cilj::Objekt(sloj, objekt) = h.cilj else {
            return false;
        };
        let tocka = projekt
            .objekt(sloj, objekt)
            .is_some_and(|o| matches!(o.geometrija, Geometrija::Tocka(_)));
        if tocka {
            return false;
        }
        self.tocka = Some((h.prsten, h.i));
        true
    }

    fn promijeni_odabir(&mut self, objekt: (IdSloja, IdObjekta), shift: bool) {
        self.tocka = None;
        if shift {
            self.odabrani.retain(|o| *o != objekt);
        } else if !self.odabrani.contains(&objekt) {
            self.odabrani.push(objekt);
        }
    }

    fn zatvori_okvir(
        &mut self,
        a: Pos2,
        b: Pos2,
        shift: bool,
        projekt: &Projekt,
        ok: &Okolina<'_>,
        p: &dyn Pretvorba,
    ) {
        let okvir = Rect::from_two_pos(a, b);
        let dodir = b.x < a.x;
        let pogodjeni: Vec<(IdSloja, IdObjekta)> = slojevi_odozgo(projekt)
            .into_iter()
            .filter(|s| (ok.vidljiv)(*s))
            .filter_map(|s| projekt.sloj(s).map(|sloj| (s, sloj)))
            .flat_map(|(s, sloj)| {
                sloj.objekti
                    .values()
                    .filter(|o| u_okviru(&o.geometrija, okvir, dodir, p))
                    .map(move |o| (s, o.id))
            })
            .collect();
        for objekt in pogodjeni {
            self.promijeni_odabir(objekt, shift);
        }
    }

    /// Okvir koji se upravo crta: prvi kut je zapamćen na karti, a drugi je miš.
    pub(crate) fn okvir_odabira(&self) -> Option<slojevi::Vrh> {
        self.okvir
    }

    /// Briše sve odabrane objekte u jednom koraku poništavanja. Objekti u zaključanom sloju ostaju.
    pub fn obrisi_odabrane(&mut self, u: &mut Uredjivac) -> bool {
        let projekt = u.projekt();
        let ciljevi: Vec<(IdSloja, IdObjekta)> = self
            .odabrani
            .iter()
            .copied()
            .filter(|(s, _)| projekt.sloj(*s).is_some_and(|x| x.stanje.smije_mijenjati()))
            .collect();
        if ciljevi.is_empty() {
            return false;
        }
        let ishod = u.skupina(|g| {
            for (sloj, objekt) in ciljevi {
                g.izvrsi(NaredbaObjekta::Obrisi { sloj, objekt })?;
            }
            Ok(())
        });
        match ishod {
            Ok(()) => {
                self.postavi_odabrani(None);
                true
            }
            Err(g) => {
                self.poruka = Some(Poruka::Naredba(g));
                false
            }
        }
    }
}

/// Najgornji vidljivi objekt pod mišem.
fn objekt_pod(
    m: Pos2,
    projekt: &Projekt,
    ok: &Okolina<'_>,
    p: &dyn Pretvorba,
) -> Option<(IdSloja, IdObjekta)> {
    slojevi_odozgo(projekt)
        .into_iter()
        .filter(|s| (ok.vidljiv)(*s))
        .find_map(|s| {
            let sloj = projekt.sloj(s)?;
            let id = najblizi_objekt(sloj.objekti.values().collect(), m, p)?;
            Some((s, id))
        })
}

/// Je li objekt cijeli u okviru, a uz `dodir` dotiče li ga okvir.
pub(crate) fn u_okviru(g: &Geometrija, okvir: Rect, dodir: bool, p: &dyn Pretvorba) -> bool {
    let nizovi: Vec<Vec<Pos2>> = prstenovi(g).iter().map(|r| u_tocke(r, p)).collect();
    let mut tocke = nizovi.iter().flatten();
    if tocke.clone().next().is_none() {
        return false;
    }
    if tocke.all(|t| okvir.contains(*t)) {
        return true;
    }
    if !dodir {
        return false;
    }
    let zatvoren = matches!(g, Geometrija::Poligon(_));
    nizovi
        .iter()
        .any(|niz| rubovi(niz, zatvoren).any(|(a, b)| duzina_dira(a, b, okvir)))
        || (zatvoren && pogada(g, okvir.center(), p))
}

fn rubovi(niz: &[Pos2], zatvoren: bool) -> impl Iterator<Item = (Pos2, Pos2)> + '_ {
    let zatvaranje = (zatvoren && niz.len() >= 3).then(|| (niz[niz.len() - 1], niz[0]));
    niz.windows(2).map(|d| (d[0], d[1])).chain(zatvaranje)
}

/// Dužina dira okvir kad joj je kraj u njemu ili kad siječe neki njegov rub.
fn duzina_dira(a: Pos2, b: Pos2, okvir: Rect) -> bool {
    if okvir.contains(a) || okvir.contains(b) {
        return true;
    }
    let [k0, k1, k2, k3] = [
        okvir.left_top(),
        okvir.right_top(),
        okvir.right_bottom(),
        okvir.left_bottom(),
    ];
    [(k0, k1), (k1, k2), (k2, k3), (k3, k0)]
        .into_iter()
        .any(|(c, d)| sijeku(a, b, c, d))
}

fn sijeku(a: Pos2, b: Pos2, c: Pos2, d: Pos2) -> bool {
    let strana = |p: Pos2, q: Pos2, r: Pos2| (q - p).x * (r - p).y - (q - p).y * (r - p).x;
    strana(c, d, a) * strana(c, d, b) < 0.0 && strana(a, b, c) * strana(a, b, d) < 0.0
}

#[cfg(test)]
#[path = "odabir_testovi.rs"]
mod testovi;
