#!/usr/bin/env python3
"""Exporte uniquement la progression, en lecture seule ; aucun mot de passe ni session."""
import argparse,json,pathlib,sqlite3
p=argparse.ArgumentParser();p.add_argument('source',help='Ancienne base SQLite privée ou export JSON');p.add_argument('--output',default='.local-saves/legacy-profile.json');a=p.parse_args();source=pathlib.Path(a.source).resolve()
if source.suffix in ('.sqlite','.db'):
    with sqlite3.connect(source.as_uri()+'?mode=ro',uri=True) as db:
        row=db.execute('SELECT profile FROM account WHERE id=1').fetchone()
        if not row or not row[0]:raise SystemExit('Aucune progression dans cette base.')
        profile=json.loads(row[0])
else:
    profile=json.loads(source.read_text())
    if 'profile' in profile:profile=profile['profile']
profile={k:v for k,v in profile.items() if k in ('classId','level','xp','kamas','inventory','equipment','completed')};profile['version']=1
out=pathlib.Path(a.output);out.parent.mkdir(parents=True,exist_ok=True)
with out.open('x') as f:json.dump(profile,f,ensure_ascii=False,indent=2)
print(f'Progression exportée dans {out}. Importer avec bouftourust --import-profile {out}')
