use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

#[derive(Debug, Clone)]
pub struct Opis {
    pub ime: String,
    pub cc: u32,
    pub pocetak: (usize, usize),
    pub kraj: (usize, usize),
}

pub fn analiziraj(kod: &str) -> Result<Vec<Opis>, String> {
    let datoteka = syn::parse_file(kod).map_err(|e| e.to_string())?;
    if testna(&datoteka.attrs) {
        return Ok(Vec::new());
    }
    let mut popis = Popis::default();
    popis.visit_file(&datoteka);
    match popis.greska {
        Some(e) => Err(e),
        None => Ok(popis.funkcije),
    }
}

#[derive(Default)]
struct Popis {
    put: Vec<String>,
    funkcije: Vec<Opis>,
    greska: Option<String>,
}

fn testna(atributi: &[syn::Attribute]) -> bool {
    atributi.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg") && a.meta.to_token_stream().to_string() == "cfg (test)")
    })
}

fn polozaj(span: Span, kraj: bool) -> (usize, usize) {
    let p = if kraj { span.end() } else { span.start() };
    (p.line, p.column + 1)
}

impl Popis {
    fn dodaj(&mut self, ime: String, potpis: &syn::Signature, tijelo: &syn::Block) {
        let mut broj = Brojac {
            cc: 1,
            greska: None,
        };
        broj.visit_block(tijelo);
        self.greska = self.greska.take().or(broj.greska);
        self.put.push(ime);
        self.funkcije.push(Opis {
            ime: self.put.join("::"),
            cc: broj.cc,
            pocetak: polozaj(potpis.span(), false),
            kraj: polozaj(tijelo.span(), true),
        });
        visit::visit_block(self, tijelo);
        self.put.pop();
    }
}

impl<'ast> Visit<'ast> for Popis {
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        if testna(&m.attrs) {
            return;
        }
        self.put.push(m.ident.to_string());
        visit::visit_item_mod(self, m);
        self.put.pop();
    }
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let tip = i.self_ty.to_token_stream().to_string();
        let ime = i.trait_.as_ref().map_or(tip.clone(), |(_, t, _)| {
            format!("{} for {tip}", t.to_token_stream())
        });
        self.put.push(ime);
        visit::visit_item_impl(self, i);
        self.put.pop();
    }
    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        self.put.push(i.ident.to_string());
        visit::visit_item_trait(self, i);
        self.put.pop();
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        if !testna(&f.attrs) {
            self.dodaj(f.sig.ident.to_string(), &f.sig, &f.block);
        }
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        if !testna(&f.attrs) {
            self.dodaj(f.sig.ident.to_string(), &f.sig, &f.block);
        }
    }
    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        if let Some(b) = &f.default {
            self.dodaj(f.sig.ident.to_string(), &f.sig, b);
        }
    }
    fn visit_item_macro(&mut self, m: &'ast syn::ItemMacro) {
        self.greska = Some(crate::poruke::tekst(
            "nepodrzan-makro",
            &[("mjesto", m.mac.path.to_token_stream().to_string())],
        ));
    }
}

struct Brojac {
    cc: u32,
    greska: Option<String>,
}

impl<'ast> Visit<'ast> for Brojac {
    fn visit_type_macro(&mut self, _: &'ast syn::TypeMacro) {}
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    fn visit_expr(&mut self, izraz: &'ast syn::Expr) {
        self.cc += match izraz {
            e if odluka(e) => 1,
            syn::Expr::Match(m) => m.arms.len().saturating_sub(1) as u32,
            syn::Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) => 1,
            _ => 0,
        };
        visit::visit_expr(self, izraz);
    }
    fn visit_arm(&mut self, grana: &'ast syn::Arm) {
        self.cc += u32::from(grana.guard.is_some());
        visit::visit_arm(self, grana);
    }
    fn visit_pat_or(&mut self, uzorak: &'ast syn::PatOr) {
        self.cc += uzorak.cases.len().saturating_sub(1) as u32;
        visit::visit_pat_or(self, uzorak);
    }
    fn visit_local(&mut self, lokalna: &'ast syn::Local) {
        self.cc += u32::from(lokalna.init.as_ref().is_some_and(|i| i.diverge.is_some()));
        visit::visit_local(self, lokalna);
    }
    fn visit_macro(&mut self, makro: &'ast syn::Macro) {
        if makro.path.segments.last().is_some_and(|s| {
            ["braced", "bracketed", "parenthesized"].contains(&s.ident.to_string().as_str())
        }) {
            self.cc += 1;
            return;
        }
        match izrazi_makroa(makro) {
            Ok(izrazi) => {
                for izraz in &izrazi {
                    self.visit_expr(izraz);
                }
            }
            Err(_) => {
                self.greska = Some(crate::poruke::tekst(
                    "nepodrzan-makro",
                    &[("mjesto", makro.path.to_token_stream().to_string())],
                ))
            }
        }
    }
}

