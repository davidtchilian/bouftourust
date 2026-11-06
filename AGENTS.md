# BouftouRush Rust

- Jeu PC natif en français, Rust + Bevy, 100 % local ; ne pas ajouter de backend ou de compte.
- La référence fonctionnelle est décrite dans `docs/ANALYSE.md`. Conserver les sept donjons et les quatre classes réellement présents dans la source.
- Le lot `assets/` utilisé par le jeu est versionné. Archives, sources inutilisées, outils et sauvegardes restent dans les dossiers `.local-*`, ignorés par Git.
- N'utiliser que les graphismes Dofus Retro Remastered 1.40.13. Provenance et récupération : `docs/ASSETS.md`, releases https://github.com/dofera/dofedex/releases.
- Ne jamais importer les secrets, comptes ou sessions de l'ancien projet. L'outil de migration de progression lit uniquement la colonne `profile`.
- Vérification : `cargo fmt --all -- --check`, `cargo test --locked --no-default-features`, `python3 scripts/import-assets.py --check`.
- Build Windows natif via `scripts/build-windows.ps1` ou son wrapper WSL ; distribuer le dossier d'assets à côté de l'exécutable.
