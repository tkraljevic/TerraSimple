//! Ploča Stil (zasloni C2, C3 i C4): boja, debljina i prozirnost cijelog sloja ili jednog objekta.
//! Otvara se iz izbornika retka u ploči Slojevi i crta se na njezinu mjestu.

use crate::ikone::SkladisteIkona;
use crate::poruke::{KatalogPoruka, broj, postotak};
use crate::sastavnice::{
    OpisGumbaIkone, OpisKlizaca, cip, gumb_ikona, klizac, tekst_desno, tekst_lijevo, tekst_sredina,
    zaobljeni_okvir,
};
use crate::stanje::{StanjePrikaza, UredjivanjeStila};
use crate::tema::{Boje, font_obicni, font_srednji};
use eframe::egui::{
    self, Color32, Pos2, Rect, Response, Role, Sense, Stroke, WidgetInfo, ecolor::Hsva, pos2, vec2,
};
use slojevi::{
    Boja, IzmjenaObjekta, Naredba, NaredbaObjekta, NaredbaStabla, Projekt, Stil, StilObjekta,
    Uredjivac, VrstaGeometrije,
};

/// Ponuđene boje, redom kao u nacrtu. Prva je zadana boja crteža.
pub const PONUDJENE_BOJE: [(Boja, &str); 8] = [
    (rgb(0xE6, 0x51, 0x00), "boja-narancasta"),
    (rgb(0xD3, 0x2F, 0x2F), "boja-crvena"),
    (rgb(0x8E, 0x24, 0xAA), "boja-ljubicasta"),
    (rgb(0x15, 0x65, 0xC0), "boja-plava"),
    (rgb(0x00, 0x83, 0x8F), "boja-tirkizna"),
    (rgb(0x2E, 0x7D, 0x32), "boja-zelena"),
    (rgb(0xF9, 0xA8, 0x25), "boja-zuta"),
    (rgb(0x37, 0x47, 0x4F), "boja-tamnosiva"),
];

const fn rgb(r: u8, g: u8, b: u8) -> Boja {
    Boja { r, g, b, a: 255 }
}

/// Jedna izmjena iz ploče. Prozirnost je od 0 (puno) do 1 (nevidljivo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IzmjenaStila {
    Boja(Boja),
    Debljina(f64),
    Prozirnost(f64),
    ProzirnostIspune(f64),
    Velicina(f64),
}

fn alfa(prozirnost: f64) -> u8 {
    ((1.0 - prozirnost.clamp(0.0, 1.0)) * 255.0).round() as u8
}

#[must_use]
pub fn prozirnost(boja: Boja) -> f64 {
    1.0 - f64::from(boja.a) / 255.0
}

/// Stil s jednom izmjenom. Boja mijenja i crtu i ispunu, a njihova prozirnost ostaje.
#[must_use]
pub fn primijeni(stil: Stil, izmjena: IzmjenaStila) -> Stil {
    let mut s = stil;
    match izmjena {
        IzmjenaStila::Boja(b) => {
            s.boja_crte = Boja {
                a: s.boja_crte.a,
                ..b
            };
            s.boja_ispune = Boja {
                a: s.boja_ispune.a,
                ..b
            };
        }
        IzmjenaStila::Debljina(d) => s.debljina_mm = d,
        IzmjenaStila::Prozirnost(p) => s.boja_crte.a = alfa(p),
        IzmjenaStila::ProzirnostIspune(p) => s.boja_ispune.a = alfa(p),
        IzmjenaStila::Velicina(v) => s.velicina_znaka_mm = v,
    }
    s
}

