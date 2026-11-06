# launcher

Launcher de jeux unifié, open source et 100 % local : une seule bibliothèque pour
Steam, Epic, GOG (puis EA, Ubisoft, Battle.net), chaque jeu lancé via sa boutique.
Écrit en **Rust**, interface en **iced**. Nom de travail : `launcher`.

> État : **Jalon 1** (Steam de bout en bout), à tester sur un PC avec Steam. Compte rendu : `docs/jalons/jalon-1.md`.

## Essayer

- `app\launcher.exe` : la page Bibliothèque avec les jeux installés sur ce PC.
  Les jeux sont gardés dans `%APPDATA%\launcher\library.db` et s'affichent tout de suite
  au démarrage suivant ; les réglages sont dans `%APPDATA%\launcher\user.toml`.
- `app\launcher.exe --demo 2000` : même chose avec 2 000 jeux fictifs.
- `app\launcher.exe --mesures` : 2 000 jeux, défilement automatique, transitions,
  puis écrit `mesures-jalon0.txt` à côté de l'exe et se ferme (environ 40 s).

Dans la fenêtre : `F2` images/s, `F3` mode (normal, pixel carré, pixel arrondi),
`F4` type de transition, `F5` couleur d'accent, `↑` `↓` pour choisir un jeu, `Entrée` pour jouer.
Le cadre « ESSAI » en haut fait la même chose à la souris.

## Compiler

1. Installer Rust : <https://rustup.rs>.
2. `cargo run -p launcher --release` (ou `cargo run -p launcher --release -- --mesures`).
3. Vérifications de la CI : `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## Organisation (PROMPT §5)

| Crate | Rôle | État |
|---|---|---|
| `crates/core` | Domaine pur : `InstalledGame`, `StoreId`, `LaunchAction`, tri, recherche floue. Aucune E/S. | Jalon 1 |
| `crates/db` | `library.db` (SQLite) : migrations versionnées, enregistrement des scans. | Jalon 1 |
| `crates/services` | Seule porte de l'interface : bibliothèque (base + scan), `user.toml`, lancement. | Jalon 1 |
| `crates/stores` | Steam, Epic, GOG : jeux installés, état du client, commande de lancement. Fixtures de test. | Jalon 0 (repris du prototype) |
| `crates/platform` | Registre Windows, chemins, ouverture des protocoles, processus. | Jalon 0 |
| `crates/ui` | iced : thème, logo recoloré, mode pixel, transitions, pages Bibliothèque et « Steam est introuvable », mesures. | Jalon 1 |
| `app` | Binaire principal, icône Windows. | Jalon 0 |
| `dedupe`, `sessions`, `prices`, `i18n` | Prévus aux jalons suivants. | — |

## Décisions

- `docs/adr/0001-choix-interface.md` : iced, et ce qu'il reste à prouver.
- `docs/adr/0002-licence.md` : choix de la licence (GPL-3.0, acceptée).
- `docs/adr/0003-base-de-donnees.md` : SQLite avec rusqlite, migrations par `user_version`.
- `docs/stores.md` : chemins et protocoles des boutiques, ce qui est vérifié ou non.
- `docs/idees.md` : idées notées au lieu d'être codées.

## Polices

Sora, Pixelify Sans et IBM Plex Mono sont sous licence SIL Open Font License 1.1
(`crates/ui/assets/fonts/OFL-Sora.txt`), intégrées dans l'exe.

## Licence

GPL-3.0 ou ultérieure : voir `LICENSE`. Le dossier `vendor/iced_wgpu` reste sous licence MIT (auteurs d'iced).
