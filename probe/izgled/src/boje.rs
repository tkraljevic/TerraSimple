//! Boje iz nacrta (dizajn/skripte/temelj.py), iste vrijednosti u obje teme.

use eframe::egui::Color32;

const fn c(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

#[derive(Clone, Copy)]
pub struct Boje {
    pub vrh: Color32,
    pub ploha: Color32,
    pub visoko: Color32,
    pub rub: Color32,
    pub tekst: Color32,
    pub sivo: Color32,
    pub blijedo: Color32,
    pub glavna: Color32,
    pub na_glavnoj: Color32,
    pub tint: Color32,
    pub na_tintu: Color32,
    pub crtez: Color32,
    pub crtez_ploha: Color32,
    pub sjena: Color32,
    pub k_zemlja: Color32,
    pub k_suma: Color32,
    pub k_naselje: Color32,
    pub k_voda: Color32,
    pub k_cesta: Color32,
    pub k_cesta_rub: Color32,
    pub k_glavna: Color32,
    pub k_natpis: Color32,
    pub k_voda_natpis: Color32,
}

pub const SVIJETLA: Boje = Boje {
    vrh: c(0xFAF9FD),
    ploha: c(0xFFFFFF),
    visoko: c(0xE9E8EC),
    rub: c(0xC4C6D0),
    tekst: c(0x1A1C1E),
    sivo: c(0x44474F),
    blijedo: c(0x74777F),
    glavna: c(0x0B5C56),
    na_glavnoj: c(0xFFFFFF),
    tint: c(0xCFE8E3),
    na_tintu: c(0x00302C),
    crtez: c(0xE8590C),
    crtez_ploha: Color32::from_rgba_premultiplied(58, 22, 3, 64),
    sjena: Color32::from_black_alpha(38),
    k_zemlja: c(0xEEF0E8),
    k_suma: c(0xD3E3C1),
    k_naselje: c(0xE5E1D9),
    k_voda: c(0x7DB4E0),
    k_cesta: c(0xFFFFFF),
    k_cesta_rub: c(0xD4D1C7),
    k_glavna: c(0xF3D08A),
    k_natpis: c(0x50535A),
    k_voda_natpis: c(0x3B6E9C),
};

pub const TAMNA: Boje = Boje {
    vrh: c(0x1E1F23),
    ploha: c(0x26272C),
    visoko: c(0x303136),
    rub: c(0x44474F),
    tekst: c(0xE3E2E6),
    sivo: c(0xC4C6D0),
    blijedo: c(0x8E9099),
    glavna: c(0x7FD4C8),
    na_glavnoj: c(0x00302C),
    tint: c(0x1F4A45),
    na_tintu: c(0xCFE8E3),
    crtez: c(0xFF8A4C),
    crtez_ploha: Color32::from_rgba_premultiplied(64, 35, 19, 64),
    sjena: Color32::from_black_alpha(102),
    k_zemlja: c(0x1D211E),
    k_suma: c(0x25301F),
    k_naselje: c(0x2C2C2A),
    k_voda: c(0x2F6085),
    k_cesta: c(0x494C49),
    k_cesta_rub: c(0x2F312F),
    k_glavna: c(0x6B5A33),
    k_natpis: c(0xB8BBC2),
    k_voda_natpis: c(0x8DBDE6),
};
