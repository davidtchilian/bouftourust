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

Évolutions du port : le Iop remplace Intimidation (E) par Puissance, qui double ses dégâts pendant 5 secondes (recharge 16 secondes). Colère de Iop cible une zone de rayon 60 autour du curseur, à une portée maximale de 60, avec un trajet dégagé. Son multiplicateur croissant est conservé. Puissance et Châtiment affichent une animation attachée au personnage et leur durée restante ; les effets actifs sont regroupés en grand à droite des commandes du bandeau inférieur.

La Bworkette soigne ses alliés vivants de 500 PV toutes les 12 secondes (plafond : vie maximale). Le Blop Multicolore Royal invoque un Blop Royal élémentaire aléatoire toutes les 5 secondes. Le Scarabosse Doré devient invulnérable pendant 2 secondes toutes les 5 secondes, avec un effet de protection et une mention au-dessus de sa vie. Le boss Tofu existant, le Batofu, gagne une vitesse ×5 pendant 1 seconde toutes les 10 secondes. Les délais commencent à l’apparition du monstre et suivent les pauses. Le Gloutoblop inflige 1 000 000 de dégâts fixes au contact, sans multiplicateur de vague. Les icônes de sorts deviennent monochromes pendant leur recharge.

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

## Donjon Ancestral ajouté

Huitième donjon, après le Blop Multicolore : Abraknydes, Abraknydes Sombres, Abraknes et Abraknes Sombres, trois vagues puis l’Abraknyde Ancestral. Vitesse du boss : 10 unités/s. Toutes les 3 secondes, un tirage uniforme sélectionne un monstre ordinaire (99,5 % au total) ou un autre Ancestral (0,5 %). Chaque Ancestral invoqué conserve cette capacité. Les PV et dégâts des invocations suivent la vague et le mode ; les délais suivent le temps de combat.

Panoplie Ancestrale : six pièces natives (8463–8468), axées Vie très élevée, Puissance élevée, mobilité moyenne et récupération élevée selon le TODO. Drop normal par monstre, récompense complète en Extrême. Coiffe et cape utilisent leurs vues Remastered natives.

## Donjon des Canidés

Neuvième donjon, tier 9 après l’Ancestral : trois vagues de Milimulous, Crocs Glands, Kanigrous, Mulous et Ouginaks, puis le Meulou et son escorte. Invocation de deux Mulous toutes les 5 secondes après son apparition. Après chaque paire, tous ses alliés vivants gagnent +50 dégâts fixes, nouveaux invoqués compris. Le bonus se cumule par entité jusqu’à +300 et dure jusqu’à sa mort ; il persiste après la mort du Meulou, qui ne reçoit pas ce bonus lui-même. Les pauses suspendent les délais. Les invocations suivent les statistiques de vague et de mode.

Panoplie du Meulou : six pièces natives (8262–8267), Vie/Puissance très élevées, mobilité/récupération élevées, bonus de 2 à 6 pièces. Loot en donjon et panoplie complète en Extrême.

## Équilibrage des équipements

La passe du 7 novembre 2026 remplace les statistiques des dix panoplies jouables, en conservant leurs identifiants, graphismes, emplacements et conditions d’obtention. Les chiffres et objectifs actuels figurent dans [EQUILIBRAGE.md](EQUILIBRAGE.md) ; ils remplacent les descriptions de puissance et de polyvalence antérieures, notamment pour Meulou.

## Cinquième classe : Sram

Les quatre classes du port sont conservées. Le Sram ajoute Sournoiserie (A, coefficient 1,2, recharge 0,6 s, portée 200), Piège de Masse (Z, coefficient 2,8, recharge 1,5 s, portée infinie, rayon d’explosion 75), Invisibilité (E, durée 4 s, recharge 14 s) et Attaque Mortelle (R, coefficient 4,5, recharge 4 s, portée 40, probabilité 50 % de dégâts ×3). Les recharges profitent de la récupération de l’équipement.

