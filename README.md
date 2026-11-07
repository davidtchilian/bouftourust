# BouftouRush — Rust / Bevy

Jeu PC 2D temps réel **entièrement local**, sans serveur, compte ou backend. Port de BouftouRush : Astrub, neuf donjons, modes Normal/Extrême, cinq classes dont le Crâ et ses sorts AZER, boss, loot, XP, kamas, inventaire, équipements et panoplies.

Les cartes, sprites, accessoires et icônes Remastered sont inclus dans `assets/`. Les graphismes restent la propriété de leurs ayants droit ; ce fangame gratuit n'est pas affilié à Ankama. Voir [provenance et récupération Dofedex](docs/ASSETS.md).

## Développement sous WSL2

Rust stable, pilote graphique WSLg et bibliothèques de développement X11 requis. Les dépendances Rust se téléchargent à la compilation ; le jeu fonctionne ensuite sans connexion.

```sh
cargo run --locked
cargo test --locked --no-default-features
python3 scripts/import-assets.py --check
cargo run --locked -- --validate
```

Pour accélérer les compilations WSL, placer les artefacts dans le système de fichiers Linux :

```sh
export CARGO_TARGET_DIR="$HOME/.cache/bouftourust-target"
cargo run --release --locked
```

## Produire l'exécutable Windows natif

