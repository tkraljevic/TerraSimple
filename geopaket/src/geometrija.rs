use crate::Greska;
use geozero::{
    CoordDimensions, GeomProcessor, GeozeroGeometry, ToWkb,
    error::GeozeroError,
    wkb::{GpkgWkb, WkbDialect},
};
use koordinate::{Pretvornik, Sustav};
use rusqlite::{Connection, functions::FunctionFlags};
use slojevi::{Geometrija, Vrh};

#[derive(Default)]
struct Citac {
    prstenovi: Vec<Vec<Vrh>>,
    vrsta: u32,
}
impl GeomProcessor for Citac {
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions::xyz()
    }
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> geozero::error::Result<()> {
        self.coordinate(x, y, None, None, None, None, 0)
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> geozero::error::Result<()> {
        let p = self
            .prstenovi
            .last_mut()
            .ok_or_else(|| GeozeroError::Geometry("Nema početka geometrije.".into()))?;
        p.push(Vrh {
            duzina: x,
            sirina: y,
            visina: z.filter(|v| v.is_finite()),
        });
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> geozero::error::Result<()> {
        self.vrsta = 1;
        self.prstenovi.push(Vec::new());
        Ok(())
    }
    fn linestring_begin(
        &mut self,
        tagged: bool,
        _size: usize,
        _idx: usize,
    ) -> geozero::error::Result<()> {
        if tagged {
            self.vrsta = 2;
        }
        self.prstenovi.push(Vec::new());
        Ok(())
    }
    fn polygon_begin(
        &mut self,
        _tagged: bool,
        _size: usize,
        _idx: usize,
    ) -> geozero::error::Result<()> {
        self.vrsta = 3;
        Ok(())
    }
}
fn velicina_omotnice(flag: u8) -> Result<usize, Greska> {
    match (flag >> 1) & 7 {
        0 => Ok(0),
        1 => Ok(32),
        2 | 3 => Ok(48),
        4 => Ok(64),
        _ => Err("Omotnica nije valjana.".into()),
    }
}
fn pocetak(b: &[u8]) -> Result<usize, Greska> {
    if b.get(..2) != Some(b"GP") {
        return Err("Geometrija nema GeoPackage zaglavlje.".into());
    }
    Ok(8 + velicina_omotnice(*b.get(3).ok_or("Zaglavlje je odrezano.")?)?)
}
fn tip_bajtovi(b: &[u8], poc: usize) -> Result<[u8; 4], Greska> {
    let s = b
        .get(poc + 1..poc + 5)
        .ok_or("WKB zaglavlje je odrezano.")?;
    Ok([s[0], s[1], s[2], s[3]])
}
fn tip(b: &[u8]) -> Result<u32, Greska> {
    let poc = pocetak(b)?;
    let endian = *b.get(poc).ok_or("Nema WKB zaglavlja.")?;
    let a = tip_bajtovi(b, poc)?;
    match endian {
        0 => Ok(u32::from_be_bytes(a)),
        1 => Ok(u32::from_le_bytes(a)),
        _ => Err("Redoslijed bajtova nije valjan.".into()),
    }
}

fn sirovo(b: &[u8]) -> Result<Citac, Greska> {
    let v = tip(b)?;
    if ![1, 2, 3, 1001, 1002, 1003].contains(&v) {
        return Err("Vrsta geometrije nije podržana; izvorni zapis ostaje u datoteci.".into());
    }
    let mut c = Citac::default();
    GpkgWkb(b).process_geom(&mut c)?;
    Ok(c)
}
pub fn procitaj(b: &[u8], sustav: Sustav) -> Result<Geometrija, Greska> {
    let c = sirovo(b)?;
    if sustav == Sustav::Wgs84 {
        return sastavi(c.vrsta, c.prstenovi);
    }
    let p = Pretvornik::novi(sustav, Sustav::Wgs84)?;
    let prstenovi = c
        .prstenovi
        .into_iter()
        .map(|v| v.into_iter().map(|v| pretvori_vrh(v, &p)).collect())
        .collect::<Result<Vec<_>, Greska>>()?;
    sastavi(c.vrsta, prstenovi)
}
fn sastavi(v: u32, mut p: Vec<Vec<Vrh>>) -> Result<Geometrija, Greska> {
    match v {
        1 => Ok(Geometrija::Tocka(
            *p.first()
                .and_then(|r| r.first())
                .ok_or("Točka je prazna.")?,
        )),
        2 => Ok(Geometrija::Linija(p.pop().ok_or("Linija je prazna.")?)),
        3 => Ok(Geometrija::Poligon(p)),
        _ => Err("Nije zabilježena podržana vrsta geometrije.".into()),
    }
}
fn pretvori_vrh(mut v: Vrh, p: &Pretvornik) -> Result<Vrh, Greska> {
    let t = p.pretvori([v.duzina, v.sirina])?;
    v.duzina = t[0];
    v.sirina = t[1];
    Ok(v)
}
struct Pisac<'a> {
    g: &'a Geometrija,
    p: Option<Pretvornik>,
}
fn vrhovi(g: &Geometrija) -> Vec<&[Vrh]> {
    match g {
        Geometrija::Tocka(v) => vec![std::slice::from_ref(v)],
        Geometrija::Linija(v) => vec![v],
        Geometrija::Poligon(p) => p.iter().map(Vec::as_slice).collect(),
    }
}
impl GeozeroGeometry for Pisac<'_> {
    fn process_geom<P: GeomProcessor>(&self, p: &mut P) -> geozero::error::Result<()> {
        match self.g {
            Geometrija::Tocka(v) => {
                p.point_begin(0)?;
                self.koordinata(p, *v, 0)?;
                p.point_end(0)
            }
            Geometrija::Linija(v) => self.crta(p, v, true, 0),
            Geometrija::Poligon(v) => self.poligon(p, v),
        }
    }
}
impl Pisac<'_> {
    fn poligon<P: GeomProcessor>(&self, p: &mut P, v: &[Vec<Vrh>]) -> geozero::error::Result<()> {
        p.polygon_begin(true, v.len(), 0)?;
        for (i, r) in v.iter().enumerate() {
            self.crta(p, r, false, i)?;
        }
        p.polygon_end(true, 0)
    }

    fn koordinata<P: GeomProcessor>(
        &self,
        p: &mut P,
        v: Vrh,
        i: usize,
    ) -> geozero::error::Result<()> {
        let t = match &self.p {
            Some(pretvornik) => pretvornik
                .pretvori([v.duzina, v.sirina])
                .map_err(|e| GeozeroError::Geometry(e.to_string()))?,
            None => [v.duzina, v.sirina],
        };
        if p.multi_dim() {
            p.coordinate(t[0], t[1], v.visina, None, None, None, i)
        } else {
            p.xy(t[0], t[1], i)
        }
    }
    fn crta<P: GeomProcessor>(
        &self,
        p: &mut P,
        v: &[Vrh],
        tagged: bool,
        i: usize,
    ) -> geozero::error::Result<()> {
        let zatvori = !tagged && v.first() != v.last();
        p.linestring_begin(tagged, v.len() + usize::from(zatvori), i)?;
        for (n, t) in v.iter().enumerate() {
            self.koordinata(p, *t, n)?;
        }
        if let Some(t) = v.first().filter(|_| zatvori) {
            self.koordinata(p, *t, v.len())?;
        }
        p.linestring_end(tagged, i)
    }
}
fn pretvornik(s: Sustav) -> Result<Option<Pretvornik>, Greska> {
    if s == Sustav::Wgs84 {
        Ok(None)
    } else {
        Pretvornik::novi(Sustav::Wgs84, s)
            .map(Some)
            .map_err(Into::into)
    }
}
pub fn zapisi(g: &Geometrija, sustav: Sustav, srs: i32) -> Result<Vec<u8>, Greska> {
    let z = vrhovi(g)
        .iter()
        .flat_map(|v| v.iter())
        .any(|v| v.visina.is_some());
    let dims = if z {
        CoordDimensions::xyz()
    } else {
        CoordDimensions::xy()
    };
    let pisac = Pisac {
        g,
        p: pretvornik(sustav)?,
    };
    Ok(pisac.to_wkb_dialect(WkbDialect::Geopackage, dims, Some(srs), Vec::new())?)
}
fn obuhvat(b: &[u8], os: usize) -> Option<f64> {
    let c = sirovo(b).ok()?;
    c.prstenovi
        .iter()
        .flatten()
        .map(|v| [v.duzina, v.sirina][os % 2])
        .filter(|v| v.is_finite())
        .reduce(|a, b| if os < 2 { a.min(b) } else { a.max(b) })
}
pub fn funkcije(c: &Connection) -> Result<(), Greska> {
    let f = FunctionFlags::SQLITE_UTF8
        | FunctionFlags::SQLITE_DETERMINISTIC
        | FunctionFlags::SQLITE_INNOCUOUS;
    c.create_scalar_function("ST_IsEmpty", 1, f, |ctx| {
        Ok(ctx
            .get_raw(0)
            .as_blob()
            .ok()
            .map(|b| b.get(3).is_none_or(|v| v & 16 != 0)))
    })?;
    for (ime, os) in [
        ("ST_MinX", 0),
        ("ST_MinY", 1),
        ("ST_MaxX", 2),
        ("ST_MaxY", 3),
    ] {
        c.create_scalar_function(ime, 1, f, move |ctx| {
            Ok(ctx.get_raw(0).as_blob().ok().and_then(|b| obuhvat(b, os)))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod testovi {
    use super::*;
    fn v(x: f64, y: f64, z: Option<f64>) -> Vrh {
        Vrh {
            duzina: x,
            sirina: y,
            visina: z,
        }
    }
    #[test]
    fn svi_oblici_i_visine_prolaze_obje_dimezije() {
        for z in [None, Some(42.5)] {
            let a = v(18.0, 45.0, z);
            let b = v(18.1, 45.1, z);
            let c = v(18.2, 45.0, z);
            let g = [
                Geometrija::Tocka(a),
                Geometrija::Linija(vec![a, b]),
                Geometrija::Poligon(vec![vec![a, b, c, a], vec![a, b, c, a]]),
            ];
            for g in g {
                for sustav in [Sustav::Wgs84, Sustav::Htrs96Tm] {
                    let blob = zapisi(&g, sustav, sustav.epsg() as i32).unwrap();
                    let r = procitaj(&blob, sustav).unwrap();
                    assert_eq!(r.vrsta(), g.vrsta());
                    let o = vrhovi(&r);
                    assert_eq!(o[0][0].visina, z);
                    assert!((o[0][0].duzina - a.duzina).abs() < 1e-9);
                    assert!(obuhvat(&blob, 0).is_some());
                    assert!(obuhvat(&blob, 3).is_some());
                }
            }
        }
    }
    #[test]
    fn prazno_nepodrzano_i_odrezano_se_odbijaju() {
        let mut b = zapisi(&Geometrija::Tocka(v(18.0, 45.0, None)), Sustav::Wgs84, 4326).unwrap();
        for n in 0..b.len() {
            assert!(procitaj(&b[..n], Sustav::Wgs84).is_err());
        }
        let stari = b.clone();
        b[0] = 0;
        assert!(tip(&b).is_err());
        b = stari.clone();
        b[3] = 15;
        assert!(tip(&b).is_err());
        b = stari.clone();
        b[8] = 7;
        assert!(tip(&b).is_err());
        b = stari.clone();
        b[9] = 9;
        assert!(sirovo(&b).is_err());
        let mut p = Citac::default();
        assert!(p.xy(1.0, 2.0, 0).is_err());
        p.point_begin(0).unwrap();
        p.xy(1.0, 2.0, 0).unwrap();
        assert!(sastavi(1, vec![]).is_err());
        assert!(sastavi(2, vec![]).is_err());
        assert!(obuhvat(&[], 0).is_none());
        b = stari;
        b[3] |= 16;
        let n = f64::NAN.to_le_bytes();
        b[13..21].copy_from_slice(&n);
        b[21..29].copy_from_slice(&n);
        assert!(procitaj(&b, Sustav::Wgs84).is_err());
    }
    #[test]
    fn funkcije_rade_i_kad_gdal_nije_ucitan() {
        let c = Connection::open_in_memory().unwrap();
        funkcije(&c).unwrap();
        let b = zapisi(
            &Geometrija::Linija(vec![v(18., 45., None), v(19., 46., None)]),
            Sustav::Wgs84,
            4326,
        )
        .unwrap();
        let r: (bool, f64, f64, f64, f64) = c
            .query_row(
                "SELECT ST_IsEmpty(?1),ST_MinX(?1),ST_MinY(?1),ST_MaxX(?1),ST_MaxY(?1)",
                [b],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(r, (false, 18., 45., 19., 46.));
        let n: Option<f64> = c
            .query_row("SELECT ST_MinX(NULL)", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, None);
    }
}

#[cfg(test)]
mod prekidi {
    use super::*;
    struct Pad(usize);
    impl Pad {
        fn korak(&mut self) -> geozero::error::Result<()> {
            if self.0 == 0 {
                Err(GeozeroError::Geometry("Prekid izlaza.".into()))
            } else {
                self.0 -= 1;
                Ok(())
            }
        }
    }
    impl GeomProcessor for Pad {
        fn xy(&mut self, _x: f64, _y: f64, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
        fn point_begin(&mut self, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
        fn point_end(&mut self, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
        fn linestring_begin(
            &mut self,
            _t: bool,
            _n: usize,
            _i: usize,
        ) -> geozero::error::Result<()> {
            self.korak()
        }
        fn linestring_end(&mut self, _t: bool, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
        fn polygon_begin(&mut self, _t: bool, _n: usize, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
        fn polygon_end(&mut self, _t: bool, _i: usize) -> geozero::error::Result<()> {
            self.korak()
        }
    }
    #[test]
    fn prekidi_pisaca_i_nevaljane_koordinate() {
        let a = Vrh::novi(18., 45.);
        let b = Vrh::novi(19., 45.);
        let c = Vrh::novi(19., 46.);
        for g in [
            Geometrija::Tocka(a),
            Geometrija::Linija(vec![a, b]),
            Geometrija::Poligon(vec![vec![a, b, c]]),
        ] {
            let pisac = Pisac { g: &g, p: None };
            assert!(pisac.process_geom(&mut Pad(0)).is_err());
            assert!(pisac.process_geom(&mut Pad(30)).is_ok());
            let poziva = match g {
                Geometrija::Tocka(_) => 3,
                Geometrija::Linija(_) => 4,
                Geometrija::Poligon(_) => 8,
            };
            for n in 0..poziva {
                assert!(pisac.process_geom(&mut Pad(n)).is_err());
            }

            let b = zapisi(&g, Sustav::Wgs84, 4326).unwrap();
            assert!(procitaj(&b, Sustav::Wgs84).is_ok());
        }
        let los = Geometrija::Tocka(Vrh::novi(f64::NAN, 45.));
        assert!(zapisi(&los, Sustav::Htrs96Tm, 3765).is_err());
        let mut b = zapisi(&Geometrija::Tocka(a), Sustav::Wgs84, 4326).unwrap();
        b[13..21].copy_from_slice(&f64::INFINITY.to_le_bytes());
        assert!(procitaj(&b, Sustav::Htrs96Tm).is_err());
        for (flag, n) in [(0, 0), (2, 32), (4, 48), (6, 48), (8, 64)] {
            assert_eq!(velicina_omotnice(flag).unwrap(), n);
        }
        let b = [b'G', b'P', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        assert_eq!(tip(&b).unwrap(), 1);
    }
}

pub fn otvoreni(g: &Geometrija) -> Option<String> {
    match g {
        Geometrija::Poligon(p) => Some(
            serde_json::json!(p.iter().map(|r| r.first() != r.last()).collect::<Vec<_>>())
                .to_string(),
        ),
        _ => None,
    }
}
pub fn vrati_otvorene(g: &mut Geometrija, s: &str) -> Result<(), Greska> {
    let otvoreni: Vec<bool> = serde_json::from_str(s)?;
    if let Geometrija::Poligon(p) = g {
        for (r, otvoren) in p.iter_mut().zip(otvoreni) {
            otvori_prsten(r, otvoren);
        }
    }
    Ok(())
}
fn otvori_prsten(r: &mut Vec<Vrh>, otvoren: bool) {
    if otvoren && r.first() == r.last() {
        r.pop();
    }
}

#[cfg(test)]
mod prstenovi {
    use super::*;
    #[test]
    fn zastavice_ne_mijenjaju_tocku_a_pokvarene_se_odbijaju() {
        let mut g = Geometrija::Tocka(Vrh::novi(18., 45.));
        let prije = g.clone();
        vrati_otvorene(&mut g, "[]").unwrap();
        assert_eq!(g, prije);
        assert!(vrati_otvorene(&mut g, "krivo").is_err());
    }
}

pub fn ista(a: &Geometrija, b: &Geometrija) -> bool {
    match (a, b) {
        (Geometrija::Poligon(a), Geometrija::Poligon(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| bez_zatvaranja(a) == bez_zatvaranja(b))
        }
        _ => a == b,
    }
}
fn bez_zatvaranja(r: &[Vrh]) -> &[Vrh] {
    match r.split_last() {
        Some((z, ostali)) if r.first() == Some(z) => ostali,
        _ => r,
    }
}

#[cfg(test)]
mod ugovor_geometrije {
    use super::*;
    #[test]
    fn usporedba_prstenova_razlikuje_sadrzaj_a_ne_zatvaranje() {
        let a = Vrh::novi(18., 45.);
        let b = Vrh::novi(19., 45.);
        let c = Vrh::novi(19., 46.);
        let otvoren = Geometrija::Poligon(vec![vec![a, b, c]]);
        let zatvoren = Geometrija::Poligon(vec![vec![a, b, c, a]]);
        assert!(ista(&otvoren, &zatvoren));
        assert!(!ista(&otvoren, &Geometrija::Poligon(vec![vec![a, c, b]])));
        assert!(!ista(
            &otvoren,
            &Geometrija::Poligon(vec![vec![a, b, c], vec![a, b, c]])
        ));
        assert!(ista(
            &Geometrija::Poligon(vec![vec![]]),
            &Geometrija::Poligon(vec![vec![]])
        ));
        assert!(sastavi(0, vec![]).is_err());
    }
    #[test]
    fn sqlite_funkcije_rade_u_indeksu_i_nepouzdanom_okidacu() {
        let c = Connection::open_in_memory().unwrap();
        funkcije(&c).unwrap();
        c.execute_batch("PRAGMA trusted_schema=OFF; CREATE TABLE t(geom BLOB); CREATE INDEX ix ON t(ST_MinX(geom)); CREATE TABLE r(prazno INTEGER); CREATE TRIGGER pr AFTER INSERT ON t BEGIN INSERT INTO r VALUES(ST_IsEmpty(NEW.geom)); END;").unwrap();
        let b = zapisi(&Geometrija::Tocka(Vrh::novi(18., 45.)), Sustav::Wgs84, 4326).unwrap();
        c.execute("INSERT INTO t VALUES(?1)", [&b]).unwrap();
        assert_eq!(
            c.query_row("SELECT prazno FROM r", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        c.execute("INSERT INTO t VALUES(NULL)", []).unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM r WHERE prazno IS NULL", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        let mut prazno = b.clone();
        prazno[3] |= 16;
        c.execute("INSERT INTO t VALUES(?1)", [&prazno]).unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM r WHERE prazno=1", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
}
