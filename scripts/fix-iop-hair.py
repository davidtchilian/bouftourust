#!/usr/bin/env python3
"""Exporte les têtes natives du Iop prévues pour les coiffes (frame 2).

Nécessite CairoSVG et Pillow, Java/JPEXS et l'archive source Remastered.
Les sources et SVG restent dans .local-assets ; seuls les PNG utilisés et
leurs compositions sont versionnés. Aucune découpe approximative du visage.
"""
import argparse
import concurrent.futures
import copy
import hashlib
import json
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET
import zipfile

import cairosvg

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--ffdec', type=Path, default=ROOT.parent / 'bouftourush/.local-assets/tools/ffdec/ffdec.jar')
p.add_argument('--archive', type=Path, default=ROOT / '.local-assets/dofus-1.40.13-remastered.zip')
args = p.parse_args()
cache = ROOT / '.local-assets/iop-heads'
cache.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(args.archive) as archive:
    source = archive.read('clips/sprites/80.swf')
assert hashlib.sha256(args.archive.read_bytes()).hexdigest() == 'aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5'
swf = cache / '80.swf'
swf.write_bytes(source)
manifest_path = ROOT / 'assets/manifest.json'
m = json.loads(manifest_path.read_text())
SVG = '{http://www.w3.org/2000/svg}'
XL = '{http://www.w3.org/1999/xlink}'
FF = '{https://www.free-decompiler.com/flash}'
IDENTITY = [1, 0, 0, 1, 0, 0]

def export(cid, count, start=1):
    out = cache / f'{cid}-{start}-{count}'
    marker = out / 'done'
    if not marker.exists():
        out.mkdir(exist_ok=True)
        with (out / 'export.log').open('w') as log:
            subprocess.run(['java', '-Xmx1g', '-Djava.awt.headless=true', '-jar', str(args.ffdec),
                '-onerror', 'abort', '-selectid', str(cid), '-select', f'{cid}:{start}',
                '-sublength', str(count), '-format', 'sprite:svg', '-export', 'sprite', str(out), str(swf)],
                stdout=log, stderr=subprocess.STDOUT, check=True)
        marker.touch()
    paths = sorted(out.rglob('*.svg'), key=lambda f: int(f.stem))
    assert len(paths) >= count, (cid, count, len(paths))
    return paths[:count]

def matrix(node):
    value = node.get('transform')
    return [float(v) for v in re.split(r'[,\s]+', re.fullmatch(r'matrix\(([^)]+)\)', value)[1].strip())] if value else IDENTITY

def multiply(a, b):
    return [a[0]*b[0]+a[2]*b[1], a[1]*b[0]+a[3]*b[1], a[0]*b[2]+a[2]*b[3],
        a[1]*b[2]+a[3]*b[3], a[0]*b[4]+a[2]*b[5]+a[4], a[1]*b[4]+a[3]*b[5]+a[5]]

heads = {55: 'S', 115: 'R', 148: 'L', 192: 'F', 219: 'B'}
for cid, side in heads.items():
    path = export(cid, 1, 2)[0]
    root = ET.parse(path).getroot()
    width, height = [float(root.get(k).removesuffix('px')) for k in ['width', 'height']]
    transform = matrix(root.find(SVG+'g'))
    # Le groupe racine contient la translation de cadrage, conservée au raster.
    asset_id = f'iop.head.hat.{side}'
    dest = ROOT / f'assets/iop-heads/{side}.png'
    dest.parent.mkdir(exist_ok=True)
    cairosvg.svg2png(bytestring=ET.tostring(root), write_to=str(dest), scale=10)
    asset = copy.deepcopy(m['assets']['piece.iop.d6a7493d00d90a79ee6cfcc1'])
    asset.update(size=[width, height], pivot=[transform[4]/width, transform[5]/height],
        offset=[0, 0], loop=False, flipX=False, frames=[{'path': f'iop-heads/{side}.png', 'durationMs': 50}])
    asset['provenance'].update(originalFile=f'clips/sprites/80.swf#DefineSprite_{cid}:2',
        conversion='scripts/fix-iop-hair.py ; tête native pour coiffe, frame 2 ; CairoSVG densité 10')
    m['assets'][asset_id] = asset

symbols = {'staticS': 226, 'staticR': 229, 'staticL': 245, 'staticF': 257, 'staticB': 260,
    'runS': 262, 'runR': 265, 'runL': 268, 'runF': 270, 'runB': 274,
    'hitR': 285, 'hitL': 287, 'dieR': 472, 'dieL': 474, 'anim0R': 635, 'anim0L': 637}
directions = dict(east='S', southeast='R', south='F', southwest='R', west='S', northwest='L', north='B', northeast='L')
actions = dict(idle='static', run='run', hit='hit', death='die', cast='anim0', buff='anim0')
jobs = {}
for name, pose in m['composition']['actors'].items():
    if name.startswith('iop.'):
        _, action, side = name.split('.')
        cid = symbols[actions[action] + directions.get(side, side)]
        jobs[cid] = len(pose['frames'])
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    results = dict(zip(jobs, pool.map(lambda job: export(*job), jobs.items())))

def head_transform(path):
    root = ET.parse(path).getroot()
    defs = {e.get('id'): e for e in root.iter() if e.get('id')}
    found = []
    def visit(node, parent, seen=frozenset()):
        transform = multiply(parent, matrix(node))
        cid = int(node.get(FF+'characterId', '0'))
        if cid in heads:
            found.append((heads[cid], transform))
            return
        if node.tag == SVG+'use':
            ref = node.get(XL+'href', '')[1:]
            if ref in defs and ref not in seen:
                visit(defs[ref], transform, seen | {ref})
        else:
            for child in node:
                visit(child, transform, seen)
    # Ne pas appliquer la translation de cadrage de l'export à la pose.
    for child in root.find(SVG+'g'):
        visit(child, IDENTITY)
    assert len(found) <= 1, (path, found)
    return found[0] if found else None
transforms = {cid: [head_transform(path) for path in paths] for cid, paths in results.items()}
count = 0
for name, pose in m['composition']['actors'].items():
    if not name.startswith('iop.'):
        continue
    _, action, side = name.split('.')
    cid = symbols[actions[action] + directions.get(side, side)]
    for index, frame in enumerate(pose['frames']):
        layers = [part for part in frame['layers'] if not part.get('showWithHat')]
        hair = next((i for i, part in enumerate(layers) if part.get('hideWithHat')), None)
        if hair is None:
            assert transforms[cid][index] is None, (name, index)
            continue
        assert transforms[cid][index] is not None, (name, index)
        layers[hair+1]['hideWithHat'] = True
        side, transform = transforms[cid][index]
        layers.insert(hair+2, dict(assetId=f'iop.head.hat.{side}', matrix=transform, showWithHat=True))
        frame['layers'] = layers
        count += 1
manifest_path.write_text(json.dumps(m, separators=(',', ':'))+'\n')
paths = sorted({f['path'] for a in m['assets'].values() for f in a['frames']})
records = [{'path': p, 'bytes': (ROOT/'assets'/p).stat().st_size,
    'sha256': hashlib.file_digest((ROOT/'assets'/p).open('rb'), 'sha256').hexdigest()} for p in paths]
(ROOT/'assets/data/asset-files.json').write_text(json.dumps(records, indent=2)+'\n')
print(f'{count} poses corrigées avec les 5 têtes natives pour coiffes.')
