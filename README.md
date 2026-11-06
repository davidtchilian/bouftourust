# BouftouRush — Rust / Bevy

Jeu PC 2D temps réel **entièrement local**, sans serveur, compte ou backend. Port de BouftouRush : Astrub, sept donjons, modes Normal/Extrême, quatre classes dont le Crâ et ses sorts AZER, boss, loot, XP, kamas, inventaire, équipements et panoplies.

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
./scripts/build-windows.sh
```

Le bundle est créé dans `dist/BouftouRush-Windows/` : ouvrir `BouftouRush.exe`. Garder son dossier `assets/` à côté. La compilation native Windows n'est pas une cross-compilation Linux ; le script appelle PowerShell et la chaîne Rust Windows. Une chaîne Rust GNU avec LLVM-MinGW fonctionne également : fournir `-CompilerBin` si elle n’est pas détectée dans `.local-tools/native/`. Le script utilise alors le linker LLD et le CRT fournis avec Rust. Voir [compilation Windows](docs/WINDOWS.md).

Le workflow [Windows natif](.github/workflows/windows.yml) compile, teste et prépare un artefact téléchargeable dans GitHub Actions à chaque push.

## Jouer

- **Clic droit** : rejoindre le point visé. Maintenir pour suivre la souris, relâcher pour conserver la destination.
- **A Z E R** : quatre sorts de la classe, dirigés vers le curseur. Crâ : Flèche explosive, enflammée, magique, Tir puissant.
- **I** : inventaire, filtres de panoplie, aperçu équipé rotatif. Double-clic sur un objet pour l'équiper ; cliquer un emplacement porté pour le retirer.
- **Échap** : fermer la fenêtre ouverte ; en combat sans fenêtre, mettre en pause ou reprendre. Le bouton « Reprendre » relance aussi le combat. Les menus donjons et inventaire mettent la simulation en pause.
- **Donjons** : entrée libre, trois vagues puis le boss. Extrême multiplie les monstres par cinq et leur Vie par 1,5, et donne la panoplie complète à la victoire.
- **F3** : afficher FPS, nombre d'acteurs et de textures. L'aide propose une réduction des animations d'aperçu.

Les classes disponibles sont Crâ, Sacrieur, Iop et Féca. Les donjons sont Incarnam, Ensablé, Bouftou, Tofu, Scarafeuille, Bwork et Blop Multicolore Royal. Les gains sont conservés après mort ou abandon ; une nouvelle tentative repart à la première vague avec toute la Vie.

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
