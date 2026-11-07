#!/usr/bin/env python3
"""Télécharge les icônes officielles Xixou et prépare les PNG locaux (CairoSVG)."""
import hashlib
import json
from pathlib import Path
import urllib.request

import cairosvg

ROOT = Path(__file__).resolve().parents[1]
BASE = 'https://xixou.io/wp-content/uploads/xixou-sorts-icones/officiel/'
SPELL_IDS = {
    'explosive': 179, 'flaming': 165, 'magic': 161, 'power': 166,
    'dissolution': 439, 'chastisement': 431, 'transposition': 438,
    'punishment': 446, 'sword': 160, 'jump': 142, 'intimidation': 143,
    'wrath': 159, 'glyph': 10, 'cloudy': 9, 'teleport': 11, 'truce': 5,
    'iop_power': 153, 'sournoiserie': 61, 'mass_trap': 79,
    'invisibility': 72, 'mortal_attack': 76,
}


def download(url):
    request = urllib.request.Request(url, headers={
        'User-Agent': 'Mozilla/5.0', 'Referer': 'https://xixou.io/sorts/',
    })
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def main():
    assets = ROOT / 'assets'
    manifest_path = assets / 'manifest.json'
    manifest = json.loads(manifest_path.read_text())
    game = json.loads((assets / 'data/game.json').read_text())
    if set(game['spells']) != set(SPELL_IDS):
        raise ValueError('Actualiser SPELL_IDS pour couvrir tous les sorts du jeu.')
    catalog = json.loads(download(
        'https://xixou.io/wp-content/plugins/xixou-sorts/data/fr/spells.json'
    ))
    names = {spell['id']: spell['name'] for spell in catalog['spells']}
    originals = ROOT / '.local-assets/xixou-spells'
    originals.mkdir(parents=True, exist_ok=True)
    # Tout télécharger et convertir avant de modifier les fichiers runtime.
    prepared = []
    for key, spell_id in SPELL_IDS.items():
        url = f'{BASE}{spell_id}.svg'
        svg = download(url)
        (originals / f'{spell_id}.svg').write_bytes(svg)
        png = cairosvg.svg2png(bytestring=svg, output_width=192, output_height=192)
        prepared.append((key, spell_id, url, svg, png))
    references = {}
    for key, spell_id, url, svg, png in prepared:
        asset = manifest['assets'][f'ui.spell.{key}']
        path = f'textures/ui-spell-{key}/000.png'
        (assets / path).parent.mkdir(parents=True, exist_ok=True)
        (assets / path).write_bytes(png)
        # Les SVG complets ont un canevas de 40 × 40, sans recadrage.
        asset.update(frames=[{'path': path, 'durationMs': 50}],
                     size=[40, 40], pivot=[0, 0], offset=[0, 0],
                     flipX=False, **{'loop': False})
        reference = {'id': spell_id, 'name': names[spell_id],
                     'page': f'https://xixou.io/sorts/{spell_id}/',
                     'image': url, 'svgSha256': hashlib.sha256(svg).hexdigest()}
        references[key] = reference
        asset['provenance'] = {
            'source': url, 'version': 'xixou-official',
            'originalFile': f'{spell_id}.svg', 'referencePage': reference['page'],
            'conversion': 'scripts/import-spell-icons.py ; CairoSVG ; PNG RGBA 192 px ; canevas SVG complet',
            'usageRights': 'Visuels Dofus : propriété de leurs ayants droit ; aucune autorisation de redistribution établie.',
        }
    manifest_path.write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
    (assets / 'data/xixou-spells.json').write_text(
        json.dumps(references, ensure_ascii=False, indent=2) + '\n')
    paths = sorted({f['path'] for a in manifest['assets'].values() for f in a['frames']})
    records = [{'path': p, 'bytes': (assets / p).stat().st_size,
                'sha256': hashlib.sha256((assets / p).read_bytes()).hexdigest()} for p in paths]
    (assets / 'data/asset-files.json').write_text(json.dumps(records, indent=2) + '\n')
    print(f'{len(references)} icônes Xixou intégrées ; SVG conservés sous {originals}.')


if __name__ == '__main__':
    main()
