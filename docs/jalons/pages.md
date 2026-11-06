# Toutes les pages, fidèles aux maquettes

Demande de tim (06/11) : « copie fidèle mais avec les vraies données ». Chaque page suit sa
maquette (`launcher-design/maquettes-html`) dans les trois styles : normal, pixel carré,
pixel arrondi. Ce qui n'a pas encore de données réelles reste vide, avec le jalon qui
l'apportera. Rien n'est inventé (ni prix, ni amis, ni succès).

| Page (maquette) | Données réelles affichées | Vide pour l'instant (jalon) |
|---|---|---|
| Bienvenue | boutiques détectées, nombre de jeux, « Choisir le dossier » / « Télécharger » | EA, Ubisoft, Battle.net (J3) |
| Bibliothèque (Mix2) | jeux, temps, dernière partie, pastilles Collections / Doublons · N / Statistiques | — |
| Grille | vignettes, tri (récents, nom, temps, taille), filtres par boutique | Favoris, À finir (J4) |
| Jeu-Possede | temps de jeu, dernière session, taille, dossier, « Vérifier les fichiers » et « Désinstaller » confiés à Steam | succès, actualités, cloud (§13) |
| Lancement | boutique trouvée, demande transmise, écran fermé après 6 s | — |
| Paramètres › Comptes liés | Steam / Epic / GOG détectés + nombre de jeux | EA, Ubisoft, Battle.net (J3) |
| Paramètres › Dossiers | dossiers d'installation réels, jeux et taille par dossier | téléchargements gérés par le launcher |
| Paramètres › Apparence | accent, pixel, coins, transitions (fondu ou blocs), vue au démarrage, enregistrés dans `user.toml` | thème clair (plus tard) |
| Statistiques | temps total, jeux lancés, dernière partie, part par boutique, plus joués | graphique par jour, « Quand tu joues » (J5) |
| Profil | nom et avatar du compte Steam de ce PC, temps par boutique, plus joués | niveau, succès, amis |
| Doublons | jeux installés sur plusieurs boutiques (titre normalisé), copie la plus jouée | regroupement (J4) |
| Collections | « Tous les jeux », « Jamais lancés » | collections et étiquettes à soi (J4) |
| Recherche | recherche floue dans la bibliothèque, Jouer | boutiques (J6), amis (J7) |
| Code | ouvre la page officielle d'activation (Steam et GOG avec le code), copie le code | EA, Ubisoft, Battle.net (J3) |
| Téléchargements | espace utilisé par boutique | file d'attente (non gérée par le launcher) |
| Boutique, Souhaits | « Utiliser un code » | prix et souhaits (J6, à cadrer §13) |
| Amis, Messages | — | J7, à cadrer (§13) |

Captures de contrôle : `LAUNCHER_PAGE=<page>` ouvre une page au démarrage, dans les
versions de développement seulement (`cargo run`), jamais dans l'exe livré.