Installer **Rust Windows** depuis [rustup.rs](https://rustup.rs/) avec les outils Visual Studio C++ / SDK Windows proposés par l'installateur. Depuis PowerShell :

```powershell
.\scripts\build-windows.ps1
```

Ou depuis WSL2, en utilisant réellement Cargo Windows :

```sh
make
```

Le Makefile nécessite GNU Make et appelle le script Windows existant (`make windows` est équivalent à `make`). Utiliser `make windows-debug` pour une compilation debug. Il fonctionne depuis WSL2 ou Windows avec GNU Make. Pour fournir un compilateur : `make WINDOWS_ARGS='-CompilerBin "C:\chemin\llvm-mingw\bin"'`. Sans Make, utiliser directement `./scripts/build-windows.sh` depuis WSL2.

Le bundle est créé dans `dist/BouftouRush-Windows/` : ouvrir `BouftouRush.exe`. Garder son dossier `assets/` à côté. La compilation native Windows n'est pas une cross-compilation Linux ; le script appelle PowerShell et la chaîne Rust Windows. Une chaîne Rust GNU avec LLVM-MinGW fonctionne également : fournir `-CompilerBin` si elle n’est pas détectée dans `.local-tools/native/`. Le script utilise alors le linker LLD et le CRT fournis avec Rust. Voir [compilation Windows](docs/WINDOWS.md).

Le workflow [Windows natif](.github/workflows/windows.yml) compile, teste et prépare un artefact téléchargeable dans GitHub Actions à chaque push.

## Jouer

- **Clic droit** : rejoindre le point visé. Maintenir pour suivre la souris, relâcher pour conserver la destination.
- **A Z E R** : quatre sorts de la classe, dirigés vers le curseur. Crâ : Flèche explosive, enflammée, magique, Tir puissant.
- **I** : inventaire, filtres de panoplie, aperçu équipé rotatif. Double-clic sur un objet pour l'équiper ; un clic sur un équipement porté affiche ses effets, un double-clic le retire.
- **Échap** : fermer la fenêtre ouverte ; en combat sans fenêtre, mettre en pause ou reprendre. Le bouton « Reprendre » relance aussi le combat. Les menus donjons et inventaire mettent la simulation en pause.
- **Donjons** : entrée libre, trois vagues puis le boss. Extrême multiplie les monstres par cinq et leur Vie par 1,5, et donne la panoplie complète à la victoire.
- **F3** : afficher FPS, nombre d'acteurs et de textures. L'aide propose une réduction des animations d'aperçu.

Les classes disponibles sont Crâ, Sacrieur, Iop, Féca et Sram. Les donjons sont Incarnam, Ensablé, Bouftou, Tofu, Scarafeuille, Bwork, Blop Multicolore Royal, Abraknyde Ancestral et Canidés. Les gains sont conservés après mort ou abandon ; une nouvelle tentative repart à la première vague avec toute la Vie.

## Sauvegarde et ancienne progression

La progression est enregistrée en JSON dans le dossier de données utilisateur : `%LOCALAPPDATA%\BouftouRush\BouftouRush\data\profile.json` sous Windows, `$XDG_DATA_HOME/bouftourush/profile.json` ou `~/.local/share/bouftourush/profile.json` sous Linux. Un fichier `.bak` conserve la version précédente. Un profil illisible provoque une erreur explicite et reste intact.

`BOUFTOURUSH_SAVE` permet de choisir un fichier de sauvegarde ; `BOUFTOURUSH_ASSETS` permet de choisir le dossier d'assets. Aucun chemin de l'ancien serveur n'est utilisé automatiquement.

Importer explicitement la progression depuis un export JSON ou l'ancienne base, sans toucher au serveur :

```sh
python3 scripts/export-legacy-save.py ../bouftourush/.server-data/game.sqlite
cargo run -- --import-profile .local-saves/legacy-profile.json
```

L'export SQLite est en lecture seule et ne sélectionne que le champ de progression. L'import valide les objets et équipements et sauvegarde le précédent profil dans `.bak`.

[Analyse fonctionnelle et architecture](docs/ANALYSE.md) · [Assets et sources Dofedex](docs/ASSETS.md)

Le donjon Ancestral propose trois vagues puis l’Abraknyde Ancestral : déplacement très lent, invocation toutes les 3 secondes, avec 0,5 % de chance d’invoquer un autre Ancestral. Les six pièces de la panoplie Ancestrale tombent dans ce donjon ; la victoire en Extrême donne la panoplie complète.

Le donjon des Canidés abrite le Meulou. Toutes les 5 secondes, il invoque deux Mulous et donne +50 dégâts à chacun de ses alliés vivants, nouveaux Mulous compris. Chaque monstre cumule jusqu’à +300 dégâts pour le reste de sa vie ; le Meulou ne se buffe pas lui-même. Les six pièces de sa panoplie se gagnent en loot ou ensemble à la victoire en Extrême.

L’équilibrage des panoplies et les comparaisons avant/après sont détaillés dans [EQUILIBRAGE.md](docs/EQUILIBRAGE.md). Tofu privilégie la mobilité, Bwork et Ancestrale la résistance, Blop la récupération, Meulou les dégâts. Les objets possédés sont conservés ; leurs statistiques suivent ces nouvelles valeurs.

Sram : **A** Sournoiserie (portée 200, recharge 0,6 s) ; **Z** Piège de Masse (portée infinie, empilable sur une case, recharge 1,5 s, explosion de rayon 75) ; **E** Invisibilité (4 s, recharge 14 s, recherche à la dernière position connue) ; **R** Attaque Mortelle (portée 40, recharge 4 s, 50 % de chance de dégâts ×3). Les attaques directes dissipent l’invisibilité ; les pièges la conservent. Les pièges disparaissent à la mort, à la victoire et au changement de carte.

Le filtre de panoplies ne propose que les familles présentes dans le sac, hors objets équipés. Les objets hors panoplie ont leur propre filtre. Les effets d’un objet sont affichés en liste verticale sans puces, sans statistiques nulles. Le Gelano est un anneau hors panoplie : +10 points de récupération de délai (plafond normal de 30 %), drop indépendant de 0,5 % sur chaque Blop, Biblop et membre de la famille, y compris les Royaux. Le Bouftou Royal avance désormais à 53 unités/s (contre 49), avec son accélération périodique conservée.

Fusion : sélectionner dans le sac un objet dont trois exemplaires libres sont disponibles au même niveau, puis cliquer sur « Fusionner ». Normal → ** (×1,5, bleu), ** → *** (×2, violet), *** → **** (×3, doré). Les multiplicateurs s’appliquent aux statistiques de base de l’objet ; les bonus de panoplie et plafonds globaux restent inchangés. Les équipements portés ne sont pas consommés ; les objets fusionnés sont protégés de la vente de doublons.

Chaque niveau gagné apporte +10 Vie, +5 Puissance, +0,2 point de réduction de délai et +0,2 point de vitesse de déplacement. Ces gains s’ajoutent à l’équipement et sont recalculés pour les personnages existants. Les plafonds globaux de statistiques restent appliqués. Au niveau 100, sans équipement : 1 170 Vie, 495 Puissance, 19,8 % de réduction de délai et 19,8 % de vitesse supplémentaire.
