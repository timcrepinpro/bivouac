# vendor/

`iced_wgpu/` est une copie du crate `iced_wgpu` 0.14.0 (licence MIT, auteurs d'iced),
branchée via `[patch.crates-io]` dans le `Cargo.toml` racine.

Ajouts : `src/pixel.rs` et une vingtaine de lignes dans `Renderer::present`
(`src/lib.rs`). Quand le facteur pixel vaut N > 1, toute la fenêtre est dessinée
dans une texture N fois plus petite, puis agrandie sur l'écran par un petit shader
WGSL qui lit le texel le plus proche (aucun lissage). Le launcher règle N avec
`iced_wgpu::pixel::set_factor` quand on change de mode (F3).

Pour passer à une version plus récente d'iced : recopier le crate officiel et
réappliquer ces deux modifications.

Deuxième ajout : dans `src/window/compositor.rs`, sous Windows, Vulkan est choisi en
priorité s'il est disponible (démarrage 382 ms au lieu de 2 252 ms en DirectX 12 sur
un Intel N95). `WGPU_BACKEND` reste prioritaire.
