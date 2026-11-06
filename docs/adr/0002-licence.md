# ADR 0002 — Licence du projet

Statut : **acceptée** le 06/11/2026 : tim a choisi la GPL-3.0 (`GPL-3.0-or-later`, texte dans `LICENSE`).

Dépendances principales : iced (MIT), wgpu (MIT ou Apache-2.0), serde (MIT ou Apache-2.0),
winreg (MIT), polices sous OFL 1.1 (compatibles avec toutes les options ci-dessous).

| Option | Ce que ça veut dire | Pour | Contre |
|---|---|---|---|
| **MIT ou Apache-2.0 (double)** | Tout le monde peut réutiliser le code, même dans un logiciel fermé | Standard de l'écosystème Rust, aucune friction | Quelqu'un peut en faire une version fermée |
| **GPL-3.0** | Toute version modifiée et distribuée doit rester open source | Protège le projet d'une reprise fermée | Moins de contributions d'entreprises |
| **MPL-2.0** | Les fichiers modifiés restent ouverts, le reste est libre | Compromis | Moins connue |

Recommandation : **GPL-3.0** si tu veux qu'un launcher dérivé reste toujours ouvert
(c'est l'esprit « l'utilisateur possède ses données ») ; sinon MIT/Apache-2.0.
