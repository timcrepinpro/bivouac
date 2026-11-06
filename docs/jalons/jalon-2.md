# Jalon 2 — Identité visuelle : compte rendu

## Ce qui marche

- **Thème sombre** des maquettes sur toutes les pages, polices Sora / Pixelify Sans /
  IBM Plex Mono (instances fixes, plus de repli sur une autre police).
- **Paramètres › Apparence** : couleur d'accent (le logo feu de camp prend la couleur
  choisie), style **normal**, **pixel carré** ou **pixel arrondi** (coins en escalier dans
  le shader). Tout change en direct, sans redémarrer.
- **Transitions entre pages** faites dans le shader d'écran (PROMPT §11) :
  - Fluide : fondu, glissement, zoom ;
  - Pixel : dissolution, balayage, rideau, sur une grille de 16 × 6 blocs ;
  - vitesse 150 / 300 / 500 ms (300 par défaut), ou « Aucune ».
- **Réduire les animations** : si Windows a « Afficher les animations » désactivé, les
  transitions sont coupées et la page Apparence le dit.
- Réglages enregistrés dans `user.toml`, section `[apparence]` (accent, mode,
  transition, vue, vitesse).
- Tests : réglages (lecture, écriture, valeurs par défaut), ordre des blocs des
  transitions pixel. Clippy sans avertissement sous Linux et Windows.

- **Anglais** (PROMPT §2.7), ajouté à ta demande avant le Jalon 3 :
  - tous les textes de l'interface sont dans `crates/i18n/locales/fr.toml` et
    `en.toml`, plus aucun texte en dur dans les pages ;
  - Paramètres › Apparence › Affichage › Langue : Français / English, en direct ;
  - enregistré dans `user.toml` (`langue = "fr"` ou `"en"`), français par défaut ;
  - un test vérifie que les deux fichiers ont les mêmes clés et les mêmes `{valeurs}`.

## Ce qui manque

- La densité de l'interface (prévue dans `user.toml` par le PROMPT §6) : pas de maquette,
  pas encore de réglage.
- Les messages d'erreur techniques rares (dossier de données introuvable, `user.toml`
  illisible) restent en français.
- Thème clair : non prévu par les maquettes, noté dans `docs/idees.md`.

## Ce que j'ai supposé

- Le fondu passe par le fond de l'appli (ancienne page → fond → nouvelle page), comme
  dans les maquettes.
- Le réglage Windows lu est `SPI_GETCLIENTAREAANIMATION`.

## Proposé ensuite

Jalon 3 : Epic, GOG et les autres boutiques (EA, Ubisoft, Battle.net), avec leurs écrans
« introuvable ».
