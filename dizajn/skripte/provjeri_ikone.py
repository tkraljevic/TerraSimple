"""Provjera izvoza, veličina, prozirnosti i pregleda ikona."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as E
import zipfile

from PIL import Image, ImageChops, ImageStat
from izradi_satin_glass import VELICINE, dijelovi, geometrija, profil
from temelj import IKONE

NS='http://www.w3.org/2000/svg'


def provjeri_png(path,size):
    with Image.open(path) as im:
        assert im.size==(size,size),(path.name,im.size,size)
        alpha=im.convert('RGBA').getchannel('A')
        assert alpha.getextrema()[0]==0 and alpha.getextrema()[1]>128,path.name
        edges=[alpha.crop((0,0,size,1)),alpha.crop((0,size-1,size,size)),alpha.crop((0,0,1,size)),alpha.crop((size-1,0,size,size))]
        return max(e.getextrema()[1] for e in edges)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--zastita',help='Ranije spremljeni sažeci zaštićenih datoteka')
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[2];base=root/'resursi/ikone/sucelje/efekti'
    m=json.loads((base/'manifest.json').read_text());svg_count=png_count=0;edge=0
    for record in m['izvozi']:
        v=record['crtez'];canvas=v+2*profil(v)['rub'];s=E.parse(base/record['svg']).getroot()
        assert int(s.attrib['width'])==canvas and int(s.attrib['height'])==canvas
        assert s.attrib['viewBox']==f'0 0 {canvas} {canvas}'
        assert not s.findall('.//{'+NS+'}image')
        meta=s.find('{'+NS+'}metadata');assert meta is not None and 'Lucide Icons and Contributors' in meta.text
        svg_count+=1
        for key,density in [('png1x',1),('png2x',2)]:
            edge=max(edge,provjeri_png(base/record[key],canvas*density));png_count+=1
    expected=Counter((r['tema'],r['crtez']) for r in m['izvozi'] if r['stanje']=='obicno')
    assert len(expected)==10 and set(expected.values())=={60}
    for theme,icons in m['teme'].items():
        assert set(icons)==set(IKONE)
        for name,r in icons.items():
            assert E.parse(base/r['svg']).getroot().get('width')=='32'
            edge=max(edge,provjeri_png(base/r['png32'],32),provjeri_png(base/r['png64'],64))
    table=[]
    for v in VELICINE:
        parts=dijelovi('tablica',v);rect=geometrija(parts[0][0],parts[0][1:],v)
        x,y,w,h=[rect[k] for k in ('x','y','width','height')]
        for part in parts[1:]:
            g=geometrija(part[0],part[1:],v)
            coords=[float(n) for n in re.findall(r'-?\d+(?:\.\d+)?',g['path'])]
            a,b,c,d=coords
            assert (a==x and c==x+w and b==d) or (b==y and d==y+h and a==c),(v,coords,rect)
        table.append(dict(velicina=v,unutarnje_crte=len(parts)-1,zavrseci_na_okviru=True))
    reviews=root/'dizajn/pregledi-ikona';comparisons=[];nn_count=0
    for theme in ('svijetla','tamna'):
        for v in VELICINE:
            for density in (1,2):
                images=[]
                for renderer in ('figcraft','resvg'):
                    path=reviews/renderer/f'{theme}-{v}-{density}x.png'
                    image=Image.open(path).convert('RGB');assert image.size==(888*density,760*density)
                    larger=Image.open(reviews/renderer/'najblizi-4x'/path.name).convert('RGB')
                    nearest=image.resize((image.width*4,image.height*4),Image.Resampling.NEAREST)
                    assert larger.size==nearest.size and ImageChops.difference(larger,nearest).getbbox() is None
                    nn_count+=1;images.append(image)
                maes=[]
                for i,name in enumerate(IKONE):
                    x,y=(24+i%6*144)*density,(78+i//6*67)*density
                    side=(v+2*profil(v)['rub'])*density
                    a,b=[im.crop((x,y,x+side,y+side)) for im in images]
                    diff=ImageChops.difference(a,b);maes.append((sum(ImageStat.Stat(diff).mean)/3,name))
                comparisons.append(dict(tema=theme,velicina=v,skala=density,
                    srednja_razlika_rgb=round(sum(p[0] for p in maes)/len(maes),4),
                    najveca_srednja_razlika=round(max(maes)[0],4),ikona_najvece_razlike=max(maes)[1]))
    with zipfile.ZipFile(root/'dizajn/TerraSimple-ikone-efekti.figcraft') as z:
        data=z.read('document.json');native=json.loads(data)
    assert re.search(rb"/(?:Users|home|tmp)/|file[:][/][/]", data) is None
    nodes=native['nodes'];frames=[];logos=[]
    for pg in native['pages'][:6]:
        for ident in nodes[str(pg)].get('children',[]):
            n=nodes[str(ident)]
            if n['type']=='frame':frames.append(ident)
    def walk(i):
        n=nodes[str(i)]
        if n.get('name')=='TerraSimple · logotip T':logos.append(i)
        for c in n.get('children',[]):walk(c)
    for f in frames:walk(f)
    assert len(frames)==24 and len(logos)==24
    protected=0
    if args.zastita:
        before=json.loads(Path(args.zastita).read_text())
        for p,sha in before.items():assert hashlib.sha256((root/p).read_bytes()).hexdigest()==sha,p
        protected=len(before)
    result=dict(svg=svg_count,png_velicine=png_count,png_kompatibilni=240,
        najveca_alfa_ruba=edge,tablica=table,pregledi=40,povecanja_najblizi_susjed=nn_count,
        zasloni=24,logotipi=24,zasticene_datoteke=protected,
        usporedba_iscrtavanja=comparisons,
        napomena='Razlike dvaju iscrtavanja su izmjerene, ne pretpostavlja se jednakost piksela.')
    (reviews/'provjera.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('tablica','usporedba_iscrtavanja')},ensure_ascii=False))


if __name__=='__main__':main()
