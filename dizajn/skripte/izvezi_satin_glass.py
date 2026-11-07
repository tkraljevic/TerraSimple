"""Izvoz svih stvarnih veličina i pregleda satin glass ikona.

Za iscrtavanje su potrebni resvg i FigCraft. Gotovi PNG ne služi kao izvor
za druge veličine. Dodatni povećani pregled koristi samo najbližeg susjeda.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import subprocess
import xml.etree.ElementTree as E

from PIL import Image
from izradi_satin_glass import VELICINE, profil, svg
from temelj import IKONE

E.register_namespace('', 'http://www.w3.org/2000/svg')
NS='http://www.w3.org/2000/svg'
OPISI_STANJA = {
    'odabrani_alat': 'Svijetli potez na tamnozelenom gumbu',
    'opasna_radnja': 'Obriši ili obriši sloj',
    'negativna_oznaka': 'Sadržaj koji se ne može izvesti',
    'priguseno': 'Prigušena ikona; boja sama ne dokazuje onemogućenu radnju',
    'narancasti_poligon': 'Boja poligonskog sloja ili objekta',
    'plava_linija': 'Boja linijskog sloja',
    'crvena_tocka': 'Boja točkastog sloja',
    'plava_podloga': 'Plava oznaka točke na karti',
    'potvrda': 'Pozitivna oznaka u sažetku promjena',
}


def zapisi_svg(path,ime,vel,tema,stanje='obicno',platno=None):
    path.parent.mkdir(parents=True,exist_ok=True)
    root=E.fromstring(svg(ime,vel,tema,stanje))
    if platno:
        root.set('width',str(platno));root.set('height',str(platno));root.set('viewBox',f'0 0 {platno} {platno}')
        root.find('{'+NS+'}g').set('transform',f'translate({(platno-vel)/2} {(platno-vel)/2})')
    path.write_text(E.tostring(root,encoding='unicode')+'\n')


def stranica(vel,tema,skala):
    w,h=888,760
    root=E.Element('{'+NS+'}svg',width=str(w*skala),height=str(h*skala),viewBox=f'0 0 {w} {h}')
    bg='#1D2422' if tema=='tamna' else '#F3F7F4';fg='#D8F3E8' if tema=='tamna' else '#174A3F'
    E.SubElement(root,'{'+NS+'}rect',width=str(w),height=str(h),fill=bg)
    t=E.SubElement(root,'{'+NS+'}text',x='24',y='28',fill=fg,**{'font-family':'sans-serif','font-size':'18'})
    t.text=f'Satin glass · {tema} · crtež {vel} px · {skala}×'
    t=E.SubElement(root,'{'+NS+'}text',x='24',y='51',fill=fg,**{'font-family':'sans-serif','font-size':'12'})
    t.text='Odluke (7. 11. 2026.) · izravno iscrtavanje SVG-a'
    for i,ime in enumerate(IKONE):
        icon=E.fromstring(svg(ime,vel,tema,predznak=f'c{i}'))
        x,y=24+i%6*144,78+i//6*67
        icon.set('x',str(x));icon.set('y',str(y));root.append(icon)
        t=E.SubElement(root,'{'+NS+'}text',x=str(x),y=str(y+45),fill=fg,**{'font-family':'sans-serif','font-size':'11'})
        t.text=ime
    return E.tostring(root,encoding='unicode')+'\n'


def iscrtaj_figcraft(posao):
    cli,src,dst=posao
    r=subprocess.run([cli,'convert',str(src),str(dst)],capture_output=True,text=True,check=True)
    if 'warning' in (r.stdout+r.stderr).lower():
        raise RuntimeError('Upozorenje pri iscrtavanju: '+src.name+' '+r.stdout+r.stderr)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--figcraft',required=True)
    p.add_argument('--resvg-moduli',required=True)
    p.add_argument('--stanja',required=True,help='JSON popis stanja zatečenih u nacrtu')
    p.add_argument('--font',required=True,help='Datoteka slova za natpise pregleda')
    args=p.parse_args()
    root=Path(__file__).resolve().parents[2]
    res=root/'resursi/ikone/sucelje/efekti';reviews=root/'dizajn/pregledi-ikona'
    used=json.loads(Path(args.stanja).read_text())
    manifest=dict(verzija=2,odluka='Odluke (7. 11. 2026.)',stil='satin glass',
        velicine={str(v):dict(profil(v),platno=v+2*profil(v)['rub']) for v in VELICINE},
        stanja={'obicno':dict(ikone=list(IKONE),namjena='Redovno stanje')},teme={},izvozi=[])
    for state,icons in used.items():
        manifest['stanja'][state]=dict(ikone=list(icons),primjeri=icons,namjena=OPISI_STANJA[state])
    env=dict(os.environ,NODE_PATH=args.resvg_moduli,TERRA_FONT_FILE=args.font)
    version=subprocess.run(['node','-p',"require('@resvg/resvg-js/package.json').version"],env=env,capture_output=True,text=True,check=True).stdout.strip()
    fig_version=subprocess.run([args.figcraft,'--version'],capture_output=True,text=True,check=True).stdout.strip()
    manifest.update(licenca='../LICENCA-LUCIDE.txt',podrijetlo='../LUCIDE-PODRIJETLO.md',
        iscrtavanje=dict(paket='@resvg/resvg-js',resvg=version,figcraft=fig_version,skale=[1,2],izvor='SVG za svaku veličinu',
                        polozaj='Platno se crta na položaju ikone umanjenom za rub; ne smanjuje se u okvir crteža.'))
    jobs=[]
    def raster(src,dst,scale):jobs.append(dict(ulaz=str(src),izlaz=str(dst),skala=scale))
    for tema in ('svijetla','tamna'):
        manifest['teme'][tema]={}
        for ime in IKONE:
            src=res/tema/(ime+'.svg');zapisi_svg(src,ime,22,tema,platno=32)
            (res/tema/'32').mkdir(exist_ok=True)
            raster(src,res/tema/(ime+'.png'),2);raster(src,res/tema/'32'/(ime+'.png'),1)
            manifest['teme'][tema][ime]=dict(svg=f'{tema}/{ime}.svg',png32=f'{tema}/32/{ime}.png',png64=f'{tema}/{ime}.png')
        states={'obicno':list(IKONE),**{s:list(items) for s,items in used.items()}}
        for state,names in states.items():
            for vel in VELICINE:
                pr=profil(vel)
                for ime in names:
                    rel=Path('velicine')/tema/state/str(vel)/ime
                    src=res/rel.with_suffix('.svg');zapisi_svg(src,ime,vel,tema,state)
                    a=res/rel.with_name(ime+'@1x.png');b=res/rel.with_name(ime+'@2x.png')
                    raster(src,a,1);raster(src,b,2)
                    manifest['izvozi'].append(dict(ikona=ime,tema=tema,stanje=state,crtez=vel,
                        platno=vel+2*pr['rub'],rub=pr['rub'],svg=str(src.relative_to(res)),
                        png1x=str(a.relative_to(res)),png2x=str(b.relative_to(res))))
    figjobs=[]
    for tema in ('svijetla','tamna'):
        for vel in VELICINE:
            for scale in (1,2):
                name=f'{tema}-{vel}-{scale}x';src=reviews/'svg'/(name+'.svg')
                src.parent.mkdir(parents=True,exist_ok=True);src.write_text(stranica(vel,tema,scale))
                for renderer in ('resvg','figcraft'):
                    dst=reviews/renderer/(name+'.png');dst.parent.mkdir(exist_ok=True)
                    if renderer=='resvg':raster(src,dst,1)
                    else:figjobs.append((args.figcraft,src,dst))
    # Popis putanja je samo privremen ulaz iscrtavanju, ne dio spremljenih resursa.
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        queue=Path(tmp)/'poslovi.json';queue.write_text(json.dumps(jobs))
        subprocess.run(['node',str(root/'dizajn/skripte/iscrtaj_resvg.cjs'),'--popis',str(queue)],env=env,check=True)
    with ThreadPoolExecutor(max_workers=4) as pool:list(pool.map(iscrtaj_figcraft,figjobs))
    for renderer in ('resvg','figcraft'):
        folder=reviews/renderer/'najblizi-4x';folder.mkdir(exist_ok=True)
        for image in (reviews/renderer).glob('*.png'):
            with Image.open(image) as im:im.resize((im.width*4,im.height*4),Image.Resampling.NEAREST).save(folder/image.name)
    (res/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(svg_velicine=len(manifest['izvozi']),png_poslovi=len(jobs),pregledi_figcraft=len(figjobs))))


if __name__=='__main__':main()
