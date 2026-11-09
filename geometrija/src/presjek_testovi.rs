use super::*;

fn p(a: [f64; 2], b: [f64; 2]) -> [Tocka; 2] {
    [a, b]
}

/// Spora inačica za usporedbu: svaki rub sa svakim.
fn spora<P: AsRef<[Tocka]>>(prstenovi: &[P]) -> bool {
    let prstenovi = prstenovi_lika(prstenovi);
    if prstenovi.iter().any(|p| povratak(p).is_some()) {
        return true;
    }
    let r = rubovi(&prstenovi);
    let mut dodiri = DodiriPrstenova::default();
    (0..r.len())
        .any(|i| (i + 1..r.len()).any(|j| presjek_dvaju(&r[i], &r[j], &mut dodiri).is_some()))
}

#[test]
fn dodir_dviju_duzina() {
    // Križanje u sredini, s točnim mjestom.
    assert_eq!(
        dodir(p([0., 0.], [2., 2.]), p([0., 2.], [2., 0.])),
        Some(Dodir::Krizanje([1., 1.]))
    );
    assert_eq!(
        dodir(p([0., 0.], [4., 0.]), p([1., -1.], [1., 3.])),
        Some(Dodir::Krizanje([1., 0.]))
    );
    // Odvojene i usporedne bez dodira.
    assert_eq!(dodir(p([0., 0.], [1., 0.]), p([0., 1.], [1., 1.])), None);
    assert_eq!(dodir(p([0., 0.], [1., 1.]), p([3., 0.], [2., 1.])), None);
    // Pravci se sijeku, ali izvan dužina.
    assert_eq!(dodir(p([0., 0.], [1., 0.]), p([2., -1.], [2., 1.])), None);
    assert_eq!(dodir(p([0., 0.], [1., 1.]), p([0., 3.], [3., 0.])), None);
    // Kraj jedne dužine na drugoj, sa svake strane i svakim krajem.
    assert_eq!(
        dodir(p([1., 0.], [1., 5.]), p([0., 0.], [2., 0.])),
        Some(Dodir::Kraj([1., 0.]))
    );
    assert_eq!(
        dodir(p([1., 5.], [1., 0.]), p([0., 0.], [2., 0.])),
        Some(Dodir::Kraj([1., 0.]))
    );
    assert_eq!(
        dodir(p([0., 0.], [2., 0.]), p([1., 0.], [1., 5.])),
        Some(Dodir::Kraj([1., 0.]))
    );
    assert_eq!(
        dodir(p([0., 0.], [2., 0.]), p([1., 5.], [1., 0.])),
        Some(Dodir::Kraj([1., 0.]))
    );
    // Na pravcu druge dužine, ali izvan nje.
    assert_eq!(dodir(p([3., 0.], [3., 5.]), p([0., 0.], [2., 0.])), None);
    assert_eq!(dodir(p([0., 3.], [5., 3.]), p([0., 0.], [0., 2.])), None);
}

#[test]
fn dodir_na_istom_pravcu() {
    // Vodoravno: preklapanje, dodir krajem, razmak.
    assert_eq!(
        dodir(p([0., 0.], [2., 0.]), p([1., 0.], [3., 0.])),
        Some(Dodir::Preklapanje([1., 0.]))
    );
    assert_eq!(
        dodir(p([2., 0.], [0., 0.]), p([3., 0.], [1., 0.])),
        Some(Dodir::Preklapanje([1., 0.]))
    );
    assert_eq!(
        dodir(p([0., 0.], [1., 0.]), p([1., 0.], [2., 0.])),
        Some(Dodir::Kraj([1., 0.]))
    );
    assert_eq!(dodir(p([0., 0.], [1., 0.]), p([2., 0.], [3., 0.])), None);
    // Okomito, gdje redoslijed daje y.
    assert_eq!(
        dodir(p([0., 0.], [0., 2.]), p([0., 1.], [0., 3.])),
        Some(Dodir::Preklapanje([0., 1.]))
    );
    assert_eq!(
        dodir(p([0., 0.], [0., 1.]), p([0., 1.], [0., 2.])),
        Some(Dodir::Kraj([0., 1.]))
    );
    assert_eq!(dodir(p([0., 0.], [0., 1.]), p([0., 2.], [0., 3.])), None);
    // Koso, i jedna unutar druge.
    assert_eq!(
        dodir(p([0., 0.], [3., 3.]), p([1., 1.], [2., 2.])),
        Some(Dodir::Preklapanje([1., 1.]))
    );
}

#[test]
fn unutar_i_polozaj() {
    assert!(unutar([0., 0.], [2., 0.], [0., 0.]));
    assert!(unutar([0., 0.], [2., 0.], [2., 0.]));
    assert!(!unutar([0., 0.], [2., 0.], [2.5, 0.]));
    assert!(!unutar([0., 0.], [2., 0.], [-0.5, 0.]));
    assert!(unutar([0., 0.], [0., 2.], [0., 2.]));
    assert!(unutar([0., 0.], [0., 2.], [0., 0.]));
    assert!(!unutar([0., 0.], [0., 2.], [0., 2.5]));
    assert!(!unutar([0., 0.], [0., 2.], [0., -0.5]));
    assert_eq!(polozaj([0., 0.], [0., 2.], [5., 7.]), 7.);
    assert_eq!(polozaj([0., 0.], [1., 2.], [5., 7.]), 5.);
}