/// Objekt dobiva vlastitu vrijednost samo za ono što je izmijenjeno, a ostalo i dalje uzima od sloja.
#[must_use]
pub fn primijeni_na_objekt(objekt: StilObjekta, sloj: &Stil, izmjena: IzmjenaStila) -> StilObjekta {
    let novi = primijeni(objekt.na_stilu_sloja(sloj), izmjena);
    let mut s = objekt;
    match izmjena {
        IzmjenaStila::Boja(_) => {
            s.boja_crte = Some(novi.boja_crte);
            s.boja_ispune = Some(novi.boja_ispune);
        }
        IzmjenaStila::Debljina(_) => s.debljina_mm = Some(novi.debljina_mm),
        IzmjenaStila::Prozirnost(_) => s.boja_crte = Some(novi.boja_crte),
        IzmjenaStila::ProzirnostIspune(_) => s.boja_ispune = Some(novi.boja_ispune),
        IzmjenaStila::Velicina(_) => s.velicina_znaka_mm = Some(novi.velicina_znaka_mm),
    }
    s
}

/// Ono što ploča prikazuje: naziv u naslovu, vrsta sloja i stil kojim se crta.
#[derive(Clone, Debug, PartialEq)]
pub struct PrikazStila {
    pub naziv: String,
    pub vrsta: VrstaGeometrije,
    pub stil: Stil,
    pub uredivo: bool,
}

/// Prazno kad sloja ili objekta više nema, pa se ploča zatvara.
#[must_use]
pub fn prikaz(projekt: &Projekt, u: &UredjivanjeStila) -> Option<PrikazStila> {
    let sloj = projekt.sloj(u.sloj)?;
    let (naziv, stil) = match u.ciljni_objekt() {
        Some(o) => {
            let objekt = sloj.objekti.get(&o)?;
            (objekt.naziv.clone(), objekt.stil.na_stilu_sloja(&sloj.stil))
        }
        None => (sloj.naziv.clone(), sloj.stil),
    };
    Some(PrikazStila {
        naziv,
        vrsta: sloj.vrsta,
        stil,
        uredivo: sloj.stanje.smije_mijenjati(),
    })
}

/// Izvršava izmjenu nad slojem ili objektom. Dok miš drži klizač, sve je jedan korak poništavanja.
pub fn izvrsi(uredjivac: &mut Uredjivac, u: &mut UredjivanjeStila, izmjena: IzmjenaStila) -> bool {
    let Some(sloj) = uredjivac.projekt().sloj(u.sloj) else {
        return false;
    };
    let naredba: Naredba = match u.ciljni_objekt() {
        Some(objekt) => {
            let Some(o) = sloj.objekti.get(&objekt) else {
                return false;
            };
            NaredbaObjekta::Izmijeni {
                sloj: u.sloj,
                objekt,
                izmjena: IzmjenaObjekta::Stil(primijeni_na_objekt(o.stil, &sloj.stil, izmjena)),
            }
            .into()
        }
        None => NaredbaStabla::PostaviStil {
            sloj: u.sloj,
            stil: primijeni(sloj.stil, izmjena),
        }
        .into(),
    };
    posalji(uredjivac, u, naredba)
}

/// Sloj dobiva zadani stil, a objekt opet uzima stil sloja.
pub fn vrati_zadano(uredjivac: &mut Uredjivac, u: &mut UredjivanjeStila) -> bool {
    u.potez = false;
    let naredba: Naredba = match u.ciljni_objekt() {
        Some(objekt) => NaredbaObjekta::Izmijeni {
            sloj: u.sloj,
            objekt,
            izmjena: IzmjenaObjekta::Stil(StilObjekta::default()),
        }
        .into(),
        None => NaredbaStabla::PostaviStil {
            sloj: u.sloj,
            stil: Stil::default(),
        }
        .into(),
    };
    posalji(uredjivac, u, naredba)
}

