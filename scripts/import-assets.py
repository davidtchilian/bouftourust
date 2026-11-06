#!/usr/bin/env python3
"""Import reproductible du lot runtime Remastered ; aucune donnée de compte."""
import argparse, hashlib, json, pathlib, shutil
p=argparse.ArgumentParser();p.add_argument('source',nargs='?',default='../bouftourush/public/assets');p.add_argument('--check',action='store_true');a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[1];source=pathlib.Path(a.source);target=root/'assets';
if a.check: source=target
m=json.loads((source/'manifest.json').read_text()); paths=set()
for name,asset in m['assets'].items():
    if asset['provenance']['version']!='1.40.13-remastered': raise ValueError(f'Source interdite : {name}')
    for f in asset['frames']:
        path=pathlib.PurePosixPath(f['path'])
        if path.is_absolute() or '..' in path.parts: raise ValueError(path)
        paths.add(str(path))
records=[]
for path in sorted(paths):
    src=source/path
    if not src.is_file(): raise FileNotFoundError(f'{src} — voir docs/ASSETS.md')
    dst=target/path
    if not a.check:
        dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
    records.append({'path':path,'bytes':src.stat().st_size,'sha256':hashlib.file_digest(src.open('rb'),'sha256').hexdigest()})
if not a.check:
    shutil.copy2(source/'manifest.json',target/'manifest.json')
    (target/'data'/'asset-files.json').write_text(json.dumps(records,indent=2)+'\n')
else:
    expected=json.loads((target/'data'/'asset-files.json').read_text())
    if records!=expected: raise ValueError('Empreintes des assets différentes')
print(f'{len(records)} fichiers vérifiés, {sum(r["bytes"] for r in records)/1024**2:.1f} Mio, {len(m["assets"])} animations/pièces.')
