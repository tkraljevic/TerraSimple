# TerraSimple i goCOP: most preko plugina

Razgovor 27. 10. 2026. Ideja, ništa nije odlučeno.

## Zašto

U goCOP-u se gotovo sve može vezati za mjesto na karti: letve, nasipi i
dionice, objekti, prijave s terena, obilasci i zadaci vodočuvara, stadiji
obrane po branjenom području, akti. TerraSimple bi bio most između goCOP-a
i QGIS-a, a istovremeno jednostavan alat za crtanje za sve ostale
korisnike, i one koji goCOP nemaju.

## Pluginovi u TerraSimpleu

- Plugin je zaseban program koji s TerraSimpleom razgovara JSON-RPC-om
  preko stdin i stdout (kao LSP ili MCP). Može biti u bilo kojem jeziku, a
  pad plugina ne ruši program.
- Dinamičke biblioteke ne (nema stabilnog ABI-ja u Rustu). WASM kasnije,
  kao drugi tip plugina sa sandboxom, za pluginove iz zajednice.
- Manifest `terrasimple-plugin.toml`: ime, inačica, inačica plugin API-ja,
  izvršna datoteka po platformi, dopuštenja (mreža, datoteke) koja korisnik
  vidi pri instalaciji.
- Plugin smije ponuditi: izvor slojeva (čitanje, upis, sukobi), naredbe u
  izborniku, deklarativni panel uz odabrani objekt (polja, popisi, gumbi),
  uvoz i izvoz formata, otvaranje adresa `terrasimple://…`.
- TerraSimple daje pluginu: pomicanje karte, osvježavanje sloja, obavijesti
  i napredak, spremanje tajni u sustavni spremnik ključeva.
- Model slojeva od faze 0–2 zna za vanjske izvore: sloj može biti samo za
  čitanje, uređivan, „čeka slanje” ili „u sukobu”. Plugin API ide u
  specifikaciju v1.0, makar s jednim pluginom (goCOP).

## Plugin za goCOP (`gocop-terrasimple`)

U repozitoriju goCOP-a, u Go-u, izlazi s istim izdanjem i postavlja ga
Postava. Veza s čvorom zato ne mora biti zamrznuti javni API.

- **Prijava:** QR s profila i PIN, jednom; ključ uređaja u sustavnom
  spremniku; opoziv i rok kao za mobitel.
- **Slojevi:** u „Mjestima” mapa „goCOP · <centar>” po sektorima i
  područjima; vrijeme na vremenskom klizaču. Po pravima korisnika; imena
  osoba ne idu u geometriju.
- **Panel:** letva (vodostaj, stadij, zadnje očitanje), prijava (opis,
  slike).
- **Upis:** sloj Crteži izravno; izmjene službenih slojeva (nasipi,
  dionice, rkm) kao prijedlog koji administrator prihvati u goCOP-u. Uz
  izmjenu ide oznaka verzije; sukob se pokaže u panelu. Verzije u knjizi
  radi čvor, nikad TerraSimple.
- **Arhiva:** „Otvori .cop arhivu” pokazuje stanje na dan, samo za čitanje,
  istim Go kodom kao goCOP (`CitajCop`). Backup servis ostaje zaseban.
- **Iz goCOP-a:** gumb „Otvori u TerraSimpleu” → `terrasimple://gocop/…`.

## Što bi trebalo u goCOP-u

Jedno pravilo kako se geometrija veže uz bilo koji zapis, da plugin svaki
zapis s mjestom može pokazati kao sloj bez posebnog posla za svaku vrstu:
GeoJSON u WGS84 i, gdje ima smisla, položaj na nasipu ili vodotoku u rkm
(stacionaža). Nova vrsta zapisa onda sama postaje sloj.

OGC API – Features vrijedi napraviti zbog QGIS-a, ali ne žuri.
