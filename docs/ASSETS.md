# Assets du jeu

Les graphismes utilisés sont issus du client **Dofus Retro Remastered 1.40.13**, à l’exception des **icônes de sorts fournies par Xixou**. Ils restent la propriété de leurs ayants droit. Le projet est un fangame gratuit, non affilié à Ankama. Aucune autorisation de redistribution d'Ankama n'est établie ; le présent stockage Git répond à la demande du propriétaire du projet et ne constitue pas une telle autorisation.

## Sorts : référence et icônes Xixou

[Xixou — Sorts](https://xixou.io/sorts/) devient la référence pour tous les sorts du projet ; exemple : [Dissolution, n° 439](https://xixou.io/sorts/439-dissolution/). Le site annonce Dofus Rétro 1.49 : ces icônes ne sont donc pas étiquetées Remastered 1.40.13. Les animations de sorts restent issues du client Remastered.

Les 21 sorts définis dans `assets/data/game.json` sont couverts, y compris Intimidation, encore défini mais absent des raccourcis actuels. `assets/data/xixou-spells.json` relie chaque identifiant interne à l’identifiant Xixou, son nom sur le site, sa page, l’URL du SVG et son empreinte SHA-256. Châtiment correspond à Châtiment Forcé (431).

Les SVG officiels complets (`wp-content/uploads/xixou-sorts-icones/officiel/<id>.svg`) sont téléchargés sous `.local-assets/xixou-spells/`, puis convertis sans recadrage en PNG RGBA de 192 × 192 pixels. Le manifeste référence ces PNG versionnés sous `assets/textures/ui-spell-*/000.png`, utilisés dans la barre de sorts et les indicateurs de buffs. Aucun accès réseau n’est nécessaire en jeu. Les règles et valeurs de combat spécifiques au fangame restent dans `game.json`.

Pour reproduire l’import (CairoSVG nécessaire uniquement à la préparation) :

```sh
.local-tools/python/bin/python scripts/import-spell-icons.py
python3 scripts/import-assets.py --check
```

Lors de l’ajout d’un sort, compléter `SPELL_IDS` dans le script avec l’identifiant confirmé sur Xixou, puis relancer l’import. Les anciennes extractions SWF d’icônes décrites plus bas documentent la provenance historique ; elles sont remplacées par Xixou dans le lot actuel.

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
.local-tools/python/bin/pip install Pillow CairoSVG
.local-tools/python/bin/python scripts/prepare-masks.py
.local-tools/python/bin/python scripts/fix-iop-hair.py --ffdec ../bouftourush/.local-assets/tools/ffdec/ffdec.jar
.local-tools/python/bin/python scripts/import-spell-icons.py
python3 scripts/import-assets.py --check
```

`fix-iop-hair.py` extrait les cinq têtes natives prévues pour une coiffe (frame 2 des symboles de tête du SWF du Iop). Il les place selon les matrices originales de chaque frame, uniquement quand une coiffe est équipée. Cela remplace la tête aux cheveux longs sans supprimer le visage, à l’arrêt, en mouvement et dans le portrait. Les cinq PNG sont versionnés dans `assets/iop-heads/`. Java et JPEXS sont nécessaires uniquement à cette régénération ; les SVG intermédiaires restent dans `.local-assets/iop-heads/`.

`prepare-masks.py` applique les masques de disparition dans le repère source à chaque pièce/accessoire compatible. Les images dérivées sont petites, transparentes et versionnées. Ce précalcul préserve les matrices GAC et évite une génération d'images pendant le jeu.

Le script d'import refuse les chemins sortant du dossier et toute version autre que `1.40.13-remastered`, avec une exception limitée aux icônes `ui.spell.*` de provenance `xixou-official` dont l’URL correspond au relevé Xixou. Il n'importe ni `.server-data/`, ni compte, ni sessions, ni mot de passe. Les fichiers PNG pointés par le manifeste sont copiés une seule fois même s'ils servent à plusieurs animations. Les atlas partagés peuvent contenir des régions non affichées ; conserver la page évite de dupliquer et réencoder les textures source.

Si seul le ZIP client est disponible, utiliser la chaîne d'extraction SWF du projet BouftouRush décrite dans ASSETS-REFERENCE.md pour reconstituer les cartes et atlas, puis importer son dossier runtime. Le ZIP brut seul ne remplace pas un manifeste préparé : le jeu ne contient pas d'interpréteur Flash.

L’icône de Puissance du Iop utilise `clips/spells/icons/up/153.swf` de cette même archive (JPEXS : export frame SVG, puis CairoSVG à 192 px). Les icônes du Sacrieur ont été comparées directement aux frames natives : Dissolution 439, Châtiment Forcé 431, Transposition 438, Punition 446.

Ancestral : monstres `clips/sprites/{1001,1158,1333,1334,1458}.swf`, six icônes `clips/items/{16/174,17/126,1/118,9/123,11/96,10/112}.swf`, coiffe `a1.swf#16_174` et cape `a2.swf#17_126`. Exports JPEXS 26.4.0, timelines et pivots SWF natifs. Carte 8721 : disposition publique non chiffrée du champ mapData du dump de jeu Ancestra, graphismes du seul client Remastered 1.40.13 ; aucun compte ni clé de chiffrement importé. Les cellules actives franchissables de la plus grande composante forment la navigation ; les décors bloquants sont exclus.

Canidés / Meulou : sources Remastered `clips/sprites/{1107,1039,1069,1196,1201,1134}.swf`. Icônes `clips/items/{1/107,9/113,11/85,17/110,10/103,16/144}.swf`, accessoires `a1.swf#16_144` et `a2.swf#17_110`. Timelines et pivots natifs JPEXS 26.4.0 ; accessoires vectoriels CairoSVG densité 3. Carte 9648 : disposition publique mapData du dump de jeu Ancestra et bibliothèques graphiques Remastered, sans comptes ni clés importés. Navigation par cellules franchissables de la plus grande composante, hors décors bloquants. Sources inutilisées et exports intermédiaires conservés sous `.local-assets/meulou/` ; runtime regroupé sous `assets/atlases/meulou/`.

Sram : `clips/sprites/40.swf`, 24 compositions natives (huit directions en repos/course, deux directions pour lancer, buff, réaction et mort), matrices GAC et masques source. Icônes Sournoiserie 61, Piège de Masse 79, Invisibilité 72 et Attaque Mortelle 76 dans `clips/spells/icons/up/`. Animations `402.swf#16` (invisibilité), `412.swf#14` (attaque mortelle) et `410.swf#6` (explosion du piège). Correspondances confirmées dans les tables publiques de sorts du jeu ; graphismes exclusivement Remastered 1.40.13. Les marqueurs de case réutilisent `clips/gfx/cell.swf#s1`. Export JPEXS 26.4.0 et rasterisation vectorielle avec pivots et durées d’origine ; pièces regroupées dans `assets/atlases/sram/`, masques précalculés par `scripts/prepare-masks.py`. Intermédiaires sous `.local-assets/sram-*`.

Gelano : icône native `clips/items/9/47.swf`, correspondance 2469 / type 9 / graphique 47 vérifiée dans les données publiques d’objets. Export vectoriel JPEXS 26.4.0, retrait du seul fond blanc de scène Flash et recadrage transparent sur le dessin, rasterisation à 192 px. Aucun remplacement des graphismes Remastered.

Les effets Sram conservent aussi la timeline principale des clips multiframe, avec leurs frames distinctes. Les particules originales du piège sont disposées radialement dans son rayon de dégâts ; l’effet d’invisibilité est réduit à l’échelle du personnage. Atlas dédiés sous `assets/atlases/sram-fx/`.
