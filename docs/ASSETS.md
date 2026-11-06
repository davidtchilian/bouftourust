# Assets Remastered

Les graphismes utilisés sont issus du client **Dofus Retro Remastered 1.40.13**. Ils restent la propriété de leurs ayants droit. Le projet est un fangame gratuit, non affilié à Ankama. Aucune autorisation de redistribution d'Ankama n'est établie ; le présent stockage Git répond à la demande du propriétaire du projet et ne constitue pas une telle autorisation.

## Jouer après un clone

`assets/` contient le manifeste, les atlas, les images runtime et les données. Ces fichiers doivent être versionnés. Aucun téléchargement en jeu n'est effectué. `assets/data/asset-files.json` contient le chemin, la taille et le SHA-256 de chaque PNG. Vérifier le lot :

```sh
python3 scripts/import-assets.py --check
cargo run -- --validate
```

## Récupérer les sources si elles manquent

- [Releases GitHub de Dofedex](https://github.com/dofera/dofedex/releases)
- [Release 1.40.13](https://github.com/dofera/dofedex/releases/tag/1.40.13)
- [Archive Remastered 1.40.13](https://github.com/dofera/dofedex/releases/download/1.40.13/dofus-1.40.13-remastered.zip)

SHA-256 de l'archive source : `aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5` ; taille 474 901 966 octets.

Conserver les archives et exports non utilisés sous `.local-assets/`, exclu de Git. Ne pas utiliser l'édition Classic comme remplacement. Le ZIP source a été conservé localement dans ce dossier ; il ne fait pas partie du bundle.

## Reproduire l'import

Le lot provient de `../bouftourush/public/assets/`. Chaque animation garde sa provenance source/version/fichier/export dans le manifeste. Les paramètres historiques et chemins SWF sont documentés dans [ASSETS-REFERENCE.md](ASSETS-REFERENCE.md), copie du relevé d'origine. Les chemins `.local-assets/` décrits dans cette copie sont ceux de l'ancien projet.

```sh
python3 scripts/import-assets.py ../bouftourush/public/assets
python3 -m venv .local-tools/python
.local-tools/python/bin/pip install Pillow
.local-tools/python/bin/python scripts/prepare-masks.py
python3 scripts/import-assets.py --check
```

`prepare-masks.py` applique les masques de disparition dans le repère source à chaque pièce/accessoire compatible. Les images dérivées sont petites, transparentes et versionnées. Ce précalcul préserve les matrices GAC et évite une génération d'images pendant le jeu.

Le script d'import refuse les chemins sortant du dossier et toute version autre que `1.40.13-remastered`. Il n'importe ni `.server-data/`, ni compte, ni sessions, ni mot de passe. Les fichiers PNG pointés par le manifeste sont copiés une seule fois même s'ils servent à plusieurs animations. Les atlas partagés peuvent contenir des régions non affichées ; conserver la page évite de dupliquer et réencoder les textures source.

Si seul le ZIP client est disponible, utiliser la chaîne d'extraction SWF du projet BouftouRush décrite dans ASSETS-REFERENCE.md pour reconstituer les cartes et atlas, puis importer son dossier runtime. Le ZIP brut seul ne remplace pas un manifeste préparé : le jeu ne contient pas d'interpréteur Flash.