#[test]
fn ocisti_mice_ponovljene_i_zatvarajucu() {
    let a = [0., 0.];
    let b = [1., 0.];
    let c = [1., 1.];
    assert_eq!(ocisti(&[a, a, b, b, c, a, a]), vec![a, b, c]);
    assert_eq!(ocisti(&[a, b, c]), vec![a, b, c]);
    assert_eq!(ocisti(&[a]), vec![a]);
    assert_eq!(ocisti(&[a, a]), vec![a]);
    assert!(ocisti(&[]).is_empty());
}

#[test]
fn povratak_po_istom_rubu() {
    // Šiljak: iz (2,0) natrag prema (1,0).
    assert_eq!(
        povratak(&[[0., 0.], [2., 0.], [1., 0.], [1., 1.]]),
        Some([2., 0.])
    );
    // Ravan vrh na istom pravcu, ali naprijed, nije povratak.
    assert_eq!(povratak(&[[0., 0.], [1., 0.], [2., 0.], [1., 1.]]), None);
    // Šiljak u prvom vrhu, gdje prethodnik je zadnji vrh.
    assert_eq!(
        povratak(&[[1., 0.], [0., 1.], [0., 0.], [2., 0.]]),
        Some([2., 0.])
    );
    assert_eq!(povratak(&[[0., 0.], [1., 0.], [0., 1.]]), None);
    // Povratak s druge strane: susjed iza je bliže od susjeda ispred.
    assert_eq!(
        povratak(&[[1., 0.], [2., 0.], [0., 0.], [1., 1.]]),
        Some([2., 0.])
    );
}

#[test]
fn osmica_i_obicni_likovi() {
    let kvadrat = [[0., 0.], [2., 0.], [2., 2.], [0., 2.]];
    assert_eq!(presjek_lika(&[kvadrat]), None);
    let mut zatvoren = kvadrat.to_vec();
    zatvoren.push(kvadrat[0]);
    assert_eq!(presjek_lika(&[zatvoren]), None);
    let mut obratno = kvadrat;
    obratno.reverse();
    assert_eq!(presjek_lika(&[obratno]), None);
    let osmica = [[0., 0.], [2., 2.], [2., 0.], [0., 2.]];
    assert_eq!(presjek_lika(&[osmica]), Some([1., 1.]));
    // Ponovljena ista točka zaredom nije presjek.
    let ponovljena = [[0., 0.], [2., 0.], [2., 0.], [2., 2.], [0., 2.]];
    assert_eq!(presjek_lika(&[ponovljena]), None);
    // Manje od tri različite točke još nije lik.
    assert_eq!(presjek_lika(&[[[0., 0.], [1., 0.], [0., 0.]]]), None);
    assert_eq!(presjek_lika::<[Tocka; 0]>(&[]), None);
}

#[test]
fn prsten_koji_dotice_sam_sebe() {
    // Vrh (2,0) leži na rubu (0,0)–(4,0), koji nije njegov susjed.
    let dodir_vrhom = [[0., 0.], [4., 0.], [4., 4.], [2., 0.1], [2., 0.], [0., 4.]];
    assert_eq!(presjek_lika(&[dodir_vrhom]), Some([2., 0.]));
    let na_rubu = [[0., 0.], [4., 0.], [4., 4.], [2., 0.], [0., 4.]];
    assert_eq!(presjek_lika(&[na_rubu]), Some([2., 0.]));
    // Isti vrh dvaput, ne zaredom: prsten se dotiče u točki.
    let dvaput = [[0., 0.], [2., 2.], [4., 0.], [4., 4.], [2., 2.], [0., 4.]];
    assert_eq!(presjek_lika(&[dvaput]), Some([2., 2.]));
    // Dva ruba istog prstena na istom pravcu.
    let preklop = [
        [0., 0.],
        [4., 0.],
        [4., 2.],
        [3., 2.],
        [3., 0.],
        [1., 0.],
        [1., 2.],
        [0., 2.],
    ];
    assert!(presjek_lika(&[preklop]).is_some());
    // Šiljak se vraća po istom rubu.
    let siljak = [[0., 0.], [4., 0.], [2., 0.], [2., 2.]];
    assert_eq!(presjek_lika(&[siljak]), Some([4., 0.]));
}

