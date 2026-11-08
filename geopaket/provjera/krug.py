"""Izmišljeni podaci, oba smjera razmjene i službeni validator; bez prozora."""
import json, os, pathlib, sqlite3, subprocess, sys
from qgis.core import (Qgis, QgsApplication, QgsVectorLayer, QgsFeature, QgsGeometry, QgsVectorFileWriter, QgsCoordinateTransformContext, QgsProviderRegistry, QgsField, QgsSymbol, QgsLineSymbol, QgsSingleSymbolRenderer)
from qgis.PyQt.QtCore import QVariant
app=QgsApplication([],False);app.initQgis()
from osgeo import gdal
print('Verzije:', Qgis.QGIS_VERSION, gdal.VersionInfo('--version'), flush=True)
program,mapa,validator=sys.argv[1:]
mapa=pathlib.Path(mapa);mapa.mkdir(parents=True,exist_ok=True)
rez=[]
def provjeri(ime,uvjet):
    rez.append({"provjera":ime,"prosla":bool(uvjet)})
    if not uvjet: raise AssertionError(ime)
def rust(n,p):
    return subprocess.run([program,n,str(p)],check=True,capture_output=True,text=True).stdout
def sloj(p,t): return QgsVectorLayer(f"{p}|layername={t}",t,"ogr")
def val(p):
    r=subprocess.run([sys.executable,validator,str(p)],capture_output=True,text=True)
    tekst='\n'.join(x for x in (r.stdout+r.stderr).splitlines() if x.strip() and 'platform' not in x)
    if tekst: print(tekst,flush=True)
    provjeri('GDAL validator: '+p.name,r.returncode==0 and not tekst)
def ocisti(p):
    if p.exists(): p.unlink()
nas=mapa/'nas.gpkg';ocisti(nas);rust('stvori',nas)
val(nas)
for t in ['sloj_1','sloj_3','sloj_5']:
    l=sloj(nas,t);provjeri('QGIS otvara '+t,l.isValid() and l.featureCount()==1)
    provjeri('Z ostaje '+t,all(abs(v.z()-123.5)<1e-10 for f in l.getFeatures() for v in f.geometry().vertices()))
    provjeri('stil se otvara '+t,l.renderer().type()=='singleSymbol')
    provjeri('skriveni stilovi '+t,all(c.hidden for c in l.attributeTableConfig().columns() if c.name.startswith('stil_')))
    l=None
