//! Samo čitanje mapa i metapodataka: ne pokreće QGIS ni upit --version.
fn main() -> Result<(), qgis::Greska> {
    let q = qgis::pronadji_u(&qgis::Okolina::trenutna(), None, |_| {
        Err(std::io::Error::other("Ovaj pregled ne pokreće programe."))
    })?;
    println!("{} {:?}", q.staza.display(), q.inacica);
    Ok(())
}