#[test]
fn rupe() {
    let vanjski = [[0., 0.], [10., 0.], [10., 10.], [0., 10.]];
    let unutra = [[2., 2.], [4., 2.], [4., 4.], [2., 4.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &unutra[..]]), None);
    // Rupa koja prelazi vanjski rub.
    let preko = [[8., 2.], [12., 2.], [12., 4.], [8., 4.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &preko[..]]), Some([10., 2.]));
    // Rupa koja dotiče vanjski rub u jednoj točki je dopuštena, kao u GEOS-u.
    let dotice = [[5., 0.], [6., 2.], [4., 2.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &dotice[..]]), None);
    // Rupa s rubom na vanjskom rubu.
    let na_rubu = [[2., 0.], [4., 0.], [4., 2.], [2., 2.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &na_rubu[..]]), Some([2., 0.]));
    // Vanjski rub prolazi kroz dva vrha rupe i dijeli je napola (nađeno usporedbom s GEOS-om).
    let kroz_vrhove = [[3., 1.], [3.5, 1.], [3.5, 1.5], [3., 1.5]];
    let trokut = [[5., 3.], [4., 1.], [2., 0.]];
    let t = presjek_lika(&[&trokut[..], &kroz_vrhove[..]]);
    assert!(t == Some([3., 1.]) || t == Some([3.5, 1.5]), "{t:?}");
    // Dvije rupe koje se dotiču u istoj točki, i rupa koja dotiče vanjski rub u istoj točki
    // preko dvaju svojih rubova: to je jedno mjesto dodira, ne dva.
    let a = [[2., 2.], [4., 2.], [4., 4.], [2., 4.]];
    let b = [[4., 4.], [6., 4.], [6., 6.], [4., 6.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &a[..], &b[..]]), None);
    // Dvije rupe koje se križaju.
    let druga = [[3., 3.], [5., 3.], [5., 5.], [3., 5.]];
    let t = presjek_lika(&[&vanjski[..], &unutra[..], &druga[..]]);
    assert!(t == Some([4., 3.]) || t == Some([3., 4.]), "{t:?}");
    // Rupa s osmicom.
    let osmica = [[2., 2.], [4., 4.], [4., 2.], [2., 4.]];
    assert_eq!(presjek_lika(&[&vanjski[..], &osmica[..]]), Some([3., 3.]));
}

#[test]
fn trokut_i_cetverokut_susjedi() {
    let r = rubovi(&[vec![[0., 0.], [1., 0.], [0., 1.]]]);
    assert!(
        r.iter()
            .all(|a| r.iter().filter(|b| b.redni != a.redni).all(|b| a.susjed(b)))
    );
    let r = rubovi(&[
        vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
        vec![[0., 0.], [1., 0.], [1., 1.]],
    ]);
    assert!(r[0].susjed(&r[1]) && r[0].susjed(&r[3]) && !r[0].susjed(&r[2]));
    assert!(
        !r[0].susjed(&r[4]),
        "rubovi različitih prstenova nisu susjedi"
    );
    assert_eq!(r[3].krajevi, [[0., 1.], [0., 0.]]);
    assert_eq!(r[1].x_raspon(), [1., 1.]);
    assert_eq!(r[0].x_raspon(), [0., 1.]);
}

#[test]
fn neispravan_broj_nije_presjek() {
    let nan = [[0., 0.], [f64::NAN, 0.], [2., 2.], [0., 2.]];
    assert_eq!(presjek_lika(&[nan]), None);
}

/// Jednostavan generator sa stalnim sjemenom, bez vanjskih paketa.
struct Lcg(u64);

impl Lcg {
    fn broj(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }
}

fn nasumican_lik(g: &mut Lcg) -> Vec<Vec<Tocka>> {
    // Male cijele koordinate daju mnogo dodira, preklapanja i ponovljenih točaka.
    let mreza = g.broj(2) == 0;
    let tocka = |g: &mut Lcg| {
        if mreza {
            [g.broj(6) as f64, g.broj(6) as f64]
        } else {
            [g.broj(10_000) as f64 / 997.0, g.broj(10_000) as f64 / 991.0]
        }
    };
    (0..1 + g.broj(3))
        .map(|_| (0..3 + g.broj(8)).map(|_| tocka(g)).collect())
        .collect()
}

#[test]
fn brza_i_spora_se_slazu_na_10000_likova() {
    let mut g = Lcg(20_261_109);
    let (mut s, mut bez) = (0, 0);
    for _ in 0..10_000 {
        let lik = nasumican_lik(&mut g);
        let brza = presjek_lika(&lik).is_some();
        assert_eq!(brza, spora(&lik), "{lik:?}");
        if brza { s += 1 } else { bez += 1 }
    }
    assert!(s > 500 && bez > 500, "{s} s presjekom, {bez} bez");
}

/// Krug od mnogo vrhova bez presjeka, i isti s jednim prekriženim parom vrhova.
#[test]
fn veliki_krug_s_jednim_krizanjem() {
    // Dovoljno velik da sporiji račun ne prođe neopaženo, a dovoljno malen da bez optimizacije
    // prevođenja ne istekne ni kad brza pretraga zakaže. Brzinu mjeri `examples/brzina_presjeka.rs`.
    let n = 4_000;
    let krug: Vec<Tocka> = (0..n)
        .map(|i| {
            let k = i as f64 / n as f64 * std::f64::consts::TAU;
            [18.0 + k.cos(), 45.0 + k.sin()]
        })
        .collect();
    assert_eq!(presjek_lika(&[&krug]), None);
    let mut krivi = krug.clone();
    krivi.swap(1_000, 1_001);
    assert!(presjek_lika(&[&krivi]).is_some());
}