fn posalji(uredjivac: &mut Uredjivac, u: &mut UredjivanjeStila, naredba: Naredba) -> bool {
    let ishod = if u.potez {
        uredjivac.izvrsi_uz_zadnji(naredba)
    } else {
        uredjivac.izvrsi(naredba)
    };
    u.potez = ishod.is_ok();
    ishod.is_ok()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Radnja {
    Natrag,
    Zatvori,
    VratiZadano,
}

/// Crta ploču Stil u okviru ploče Slojevi.
pub fn crtaj_plocu_stila(
    ui: &mut egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    okvir: Rect,
    boje: &Boje,
) {
    let Some(mut u) = stanje.stil else {
        return;
    };
    if !ui.input(|i| i.pointer.primary_down()) {
        u.potez = false;
    }
    let Some(p) = prikaz(uredjivac.projekt(), &u) else {
        stanje.stil = None;
        return;
    };
    let tamna = stanje.tema_tamna;
    let mut radnja = zaglavlje(ui, ikone, katalog, okvir, &p.naziv, tamna, boje);
    let mut y = okvir.min.y + 66.0;
    if u.objekt.is_some() {
        izbor_cilja(
            ui,
            ikone,
            katalog,
            &mut u,
            pos2(okvir.min.x + 16.0, y),
            tamna,
            boje,
        );
        y += 52.0;
    }
    if p.uredivo {
        if let Some(izmjena) = kontrole(ui, katalog, &p, okvir, y, boje) {
            let _ = izvrsi(uredjivac, &mut u, izmjena);
        }
        radnja = radnja.or(podnozje(ui, katalog, okvir, boje));
    } else {
        napomena(ui, katalog, "stil-zakljucano", okvir, y, boje);
    }
    obradi_radnju(uredjivac, stanje, u, radnja);
}

fn obradi_radnju(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    mut u: UredjivanjeStila,
    radnja: Option<Radnja>,
) {
    match radnja {
        Some(Radnja::Natrag) => stanje.stil = None,
        Some(Radnja::Zatvori) => {
            stanje.stil = None;
            stanje.ploca_otvorena = false;
        }
        Some(Radnja::VratiZadano) => {
            let _ = vrati_zadano(uredjivac, &mut u);
            stanje.stil = Some(u);
        }
        None => stanje.stil = Some(u),
    }
}

fn zaglavlje(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    okvir: Rect,
    naziv: &str,
    tamna: bool,
    boje: &Boje,
) -> Option<Radnja> {
    let (px, py, pw) = (okvir.min.x, okvir.min.y, okvir.width());
    let natrag = OpisGumbaIkone {
        okvir: Rect::from_min_size(pos2(px + 10.0, py + 11.0), vec2(24.0, 24.0)),
        ikona: "natrag",
        stanje: "obicno",
        crtez: 20,
        obrubljen: false,
        aktivno: false,
        labela: &katalog.tekst("stil-natrag"),
    };
    let natrag = gumb_ikona(ui, ikone, &natrag, tamna, boje).clicked();
    tekst_lijevo(
        ui,
        pos2(px + 42.0, py + 23.0),
        &katalog.s_tekstovima("stil-naslov", &[("naziv", naziv.to_owned())]),
        font_srednji(18.0),
        boje.tekst,
    );
    let zatvori = OpisGumbaIkone {
        okvir: Rect::from_min_size(pos2(px + pw - 36.0, py + 11.0), vec2(24.0, 24.0)),
        ikona: "x",
        stanje: "obicno",
        crtez: 20,
        obrubljen: false,
        aktivno: false,
        labela: &katalog.tekst("stil-zatvori"),
    };
    let zatvori = gumb_ikona(ui, ikone, &zatvori, tamna, boje).clicked();
    ui.painter().line_segment(
        [pos2(px, py + 44.0), pos2(px + pw, py + 44.0)],
        Stroke::new(1.0, boje.obrub),
    );
    if natrag {
        Some(Radnja::Natrag)
    } else {
        zatvori.then_some(Radnja::Zatvori)
    }
}

/// „Cijeli sloj” ili „Samo ovaj objekt”, kad je ploča otvorena iz retka objekta.
fn izbor_cilja(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    u: &mut UredjivanjeStila,
    pocetak: Pos2,
    tamna: bool,
    boje: &Boje,
) {
    let sloj = Rect::from_min_size(pocetak, vec2(112.0, 30.0));
    let objekt = Rect::from_min_size(pos2(sloj.max.x + 8.0, pocetak.y), vec2(144.0, 30.0));
    let tekst_sloj = katalog.tekst("stil-cijeli-sloj");
    let tekst_objekt = katalog.tekst("stil-samo-objekt");
    if cip(ui, ikone, sloj, &tekst_sloj, !u.samo_objekt, tamna, boje).clicked() {
        u.samo_objekt = false;
        u.potez = false;
    }
    if cip(ui, ikone, objekt, &tekst_objekt, u.samo_objekt, tamna, boje).clicked() {
        u.samo_objekt = true;
        u.potez = false;
    }
}

/// Jedan klizač u ploči: naslov lijevo, vrijednost desno i staza ispod.
struct RedKlizaca {
    kljuc: &'static str,
    vrijednost: f64,
    najmanje: f64,
    najvise: f64,
    korak: f64,
    prikaz: String,
    izmjena: fn(f64) -> IzmjenaStila,
}

fn mm(v: f64) -> String {
    format!("{} mm", broj(v, 1))
}

fn posto(p: f64) -> String {
    postotak((p * 100.0).round() as u32)
}

fn klizaci(vrsta: VrstaGeometrije, s: &Stil) -> Vec<RedKlizaca> {
    let debljina = |kljuc| RedKlizaca {
        kljuc,
        vrijednost: s.debljina_mm,
        najmanje: 0.1,
        najvise: 5.0,
        korak: 0.1,
        prikaz: mm(s.debljina_mm),
        izmjena: IzmjenaStila::Debljina,
    };
    match vrsta {
        VrstaGeometrije::Tocka => vec![RedKlizaca {
            kljuc: "stil-velicina",
            vrijednost: s.velicina_znaka_mm,
            najmanje: 1.0,
            najvise: 10.0,
            korak: 0.5,
            prikaz: mm(s.velicina_znaka_mm),
            izmjena: IzmjenaStila::Velicina,
        }],
        VrstaGeometrije::Linija => vec![
            debljina("stil-debljina"),
            RedKlizaca {
                kljuc: "stil-prozirnost",
                vrijednost: prozirnost(s.boja_crte) * 100.0,
                najmanje: 0.0,
                najvise: 90.0,
                korak: 1.0,
                prikaz: posto(prozirnost(s.boja_crte)),
                izmjena: |v| IzmjenaStila::Prozirnost(v / 100.0),
            },
        ],
        VrstaGeometrije::Poligon => vec![
            RedKlizaca {
                kljuc: "stil-prozirnost-ispune",
                vrijednost: prozirnost(s.boja_ispune) * 100.0,
                najmanje: 0.0,
                najvise: 100.0,
                korak: 1.0,
                prikaz: posto(prozirnost(s.boja_ispune)),
                izmjena: |v| IzmjenaStila::ProzirnostIspune(v / 100.0),
            },
            debljina("stil-debljina-ruba"),
        ],
    }
}

fn kontrole(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    p: &PrikazStila,
    okvir: Rect,
    y: f32,
    boje: &Boje,
) -> Option<IzmjenaStila> {
    let x = okvir.min.x + 16.0;
    let sirina = okvir.width() - 32.0;
    let mut izmjena = paleta(ui, katalog, p.stil.boja_crte, pos2(x, y), boje);
    let mut y = y + 72.0;
    for red in klizaci(p.vrsta, &p.stil) {
        izmjena = izmjena.or(red_klizaca(ui, katalog, &red, pos2(x, y), sirina, boje));
        y += 56.0;
    }
    izmjena
}

fn red_klizaca(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    red: &RedKlizaca,
    pocetak: Pos2,
    sirina: f32,
    boje: &Boje,
) -> Option<IzmjenaStila> {
    let naslov = katalog.tekst(red.kljuc);
    tekst_lijevo(ui, pocetak, &naslov, font_obicni(13.0), boje.tekst_2);
    tekst_desno(
        ui,
        pos2(pocetak.x + sirina, pocetak.y),
        &red.prikaz,
        font_obicni(13.0),
        boje.tekst,
    );
    let opis = OpisKlizaca {
        okvir: Rect::from_min_size(pos2(pocetak.x, pocetak.y + 12.0), vec2(sirina, 24.0)),
        vrijednost: red.vrijednost,
        najmanje: red.najmanje,
        najvise: red.najvise,
        korak: red.korak,
        labela: &naslov,
    };
    klizac(ui, &opis, boje).map(red.izmjena)
}

/// Osam ponuđenih boja i krug „+” za bilo koju drugu.
fn paleta(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    trenutna: Boja,
    pocetak: Pos2,
    boje: &Boje,
) -> Option<IzmjenaStila> {
    tekst_lijevo(
        ui,
        pocetak,
        &katalog.tekst("stil-boja"),
        font_obicni(13.0),
        boje.tekst_2,
    );
    let srediste = |i: usize| pos2(pocetak.x + 13.0 + i as f32 * 35.0, pocetak.y + 32.0);
    let mut izmjena = None;
    for (i, (ponudjena, kljuc)) in PONUDJENE_BOJE.iter().enumerate() {
        let odabrana = ista_boja(*ponudjena, trenutna);
        let labela = katalog.tekst(kljuc);
        if krug_boje(ui, srediste(i), *ponudjena, odabrana, &labela, boje).clicked() && !odabrana {
            izmjena = Some(IzmjenaStila::Boja(*ponudjena));
        }
    }
    let druga = druga_boja(ui, katalog, trenutna, srediste(PONUDJENE_BOJE.len()), boje);
    izmjena.or(druga)
}

const fn ista_boja(a: Boja, b: Boja) -> bool {
    a.r == b.r && a.g == b.g && a.b == b.b
}

fn krug_boje(
    ui: &mut egui::Ui,
    srediste: Pos2,
    boja: Boja,
    odabrana: bool,
    labela: &str,
    boje: &Boje,
) -> Response {
    let crtac = ui.painter();
    crtac.circle_filled(srediste, 11.0, Color32::from_rgb(boja.r, boja.g, boja.b));
    if odabrana {
        crtac.circle_stroke(srediste, 14.5, Stroke::new(2.0, boje.tekst));
    }
    let okvir = Rect::from_center_size(srediste, vec2(30.0, 30.0));
    let id = ui.make_persistent_id(("boja", labela));
    let odgovor = ui.interact(okvir, id, Sense::click());
    odgovor.widget_info(|| WidgetInfo::selected(Role::Button, ui.is_enabled(), odabrana, labela));
    odgovor
}

/// Krug „+” otvara birač boje. Svaki pomak u biraču odmah boji kartu.
fn druga_boja(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    trenutna: Boja,
    srediste: Pos2,
    boje: &Boje,
) -> Option<IzmjenaStila> {
    let crtac = ui.painter();
    crtac.circle_stroke(srediste, 11.0, Stroke::new(1.0, boje.obrub_2));
    tekst_sredina(ui, srediste, "+", font_srednji(16.0), boje.tekst_2);
    let labela = katalog.tekst("stil-druga-boja");
    let id = ui.make_persistent_id("druga_boja");
    let odgovor = ui.interact(
        Rect::from_center_size(srediste, vec2(30.0, 30.0)),
        id,
        Sense::click(),
    );
    odgovor.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &labela));
    egui::Popup::from_toggle_button_response(&odgovor)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| birac_boje(ui, katalog, trenutna, boje))
        .and_then(|o| o.inner)
        .map(IzmjenaStila::Boja)
}

