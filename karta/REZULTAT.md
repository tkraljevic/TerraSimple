# Rezultat paketa `karta`

Karta, crtanje i mjerenje opisani su u `docs/rezultati.md`, u rezultatima paketa `karta`.

## Lik koji siječe sam sebe (9. 11. 2026.)

Dok se crta ili mjeri lik, karta na mjestu gdje rub siječe sam sebe crta mali crveni krug s bijelim
obrubom, i to već s točkom pod mišem. Isto vrijedi za odabran gotov lik. Crvena s bijelim obrubom
vidi se na svijetloj i tamnoj karti i na ortofotu. `Mjerenje` dobiva polje `presjek`, a
`Crtanje::presjek` daje mjesto za upozorenje ispod karte, koje ispisuje `prozor`: „Rub lika siječe
sam sebe, pa površina nije točna. Mjesto je označeno crvenim krugom.”

Test na pravoj karti s događajima miša (`karta_testovi.rs`) crta osmicu i provjerava da je jedan
crveni krug na mjestu križanja, i dok se crta i kad je lik gotov, a da ga ispravan lik nema.

Odabrani lik provjerava se u svakom okviru. Za lik od milijun vrhova to je oko 24 ms, pa bi takav
odabran lik usporio kartu; obični likovi imaju desetke vrhova.

Namjerne greške na novim funkcijama (`presjek`, `geometrija_crteza`, `mjerenje`, `nacrtaj`): 10 uhvaćeno,
0 preživjelo, 4 neizvedivo.

**Nije provjereno:** izgled oznake pravim okom na ortofotu.
