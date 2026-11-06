# Analyse de BouftouRush avant portage

Référence : `../bouftourush`, HEAD `c6a595c832937a608b4e00e94fcc19d4a7415a64`, état de travail du 6 novembre 2026. L'analyse porte sur les fichiers présents, y compris leurs éventuelles modifications locales. Le README historique est en retard : il annonce cinq donjons alors que `src/data/dungeons.ts` en active sept et `classes.ts` quatre classes.

## Périmètre constaté et repris

| Fonction | Référence TypeScript | Port Rust |
| --- | --- | --- |
| Astrub, sept arènes, placement, obstacles | manifeste, `movement/navigation.ts`, `map-navigation.json` | `assets/manifest.json`, `navigation.rs` |
| Incarnam, Ensablé, Bouftou, Tofu, Scarafeuille, Bwork, Blop multicolore | `data/dungeons.ts` | export exact `assets/data/game.json` |
| Normal, Extrême ×5 monstres / ×1,5 PV, trois vagues + boss | `dungeons/waves.ts` | directeur de vagues ECS |
| Quatre classes, 16 sorts | `data/classes.ts`, `core/simulation.ts` | `simulation/` |
| AZER, clic droit maintenu, destination conservée | `input/pointer.ts` | interface native, projection inverse du viewport |
| Immobilisation 0,12 s, cooldowns, buff, dégâts en pourcentage | `combat/`, `data/balance.ts` | calculs purs et pas fixe à 60 Hz |
| Poursuite, attaques, mort, récompenses, pas de collisions entre monstres | `core/simulation.ts` | composants Actor / Enemy / Shot |
| Accélération Royal, lance esquivable Raul Mops | `ai/speedBurst.ts`, `simulation.ts` | âge individuel, télégraphe puis projectile |
| XP, niveaux 1–100, kamas, butins pondérés | `progression/` | Profile, tirages xorshift32 |
| Butin royal indépendant à 10 %, panoplies Extrême garanties | `data/monsters.ts` et `simulation.ts` | mêmes tables et règles |
| Sac, filtres, équipement, deux anneaux, seuil de panoplie distinct | `InventoryPanel.tsx`, `profile.ts` | inventaire natif, statistiques recalculées |
| Vente de doublons à 1–4 kamas | `progression/resale.ts` | même protection des exemplaires portés |
| Portrait équipé et rotation, boss animés au survol, butin | `ui/CharacterSprite.tsx`, `DungeonPanel.tsx` | composition affine et aperçus HD |
| Coiffes, capes, arc, directions, masques | `rendering/composition*.ts` | matrices source, masques précalculés |
| Mort/victoire, nouvelle tentative, abandon, gains conservés | `core/simulation.ts` | phases locales, retour hub à pleine vie |
| Sauvegarde serveur et authentification | `server/`, `save/serverProfile.ts` | remplacées par JSON local, conformément à la demande |

La coiffe du développeur existe dans les données et ses plafonds spécifiques sont conservés. Elle ne tombe pas en butin et n'est pas donnée aux nouveaux personnages. Un ancien profil qui la possède peut être importé.

## Interface

L'original utilise un habillage sombre moderne autorisé par sa dernière évolution : carte complète au-dessus du bandeau inférieur, informations vitales à gauche, sorts au centre, statut et actions à droite. Ce découpage et les fenêtres donjons/inventaire/aide sont repris en interface PC native. Les illustrations, icônes, cartes et personnages proviennent du lot Remastered ; les cadres et boutons de la refonte sont dessinés par l'interface native. Le navigateur, le DOM et le CSS sont remplacés par Bevy/egui, ce qui implique une adaptation du rendu des contrôles et des polices, sans reprise pixel à pixel du CSS.

Les graphismes sont statiques pour les décors, animés pour les acteurs et effets. Les timelines d'attaque, de réaction et de mort sont adaptées à leurs fenêtres temps réel comme dans la source. L'arc Crâ est celui des animations sources de tir.

## Architecture et optimisation

La simulation ne dépend d'aucun service ou socket. Le moteur Bevy fournit l'ECS, le pas fixe, les ressources, le chargement asynchrone et le rendu wgpu. Les acteurs, ennemis, projectiles, effets et textes flottants sont des entités. Une ressource Session ne contient que l'état global du parcours et les glyphes. Le système de simulation utilise un accès exclusif au World pour imposer un ordre déterministe aux récompenses et aux transitions ; l'extraction et le rendu Bevy restent séparés. Un découpage supplémentaire en systèmes parallèles n'est utile qu'après mesure.

Le renderer utilise les meshes texturés de bevy_egui, superposés par profondeur logique de cellule. Ils permettent de conserver les matrices affines Flash complètes, y compris le cisaillement des accessoires ; les décomposer en translation/rotation/échelle perdrait de l'information. Les UV partagent les atlas. Les snapshots de progression sont écrits sur un thread dédié avec une file bornée, puis vidés à la fermeture ; aucun accès disque de sauvegarde n’est fait sur le thread de rendu. Les masques de composition sont précalculés, sans lecture d'image dans la boucle de jeu.

Le graphe de navigation et ses adjacences sont construits une fois par carte. Les ennemis utilisent un champ de distances commun pour leur destination ; les chemins sont lissés et les segments vérifiés tous les trois points. Les chemins sont recalculés environ toutes les 0,65 s par ennemi. Aucun test de séparation entre monstres ni sérialisation JSON des entités à chaque image. Les textures nécessaires à une scène sont préchargées avant d'avancer la simulation, les autres sont libérées après 300 images sans utilisation.

Le bundle graphique constitue l'essentiel de la taille disque. « Léger » ne signifie pas réduire la résolution des graphismes. Le profil release enlève les symboles, active LTO et n'embarque ni moteur 3D PBR, ni moteur de texte Bevy inutilisé (les textes utilisent egui), ni audio inutilisé, ni réseau. Les optimisations sont à mesurer sur les exécutables release et des pilotes GPU réels.

## Validation et limites de comparaison

`tests/reference_progression.json` est calculé par les fonctions TypeScript d'origine : statistiques de toutes les panoplies à trois niveaux, plafonds, dégâts et progression XP. Le cœur est compilable sans la fonctionnalité `client`, pour exécuter les tests sans renderer ni fenêtre. `tests/parity.rs` compare ces résultats et vérifie navigation, doublons, sauvegarde, pause, vagues, capacités et complétion des donjons. Le pilote de complétion utilise la coiffe du développeur : il vérifie le déroulement et les récompenses, pas la difficulté ni l'équilibrage ressenti.

L'original n'intègre pas de sons ; aucun son n'est donc ajouté. Aucun donjon futur uniquement présent dans les scripts de préparation n'est activé. L'interface Mon compte n'a pas de rôle dans une application locale sans compte. Le projet ne lit jamais automatiquement les identifiants ou la base privée de l'ancien jeu ; un export explicite de progression est fourni.
