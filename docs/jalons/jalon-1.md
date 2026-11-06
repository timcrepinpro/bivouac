# Jalon 1 — Steam de bout en bout : compte rendu

## Ce qui marche

- **Base locale** `%APPDATA%\launcher\library.db` (SQLite, migrations versionnées,
  ADR 0003). Au démarrage la bibliothèque connue s'affiche tout de suite, puis le scan
  des boutiques la met à jour.
- **Steam** : détection du client (registre puis dossiers habituels, ou dossier indiqué à
  la main), lecture des jeux installés (`libraryfolders.vdf`, `appmanifest_*.acf`),
  temps de jeu, images en cache, lancement par `steam://rungameid/<appid>`.
- **Recherche floue** dans la liste : insensible aux accents et à la casse, tolère les
  fautes de frappe (« holow knigt », « witcehr ») et les initiales (« gta », « rdr2 »),
  meilleurs résultats en premier.
- **Écran « Steam est introuvable »** (maquette `Lancement-App`) : s'ouvre quand on lance
  un jeu Steam sans Steam, ou par le bandeau « Steam est introuvable · Régler » de la
  liste. « Installer Steam » ouvre la page officielle ; « Indiquer où est Steam » ouvre le
  choix de dossier de Windows et enregistre le dossier dans `user.toml` ; « Annuler » ou
  Échap revient à la bibliothèque.
- Tests : base (migrations, enregistrement, jeux désinstallés, réouverture), recherche,
  services (Steam qui disparaît, dossier choisi à la main), fixtures Steam/Epic/GOG.

## Ce qui reste à vérifier

- **Critère du jalon** (« je clique sur un jeu Steam et il se lance ») : à tester sur ta
  tour, ce PC n'a ni Steam ni jeux. Testé ici avec un faux dossier Steam.
- Epic et GOG sont toujours scannés (code du Jalon 0) mais leur écran « introuvable »
  et leurs tests complets sont pour le Jalon 3.

## Ce que j'ai supposé

- Le bandeau « Steam est introuvable » dans la liste : le PROMPT §7 prévoit « un écran
  ou un bandeau » par état ; sans lui, sur un PC sans Steam ni jeu, l'écran serait
  inaccessible.
- Un dossier choisi à la main est accepté s'il contient `steam.exe` ou `steamapps`
  (on accepte aussi de choisir `steamapps` lui-même).

## Proposé ensuite

Jalon 2 (identité visuelle) : page Paramètres › Apparence, réglages enregistrés dans
`user.toml` (accent, mode pixel, coins, transitions), et retrait du cadre « ESSAI ».
