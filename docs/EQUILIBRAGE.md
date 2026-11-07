# Équilibrage des panoplies — 7 novembre 2026

Première passe sur les dix panoplies jouables. Objectif : réduire les sauts de puissance et préserver des choix de mobilité, de résistance et de récupération. Cette première passe concerne les équipements. Les gains de niveau ont ensuite été ajustés comme décrit en fin de document. Les monstres, sorts, taux de loot et récompenses ne sont pas modifiés par cet équilibrage.

Les nombres ci-dessous sont les bonus d’équipement seuls, avec une pièce de chaque objet et le bonus complet, sans statistiques de niveau ni seconde bague. Le Scarafeuille Noir et le Blop Multicolore ont quatre pièces : leurs totaux ne représentent pas une tenue de huit emplacements. Les mélanges et le second anneau sont vérifiés séparément dans les tests.

| Panoplie | Pièces | Vie avant → après | Puissance avant → après | Vitesse après | Récupération après |
| --- | ---: | ---: | ---: | ---: | ---: |
| Panoplie de l’Aventurier | 6 | 140 → 80 | 50 → 30 | 0 % | 0 % |
| Panoplie en Mousse | 7 | 260 → 160 | 110 → 70 | 0 % | 10 % |
| Panoplie du Bouftou | 7 | 460 → 300 | 260 → 145 | 0 % | 0 % |
| Panoplie du Bouftou Royal | 7 | 690 → 430 | 390 → 205 | 0 % | 0 % |
| Panoplie du Tofu | 7 | 168 → 160 | 175 → 160 | 36 % | 8 % |
| Panoplie du Scarafeuille Noir | 4 | 580 → 230 | 304 → 155 | 0 % | 0 % |
| Panoplie du Chef Bwork | 7 | 945 → 620 | 406 → 250 | 8 % | 6 % |
| Panoplie Blop Multicolore Royale | 4 | 620 → 220 | 520 → 250 | 0 % | 18 % |
| Panoplie Ancestrale | 6 | 1230 → 820 | 600 → 300 | 6 % | 14 % |
| Panoplie du Meulou | 6 | 1500 → 720 | 840 → 400 | 16 % | 8 % |

Les bonus complets de Vie et Puissance représentent environ 20 % du budget de chaque panoplie. Les paliers intermédiaires sont progressifs ; un équipement partiel ou mixte reste viable. Les pièces se spécialisent : armes et amulettes pour les dégâts, ceintures et capes pour la Vie, bottes pour la vitesse, amulettes et anneaux pour la récupération. Chaque objet ne fournit plus les quatre statistiques systématiquement.

Tofu reste la référence en mobilité. Scarafeuille Noir apporte résistance et dégâts sur quatre emplacements, sans vitesse ni récupération. Bwork privilégie la résistance, Blop la fréquence des sorts. Ancestrale conserve davantage de Vie et de récupération que Meulou ; Meulou fournit davantage de Puissance et de vitesse. La Coiffe du dev conserve ses statistiques de test et doit être retirée pour juger la difficulté.

Les sauvegardes conservent les objets et l’équipement : leurs statistiques sont recalculées au chargement. Les panoplies futures du TODO ne sont pas ajoutées par cette passe.

Vérification : profils aux niveaux 1, 25 et 100, spécialités des panoplies, mélanges de deux panoplies avec chaque anneau disponible, navigation et victoire dans les neuf donjons. Le test automatique de victoire utilise la Coiffe du dev : il valide le fonctionnement, pas la difficulté humaine. Les nouvelles valeurs constituent une base de playtest ; elles ne prouvent pas un taux de victoire ou une durée de combat cible.

Les profils historiques, cas de dégâts et seuils d’XP sont conservés dans `tests/reference_progression.json` ; les attentes d’équipement et de Vie gagnée par XP ont été actualisées, avec ajout d’Ancestrale et Meulou. Régénération indépendante du moteur Rust : `python3 scripts/export-balanced-fixtures.py`.

Mise à jour des niveaux : chaque niveau gagné ajoute désormais +10 Vie, +5 Puissance, +0,2 point de récupération et +0,2 point de vitesse. Les valeurs du tableau restent celles de l’équipement seul. Les attentes de Vie gagnée par XP suivent maintenant +10 par niveau ; les seuils d’XP et les profils de référence restent conservés.
