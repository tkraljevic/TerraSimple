"""Ponovljivi mali početni ulazi; sve koordinate i nazivi su izmišljeni."""
from pathlib import Path
import sqlite3
import struct

ovdje = Path(__file__).resolve().parent
pocetni = ovdje / 'pocetni'

def gp(wkb, flag=1):
    return b'GP\x00' + bytes([flag]) + struct.pack('<i', 4326) + wkb

def tocka(x, y):
    return gp(struct.pack('<BI2d', 1, 1, x, y))

geometrije = {
    'tocka': tocka(18., 45.),
    'visina': gp(struct.pack('<BI3d', 1, 1001, 18., 45., 7.)),
    'prazna': gp(struct.pack('<BI2d', 1, 1, float('nan'), float('nan')), 17),
    'krivulja': gp(struct.pack('<BII', 1, 9, 0)),
    'linija': gp(struct.pack('<BII4d', 1, 2, 2, 18., 45., 18.1, 45.1)),
    'poligon': gp(struct.pack('<BIII8d', 1, 3, 1, 4, 18., 45., 18.1, 45., 18., 45.1, 18., 45.)),
    'odrezano': b'GP\x00',
    'null': b'',
}
qml = '''<qgis><selection><symbol name="0"><layer><Option name="color" value="255,0,0,255"/></layer></symbol></selection><renderer-v2 type="singleSymbol"><symbols><symbol name="0"><layer class="SimpleMarker"><Option type="Map"><Option name="color" value="30,100,80,255"/><Option name="size" value="3"/></Option></layer></symbol><symbol name="1"><layer><Option type="Map"><Option name="color" value="0,0,0,255"/></Option></layer></symbol></symbols></renderer-v2></qgis>'''
for n, b in geometrije.items():
    (pocetni / 'geometrija' / (n + '.bin')).write_bytes(b)
for n, b in {'vise-simbola': qml.encode(), 'odrezano': b'<qgis><renderer-v2', 'prazno': b'', 'utf8': b'\xff\xfe', 'ugnijezdeno': ('<x>'*32 + '</x>'*32).encode()}.items():
    (pocetni / 'qml' / (n + '.xml')).write_bytes(b)
p = pocetni / 'datoteka' / 'izmisljeno.gpkg'
if p.exists():
    p.unlink()
c = sqlite3.connect(p)
c.executescript((ovdje.parent / 'geopaket/src/shema.sql').read_text())
for t, tip, zapisi in [('tocke', 'POINT', [geometrije['tocka'], geometrije['visina'], geometrije['prazna'], None]), ('krivulje', 'COMPOUNDCURVE', [geometrije['krivulja']]), ('linije', 'LINESTRING', [geometrije['linija']])]:
    c.execute(f'CREATE TABLE {t}(fid INTEGER PRIMARY KEY,geom BLOB,naziv TEXT)')
    c.execute("INSERT INTO gpkg_contents(table_name,data_type,identifier,srs_id) VALUES(?,'features',?,4326)", (t,t))
    c.execute('INSERT INTO gpkg_geometry_columns VALUES(?,?,?,4326,2,0)', (t,'geom',tip))
    for i, b in enumerate(zapisi):
        c.execute(f'INSERT INTO {t} VALUES(?,?,?)', (i+1,b,'Izmišljeno'))
c.execute("INSERT INTO layer_styles(f_table_name,f_geometry_column,styleQML,useAsDefault) VALUES('tocke','geom',?,1)", (qml,))
c.execute("UPDATE gpkg_contents SET last_change='2026-11-08T00:00:00.000Z'")
c.execute("UPDATE layer_styles SET update_time='2026-11-08T00:00:00.000Z'")
c.commit();c.close()
(pocetni / 'datoteka' / 'odrezano.gpkg').write_bytes(b'SQLite format 3\x00')
(pocetni / 'datoteka' / 'nasumicno.gpkg').write_bytes(bytes(range(64)))
