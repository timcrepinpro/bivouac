# ADR 0003 — Base de données locale

Statut : **acceptée** (Jalon 1, 06/11/2026).

## Contexte

PROMPT §6 : `library.db` en SQLite, avec « rusqlite ou sqlx » et des migrations
versionnées dès le début. Le démarrage doit rester sous 500 ms et rien ne doit bloquer
l'interface.

## Choix

- **rusqlite**, avec SQLite compilé dans l'exe (`bundled`) : rien à installer, même
  version partout. sqlx apporterait de l'asynchrone et des requêtes vérifiées à la
  compilation, mais demande une base au moment de compiler et un moteur async en plus ;
  ici les requêtes sont courtes et tournent déjà hors du fil de l'interface (tâches iced).
- **Migrations** : fichiers `crates/db/migrations/NNNN_nom.sql`, inclus dans l'exe,
  appliqués dans l'ordre selon `PRAGMA user_version`, chacun dans une transaction.
  On n'en modifie jamais une publiée. Une base plus récente que l'exe est refusée
  (message clair) au lieu d'être abîmée.
- La migration 0001 crée tout le schéma de départ du §6, plus trois colonnes dans
  `copies` (`last_played`, `covers_json`, `launch_json`) pour réafficher la bibliothèque
  sans rescanner.
- Mode WAL pour lire pendant une écriture.

## Conséquences

- Au démarrage : lecture de la base (quelques ms) et affichage, puis scan des boutiques
  en tâche de fond, enregistrement, et mise à jour de la liste.
- Une boutique introuvable ne modifie pas ses jeux en base : ils restent visibles
  (maquette `Lancement-App`).
- Un jeu absent du scan d'une boutique trouvée passe à `installed = 0` (sa ligne reste,
  pour les sessions et les doublons des jalons suivants).
- `games` regroupe pour l'instant un exemplaire par jeu ; le regroupement des doublons
  entre boutiques viendra au Jalon 4.
