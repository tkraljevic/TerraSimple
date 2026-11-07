//! Što je na ekranu i koliko je to u metrima. Ne zna za grafičku karticu.

use bytemuck::{Pod, Zeroable};

/// `pola` je pola ekrana u istim jedinicama kao širina crte, u točkama sučelja.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Kamera {
    pub sredina: [f64; 2],
    pub metara_po_px: f64,
    pub pola: [f32; 2],
}

/// Podaci koje sjenčar dobiva za jedan komad karte.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Pogled {
    pub pomak: [f32; 2],
    pub mjerilo: f32,
    pub rezerva: f32,
    pub pola: [f32; 2],
    pub rezerva2: [f32; 2],
}

impl Kamera {
    /// Najgrublja razina čija je greška ispod jednog piksela.
    pub fn razina(&self, greske: &[f64]) -> usize {
        greske
            .iter()
            .rposition(|g| *g <= self.metara_po_px)
            .unwrap_or(0)
    }

    /// Je li okvir `[min x, min y, max x, max y]` barem djelomično na ekranu.
    pub fn vidi(&self, okvir: [f64; 4]) -> bool {
        let pola = [
            f64::from(self.pola[0]) * self.metara_po_px,
            f64::from(self.pola[1]) * self.metara_po_px,
        ];
        okvir[2] >= self.sredina[0] - pola[0]
            && okvir[0] <= self.sredina[0] + pola[0]
            && okvir[3] >= self.sredina[1] - pola[1]
            && okvir[1] <= self.sredina[1] + pola[1]
    }

    /// Vrhovi su u `f32` u odnosu na ishodište komada, pa pomicanje mijenja samo ovu vrijednost.
    pub fn pogled(&self, ishodiste: [f64; 2]) -> Pogled {
        Pogled {
            pomak: [
                ((ishodiste[0] - self.sredina[0]) / self.metara_po_px) as f32,
                ((ishodiste[1] - self.sredina[1]) / self.metara_po_px) as f32,
            ],
            mjerilo: (1.0 / self.metara_po_px) as f32,
            rezerva: 0.0,
            pola: self.pola,
            rezerva2: [0.0; 2],
        }
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    fn kamera() -> Kamera {
        Kamera {
            sredina: [1000.0, 2000.0],
            metara_po_px: 2.0,
            pola: [100.0, 50.0],
        }
    }

    #[test]
    fn razina_ima_gresku_ispod_piksela() {
        let greske = [0.0, 2.0, 8.0, 32.0];
        assert_eq!(kamera().razina(&greske), 1);
        let mut k = kamera();
        k.metara_po_px = 1.0;
        assert_eq!(k.razina(&greske), 0);
        k.metara_po_px = 1000.0;
        assert_eq!(k.razina(&greske), 3);
        assert_eq!(k.razina(&[]), 0);
    }

    #[test]
    fn okvir_izvan_ekrana_se_ne_crta_ni_s_jedne_strane() {
        let k = kamera(); // Ekran je od 800 do 1200 i od 1900 do 2100.
        assert!(k.vidi([900.0, 1950.0, 950.0, 1990.0]));
        assert!(k.vidi([1200.0, 2100.0, 1300.0, 2200.0]), "dodiruje ugao");
        assert!(k.vidi([0.0, 0.0, 5000.0, 5000.0]), "veći od ekrana");
        assert!(!k.vidi([0.0, 1900.0, 799.0, 2100.0]), "lijevo");
        assert!(!k.vidi([1201.0, 1900.0, 1300.0, 2100.0]), "desno");
        assert!(!k.vidi([800.0, 0.0, 1200.0, 1899.0]), "dolje");
        assert!(!k.vidi([800.0, 2101.0, 1200.0, 3000.0]), "gore");
    }

    #[test]
    fn pogled_je_u_tockama_u_odnosu_na_sredinu() {
        let p = kamera().pogled([1100.0, 1900.0]);
        assert_eq!(p.pomak, [50.0, -50.0]);
        assert_eq!(p.mjerilo, 0.5);
        assert_eq!(p.pola, [100.0, 50.0]);
    }
}
