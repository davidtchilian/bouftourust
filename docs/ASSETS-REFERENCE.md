# Assets locaux — état du 2 novembre 2026

Source visuelle choisie à la demande de l’utilisateur : **Dofus Retro Remastered 1.40.13**. Elle remplace la cible initiale 1.29.1 pour cette version du projet ; ce n’est pas une affirmation d’identité exacte avec les graphismes 1.29. Le Remastered remplace le Classic précédemment installé, en conservant les identifiants des personnages, monstres, sorts, objets et décors.

Projet gratuit non commercial, non affilié à Ankama. **Aucune autorisation de redistribution des graphismes n’est établie.** À la demande explicite de l’utilisateur, les PNG, atlas et le manifeste prêts pour le jeu sont désormais versionnés dans `public/assets/`. Les archives, textes décompilés, outils et exports intermédiaires restent locaux et ignorés par Git. Cette décision de stockage ne constitue pas une autorisation des ayants droit. Aucun client n’a été lancé, aucune connexion de compte effectuée, aucune clé de carte récupérée ou contournée.

## Sources effectivement récupérées

| Source | Archive locale | Volume / empreinte SHA-256 |
| --- | --- | --- |
| [Dofedex au commit ec4c08459f73f058294c34baa6baa739f9373753](https://github.com/dofera/dofedex/tree/ec4c08459f73f058294c34baa6baa739f9373753) | `.local-assets/dofedex-master.zip` | 86 364 entrées, 3 152 672 207 octets décompressés ; `35659da292e6bb4dc7762bf204c0527348976202fbb85935400d2fbbdc2523ff` |
| [Client Classic 1.40.13](https://github.com/dofera/dofedex/releases/download/1.40.13/dofus-1.40.13-classic.zip) | `.local-assets/dofus-1.40.13-classic.zip` | 239 372 174 octets ZIP ; `178ca90d276d10dd9c6bd1d8b4ba17f9804da04fc80c52dad4876fa53ce81fcd` |
| [Client Remastered 1.40.13](https://github.com/dofera/dofedex/releases/download/1.40.13/dofus-1.40.13-remastered.zip) | `.local-assets/dofus-1.40.13-remastered.zip` | 474 901 966 octets ZIP ; `aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5` |
| [JPEXS FFDec 26.4.0](https://github.com/jindrapetrik/jpexs-decompiler/releases/tag/version26.4.0) | `.local-assets/tools/ffdec/ffdec.jar` | Outil local de lecture/export SWF, Java 11 utilisé |

Le premier client Classic reste extrait dans `.local-assets/client-1.40.13-classic/` : 20 271 fichiers, dont 20 230 SWF, 9 537 fichiers de carte et 5 948 fichiers dans `clips/items/`. Le catalogue inspecte 1 154 SWF de sprites.

Le client Remastered est extrait dans `.local-assets/client-1.40.13-remastered/` (20 453 entrées ZIP). Les 34 fichiers sources utilisés auparavant existent aux mêmes chemins : 20 diffèrent et 14 sont identiques, dont certaines icônes. Le relevé local `.local-assets/remastered-audit.json` conserve leurs empreintes comparées.

Le dépôt Dofedex complet est extrait dans `/home/david/.cache/bouftourush/ec4c08459f73f058294c34baa6baa739f9373753/dofedex-master/`, accessible par le lien `.local-assets/dofedex`. Le système de fichiers Linux évite la lenteur de 86 000 créations sur NTFS. Les inventaires et empreintes complets restent dans `.local-assets/dofedex-provenance.json`, `.local-assets/client-1.40.13-catalog.json` et `.local-assets/gfx-ui-catalog.json`.

Les fichiers de langue Dofedex ont leurs propres versions de tables, sans certifier une synchronisation exacte avec le client 1.40.13. Les sources ActionScript du loader servent de référence de format ; elles ne sont pas exécutées.

## Registre des graphismes intégrés

Les chemins ci-dessous sont ceux réellement vérifiés sur disque. Le manifeste du jeu associe chaque identifiant sémantique aux frames, régions d’atlas, pivots, échelles, offsets, durées et provenance : `.local-assets/game-runtime/manifest.json`, puis `public/assets/manifest.json` après import.

| Besoin / identifiant | Source originale | Conversion et intégration |
| --- | --- | --- |
| Crâ / `cra.idle.*`, `cra.run.*` | `clips/sprites/90.swf` | Cinq vues natives S/R/F/L/B et miroirs conformes au loader ; huit directions intégrées |
| Crâ cast/buff/hit/death | `clips/sprites/90.swf`, symboles `anim0R/L`, `anim10R/L`, `hitR/L`, `dieR/L` | Tir à l’arc `anim10R/L`, incantation du buff `anim0R/L`, hit/death ; timelines intégrées |
| Bouftou / `mob.1566.*` | `clips/sprites/1566.swf` | Idle, run, attack, hit, death, vues R/L |
| Bouftons / `mob.1570.*`, `mob.1571.*` | `clips/sprites/1570.swf`, `1571.swf` | Mêmes cinq états ; variantes jeunes d’Incarnam définies comme archétypes séparés |
| Chef / `mob.1573.*` | `clips/sprites/1573.swf` | Cinq états intégrés |
| Royal / `mob.1574.*` | `clips/sprites/1574.swf` | Cinq états intégrés ; boss Bouftou |
| Milimilou / `mob.1581.*` | `clips/sprites/1581.swf` | Cinq états intégrés ; `monsters_fr_1160.swf` : `M[1001]`, nom Milimilou, `g:1581`. Le XML le nomme « Milou » ; ne pas confondre avec Milimulou `1107.swf` |
| Chafer Prépubère / `mob.1014.*` | `clips/sprites/1014.swf` | Cinq états ; identité d’Incarnam confirmée dans la table monstres |
| Crabe / `mob.1009.*` | `clips/sprites/1009.swf` | Cinq états ; monstre du Donjon Ensablé |
| Étoile de la Mer d’Asse / `mob.1474.*` | `clips/sprites/1474.swf` | Cinq états ; monstre du Donjon Ensablé |
| Pichons / `mob.1483.*` à `mob.1488.*` | `clips/sprites/1483.swf`…`1488.swf` | Idle, course, attaque, hit, death ; `1483` sans `anim2`, attaque via `anim0` |
| Raul Mops / `mob.1481.*` | `clips/sprites/1481.swf` | Cinq états ; garde du boss Ensablé |
| Mob l’Éponge / `mob.1546.*` | `clips/sprites/1546.swf` | Boss du Donjon Ensablé ; absence de symboles `run`, repli sur `walk` |
| Magique / `spell.magic.*` | `clips/spells/902.swf`, `move`, `shoot` | Projectile et impact animés |
| Enflammée / `spell.flaming.*` | `clips/spells/909.swf`, sprites internes 22 et 41 | Ligne de lancement et impact animés, placement/direction par le moteur |
| Puissant / `spell.power.cast` | `clips/spells/904.swf`, sprite interne 14 | Effet de buff original, sprite 14 vérifié dans le Remastered ; déplacement repris après l’incantation |
| Explosive / `spell.explosive.*` | `clips/spells/910.swf`, `move`, `shoot` | Projectile et explosion animés |
| Cadres/boutons/slots / `ui.window`, `ui.button`, `ui.slot` | `modules/core.swf`, `UI_InventoryGridBackground`, character ID 1226 | Cellule originale 32 × 32, composée en neuf zones ; aucune couleur de remplacement dessinée |
| Vie / `ui.heart` | `modules/core.swf`, `Heart`, character ID 872, première pose | Icône originale du HUD ; Vie/cooldowns/durée du buff en texte |

Les correspondances sort → animation sont confirmées dans les lignes des sorts 161/165/166/179 du [fichier de données Ancestra-Evolutive](https://github.com/Romain-P/Ancestra-Evolutive/blob/master/sql/ancestra-evolutive-full.sql), table `sorts`, champ `sprite` : respectivement 902/909/904/910. Cette source sert à la correspondance d’identifiants ; **les graphismes restent ceux du client Dofedex**. Le dump téléchargé reste privé sous `.local-assets/ancestra.sql` et n’est pas exécuté.

L’identité du boss Incarnam est corroborée par les données locales Milimilou et le [guide du donjon pour Dofus 1.29](https://guidedofus.com/donjon-dincarnam/). Les groupes et statistiques du fangame restent des adaptations, sans prétention canonique.

### Cartes, sols et décors

| Carte | Donnée publique non chiffrée utilisée | État |
| --- | --- | --- |
| Astrub zaap 7411 | Dofedex `maps/7411/7411_0612131401.swf` | 89 couches, 293 cellules de navigation, départ devant le zaap |
| Incarnam 10360 | Dofedex `maps/10360/10360_0703281543.swf` | Arène de 162 couches, 148 cellules de navigation et huit points d’apparition |
| Bouftou 2073 | Dofedex `maps/2073/2073_0505170920.swf` | Arène de 288 couches, 202 cellules de navigation et huit points d’apparition |
| Ensablé 10155 | Dofedex `maps/10155/10155_0701221531.swf` | Arène de 153 couches, 186 cellules de navigation et huit points d’apparition ; variante `X` chiffrée non utilisée |

Les bibliothèques de dessin viennent de `clips/gfx/g1.swf`, `g2.swf`, `o1.swf`…`o12.swf`, selon l’identifiant de chaque cellule. Le fond Astrub est le symbole `71` de `g1.swf` (character ID 513) ; ce fond seul n’est pas la carte. Les objets, flips et rotations isométriques sont reconstruits depuis les données. Tous les décors restent figés sur leur première frame, végétation comprise, à la demande de l’utilisateur. Le tri respecte les couches sol/objet1/objet2 et la cellule des pieds.

Les tables `maps_fr_366.swf` et `hints_fr_1166.swf` associent Astrub 7411 à `[4,-19]`, Cité d’Astrub, et au zaap. La table des donjons `dungeons_fr_1170.swf` associe les salles 2073…2077 et 2079…2084 au donjon Bouftou. L’entrée Incarnam 10359 est identifiée par les hints. Le gameplay utilise une salle comme arène de vagues par donjon, sans reproduire le parcours des salles originales.

Les variantes récentes suffixées `X`, dont `data/maps/7411_0711291819X.swf`, contiennent des données chiffrées. **Elles ne sont pas déchiffrées par le projet.** Les dispositions historiques sont distinctes de ces cartes récentes et ne sont pas présentées comme une reproduction exacte.

### Objets et panoplies

Identités et liens `t`/`g` vérifiés dans `lang/fr/items/items_fr_450.swf`. Les statistiques et drops du fangame sont définis séparément dans `src/data/items.ts` et `src/data/monsters.ts`.

| Objet / identifiant | Source d’icône | État |
| --- | --- | --- |
| 2411 Coiffe du Bouftou | `clips/items/16/41.swf` | Icône, slot, stats, panoplie intégrés |
| 2414 Cape Bouffante | `clips/items/17/44.swf` | Idem |
| 2416 Marteau du Bouftou | `clips/items/7/23.swf` | Idem |
| 2419 Anneau de Bouze le Clerc | `clips/items/9/44.swf` | Idem, deux slots d’anneau |
| 2422 Boufbottes | `clips/items/11/26.swf` | Idem |
| 2425 Amulette du Bouftou | `clips/items/1/48.swf` | Idem |
| 2428 Ceinture du Bouftou | `clips/items/10/29.swf` | Idem |
| 2473 Cape de l’Aventurier | `clips/items/17/48.swf` | Icône, slot, stats, panoplie intégrés |
| 2474 Chapeau de l’Aventurier | `clips/items/16/44.swf` | Idem |
| 2475 Anneau de l’Aventurier | `clips/items/9/48.swf` | Idem |
| 2476 Bottes de l’Aventurier | `clips/items/11/32.swf` | Idem |
| 2477 Ceinture de l’aventurier | `clips/items/10/33.swf` | Idem |
| 2478 Amulette de l’Aventurier | `clips/items/1/52.swf` | Idem |
| 8253 Amulette en Mousse | `clips/items/1/106.swf` | Icône, slot, stats, panoplie Ensablé intégrés |
| 8254 Anneau en Mousse | `clips/items/9/112.swf` | Idem, deux slots d’anneau |
| 8255 Pelle en Mousse | `clips/items/8/44.swf` | Idem |
| 8256 Bottes en Mousse | `clips/items/11/84.swf` | Idem |
| 8257 Ceinture en Mousse | `clips/items/10/102.swf` | Idem |
| 8258 Cape en Mousse | `clips/items/17/109.swf` | Icône et panoplie Ensablé ; portée via `a2.swf#17_109` |
| 8259 Coiffe en Mousse | `clips/items/16/143.swf` | Icône et panoplie Ensablé ; portée via `a1.swf#16_143` |

Le bouclier 7077 de la panoplie n’est pas proposé, faute d’emplacement bouclier. Les huit emplacements visibles sont coiffe, cape, amulette, deux anneaux, ceinture, bottes et arme. Les coiffes et capes sont visibles sur Crâ : `a1.swf#16_41`, `16_44`, `16_143` et `a2.swf#17_44`, `17_48`, `17_109` dans `clips/sprites/accessories/`. Les matrices et profondeurs des clips `GAC.applyAccessory` servent à la composition, avec vues directionnelles et cycles de course de cape. Le tir utilise l’arc original `a4.swf#2_1` ; la table d’objets associe les arcs à `an:10`. Les icônes sont recadrées sur le dessin original avec fond transparent, au lieu d’afficher la scène Flash entière. Le slot familier n’est pas proposé. Les icônes des quatre sorts sont intégrées depuis `clips/spells/icons/up/{161,165,166,179}.swf` dans cette même archive Remastered. La précédente recherche omettait le sous-dossier `up` ; il ne s’agissait pas de ressources absentes.

## Préparation et validation reproductibles

Prérequis : Node/pnpm, Python 3.11+ (3.12 utilisé), Java 11+, JPEXS 26.4.0 et Chromium Playwright. Aucune bibliothèque Python externe.

Arborescence attendue :

```text
.local-assets/
  client-1.40.13-remastered/       # clips/, modules/, data/ à ce niveau
  dofedex/                    # maps/, lang/, loader/ à ce niveau, dossier ou lien local
  tools/ffdec/ffdec.jar
```

Sur un nouveau clone, fournir localement des sources dont vous êtes autorisé à faire cet usage, puis extraire aux chemins ci-dessus. Le dépôt Dofedex peut être réacquis avec les liens des sources ou le ZIP du commit : `https://codeload.github.com/dofera/dofedex/zip/ec4c08459f73f058294c34baa6baa739f9373753`. L’empreinte du ZIP du commit peut différer de celle du ZIP master même si son contenu correspond ; vérifier le commentaire de commit et les fichiers.

```sh
pnpm install
pnpm exec playwright install chromium
pnpm assets:prepare:game
pnpm assets:import .local-assets/game-runtime
pnpm assets:validate
pnpm dev
```

La préparation complète exporte les séquences de jeu, pièces d’équipement indépendantes et aperçus UI, puis les assemble dans des atlas 2048 × 2048. Le manifeste installé indique le nombre exact de séquences, de frames et les pages utilisées. Les icônes d’objets et éléments UI restent en PNG individuels. La composition conserve les graphismes originaux ; les bordures d’atlas sont dupliquées pour éviter les fuites UV.

Les étapes sont `prepare-hub.py`, `prepare-game.py`, `prepare-appearance.py` (entrée de `prepare-composition.py`), `rasterize-appearance.ts`, `prepare-ui-previews.py`, `rasterize-appearance.ts --previews`, puis `pack-atlas.ts` pour le jeu et avec `--previews` pour les portraits. Les pivots sont extraits des transformations SVG originales ; les arrêts littéraux `stop()` sont pris en compte pour éviter les queues vides. Les vues natives et leurs miroirs sont conservés, sans inventer des directions inexistantes. Les caches de conversion sont invalidés par les empreintes des fichiers et les paramètres d’export. Le cache de rasterisation vérifie les empreintes des SVG et des PNG produits ; il ne considère pas les icônes brutes recopiées par le préparateur comme déjà recadrées.

L’import valide d’abord les sources, prépare un dossier intermédiaire et ne copie que les fichiers référencés dans le manifeste. `.local-assets/` reste ignoré. Le lot converti de `public/assets/` est versionné à la demande explicite de l’utilisateur pour faciliter les nouvelles installations ; ses mentions de provenance et restrictions de redistribution restent inchangées. Aucun binaire n’est enregistré en localStorage.

### Inspection et exports de recherche

Ces commandes restent disponibles indépendamment du préparateur de jeu :

```sh
pnpm assets:inspect .local-assets/dofus-1.40.13-remastered.zip --output .local-assets/client-1.40.13-catalog.json
pnpm assets:export .local-assets/client-1.40.13-remastered --client-version 1.40.13-remastered
```

Le premier lot de recherche, `.local-assets/exports-1.40.13/`, contient 46 séquences et 605 PNG avec pivots non résolus. Ce manifeste de recherche n’est pas le manifeste de jeu. `assets:prepare` reste disponible pour préparer seulement le hub de base ; l’importer remplace le jeu complet par ce jalon limité.

### Contrat du manifeste

`src/game/assets/manifest.ts` décrit les frames, durées, pivot normalisé depuis le coin supérieur gauche, taille en unités monde, offset, bouclage, flip, et provenance source/version/fichier/conversion/conditions d’usage. Une frame d’atlas contient `region: [x, y, largeur, hauteur, largeurPage, hauteurPage]` en pixels. Sans région, le PNG entier est utilisé.

Les cartes contiennent bornes de cadrage, spawn joueur, couches ordonnées, cellules de navigation et spawns ennemis. Les coordonnées Y montent dans le monde ; les données de carte d’origine sont converties depuis leur axe Y descendant. Les bornes visuelles seules ne sont pas une collision.

La classification des objets et les décisions de navigation sont décrites dans `scripts/map-navigation.json`, puis appliquées aux préparateurs d’Astrub et des donjons avant le choix des spawns. Le lot runtime versionné contient les mêmes corrections. `assets:validate` vérifie aussi cette navigation avec Python 3 : objets connus, décisions par cellule, empreinte du placement, obstacles bloquants et exclusions de bord. Les graphismes, leur disposition et leurs provenances restent inchangés. Voir [la revue des collisions](NAVIGATION.md) pour les critères et les résultats par carte.

Le contrat minimal exige Astrub et idle/run Crâ dans huit directions. Avec `gameplayReady: true`, les cartes, monstres, sorts, icônes et éléments UI de ce MVP sont également obligatoires. Le validateur vérifie schéma, régions, chemins relatifs, fichiers locaux, signature PNG et provenance déclarée ; il ne certifie ni droits ni authenticité artistique.

## Limites visuelles restantes

Le moteur joue les graphismes et timelines convertis, sans émulation complète d’ActionScript. Les particules aléatoires pilotées par code, sons et personnalisations de couleurs ne sont pas reproduits intégralement. Les cartes restent des reconstructions historiques avec bibliothèques Remastered 1.40.13. Aucun graphisme de remplacement, art généré, emoji ou forme procédurale visible n’est utilisé pour masquer ces limites.

## Aperçus des nouvelles interfaces

- `ui.portrait` : `modules/core.swf#LookSelectorBackground` (1109), cadre original des portraits.
- `ui.selection` : `modules/core.swf#UI_InventoryGridHighlight` (1224), sélection originale de grille.
- `preview.cra.idle.*` et `preview.appearance.*.cra.idle.*` : mêmes vecteurs du Crâ et accessoires, rasterisés à six fois la définition de base, en huit directions.
- `preview.mob.1574.{idle,attack}.R` et `preview.mob.1581.{idle,attack}.R` : vecteurs originaux du Bouftou Royal et du Milimilou, mêmes animations que le jeu.

`prepare-ui-previews.py`, puis `rasterize-appearance.ts --previews`, préparent les séquences de portraits. Le script `assets:prepare:game` inclut ces étapes. Les aperçus utilisent des découpes CSS des atlas originaux, sans dessin de remplacement ni second contexte WebGL. Les pages nécessaires sont préchargées avant l’animation, qui respecte la réduction des animations du navigateur. Les textures des aperçus ne sont pas chargées par le moteur de jeu.

## Boutons orange Retro

Les boutons d’action utilisent `modules/core.swf#ButtonNormalUp` (31), `ButtonNormalDown` (27) pour le survol/appui. Identifiants : `ui.button`, `ui.button.hover`, `ui.button.down`. `compose-ui-buttons.ts` repositionne les trois parties originales gauche/milieu/droite, puis rasterise à six fois la définition de base pour une découpe CSS en trois zones. Aucun dégradé ou bouton de substitution n’est dessiné. Les cellules du sac et d’équipement conservent leurs vrais cadres d’inventaire ; les filtres, actions, entrées de donjon et fermetures utilisent le skin orange.

## Exclusivité Remastered et qualité obligatoire

Le manifeste refuse désormais toute provenance déclarée autre que `1.40.13-remastered`, avec un diagnostic explicite. Le lot actuellement installé a été contrôlé : chaque ressource déclare cette édition. Cela ne dispense pas de vérifier l’archive et les chemins d’origine : une déclaration de version seule ne certifie pas l’authenticité. Le Classic conservé pour l’historique n’est jamais un recours d’import.

Les aperçus UI exigent leurs propres exports haute définition `preview.*` ; ils ne reviennent plus silencieusement au petit sprite de jeu agrandi. Les gardiens sont déterminés depuis les données des monstres, y compris Mob l’Éponge, et leurs vecteurs Remastered sont exportés à six fois la définition de base. Un aperçu manquant produit un diagnostic et exige une nouvelle préparation.

Le préparateur vérifie aussi l’empreinte du sprite Crâ du client source contre celle de l’archive Remastered 1.40.13 vérifiée. Un client Classic complet est refusé avant l’export ; cette ancre ne remplace pas le contrôle de provenance de chaque fichier et de l’archive.

## Installation depuis Git

Le lot préparé (`public/assets/manifest.json`, PNG et atlas) est inclus dans le dépôt à la demande explicite de l’utilisateur. Après `git clone`, `pnpm install` puis `pnpm dev` suffisent pour jouer. La reconstruction depuis les SWF est facultative ; archives clients, Dofedex extrait, JPEXS et caches de conversion restent locaux. Cette inclusion dans Git n’établit aucun droit de redistribution des graphismes.

## Migration vers les pièces indépendantes (novembre 2026)

Les sources restent exclusivement celles de l’archive Remastered vérifiée en tête de ce document : `clips/sprites/90.swf`, `clips/sprites/accessories/a1.swf#16_41`, `#16_44`, `#16_143`, `a2.swf#17_44`, `#17_48`, `#17_109` et `a4.swf#2_1`. Aucune archive Classic n’est une entrée de cette migration.

`prepare-composition.py` exporte les vues et cycles des accessoires individuellement. Le corps est découpé en portions consécutives de l’arbre d’affichage, séparées aux attaches et aux changements de visibilité des cheveux. Les SVG de ces portions sont dédupliqués par empreinte ; leurs durées sont portées par la timeline du corps. Les matrices natives viennent des clips identifiés par les scripts `GAC.applyAccessory`, avec leur label directionnel, et sont cumulées dans la hiérarchie originale. L’ordre des couches constitue leur profondeur relative.

Les paramètres de rasterisation sont une densité 3 pour le jeu et 10 pour les pièces du portrait, toujours depuis les vecteurs. Le cadrage transparent conserve un pixel natif de marge, le pivot natif et la taille en unités du monde. Les accessoires conservent leurs cycles imbriqués. Les masques de disparition sont exportés depuis les chemins de découpe originaux ; aucun masque dessiné à la main n’est ajouté. JPEXS omet certaines références aux clips vides : pour l’effet `90.swf#284`, l’import vérifie directement que la frame d’arrêt 13 est vide avant d’accepter l’absence de dessin.

Le manifeste contient `composition.actors`, `composition.accessories` et les images `piece.body.*`, `piece.accessory.*`, `piece.mask.*`, avec leurs équivalents HD nécessaires `preview.piece.*`. Les références cartésiennes `appearance.*` et `preview.appearance.*` sont supprimées du lot publié. Les anciens SVG composés restent uniquement dans `.local-assets/` pour les comparaisons.

Pour régénérer seulement cette migration sur un export de jeu existant :

```sh
python3 scripts/prepare-appearance.py
node --import tsx scripts/rasterize-appearance.ts
python3 scripts/prepare-ui-previews.py
node --import tsx scripts/rasterize-appearance.ts --previews
pnpm assets:compare
node --import tsx scripts/pack-atlas.ts .local-assets/game-runtime
node --import tsx scripts/pack-atlas.ts .local-assets/game-runtime --previews
pnpm assets:import .local-assets/game-runtime
pnpm assets:validate
```

Le cache de scripts Crâ est vérifié par SHA-256 et réexporté automatiquement si absent ou périmé. Le pipeline complet reste `pnpm assets:prepare:game`. Les restrictions de redistribution et notices initiales s’appliquent à toutes ces pièces.

### Mesures de la migration

Mesures locales de novembre 2026, sur le même lot de cartes et monstres ; Mo désigne ici 1 000 000 octets. Les nombres de séquences ne représentent pas la taille des images : le découpage ajoute des références de petites pièces, mais retire les images répétées des associations.

| Mesure | Avant | Après |
| --- | ---: | ---: |
| Images PNG compressées, toutes ressources | 91,22 Mo | 43,23 Mo |
| Mémoire RGBA théorique de toutes les images | 791,14 Mo | 371,71 Mo |
| Frames référencées | 21 814 | 12 869 |
| Séquences cartésiennes `appearance.*` et `preview.appearance.*` | 480 | 0 |
| Fichiers PNG | 76 | 51 |

Le lot final contient 2 295 séquences/pièces, 24 timelines du Crâ, six équipements visuels et un arc. Il utilise 14 atlas de jeu et huit atlas de portraits de 2048 × 2048. Dans un combat avec trois ennemis, Three.js charge 13 textures d’atlas, soit environ 218,10 Mo RGBA théoriques sans mipmaps ; les atlas de portrait restent hors du moteur 3D. Le Crâ utilise cinq quads dans la pose mesurée, au plus neuf couches source selon la frame et l’équipement. La scène mesurée totalise 296 quads/appels de dessin, décors compris.

Avec Chromium headless, en exécution isolée : chargement local jusqu’au menu en 0,96 s, heap JS de 41,07 Mo, intervalle médian et 95e percentile des frames à 16,7 ms sur 120 frames. Ces chiffres décrivent cet environnement de test et son cache système ; ils ne garantissent pas le débit d’un téléphone ou d’un autre GPU. Un premier relevé avec plusieurs tests navigateur simultanés était plus lent et n’est pas utilisé comme mesure isolée.

L’import validé du répertoire préparé prend 8,00 s ; la reconstruction des atlas prend 11,75 s pour le jeu et 5,97 s pour les portraits. La dernière rasterisation avec cache déjà alimenté prend 74,78 s, avec un maximum RSS mesuré de 187 552 Ko pour le processus ; ce n’est pas une mesure d’export complet à froid ni de mémoire cumulée de tous les processus Chromium.

Le test de croissance résout les 2 500 associations de 50 coiffes et 50 capes via 100 entrées indépendantes sans générer d’images supplémentaires. Ce test réutilise les pièces vérifiées : il vérifie l’architecture et ne prétend pas intégrer 100 équipements originaux distincts. Pour de nouvelles pièces réelles, taille et chargement croissent avec leurs seules vues/cycles.

Les comparaisons locales vérifient les poses initiales, intermédiaires et finales, ainsi que les frames portant un masque de mort, sur les associations disponibles. Les rapports détaillés et captures restent dans `.local-assets/composition-validation/` ; les captures d’inventaire ordinateur/petit écran et de mort Three.js sont également privées. La tolérance de comparaison est de 3 % de différence normalisée des couleurs prémultipliées et de l’alpha, pour tenir compte de l’anticrénelage des pièces rasterisées séparément. La normalisation a un plancher de 256 pixels opaques au zoom de comparaison ×3 : les derniers résidus subpixel de disparition gardent ainsi une tolérance absolue inférieure à huit pixels RGBA équivalents, plutôt qu’un pourcentage instable sur une silhouette presque vide.


Validation finale : 1 056 comparaisons, écart normalisé moyen de 0,589 % et maximum de 1,671 %, sous le seuil de 3 %. Le résidu ayant initialement un pourcentage brut élevé en fin de mort couvre moins de 0,02 pixel opaque équivalent ; son erreur absolue est de 0,011 pixel équivalent. Les poses visibles et les frames masquées passent également dans le renderer Three.js, sans erreur shader. Les vérifications couvrent 40 tests Vitest, sept tests Python, dix scénarios navigateur (dont les trois donjons) et un scénario hors connexion. Les captures de portrait ordinateur et 390 × 844 ont été examinées visuellement.


## Icônes des sorts AZER

Les sources sont acquises dans l’archive Remastered SHA-256 `aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5` déjà référencée ci-dessus. `prepare-spell-icons.py` compare chaque fichier local à son entrée ZIP avant export ; aucune ressource Classic n’est utilisée. Les identifiants et noms sont confirmés par les entrées `S[161]`, `S[165]`, `S[166]` et `S[179]`, champ `i.up`, de la table locale `spells_fr_1166.swf`.

| Sort | Chemin original dans le client Remastered | SHA-256 du SWF | Identifiant runtime |
| --- | --- | --- | --- |
| Flèche explosive | `clips/spells/icons/up/179.swf` | `0e71bc46c0213f183af1776661cc7563c40cfdbef8f47d817ba9d0d755c99895` | `ui.spell.explosive` |
| Flèche enflammée | `clips/spells/icons/up/165.swf` | `08ea62f087080c3d681c31780ac5ab5d920d4e11e5357821e2453d8a17f2d643` | `ui.spell.flaming` |
| Flèche magique | `clips/spells/icons/up/161.swf` | `89e5d5c5205d87834788fc837af6ff682eef64c1208defab62aa2d0fb34ad570` | `ui.spell.magic` |
| Tir puissant | `clips/spells/icons/up/166.swf` | `35d400ebd7765bc6406069a4857ffc4d83828fa4e9ccbe041cd541dcdbd4a946` | `ui.spell.power` |

JPEXS exporte la première frame en SVG ; `rasterize-appearance.ts --spell-icons` recadre au dessin et rasterise les vecteurs jusqu’à 192 px par côté. Les couleurs natives et le fond beige du dessin sont conservés. Les quatre PNG individuels sont sous `public/assets/textures/ui-spell-*/000.png`, sans intermédiaire SWF publié. Le fond de disponibilité est le skin orange `ButtonNormalUp` original ; le fond de recharge est `UI_InventoryGridBackground`, avec texte blanc/noir et temps restant. Les icônes ne sont ni recolorées ni remplacées par un dessin de substitution.

Régénération : `pnpm assets:spells`, puis `pnpm assets:import .local-assets/game-runtime` et `pnpm assets:validate`. Ces étapes figurent aussi dans `assets:prepare:game`. Le manifeste exige les quatre icônes lorsque le jeu est déclaré prêt. Les restrictions de redistribution restent celles documentées au début de ce fichier.


### Habillage d’interface moderne — 2 novembre 2026

Sur demande explicite du joueur, les couleurs, fonds, cadres et boutons de l’interface sont maintenant définis en CSS. Cela remplace la contrainte antérieure d’assembler exclusivement les éléments de skin Retro pour l’interface. Les anciens exports `ui.window`, `ui.button`, `ui.slot`, `ui.portrait`, `ui.selection` et variantes de boutons sont conservés dans le manifeste pour compatibilité et régénération, mais ne sont plus appliqués comme habillage dans React. Les icônes de sorts, cœur, objets et aperçus animés restent issus du lot Remastered documenté ci-dessus. Aucun fichier graphique tiers nouveau n’a été importé. Les règles de provenance et de redistribution des illustrations restent inchangées.

Le favicon utilise directement `textures/item-2411/000.png` (Coiffe du Bouftou), déjà déclaré dans le manifeste comme `item.2411`, sans réexport ni copie. Source Remastered : `clips/items/16/41.swf#timeline`.

### Panoplie Bouftou Royal — ajout du 4 novembre 2026

Archive Remastered réacquise et empreinte SHA-256 revérifiée : `aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5`. Identités, noms et couples type/graphisme vérifiés dans `lang/fr/items/items_fr_450.swf` de Dofedex au commit déjà référencé. Aucun graphisme Classic utilisé.

| Objet | Icône Remastered | Accessoire porté |
| --- | --- | --- |
| 2438 Boufcoiffe Royale | `clips/items/16/42.swf` | `clips/sprites/accessories/a1.swf#16_42` |
| 2440 Epée Royale du Bouftou | `clips/items/6/30.swf` | Emplacement arme ; tir avec l’arc original existant |
| 2441 Anneau Royal du Bouftou | `clips/items/9/46.swf` | — |
| 2442 Boufbottes Royales | `clips/items/11/30.swf` | — |
| 2443 Amulette Royale du Bouftou | `clips/items/1/50.swf` | — |
| 2444 Ceinture Royale du Bouftou | `clips/items/10/31.swf` | — |
| 2445 Cape Bouffante Royale | `clips/items/17/46.swf` | `clips/sprites/accessories/a2.swf#17_46` |

Exports JPEXS 26.4.0 puis `rasterize-appearance.ts` : icônes recadrées sur le dessin transparent, côté maximal de 192 pixels ; accessoires en pièces indépendantes à densité 3, aperçus d’inventaire à densité 10 depuis les vecteurs. Les directions, durées, matrices d’attache, profondeurs et masques utilisent la composition Crâ existante. Les sorties runtime et leur provenance sont dans `public/assets/manifest.json` et `public/assets/textures/`.

La préparation complète inclut ces fichiers. Pour compléter uniquement un lot runtime existant, recopier ce lot sous `.local-assets/game-runtime/`, puis exécuter :

```sh
python3 scripts/prepare-game.py --royal-only
python3 scripts/prepare-composition.py --royal-only
pnpm exec tsx scripts/rasterize-appearance.ts
pnpm assets:import .local-assets/game-runtime
pnpm assets:validate
```

Les mêmes prérequis locaux et restrictions de redistribution s’appliquent. Aucune nouvelle ressource nécessaire à ces sept pièces ne manque.

### Lance de Raul Mops — 4 novembre 2026

Source : `clips/sprites/1481.swf` du client **Remastered 1.40.13**, comparée octet par octet à l’archive dont le SHA-256 est indiqué plus haut. L’inspection visuelle des SVG confirme : `anim0R/L` est une incantation, `anim2R/L` un coup de lance et **`anim1R/L` le lancer**. Le projectile reprend exclusivement le symbole interne **4**, la lance effectivement tenue dans cette animation (présente jusqu’à la frame 33, retirée à la frame 34). Aucun projectile de remplacement ni graphisme généré.

`prepare-enemy-abilities.py` exporte `mob.1481.throw.R/L` avec les 72 frames natives à 40 images/s et `spell.raulMops.spear` depuis le vecteur original. JPEXS 26.4.0, rasterisation à densité 3, transparence et recadrage commun des timelines ; lance à l’échelle source 0,8, tournée de 90° pour pointer à droite avant orientation par le moteur, pivot centré et décalage visuel vertical de 30 unités au-dessus de sa trajectoire au sol. Le symbole de lance a une seule frame dans le fichier original. Les durées natives restent dans le manifeste ; le jeu adapte le geste à sa fenêtre de 0,9 seconde.

La préparation complète inclut ces exports avant la création des atlas. Pour compléter un lot existant recopié sous `.local-assets/game-runtime/` :

```sh
pnpm assets:prepare:enemies
pnpm assets:import .local-assets/game-runtime
pnpm assets:validate
```

Le script vérifie les sources locales et n’effectue aucun téléchargement. Les intermédiaires SVG et empreintes de contrôle restent sous `.local-assets/game-runtime/_appearance/`. Le manifeste exige les deux gestes animés et le projectile lorsque le jeu est déclaré prêt. Aucun graphisme supplémentaire n’est nécessaire pour l’accélération du Royal ; son animation de course originale reste utilisée, avec un indicateur textuel.

### Extension Tofu / Scarafeuille Noir — état historique avant intégration

Le catalogue `scripts/tofu-scara-assets.json` relève les identités de la table
locale `items_fr_450.swf`, export `I.u` : panoplie Tofu (50), objets
8108–8114 ; panoplie Scarafeuille Noir (49), objets 8132, 8133, 8134 et 8136.
Les onze chemins `clips/items/<type>/<g>.swf` ont été vérifiés dans le client
Remastered local. `python3 scripts/prepare-tofu-scara.py` exporte leurs icônes
avec le préparateur existant (première frame, transparence et cadrage vectoriel),
après validation de la source Remastered ; le manifeste local conserve les
chemins et empreintes calculés par ce préparateur. Cela n'active ni butin ni objets.

Constat sur cette installation : `.local-assets/dofedex` et le cache historique
mentionné plus haut sont absents. Leur présence décrite dans ce document ne vaut
pas vérification de l'environnement courant. Les dispositions publiques des
arènes Tofu et Scarafeuille restent à identifier et importer ; ne pas réutiliser
une arène sans rapport ou déchiffrer les cartes récentes. Il reste à vérifier les
identités et animations des monstres/boss, exporter les accessoires portés et les
aperçus, revoir la navigation par cellule, intégrer les vagues et le butin, puis
implémenter et tester les bonus de vitesse et l'équilibrage du tableau TODO.
Les donjons et panoplies ne sont donc pas encore livrés dans le jeu.

### Réinstallation locale Dofedex du 4 novembre 2026

Le dépôt au commit `ec4c08459f73f058294c34baa6baa739f9373753` a été téléchargé
via `https://codeload.github.com/dofera/dofedex/zip/ec4c08459f73f058294c34baa6baa739f9373753`
et décompressé directement dans `.local-assets/dofedex/` (86 364 entrées,
3 152 672 207 octets décompressés). L'archive est `.local-assets/dofedex-source.zip` ;
son empreinte est enregistrée dans `.local-assets/dofedex-provenance.json`.
Ce constat remplace l'absence locale signalée dans la section précédente.
Le client graphique reste exclusivement Remastered 1.40.13, archive SHA-256
`aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5`, revérifiée.
`.gitignore` exclut `.local-assets/`, toutes les archives ZIP, et protège également
`/dofedex/` et `/dofedex-*/` en cas d'extraction à la racine.

Dispositions publiques identifiées : Tofu `maps/9529/9529_0609131545.swf`
(table MA.m, sous-zone 208, salle 8), Scarafeuille
`maps/9782/9782_0610211614.swf` (groupe Scarabosse Doré dans les données locales
Ancestra). Les variantes `X` ne sont pas utilisées. Graphismes reconstruits
uniquement depuis `client-1.40.13-remastered/clips/gfx/`.
Les sprites 1558 (Tofu), 1462 (Tofu Noir), 1380 (Batofu), 1135–1138
(Scarafeuilles colorés), 1453 (Noir), 1455 (Scarabosse Doré) existent et exposent
les animations static/run/anim0/hit/die R et L. Accessoires confirmés :
`a1.swf#16_128`, `a1.swf#16_132`, `a2.swf#17_97`, `a2.swf#17_100`.
Leur présence ne constitue pas à elle seule une validation visuelle du runtime.

Empreinte SHA-256 de l’archive Dofedex téléchargée :
`627b0cd8d92bdc693a30d247e242c5dd40ff5aa7bbb970aacf6fbaad480cf2b9`.


### Intégration jouable Tofu / Scarafeuille Noir — 5 novembre 2026

Le lot runtime de cette extension est préparé depuis les sources locales Remastered
1.40.13 (archive SHA-256
`aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5`).
Les constats « non jouable » ci-dessus décrivent l'étape de préparation antérieure.
Les dispositions publiques restent les fichiers 9529 et 9782 identifiés ci-dessus ;
aucune donnée protégée n'est déchiffrée. Les bibliothèques `clips/gfx/` Remastered
fournissent tous les graphismes, figés à leur première frame.

- Tofu : 142 cellules navigables, quatre décisions de décor traversable et six
  exclusions de bord ; huit positions d'apparition dans la composante du spawn.
- Scarafeuille : 178 cellules navigables, 24 décisions de décor traversable et 21
  exclusions de bord ; huit positions d'apparition dans la composante du spawn.
- La revue de `scripts/map-navigation.json` distingue cages, caisses, piliers,
  troncs et rochers des herbes, fleurs, paille, petits débris et racines suspendues.
  Les œufs décoratifs ne créent pas d'obstacle implicite. Les limites source restent
  fermées. Les mêmes cellules servent au joueur, aux monstres et aux projectiles.
- Sprites : `clips/sprites/{1558,1462,1380,1135,1136,1137,1138,1453,1455}.swf`,
  symboles `static`, `run`, `anim0`, `hit`, `die`, directions R/L, timelines complètes.
  `prepare-dungeon-vectors.py` réexporte les vecteurs à densité 3 pour le jeu,
  à densité 2 pour les décors. Les portraits des boss sont exportés à densité 6.
- Les onze icônes suivent les chemins exacts de `scripts/tofu-scara-assets.json`,
  recadrage transparent au dessin original, dimension maximale de 192 pixels.
- Accessoires : `clips/sprites/accessories/a1.swf#16_128` (8114), `#16_132`
  (8133), `a2.swf#17_97` (8112), `#17_100` (8134). Les pièces indépendantes
  gardent les attaches GAC, les vues directionnelles et les timelines animées,
  densité 3 en jeu et 10 pour les pièces des portraits.

Le rasteriseur mesure chaque SVG séparément : les identifiants de symboles JPEXS
réutilisés entre frames ne doivent pas fausser le cadrage commun de l'animation.
Les sorties sont les séquences `mob.*`, `map.*`, `item.*`, `piece.accessory.*`
et leurs aperçus ; le manifeste conserve les chemins SWF et symboles originaux.
Les atlas PNG sont ensuite importés dans `public/assets/`.

Pour compléter un export local existant, avec Java/JPEXS, Chromium et les deux
sources locales installés comme indiqué plus haut :

```sh
pnpm assets:prepare:tofu-scara
pnpm assets:import .local-assets/game-runtime
pnpm assets:validate
```

`assets:prepare:game` inclut également l'extension lors d'une reconstruction complète.
Les sources, SVG et caches restent dans `.local-assets/`, ignoré par Git ; seuls les
PNG/atlas et le manifeste runtime sont versionnés selon la demande du joueur.
Cette intégration n'établit aucun droit de redistribution ni accord d'Ankama.

Le lot intégré contient 2 543 séquences : 92 atlas de jeu et 25 atlas d'aperçus,
plus les icônes et éléments UI séparés. Le moteur charge les pièces du Crâ, les
sorts et Astrub au démarrage (19 pages utilisées par ce lot), puis les cartes et
familles nécessaires à l'entrée dans chaque donjon. Une attente de textures
suspend le pas de simulation, sans avancer les délais de combat ; un échec affiche
le chemin à réimporter. Les portraits et icônes restent chargés par leur interface.


Les six fonds `map.baked.*` sont des compositions PNG à densité 2 des couches
originales d'ordre inférieur à 2000 (sol et décor de fond). `bake-static-maps.ts`
conserve positions, pivots, rotations, miroirs et ordre de dessin ; les objets
`object2` restent indépendants pour leur occlusion avec les personnages. Les
couches originales restent dans le manifeste et servent toujours à la validation
et à la navigation. La scène en pause ne redessine que lorsqu'un état visuel ou
le cadrage change. Il n'y a ni nouveau graphisme ni animation de décor.

La préparation des six panoplies suivantes est décrite dans
[PREPARED-ASSETS.md](PREPARED-ASSETS.md). Son manifeste est indépendant du jeu actif.
Le cache ZIP de téléchargement Dofedex a été purgé après vérification de l'extraction
conservée ; son empreinte reste dans `.local-assets/dofedex-provenance.json`.
L'archive Remastered et le client source restent locaux.

### Lot préparé indépendant livré le 5 novembre 2026

`public/assets/prepared/` contient les six familles restantes de `TODO.md`, sans activation dans le jeu : 806 séquences Remastered, six cartes graphiques, 35 icônes, 43 sprites sources distincts, dix accessoires et neuf gardiens. Les 426 PNG occupent environ 575 Mio avec le manifeste. Les références temporelles passent de 22 051 à 19 334 par regroupement des pauses consécutives identiques ; aucune pose différente n'est retirée. Le manifeste actif ne référence pas ce lot. Catalogue, sources exactes, paramètres d'export et limites : [PREPARED-ASSETS.md](PREPARED-ASSETS.md).


### Activation jouable Chef Bwork / Blop Multi — 5 novembre 2026

Bworkette (sprite `1241`, carte publique `9760`) et Blop Multicolore Royal
(`1647`, disposition publique `11892`) sont intégrés avec leurs familles,
portraits HD, onze icônes et accessoires Bwork `6813` / `6811`.
Les chemins SWF exacts figurent dans `scripts/future-dungeons-assets.json` ;
les paramètres d’export et l’empreinte d’archive Remastered restent ceux de
[PREPARED-ASSETS.md](PREPARED-ASSETS.md#activation-chef-bwork--blop-multi--5-novembre-2026).
Cette activation remplace le statut inactif indiqué dans les notes historiques
ci-dessus pour ces deux familles. Le manifeste actif référence leurs PNG sous
`prepared/`, sans copie des atlas ni modification du lot historique. Les cartes
utilisent les couches fixes originales, avec la navigation revue et versionnée.
La validation exige les deux cartes et les accessoires Bwork.
Aucun droit de redistribution ni autorisation d’Ankama n’est établi.

### Coiffe du dev — apparence Champion Remastered, 5 novembre 2026

Objet personnalisé du fangame, nommé « Coiffe du dev » (`devHat`), sans table
de butin. Son dessin est exclusivement celui de la **Coiffe du Champion 8069** :
identité vérifiée dans `lang/fr/items/items_fr_450.swf / I.u[8069]`, au commit
Dofedex `ec4c08459f73f058294c34baa6baa739f9373753` (type 16, graphisme 127).
Les bonus personnalisés ne sont pas ceux de l’objet canonique.

Sources graphiques acquises dans l’archive **Remastered 1.40.13** SHA-256
`aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5` :

| Usage | Source originale | SHA-256 du SWF |
| --- | --- | --- |
| Icône | `clips/items/16/127.swf`, frame 1 | `acfc3abece8bdc789e646549e56e69ac5b26619e15bb99cda6f44bfc461d7a5d` |
| Coiffe portée | `clips/sprites/accessories/a1.swf#16_127` | `61c32e94c618b82818ae75359b2ffdcb2bc735f75a11309042e5e1bd5ce50b17` |

Le préparateur vérifie l’empreinte de l’archive et compare les deux SWF locaux
à leurs entrées originales avant export. JPEXS 26.4.0 exporte les vecteurs ;
Chromium les rasterise à densité 3 en jeu et 10 pour les aperçus. L’icône est
recadrée au dessin transparent, côté maximal 192 pixels. Les cinq labels source
R/L/F/B/S comportent chacun une frame native : aucune animation de la coiffe
n’est retirée. Le port suit les matrices et profondeurs GAC déjà extraites du Crâ,
pour ses directions et gestes, y compris avec les autres capes.

Les onze séquences finales et leur petit manifeste sont versionnés sous
`public/assets/dev-hat/`, avec références explicites du manifeste actif.
Aucune archive, source SWF, ActionScript ou SVG intermédiaire n’est publiée.
Les restrictions de redistribution et l’absence d’autorisation d’Ankama
restent inchangées.

Régénération après installation des sources et outils décrits plus haut :

```sh
pnpm assets:prepare:dev-hat
pnpm assets:validate
```

Le lot de travail est `.local-assets/dev-hat-runtime/`. Les préparations du jeu
réintègrent le lot versionné via `activate-bwork-blop.ts` ; l’import conserve
le dossier `dev-hat/` comme le dossier préparé des familles.

### Classes supplémentaires — 5 novembre 2026

Source téléchargée explicitement à la demande du joueur :
[release Dofedex 1.40.13](https://github.com/dofera/dofedex/releases/tag/1.40.13),
`dofus-1.40.13-remastered.zip` (474 901 966 octets), SHA-256
`aa9bccb901feb8bc06c16464c36ffbef1a46f8db27ce497bf252b9bd8c5d42d5`.
Archive et extraction dans `.local-assets/`, jamais publiées. Chaque fichier
utilisé est comparé à son entrée dans cette archive avant export.

| Classe | Corps original | Icônes des sorts AZER |
|---|---|---|
| Sacrieur | `clips/sprites/110.swf` | `clips/spells/icons/up/{439,431,438,446}.swf` |
| Iop | `clips/sprites/80.swf` | `clips/spells/icons/up/{160,142,143,159}.swf` |
| Féca | `clips/sprites/10.swf` | `clips/spells/icons/up/{10,9,11,5}.swf` |

`prepare-classes.py` exporte `staticS/R/F/L/B`, `runS/R/F/L/B`,
`anim0R/L`, `hitR/L`, `dieR/L`, avec les miroirs du client. Les gestes sont
adaptés au combat temps réel. Les couches originales et attaches
`GAC.applyAccessory` sont conservées pour toutes les coiffes et capes existantes.
JPEXS 26.4.0 produit les SVG ; Chromium rasterise les pièces à densité 6,
les effets à densité 3 et les aperçus d’inventaire à densité 10. Les icônes sont
recadrées au dessin, jusqu’à 192 pixels. Les durées natives restent au manifeste.

Effets : Châtiment `clips/spells/1051.swf#8`, Transposition `1055.swf#spire` (particules du sprite 8, transformations ActionScript transcrites avec graine 1055),
Épée de Iop `805.swf#8`, Bond `807.swf#5`, Intimidation `804.swf#13`,
Colère `809.swf#39`, Nuageuse `111.swf#14`, Trêve `101.swf#14`.
Les correspondances sont relevées dans la table `sorts` du dump Ancestra cité
plus haut ; tous les graphismes viennent exclusivement de l’archive Remastered.
Dissolution, Punition et Téléportation n’ont pas de sprite autonome dans cette
table : le jeu conserve le geste du personnage, sans effet emprunté à un autre
sort. La délimitation persistante du glyphe emploie `clips/gfx/cell.swf#s1` ;
aucune flamme de substitution n’est ajoutée.

Régénérer : `pnpm assets:prepare:classes`, puis `pnpm assets:validate`.
Le préparateur part du manifeste runtime existant, ajoute les trois classes,
puis importe uniquement les PNG/atlas et les compositions. Les sources SWF,
SVG et caches restent locaux. Les atlas supplémentaires sont séparés sous
`atlases/classes/` et `atlases/class-previews/`. Le validateur contrôle aussi les
attaches et les aperçus de ces classes. Cette publication du runtime à la demande
du joueur ne constitue pas une autorisation de redistribution d’Ankama.
