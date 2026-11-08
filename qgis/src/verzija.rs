use std::{path::Path, process::Output};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Inacica(pub u32, pub u32, pub u32);

pub(crate) fn procitaj(s: &str) -> Option<Inacica> {
    s.split(|c: char| !c.is_ascii_digit() && c != '.')
        .find_map(brojevi)
}
fn brojevi(s: &str) -> Option<Inacica> {
    let mut p = s.split('.');
    let a = p.next()?.parse().ok()?;
    let b = p.next()?.parse().ok()?;
    let c = p.next().unwrap_or("0").parse().ok()?;
    Some(Inacica(a, b, c))
}
pub(crate) fn iz_izlaza(o: Output) -> Option<Inacica> {
    if !o.status.success() {
        return None;
    }
    procitaj(&String::from_utf8_lossy(&o.stdout))
}
pub(crate) fn mac(p: &Path) -> Option<Inacica> {
    let v = plist::Value::from_file(p.join("Contents/Info.plist")).ok()?;
    procitaj(
        v.as_dictionary()?
            .get("CFBundleShortVersionString")?
            .as_string()?,
    )
}
pub(crate) fn windows(p: &Path, paket: &str) -> Option<Inacica> {
    let db = std::fs::read_to_string(p.join("etc/setup/installed.db")).ok();
    let redak = db.as_deref().and_then(|s| {
        s.lines()
            .find(|l| l.split_whitespace().next() == Some(paket))
    });
    redak
        .and_then(procitaj)
        .or_else(|| p.file_name()?.to_str().and_then(procitaj))
}
