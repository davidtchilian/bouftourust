#!/usr/bin/env python3
"""Applique les masques source dans le repère Flash ; Pillow requis à l'import seulement."""
from PIL import Image, ImageChops
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1]/'assets';path=root/'manifest.json';m=json.loads(path.read_text());cache={};generated={};n=0
# Cache borné aux atlas réellement utilisés par les rares frames masquées.
def frame_image(asset,index):
    f=asset['frames'][index%len(asset['frames'])];p=f['path']
    if p not in cache: cache[p]=Image.open(root/p).convert('RGBA')
    im=cache[p]
    if 'region' in f:
        x,y,w,h,*_=f['region'];im=im.crop((x,y,x+w,y+h))
    return im
for actor in m['composition']['actors'].values():
    for index,frame in enumerate(actor['frames']):
        for layer in frame['layers']:
            if not layer.get('maskAssetId'):continue
            possibilities=[layer['assetId']] if layer.get('assetId') else sorted({acc['views'][layer['label']] for acc in m['composition']['accessories'].values() if acc['slot']==layer['slot']})
            layer['maskedAssets']={}
            for aid in possibilities:
                asset=m['assets'][aid];fi=index%len(asset['frames']) if layer.get('slot') else 0
                key=hashlib.sha256(json.dumps([aid,fi,layer['maskAssetId'],layer['matrix']],sort_keys=True).encode()).hexdigest()[:24];ident='masked.'+key;layer['maskedAssets'][aid]=ident
                if ident in generated:continue
                mask=m['assets'][layer['maskAssetId']];im=frame_image(asset,fi);alpha=frame_image(mask,0).getchannel('A');iw,ih=im.size;mw,mh=alpha.size
                a,b,c,d,tx,ty=layer['matrix'];w,h=asset['size'];px,py=asset['pivot'];sw,sh=mask['size'];sx,sy=mask['pivot']
                # Coordonnées pixel → pièce locale → matrice GAC → pixels du masque.
                coeff=(a*w/iw*mw/sw,c*h/ih*mw/sw,(-a*px*w-c*py*h+tx+sx*sw)*mw/sw,b*w/iw*mh/sh,d*h/ih*mh/sh,(-b*px*w-d*py*h+ty+sy*sh)*mh/sh)
                clip=alpha.transform(im.size,Image.Transform.AFFINE,coeff,resample=Image.Resampling.BILINEAR)
                im.putalpha(ImageChops.multiply(im.getchannel('A'),clip));out=f'composition-masks/{key}.png';(root/out).parent.mkdir(exist_ok=True);im.save(root/out)
                generated[ident]={**asset,'frames':[{'path':out,'durationMs':50}],'loop':False,'flipX':False,'provenance':{**asset['provenance'],'conversion':asset['provenance']['conversion']+' ; masque affine source, scripts/prepare-masks.py'}};n+=1
m['assets'].update(generated);path.write_text(json.dumps(m,separators=(',',':'))+'\n');print(f'{n} pièces masquées générées.');
paths=sorted({f['path'] for a in m['assets'].values() for f in a['frames']});records=[{'path':p,'bytes':(root/p).stat().st_size,'sha256':hashlib.file_digest((root/p).open('rb'),'sha256').hexdigest()} for p in paths];(root/'data/asset-files.json').write_text(json.dumps(records,indent=2)+'\n')