Le piège est fixé au centre de la cellule praticable réellement visée, sans test de portée ni de ligne de vue de placement. Chaque pose conserve ses dégâts calculés au lancement. Aucun plafond d’empilement ou durée d’expiration ; chaque piège inflige ses propres dégâts. Seul le contact d’un ennemi avec le losange exact déclenche les pièges, sans déclenchement en chaîne par proximité des explosions. Le segment de déplacement est testé pour ne pas rater une cellule traversée rapidement. Une seule animation d’explosion par case est dessinée quand plusieurs pièges explosent ensemble. Les pièges persistent entre les vagues, puis sont effacés à la mort, à la victoire ou au changement de carte.

Invisible, le joueur reste visible pour lui-même à 30 % d’opacité avec un compteur. Les ennemis se dirigent vers sa dernière position connue sans reprendre sa position réelle ; ils ne déclenchent pas de nouvelles attaques ciblées. Les projectiles déjà lancés ou préparés restent dangereux. Une attaque directe valide révèle le Sram, la pose et le déclenchement des pièges ne le révèlent pas. La fin des 4 s rétablit la poursuite normale ; les pauses suspendent cette durée.

## Inventaire et Gelano

Filtre de panoplies limité aux objets non équipés, avec remise à zéro si la dernière pièce libre quitte le sac. Un exemplaire libre suffit même si un autre exemplaire est équipé. Filtre « Hors panoplie » uniquement si nécessaire. Un clic sur un équipement porté sélectionne sa fiche ; seul le double-clic le retire. Effets en lignes sans puces, police 16 px dans la fiche, valeurs nulles omises.

Gelano 2469 : anneau sans panoplie (`set` vide, ignoré des compteurs de panoplies), uniquement +10 points de récupération. Deux anneaux se cumulent, sous le plafond normal de 30 %. Drop bonus indépendant à 0,5 % sur les 16 espèces dont le nom contient Blop, invocations comprises ; aucune garantie par la victoire en Extrême. Les tableaux de loot de panoplies sont conservés.

Vitesse du Bouftou Royal : 49 → 53 unités/s, accélération ×2 pendant 1 seconde toutes les 5 secondes conservée.

## Fusion des équipements

Chaque instance possède un `fusionLevel` facultatif (0 normal, 1 **, 2 ***, 3 ****). Les anciennes sauvegardes sans ce champ chargent le niveau 0. Les statistiques d’une instance valent celles de la définition ×1, ×1,5, ×2 ou ×3 ; elles ne sont pas multipliées récursivement. Les demi-points sont conservés et affichés. Les limites globales de récupération/déplacement et les bonus de panoplie sont conservés, avec comptage unique de la pièce de base.

Une fusion consomme exactement trois instances non équipées de même identifiant d’objet et même niveau. L’instance sélectionnée survit et monte d’un niveau, les deux autres sont retirées. La sélection reste donc stable. Aucun coût en kamas, aucun tirage aléatoire, pas de niveau supérieur à ****. Les commandes portent le niveau attendu pour éviter qu’un clic ancien répété déclenche la fusion suivante. Les opérations invalides ne consomment rien ; toute réussite est sauvegardée. 27 objets normaux permettent un ****.

Le bouton présente le nombre d’exemplaires disponibles, sans aperçu du résultat ni affichage du multiplicateur. Les objets **, *** et **** ont respectivement un contour lumineux bleu, violet et doré, dans le sac et dans les emplacements équipés. Les fiches et infobulles présentent leur nom étoilé et leurs statistiques effectives. Les objets fusionnés sont exclus de « Vendre doublons ». Contrôle visuel isolé : `--smoke-frames 60 --smoke-scene fusion`.

## Gains par niveau

À partir du passage du niveau 1 au niveau 2 : +10 Vie, +5 Puissance, +0,2 point de récupération et +0,2 point de vitesse par niveau. Le niveau 1 conserve sa base de 180 Vie. Calcul dérivé de `level - 1`, sans stocker de bonus cumulés dans la sauvegarde : effet rétroactif sur les profils existants et absence de double attribution au chargement. Les gains se cumulent avec les équipements et bonus de panoplie avant application des plafonds existants. Un niveau gagné restaure 10 Vie comme il augmente le maximum de 10. Les pourcentages de la fiche personnage affichent une décimale. Les attentes d’équipement et de Vie gagnée via XP ont été actualisées ; les seuils d’XP et le niveau maximal ne changent pas.