podaci=QgsProviderRegistry.instance().querySublayers(str(nas))
provjeri('gpkg_ts tablice nisu slojevi',all(not p.name().startswith('gpkg_ts_') for p in podaci))
l=sloj(nas,'sloj_1');l.startEditing()
f=next(l.getFeatures());l.changeAttributeValue(f.id(),l.fields().indexOf('naziv'),'Izmijenjeno u QGIS-u')
l.addAttribute(QgsField('tudji',QVariant.String));l.updateFields();l.changeAttributeValue(f.id(),l.fields().indexOf('tudji'),'Sačuvaj me')
novi=QgsFeature(l.fields());novi['naziv']='NULL';novi.setGeometry(QgsGeometry());l.addFeature(novi)
l.changeGeometry(f.id(),QgsGeometry.fromWkt('POINT Z (18.7 45.6 123.5)'))
provjeri('QGIS sprema izmjene',l.commitChanges());l=None
l=sloj(nas,'sloj_3');l.setRenderer(QgsSingleSymbolRenderer(QgsLineSymbol.createSimple({'line_color':'#7c3aed','line_width':'1.2'})))
l.saveStyleToDatabaseV2('Iz QGIS-a','',True,'');l=None
j=json.loads(rust('citaj',nas));provjeri('naš čitač vidi naziv','Izmijenjeno u QGIS-u' in json.dumps(j,ensure_ascii=False))
provjeri('NULL je prijavljen',len(j['upozorenja'])>=1)
linija=next(s for s in j['slojevi'] if s['id']==3)
provjeri('naš čitač vidi QGIS stil','r: 124, g: 58, b: 237' in linija['stil'] and 'debljina_mm: 1.2' in linija['stil'])
provjeri('naš čitač vidi pomak vrha','18.7' in j['slojevi'][0]['objekti'][0]['geometrija'])
c=sqlite3.connect(nas);prije=c.execute('SELECT fid,hex(geom),tudji FROM sloj_1 ORDER BY fid').fetchall();c.close()
rust('promijeni',nas)
c=sqlite3.connect(nas);poslije=c.execute('SELECT fid,hex(geom),tudji FROM sloj_1 ORDER BY fid').fetchall();c.close()
provjeri('tuđi stupac i svi izvorni zapisi ostaju',prije==poslije)
l=sloj(nas,'sloj_1');provjeri('QGIS ponovno otvara',l.isValid() and l.featureCount()==2);l=None
l=sloj(nas,'sloj_3');provjeri('QGIS stil ostaje nakon našeg spremanja',l.renderer().symbol().color().name()=='#7c3aed' and abs(l.renderer().symbol().width()-1.2)<1e-9);l=None;val(nas)
# Datoteka koju od početka piše QGIS, u metričkom sustavu i s visinom.
tudja=mapa/'qgis.gpkg';ocisti(tudja)
m=QgsVectorLayer('LineStringZ?crs=EPSG:3765&field=naziv:string','QGIS linija','memory')
f=QgsFeature(m.fields());f['naziv']='QGIS izvor';f.setGeometry(QgsGeometry.fromWkt('LINESTRING Z (655120 5046380 42,655220 5046480 43)'));m.dataProvider().addFeatures([f])
o=QgsVectorFileWriter.SaveVectorOptions();o.driverName='GPKG';o.layerName='izvor'
r=QgsVectorFileWriter.writeAsVectorFormatV3(m,str(tudja),QgsCoordinateTransformContext(),o);provjeri('QGIS piše svoj paket',r[0]==QgsVectorFileWriter.NoError)
j=json.loads(rust('citaj',tudja));provjeri('naš čitač otvara QGIS sustav i Z',not j['upozorenja'] and '42.0' in json.dumps(j))
c=sqlite3.connect(tudja);prije=c.execute('SELECT hex(geom) FROM izvor').fetchone()[0];c.close()
rust('promijeni',tudja)
c=sqlite3.connect(tudja);provjeri('neizmijenjeno nema kruga preračuna',prije==c.execute('SELECT hex(geom) FROM izvor').fetchone()[0]);c.close()
l=sloj(tudja,'izvor');provjeri('QGIS otvara naš upis u svoju datoteku',next(l.getFeatures())['opis']=='Spremljeno nakon QGIS-a');l=None;val(tudja)
(mapa/'rezultat.json').write_text(json.dumps(rez,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(rez,ensure_ascii=False))
# QgsApplication se ne gasi dok Python još drži pružatelje slojeva.
# Snimljeni izmišljeni rezultat za Rust testove bez QGIS-a.
l=sloj(tudja,'izvor');l.saveNamedStyle(str(mapa/'qgis.qml'));l=None
c=sqlite3.connect(tudja)
srs=c.execute('SELECT srs_name,srs_id,organization,organization_coordsys_id,definition,description FROM gpkg_spatial_ref_sys WHERE srs_id=3765').fetchone()
blob=c.execute('SELECT hex(geom) FROM izvor').fetchone()[0];c.close()
qml=(mapa/'qgis.qml').read_text()
assert str(pathlib.Path.home()) not in qml
snimka={'izvor':'QGIS '+Qgis.QGIS_VERSION+', izmišljena linija, bez prozora','srs':srs,'geometrija_hex':blob,'qml':qml}
podaci=pathlib.Path(__file__).parent.parent/'tests'/'podaci';podaci.mkdir(exist_ok=True)
(podaci/'qgis.json').write_text(json.dumps(snimka,ensure_ascii=False,indent=2)+'\n')
# Miješanje Z i 2D mora mijenjati deklaraciju, bez prepisivanja izvornog objekta.
def mijesaj(p, naredba, visina):
    prije=json.loads(rust('citaj',p))['slojevi'][0]['id']
    c=sqlite3.connect(p);blob=c.execute('SELECT hex(geom) FROM izvor WHERE fid=1').fetchone()[0];c.close()
    rust(naredba,p)
    c=sqlite3.connect(p)
    provjeri('opcionalna Z dimenzija '+p.name,c.execute('SELECT z FROM gpkg_geometry_columns').fetchone()[0]==2)
    provjeri('izvorni bajtovi nakon dodavanja '+p.name,c.execute('SELECT hex(geom) FROM izvor WHERE fid=1').fetchone()[0]==blob);c.close()
    j=json.loads(rust('citaj',p));l=j['slojevi'][0]
    provjeri('identitet stranog sloja ostaje '+p.name,l['id']==prije)
    novi=next(o for o in l['objekti'] if o['naziv']=='Dodana linija')
    provjeri('visina dodanog objekta '+p.name,visina in novi['geometrija'])
    l=sloj(p,'izvor');provjeri('QGIS otvara miješane dimenzije '+p.name,l.isValid() and l.featureCount()==2);l=None
    val(p)
mijesaj(tudja,'dodaj-bez-visine','visina: None')
ravnina=mapa/'ravnina.gpkg';ocisti(ravnina)
ravan=QgsVectorLayer('LineString?crs=EPSG:4326&field=naziv:string','Ravnina','memory')
f=QgsFeature(ravan.fields());f['naziv']='Izmišljena 2D linija';f.setGeometry(QgsGeometry.fromWkt('LINESTRING (18 45,18.1 45.1)'));ravan.dataProvider().addFeatures([f])
o=QgsVectorFileWriter.SaveVectorOptions();o.driverName='GPKG';o.layerName='izvor'
r=QgsVectorFileWriter.writeAsVectorFormatV3(ravan,str(ravnina),QgsCoordinateTransformContext(),o);provjeri('QGIS piše 2D sloj',r[0]==QgsVectorFileWriter.NoError)
mijesaj(ravnina,'dodaj-s-visinom','visina: Some(7.0)')
# QGIS-ov HDKS zapis pod brojem 100000 mora se prepoznati iz WKT-a.
hdks=mapa/'hdks.gpkg';ocisti(hdks)
h=QgsVectorLayer('LineStringZ?crs=EPSG:3908&field=naziv:string','HDKS','memory')
f=QgsFeature(h.fields());f['naziv']='Izmišljeni HDKS';f.setGeometry(QgsGeometry.fromWkt('LINESTRING Z (6500000 5040000 7,6500100 5040100 8)'));h.dataProvider().addFeatures([f])
o=QgsVectorFileWriter.SaveVectorOptions();o.driverName='GPKG';o.layerName='hdks'
r=QgsVectorFileWriter.writeAsVectorFormatV3(h,str(hdks),QgsCoordinateTransformContext(),o);provjeri('QGIS piše HDKS',r[0]==QgsVectorFileWriter.NoError)
c=sqlite3.connect(hdks);srs=c.execute('SELECT srs_id FROM gpkg_geometry_columns').fetchone()[0];c.close();provjeri('QGIS zapis je pod 100000',srs==100000)
j=json.loads(rust('citaj',hdks));provjeri('čitanje QGIS-ova 100000',not j['upozorenja'] and len(j['slojevi'][0]['objekti'])==1);val(hdks)
krivulja=mapa/'krivulja.gpkg';ocisti(krivulja)
k=QgsVectorLayer('CompoundCurve?crs=EPSG:4326&field=naziv:string','Krivulja','memory')
f=QgsFeature(k.fields());f['naziv']='Izmišljena krivulja';f.setGeometry(QgsGeometry.fromWkt('COMPOUNDCURVE ((18 45,18.1 45.1),CIRCULARSTRING (18.1 45.1,18.2 45.3,18.3 45.1))'));k.dataProvider().addFeatures([f])
o=QgsVectorFileWriter.SaveVectorOptions();o.driverName='GPKG';o.layerName='krivulja'
r=QgsVectorFileWriter.writeAsVectorFormatV3(k,str(krivulja),QgsCoordinateTransformContext(),o);provjeri('QGIS piše CompoundCurve',r[0]==QgsVectorFileWriter.NoError)
c=sqlite3.connect(krivulja);prije=c.execute('SELECT hex(geom) FROM krivulja').fetchone()[0];c.close()
j=json.loads(rust('citaj',krivulja));provjeri('krivulja je prijavljena kao nepodržana',len(j['upozorenja'])==1)
r=subprocess.run([program,'dodaj-bez-visine',str(krivulja)],capture_output=True,text=True)
provjeri('novi upis u nepodržani sloj je odbijen',r.returncode!=0 and 'nepodržanom vrstom' in r.stderr)
rust('spremi',krivulja);c=sqlite3.connect(krivulja);provjeri('krivulja ostaje bajt za bajt',prije==c.execute('SELECT hex(geom) FROM krivulja').fetchone()[0]);c.close();val(krivulja)
(mapa/'rezultat.json').write_text(json.dumps(rez,ensure_ascii=False,indent=2)+'\n')
print('Završni broj provjera:',len(rez))
