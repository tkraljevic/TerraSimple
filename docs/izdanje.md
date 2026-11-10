# Izdanja TerraSimplea

Program je probno izdanje. Paketi ne traže administratorske ovlasti: raspakiraju
se u tvoju korisničku mapu. Računalo treba podržanu grafičku karticu i njezin
upravljački program. Pravila poslovnog računala mogu zasebno zabraniti pokretanje
programa.

## Preuzimanje

Na stranici **Releases** repozitorija odaberi izdanje i preuzmi paket za svoj
sustav. Izdanja su zasad probna (alfa). U datoteci JSON uz paket su inačica,
veličina i SHA-256.

## macOS

Raspakiraj `TerraSimple-…-macos-universal.zip` i premjesti `TerraSimple.app` u
svoju mapu `Applications` (`~/Applications`). Otvori ga dvoklikom. Isti paket
sadrži inačice za Apple Silicon i Intel.

Program je potpisan bez izdavača, pa nema potvrđen potpis ni Appleovu ovjeru. Ako
ga macOS blokira, a provjerio si odakle je preuzet, otvori **Postavke sustava →
Privatnost i sigurnost → Svejedno otvori** za TerraSimple. Ne isključuj zaštitu
cijelog sustava. Ako pravila računala ne dopuštaju iznimku, obrati se njegovoj
podršci. [Appleove upute](https://support.apple.com/en-us/102445).

## Windows x86_64

Raspakiraj `TerraSimple-…-windows-x86_64.zip`, na primjer u
`%LOCALAPPDATA%\Programs\TerraSimple`, pa otvori `TerraSimple.exe`. Program ne
otvara prozor konzole. Mapu `Dozvole` drži uz program.

Nepotpisan program može izazvati upozorenje SmartScreen. Kad provjeriš odakle je
preuzet, odaberi **Više informacija → Svejedno pokreni**, ako je ponuđeno. Ako
Smart App Control ili pravila računala blokiraju program, ne gasi zaštitu; zatraži
dopuštenje podrške računala.
[Microsoftovo objašnjenje](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

## Linux x86_64

Raspakiraj `TerraSimple-…-linux-x86_64.tar.gz` u svoju mapu. U raspakiranoj mapi
pokreni `./pokreni.sh` ili program `TerraSimple`. Paket se gradi na Ubuntu 24.04;
starije distribucije i sve kombinacije grafičkih upravljačkih programa nisu
potvrđene. Trebaju uobičajene biblioteke X11 ili Waylanda, među njima `libxkbcommon-x11`, i podrška za Vulkan.

Ako želiš prečac u izborniku, kopiraj `TerraSimple.desktop` u
`~/.local/share/applications/`. U njemu postavi `Exec` na punu stazu programa, a
`Icon` na punu stazu priloženog `terrasimple.png`. Stazu s razmacima u `Exec`
stavi u dvostruke navodnike. Samo raspakiravanje ne mijenja sustav.

## Provjera paketa i licence

Skrivena zastavica `--provjeri-pokretanje` otvara stvarni prozor s izmišljenim
primjerom bez mreže, provjerava nacrtani okvir i sama izlazi. Uspjeh je izlazni
kod 0; bez dostupne kartice ili valjane slike rezultat nije 0. Ima vlastitu
granicu od 12 sekundi. Namijenjena je provjeri gotovih paketa.

Licence su u mapi `Dozvole`; na Macu u
`TerraSimple.app/Contents/Resources/Dozvole`. Sadrže EUPL, hrvatski tekst, OFL
fonta Inter, ISC ikona, automatski popis licenci biblioteka i izvore karte.

## Za pripremu izdanja

Odluke (10. 11. 2026.): probna izdanja idu s grane `proba`. Gotov posao spaja se najprije u `proba`, a
oznaka probnog izdanja, npr. `v0.0.1`, stavlja se na nju. Kad probno izdanje prođe na pravim
računalima, `proba` se spaja u `master`. Tako `master` ima samo isprobano.

Tijek `.github/workflows/izdanje.yml` gradi tri sustava na oznaku `v*` ili ručno.
Inačicu uzima iz `prozor/Cargo.toml`. Ručno pokretanje sprema samo artefakte;
na oznaku nastaje samo **nacrt** izdanja. Tijek ne objavljuje izdanje. Oznaka mora odgovarati inačici, npr. `v0.0.1`.

Isti postupak lokalno: `python3 skripte/izdanje.py macos` (odnosno `windows`
ili `linux`). Trebaju odgovarajući Rust ciljevi, cargo-deny 0.20.2 i cargo-about
0.9.3 sa značajkom `cli`. Svi pozvani procesi imaju granicu vremena.
Za lokalnu provjeru samo Apple Silicona služi `--samo-arm`; naziv izlaza tada
sadrži `arm64-proba` i taj paket nije univerzalan. Tijek izdanja tu zastavicu ne koristi.

Na Macu se cijeli `.app` potpisuje bez izdavača (`codesign --sign -`) i provjerava
s `codesign --verify --strict`. Bez toga je potpis imala samo izvršna datoteka, a
provjera paketa je padala.

Windows se gradi sa statičkim MSVC okruženjem (`crt-static`) kako prenosivi EXE
ne bi tražio njegovu zasebnu instalaciju; [Rustove upute](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes).
Profil izdanja uklanja simbole (`strip = "symbols"`); ostale optimizacije ostaju zadane.

Identifikator `org.terrasimple.TerraSimple` zasad je prijedlog za potvrdu.
Može se zadati varijablom `TERRASIMPLE_BUNDLE_ID`. Za ovo izdanje pripremljeni su
ZIP za Mac i Windows te tar.gz za Linux; DMG i AppImage ostaju odluka za kasnije.
Stvarno izvedene provjere i mjerenja su u [priprema-izdanja-0.0.1.md](priprema-izdanja-0.0.1.md).
