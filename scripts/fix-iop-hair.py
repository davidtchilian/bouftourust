#!/usr/bin/env python3
"""Masque aussi les contours des cheveux du Iop sous une coiffe.

L'export Flash sépare la couleur (hideWithHat) de son dessin ombré,
placé immédiatement après. Les deux couches doivent être masquées.
À appliquer après un nouvel import du manifest original.
"""
import json
from pathlib import Path

path = Path(__file__).resolve().parents[1] / 'assets' / 'manifest.json'
manifest = json.loads(path.read_text())
changed = 0
for name, pose in manifest['composition']['actors'].items():
    if not name.startswith('iop.'):
        continue
    for frame in pose['frames']:
        layers = frame['layers']
        # Une seule couche de couleur de cheveux par frame ; l'ombre suit.
        index = next((i for i, part in enumerate(layers) if part.get('hideWithHat')), None)
        if index is None:
            continue
        shading = layers[index + 1]
        assert shading.get('assetId') and not shading.get('slot'), name
        if not shading.get('hideWithHat'):
            shading['hideWithHat'] = True
            changed += 1
path.write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
print(f'{changed} couches de contours masquées sous les coiffes.')
