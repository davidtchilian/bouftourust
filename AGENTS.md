# BouftouRush Rust

- Jeu PC natif en français, Rust + Bevy, 100 % local ; ne pas ajouter de backend ou de compte.
- La référence fonctionnelle est décrite dans `docs/ANALYSE.md`. Conserver les sept donjons et les cinq classes réellement présents dans la source.
- Le lot `assets/` utilisé par le jeu est versionné. Archives, sources inutilisées, outils et sauvegardes restent dans les dossiers `.local-*`, ignorés par Git.
- Pour tous les sorts du jeu, utiliser https://xixou.io/sorts/ comme référence et ses icônes officielles comme visuels (exemple : https://xixou.io/sorts/439-dissolution/). Télécharger chaque icône et utiliser son PNG local ; ne jamais charger le site en jeu. Correspondances et provenance : `assets/data/xixou-spells.json` ; récupération : `scripts/import-spell-icons.py`.
- Pour les autres graphismes (personnages, cartes, objets, animations des sorts), n'utiliser que Dofus Retro Remastered 1.40.13. L'exception Xixou concerne uniquement les icônes de sorts. Provenance et récupération : `docs/ASSETS.md`, releases https://github.com/dofera/dofedex/releases.
- Ne jamais importer les secrets, comptes ou sessions de l'ancien projet. L'outil de migration de progression lit uniquement la colonne `profile`.
- Vérification : `cargo fmt --all -- --check`, `cargo test --locked --no-default-features`, `python3 scripts/import-assets.py --check`.
- Build Windows natif via `scripts/build-windows.ps1` ou son wrapper WSL ; distribuer le dossier d'assets à côté de l'exécutable.
