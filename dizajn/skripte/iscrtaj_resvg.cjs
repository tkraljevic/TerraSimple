/* Iscrtava izvorni SVG izravno u traženom broju piksela. */
const fs = require('node:fs');
const { Resvg } = require('@resvg/resvg-js');
const fontFiles = process.env.TERRA_FONT_FILE ? [process.env.TERRA_FONT_FILE] : [];
function iscrtaj(ulaz, izlaz, skala = 1) {
  const svg = fs.readFileSync(ulaz);
  const r = new Resvg(svg, {
    fitTo: { mode: 'zoom', value: Number(skala) },
    font: { loadSystemFonts: false, fontFiles, defaultFontFamily: 'Arial' }
  });
  const slika = r.render();
  const tekst = svg.toString('utf8');
  const w = tekst.match(/\bwidth="([\d.]+)"/);
  const h = tekst.match(/\bheight="([\d.]+)"/);
  if (w && h && (slika.width !== Math.round(Number(w[1]) * Number(skala)) ||
      slika.height !== Math.round(Number(h[1]) * Number(skala)))) {
    throw new Error('Izlazna slika nema tražene dimenzije: ' + ulaz);
  }
  fs.writeFileSync(izlaz, slika.asPng());
}
const [ulaz, izlaz, skala = '1'] = process.argv.slice(2);
if (ulaz === '--popis') {
  const poslovi = JSON.parse(fs.readFileSync(izlaz, 'utf8'));
  for (const p of poslovi) iscrtaj(p.ulaz, p.izlaz, p.skala);
  console.log(JSON.stringify({ iscrtano: poslovi.length, resvg: require('@resvg/resvg-js/package.json').version }));
} else {
  if (!ulaz || !izlaz) throw new Error('Potrebni su ulazni SVG i izlazni PNG.');
  iscrtaj(ulaz, izlaz, skala);
}