fn izrazi_makroa(makro: &syn::Macro) -> syn::Result<Vec<syn::Expr>> {
    use syn::parse::Parser;
    if makro.path.is_ident("vec") {
        let tokeni = &makro.tokens;
        return Ok(vec![syn::parse2(quote::quote!([#tokeni]))?]);
    }
    if makro
        .path
        .segments
        .last()
        .is_some_and(|s| s.ident == "json")
    {
        return (|ulaz: syn::parse::ParseStream<'_>| {
            let mut izrazi = Vec::new();
            json_vrijednost(ulaz, &mut izrazi)?;
            Ok(izrazi)
        })
        .parse2(makro.tokens.clone());
    }
    if makro
        .path
        .segments
        .last()
        .is_some_and(|s| s.ident == "quote")
    {
        return Ok(Vec::new());
    }
    if makro.path.is_ident("matches") {
        return matches_izraz(makro);
    }
    poznati_makro(makro)?;
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    Ok(parser.parse2(makro.tokens.clone())?.into_iter().collect())
}

fn poznati_makro(makro: &syn::Macro) -> syn::Result<()> {
    let ime = makro
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();
    let poznati = [
        "println",
        "eprintln",
        "print",
        "eprint",
        "format",
        "format_args",
        "write",
        "writeln",
        "assert",
        "assert_eq",
        "assert_ne",
        "debug_assert",
        "debug_assert_eq",
        "debug_assert_ne",
        "panic",
        "todo",
        "unreachable",
        "include_str",
        "include_bytes",
        "env",
        "option_env",
        "params",
    ];
    if poznati.contains(&ime.as_str()) {
        Ok(())
    } else {
        Err(syn::Error::new(makro.span(), ime))
    }
}

fn odluka(izraz: &syn::Expr) -> bool {
    matches!(
        izraz,
        syn::Expr::If(_)
            | syn::Expr::ForLoop(_)
            | syn::Expr::While(_)
            | syn::Expr::Loop(_)
            | syn::Expr::Try(_)
    )
}

fn matches_izraz(makro: &syn::Macro) -> syn::Result<Vec<syn::Expr>> {
    use syn::parse::Parser;
    (|ulaz: syn::parse::ParseStream<'_>| {
        let izraz: syn::Expr = ulaz.parse()?;
        ulaz.parse::<syn::Token![,]>()?;
        let uzorak: proc_macro2::TokenStream = ulaz.parse()?;
        let kod = quote::quote!(match #izraz { #uzorak => (), _ => () });
        Ok(vec![syn::parse2(kod)?])
    })
    .parse2(makro.tokens.clone())
}

fn json_vrijednost(
    ulaz: syn::parse::ParseStream<'_>,
    izrazi: &mut Vec<syn::Expr>,
) -> syn::Result<()> {
    if ulaz.peek(syn::token::Brace) {
        let sadrzaj;
        syn::braced!(sadrzaj in ulaz);
        json_objekt(&sadrzaj, izrazi)
    } else if ulaz.peek(syn::token::Bracket) {
        let sadrzaj;
        syn::bracketed!(sadrzaj in ulaz);
        json_niz(&sadrzaj, izrazi)
    } else {
        izrazi.push(ulaz.parse()?);
        Ok(())
    }
}

fn json_objekt(ulaz: syn::parse::ParseStream<'_>, izrazi: &mut Vec<syn::Expr>) -> syn::Result<()> {
    while !ulaz.is_empty() {
        izrazi.push(ulaz.parse()?);
        ulaz.parse::<syn::Token![:]>()?;
        json_vrijednost(ulaz, izrazi)?;
        if ulaz.is_empty() {
            break;
        }
        ulaz.parse::<syn::Token![,]>()?;
    }
    Ok(())
}

fn json_niz(ulaz: syn::parse::ParseStream<'_>, izrazi: &mut Vec<syn::Expr>) -> syn::Result<()> {
    while !ulaz.is_empty() {
        json_vrijednost(ulaz, izrazi)?;
        if ulaz.is_empty() {
            break;
        }
        ulaz.parse::<syn::Token![,]>()?;
    }
    Ok(())
}
