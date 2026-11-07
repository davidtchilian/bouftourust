#!/usr/bin/env python3
"""Actualise les attentes d'équipement avec un calcul indépendant du moteur Rust.

Les profils historiques, cas de dégâts et seuils d’XP sont conservés ; les
attentes de Vie suivent le barème courant de gains par niveau. Ajouter
les panoplies apparues dans le port permet de couvrir les nouveaux équipements.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
data = json.loads((ROOT / 'assets/data/game.json').read_text())
path = ROOT / 'tests/reference_progression.json'
fixture = json.loads(path.read_text())
covered = {case['setId'] for case in fixture['cases']}
for key in sorted(data['sets'].keys() - covered):
    items = [item for item in data['items'].values() if item['set'] == key]
    for level in [1, 25, 100]:
        profile = dict(level=level, xp=0, kamas=0, inventory=[], equipment={}, completed=[])
        for index, item in enumerate(items):
            uid = f'balanced-{index}'
            profile['inventory'].append(dict(uid=uid, item=item['id']))
            slot = 'ring1' if item['category'] == 'ring' else item['category']
            profile['equipment'][slot] = uid
        fixture['cases'].append(dict(setId=key, profile=profile))
for case in fixture['cases']:
    profile = case['profile']
    instances = {i['uid']: i for i in profile['inventory']}
    worn = [(data['items'][instances[uid]['item']], (1, 1.5, 2, 3)[instances[uid].get('fusionLevel', 0)])
            for uid in profile['equipment'].values()]
    balance = data['balance']
    stats = dict(hp=balance['startingHp'] + (profile['level']-1)*balance['hpPerLevel'],
                 power=(profile['level']-1)*balance['powerPerLevel'],
                 cooldownReduction=(profile['level']-1)*balance['cooldownReductionPerLevel'],
                 movementSpeed=(profile['level']-1)*balance['movementSpeedPerLevel'])
    sets = {}
    for item, multiplier in worn:
        for key, value in item['stats'].items():
            stats[key] += value * multiplier
        if item['set']:
            sets.setdefault(item['set'], set()).add(item['id'])
    for key, ids in sets.items():
        thresholds = data['sets'][key]['thresholds']
        eligible = [int(count) for count in thresholds if int(count) <= len(ids)]
        if eligible:
            for stat, value in thresholds[str(max(eligible))].items():
                stats[stat] += value
    limits = dict(cooldownReduction=balance['maxCooldownReduction'],
                  movementSpeed=balance['maxMovementSpeedBonus'])
    for item, multiplier in worn:
        for key, value in item.get('statLimits', {}).items():
            limits[key] = max(limits[key], value)
    for key, limit in limits.items():
        stats[key] = max(-50 if key == 'movementSpeed' else 0, min(limit, stats[key]))
    case['speed'] = stats.pop('movementSpeed')
    case['stats'] = stats
for case in fixture['xp']:
    case['hp'] = (case['level'] - 1) * data['balance']['hpPerLevel']
path.write_text(json.dumps(fixture, ensure_ascii=False, indent=2) + '\n')
print(f"{len(fixture['cases'])} profils vérifiés aux niveaux 1, 25 et 100.")
