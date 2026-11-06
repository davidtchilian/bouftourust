#!/usr/bin/env python3
"""Détecte les images noires dans une séquence --smoke-capture-dir (Pillow requis)."""
import argparse
from pathlib import Path
from PIL import Image

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('directory', type=Path)
args = parser.parse_args()
frames = sorted(args.directory.glob('frame-*.png'))
if not frames:
    parser.error('Aucune capture frame-*.png dans ce dossier')
black = []
lowest = 255.0
for path in frames:
    with Image.open(path) as image:
        # Contrôler le viewport, sans que le HUD masque une scène vide.
        width, height = image.size
        pixels = list(image.crop((width//10, height//10, width*9//10, height*3//4))
                      .resize((128, 72)).convert('L').tobytes())
    average = sum(pixels)/len(pixels)
    lowest = min(lowest, average)
    if sum(pixel <= 24 for pixel in pixels)/len(pixels) > 0.99:
        black.append(path.name)
print(f'{len(frames)} images contrôlées ; luminance moyenne minimale : {lowest:.1f}/255')
if black:
    raise SystemExit('Images noires : '+', '.join(black))
print('Aucune image noire détectée.')