/// Nijansa, zasićenost i svjetlina, istim klizačima kao u ploči.
fn birac_boje(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    trenutna: Boja,
    boje: &Boje,
) -> Option<Boja> {
    let hsv = Hsva::from_srgb([trenutna.r, trenutna.g, trenutna.b]);
    let redovi = [
        ("boja-nijansa", f64::from(hsv.h) * 360.0, 360.0),
        ("boja-zasicenost", f64::from(hsv.s) * 100.0, 100.0),
        ("boja-svjetlina", f64::from(hsv.v) * 100.0, 100.0),
    ];
    let (polje, _) = ui.allocate_exact_size(vec2(240.0, 28.0), Sense::hover());
    ui.painter().rect_filled(
        polje,
        6.0,
        Color32::from_rgb(trenutna.r, trenutna.g, trenutna.b),
    );
    let mut nove = [hsv.h, hsv.s, hsv.v];
    let mut izmijenjena = false;
    for (i, (kljuc, vrijednost, najvise)) in redovi.into_iter().enumerate() {
        let (okvir, _) = ui.allocate_exact_size(vec2(240.0, 44.0), Sense::hover());
        let naslov = katalog.tekst(kljuc);
        tekst_lijevo(
            ui,
            pos2(okvir.min.x, okvir.min.y + 10.0),
            &naslov,
            font_obicni(12.0),
            boje.tekst_2,
        );
        let opis = OpisKlizaca {
            okvir: Rect::from_min_size(pos2(okvir.min.x, okvir.min.y + 18.0), vec2(240.0, 24.0)),
            vrijednost,
            najmanje: 0.0,
            najvise,
            korak: 1.0,
            labela: &naslov,
        };
        if let Some(v) = klizac(ui, &opis, boje) {
            nove[i] = (v / najvise) as f32;
            izmijenjena = true;
        }
    }
    let [r, g, b] = Hsva::new(nove[0], nove[1], nove[2], 1.0).to_srgb();
    izmijenjena.then_some(rgb(r, g, b))
}

