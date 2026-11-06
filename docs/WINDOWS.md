# Compilation native Windows depuis WSL2

Le wrapper `scripts/build-windows.sh` appelle PowerShell et **Cargo Windows**, installé dans le profil Windows. Il ne produit pas le `.exe` par cross-compilation Linux. Les caches de compilation Linux et Windows sont distincts.

## Chaîne recommandée : MSVC

Installer Rust Windows et les outils C++ / SDK Windows proposés par rustup. Depuis le dossier du projet dans PowerShell :

```powershell
.\scripts\build-windows.ps1
```

## Variante GNU / LLVM-MinGW

Le script accepte le dossier `bin` d'une chaîne LLVM-MinGW Windows :

```powershell
.\scripts\build-windows.ps1 -CompilerBin 'C:\outils\llvm-mingw\bin'
```

Une installation locale sous `.local-tools/native/llvm-mingw-*/bin` est détectée automatiquement. Les outils et archives ne sont pas versionnés.

Pour la cible GNU, les bibliothèques restent compilées avec les outils de Rust ; la liaison finale passe par **rust-lld**, avec `linker-flavor=ld.lld` et `link-self-contained=yes`. Le CRT et les bibliothèques système fournis par Rust sont ainsi conservés.

Cette sélection est nécessaire lorsqu'une dépendance contient des bibliothèques d'import produites par LLVM : les mélanger avec GNU ld peut donner un exécutable qui compile mais plante à l'appel d'une fonction Windows. Le compilateur Rust documente cette incompatibilité dans son [code de création des bibliothèques d'import](https://fuchsia.googlesource.com/third_party/github.com/rust-lang/rust/+/dd600eff3a8def8c41b5bb86b4c6631f6e4c690b/compiler/rustc_codegen_cranelift/src/archive.rs). Le port a rencontré ce cas pendant sa validation ; les appels Windows de la sauvegarde locale ont été vérifiés avec LLD.

## Bundle et vérification

Le résultat se trouve dans `dist/BouftouRush-Windows/`. Copier **tout le dossier**, puis ouvrir `BouftouRush.exe`. Aucun serveur, compte ou installation de Node/Python n'est requis pour jouer.

```powershell
.\dist\BouftouRush-Windows\BouftouRush.exe --validate
```

Le programme cherche d'abord `assets/` à côté du `.exe`. Les sauvegardes restent dans le profil utilisateur, séparées du jeu. `BOUFTOURUSH_ASSETS` et `BOUFTOURUSH_SAVE` permettent de choisir d'autres chemins.

Les tests du cœur s'exécutent sans fenêtre ni renderer : `cargo test --locked --no-default-features`. Le client complet est vérifié avec `cargo check --locked --all-targets` et par lancement graphique.

## Contrôle des clignotements

Le client Windows privilégie DirectX 12 (sauf sélection explicite par `WGPU_BACKEND`), attend la compilation des pipelines et utilise des liaisons de textures individuelles. Ce chemin évite la préparation différée des shaders et les tableaux de textures bindless dans le rendu de la scène.

Pour vérifier une séquence plutôt qu'une capture isolée :

```powershell
.\dist\BouftouRush-Windows\BouftouRush.exe --smoke-frames 360 --smoke-scene combat --smoke-capture-dir .local-tools\captures
```

Puis `python scripts/check-captures.py .local-tools/captures` avec Pillow installé. Ces lancements de contrôle utilisent un profil temporaire et ne modifient pas la progression du joueur.
