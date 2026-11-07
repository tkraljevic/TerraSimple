# Ikone sučelja

**Odabrani stil je satin glass — Odluke (7. 11. 2026.).** Potpuni komplet za svijetlu i tamnu temu
nalazi se u [`efekti/`](efekti/README.md), s manifestom i SVG/PNG datotekama.
Primijenjen je na svih 24 zaslona u `dizajn/TerraSimple-ikone-efekti.figcraft`.
Pet stvarnih veličina i zatečena stanja popisana su u manifestu. Podrijetlo
geometrije opisuje [popis Lucide](LUCIDE-PODRIJETLO.md), uz
[tekst licence](LICENCA-LUCIDE.txt).

## Raniji linijski prijedlog

Komplet od 60 ikona temelji se na geometriji iz `dizajn/skripte/temelj.py`.
Osam ciljanih dorada definirano je u `dizajn/skripte/izvezi_ikone.py`.
Ovaj raniji prijedlog sačuvan je za usporedbu. Rust proba još koristi svoje
izvorno vektorsko crtanje; ova mapa priprema resurse i nacrt sučelja.

- `svg/`: uređive ikone, `viewBox="0 0 24 24"`, prozirna podloga.
- `png/24/`: 24 × 24 px za prikaz na zaslonu pri skali 1×.
- `png/48/`: 48 × 48 px za prikaz iste ikone pri skali 2×.
- Usporedni pregled: `dizajn/ikone-sucelja-pregled.svg` i `.png`.

SVG koristi `currentColor`, uz zadanu boju `#44474f`. Pri umetnutom SVG-u
boja se može promijeniti CSS svojstvom `color` na samom SVG elementu. Pri
učitavanju SVG-a kao slike boju treba postaviti u datoteci ili kroz mogućnosti
odabranog SVG renderera. PNG ima zadanu sivu boju; za tamnu temu prednost
ima SVG kojem se boja postavlja iz teme.

Potez je 2,1 jedinicu na mreži 24 × 24, sa zaobljenim završecima i spojevima.
Pri veličini 20 px to je 1,75 px. Aktivna ikona uzima boju iz teme; stanje
se označava i podlogom gumba, a naziv alata ostaje dostupan kao opis gumba.

Ponovni izvoz, bez dodatnih Python paketa:

```sh
python3 dizajn/skripte/izvezi_ikone.py --cli /put/do/vectorcraft-cli
```

Skripta ponovno zapisuje ovaj komplet i usporedni pregled. PNG iscrtava
VectorCraft; u ikonama nema filtara, gradijenata ni ugrađenih slika.
