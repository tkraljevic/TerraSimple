//! Biblioteka za probu čitanja KML i KMZ datoteka (`probe/kml`).
#![forbid(unsafe_code)]

pub mod citac;
pub mod model;
pub mod primjeri;
pub mod slojevi;
pub mod usporedba_kml;

#[cfg(test)]
mod testovi {
    use std::path::PathBuf;

    use crate::citac::{procitaj_datoteku, procitaj_kml_iz_teksta, GreskaCitaca};
    use crate::primjeri::{
        izradi_sve_primjere, provjeri_da_kvarovi_padaju, provjeri_stablo_sve_znacajke,
        provjeri_sve_primjere, sadrzaj_duboke_mape, sadrzaj_mijesani_slojevi, sadrzaj_ostecen_kml,
        sadrzaj_prazna_geometrija, sadrzaj_sve_znacajke,
    };
    use crate::slojevi::podijeli_na_slojeve;
    use crate::usporedba_kml::procitaj_s_kml_crateom;

    fn mapa_za_test() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("primjeri")
    }

    #[test]
    fn sve_znacajke_bez_gubitka() {
        let stablo = procitaj_kml_iz_teksta(&sadrzaj_sve_znacajke()).unwrap();
        provjeri_stablo_sve_znacajke(&stablo).unwrap();
    }

    #[test]
    fn duboke_mape_15_razina() {
        let stablo = procitaj_kml_iz_teksta(&sadrzaj_duboke_mape(15)).unwrap();
        assert_eq!(stablo.statistika.najveca_dubina_mapa, 15);
        assert_eq!(stablo.statistika.najveca_dubina_spremnika, 16);
        assert_eq!(stablo.statistika.objekti, 4);
    }

    #[test]
    fn podjela_na_slojeve_mijesana_mapa() {
        let stablo = procitaj_kml_iz_teksta(&sadrzaj_mijesani_slojevi()).unwrap();
        let izvjestaj = podijeli_na_slojeve(&stablo);
        assert_eq!(izvjestaj.ukupno_spremnika, 5);
        assert_eq!(izvjestaj.nepraznih_spremnika, 3);
        assert_eq!(izvjestaj.mijesanih_spremnika, 2);
        assert_eq!(izvjestaj.mijesanih_visegeometrija, 1);
        assert_eq!(izvjestaj.slojeva_tocaka, 3);
        assert_eq!(izvjestaj.slojeva_linija, 2);
        assert_eq!(izvjestaj.slojeva_poligona, 1);
        assert_eq!(izvjestaj.ukupno_slojeva, 6);
    }

    #[test]
    fn nepouzdane_datoteke_i_kmz() {
        let mapa = mapa_za_test();
        izradi_sve_primjere(&mapa).unwrap();
        let broj = provjeri_sve_primjere(&mapa).unwrap();
        assert_eq!(broj, 8);

        assert!(procitaj_kml_iz_teksta(&sadrzaj_ostecen_kml()).is_err());
        assert!(matches!(
            procitaj_kml_iz_teksta(&sadrzaj_prazna_geometrija()),
            Err(GreskaCitaca::PraznaGeometrija { .. })
        ));
    }

    #[test]
    fn kml_015_gubi_elemente_i_pada_na_praznoj_tocki() {
        let mapa = mapa_za_test();
        izradi_sve_primjere(&mapa).unwrap();

        // Na sve_znacajke.kml biblioteka kml 0.15 tiho gubi:
        // - CDATA opise na Document i Folder (0 umjesto 2 spremnika s opisom),
        // - gx:MultiTrack/gx:Track (1 objekt ostaje bez geometrije),
        // - NetworkLink unutar Folder (1 umjesto 2),
        // - oba GroundOverlay unutar Folder (0 umjesto 2),
        // - Style unutar NetworkLink i ugrađeni Style unutar Placemark (4 umjesto 6).
        let kml_stat = procitaj_s_kml_crateom(&mapa.join("sve_znacajke.kml")).unwrap();
        let nas_stat = procitaj_datoteku(&mapa.join("sve_znacajke.kml"))
            .unwrap()
            .statistika;
        assert_eq!(nas_stat.mape_s_opisom, 2);
        assert_eq!(kml_stat.mape_s_opisom, 0);
        assert_eq!(nas_stat.objekti_bez_geometrije, 0);
        assert_eq!(kml_stat.objekti_bez_geometrije, 1);
        assert_eq!(nas_stat.mrezne_veze, 2);
        assert_eq!(kml_stat.mrezne_veze, 1);
        assert_eq!(nas_stat.slikovni_prekrivaci, 2);
        assert_eq!(kml_stat.slikovni_prekrivaci, 0);
        assert_eq!(nas_stat.stilovi, 6);
        assert_eq!(kml_stat.stilovi, 4);

        // Na prazna_geometrija.kml biblioteka kml 0.15 pada u paniku (remove(0) na praznom Vec).
        let rez_prazna = procitaj_s_kml_crateom(&mapa.join("prazna_geometrija.kml"));
        assert_eq!(rez_prazna, Err("PANIKA_U_KML_CRATEU"));
    }

    #[test]
    fn namjerni_kvarovi_ruse_provjere() {
        let padovi = provjeri_da_kvarovi_padaju().unwrap();
        assert_eq!(padovi.len(), 6);
    }
}