fn napomena(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    kljuc: &str,
    okvir: Rect,
    y: f32,
    boje: &Boje,
) {
    let galija = ui.painter().layout(
        katalog.tekst(kljuc),
        font_obicni(12.0),
        boje.tekst_2,
        okvir.width() - 76.0,
    );
    let visina = galija.size().y + 20.0;
    let polje = Rect::from_min_size(
        pos2(okvir.min.x + 16.0, y),
        vec2(okvir.width() - 32.0, visina),
    );
    zaobljeni_okvir(ui, polje, boje.ploca_2, None, 10.0);
    ui.painter().circle_stroke(
        pos2(polje.min.x + 18.0, polje.center().y),
        7.0,
        Stroke::new(1.2, boje.tekst_2),
    );
    tekst_sredina(
        ui,
        pos2(polje.min.x + 18.0, polje.center().y),
        "i",
        font_srednji(10.0),
        boje.tekst_2,
    );
    ui.painter().galley(
        pos2(polje.min.x + 38.0, polje.min.y + 10.0),
        galija,
        boje.tekst_2,
    );
}

/// Napomena o QGIS-u, „Vrati zadano” i „Gotovo”, uz dno ploče.
fn podnozje(
    ui: &mut egui::Ui,
    katalog: &KatalogPoruka,
    okvir: Rect,
    boje: &Boje,
) -> Option<Radnja> {
    napomena(
        ui,
        katalog,
        "stil-napomena",
        okvir,
        okvir.max.y - 116.0,
        boje,
    );
    let y = okvir.max.y - 52.0;
    let vrati = Rect::from_min_size(pos2(okvir.min.x + 12.0, y), vec2(116.0, 36.0));
    let gotovo = Rect::from_min_size(pos2(okvir.max.x - 108.0, y), vec2(92.0, 36.0));
    let tekst_vrati = katalog.tekst("stil-vrati-zadano");
    tekst_sredina(
        ui,
        vrati.center(),
        &tekst_vrati,
        font_srednji(13.0),
        boje.primarna_tamna,
    );
    let vrati = gumb_teksta(ui, vrati, &tekst_vrati);
    let tekst_gotovo = katalog.tekst("stil-gotovo");
    zaobljeni_okvir(ui, gotovo, boje.alat_odabran, None, 18.0);
    tekst_sredina(
        ui,
        gotovo.center(),
        &tekst_gotovo,
        font_srednji(13.0),
        boje.alat_odabran_tekst,
    );
    if gumb_teksta(ui, gotovo, &tekst_gotovo).clicked() {
        Some(Radnja::Natrag)
    } else {
        vrati.clicked().then_some(Radnja::VratiZadano)
    }
}

fn gumb_teksta(ui: &mut egui::Ui, okvir: Rect, labela: &str) -> Response {
    let odgovor = ui.interact(
        okvir,
        ui.make_persistent_id(("stil_gumb", labela)),
        Sense::click(),
    );
    odgovor.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), labela));
    odgovor
}
