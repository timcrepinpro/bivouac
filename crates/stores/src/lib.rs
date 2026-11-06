//! Boutiques. Chaque module lit les fichiers de sa boutique ; la détection des
//! chemins passe par `launcher-platform`. Jalon 0 : Steam, Epic, GOG (jeux installés).

pub mod epic;
pub mod gog;
pub mod steam;
pub mod vdf;

use std::path::PathBuf;

use launcher_core::{sort_library, Account, InstalledGame, StoreId};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("fichier illisible {path} : {reason}")]
    Parse { path: PathBuf, reason: String },
    #[error("dossier illisible {path} : {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// État du client de boutique (PROMPT §7). `NotLoggedIn` arrivera avec les API en ligne.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    Installed { path: PathBuf },
    NotFound,
}

/// Emplacements à scanner. Séparés de la détection pour tester avec des fixtures.
#[derive(Debug, Clone, Default)]
pub struct Roots {
    pub steam: Option<PathBuf>,
    pub epic_manifests: Option<PathBuf>,
    pub gog_entries: Vec<gog::GogEntry>,
    pub gog_galaxy: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub games: Vec<InstalledGame>,
    pub clients: Vec<(StoreId, ClientStatus)>,
    /// Boutiques lues sans erreur : seules celles-ci mettent la base à jour.
    pub read_ok: Vec<StoreId>,
    pub errors: Vec<String>,
    /// Comptes trouvés sur ce PC (Steam pour l'instant).
    pub accounts: Vec<Account>,
}

impl ScanResult {
    pub fn client(&self, store: StoreId) -> ClientStatus {
        self.clients
            .iter()
            .find(|(s, _)| *s == store)
            .map(|(_, c)| c.clone())
            .unwrap_or(ClientStatus::NotFound)
    }
}

/// Une boutique en panne ne bloque pas les autres (PROMPT §7).
pub fn scan(roots: &Roots) -> ScanResult {
    let mut r = ScanResult::default();
    let status = |p: &Option<PathBuf>| match p {
        Some(path) => ClientStatus::Installed { path: path.clone() },
        None => ClientStatus::NotFound,
    };
    r.clients.push((StoreId::Steam, status(&roots.steam)));
    r.clients
        .push((StoreId::Epic, status(&roots.epic_manifests)));
    r.clients.push((
        StoreId::Gog,
        if roots.gog_galaxy.is_some() || !roots.gog_entries.is_empty() {
            ClientStatus::Installed {
                path: roots.gog_galaxy.clone().unwrap_or_default(),
            }
        } else {
            ClientStatus::NotFound
        },
    ));
    if let Some(dir) = &roots.steam {
        r.accounts.extend(steam::account(dir));
        match steam::scan(dir) {
            Ok(mut g) => {
                r.games.append(&mut g);
                r.read_ok.push(StoreId::Steam);
            }
            Err(e) => r.errors.push(format!("Steam : {e}")),
        }
    }
    if let Some(dir) = &roots.epic_manifests {
        match epic::scan(dir) {
            Ok(mut g) => {
                r.games.append(&mut g);
                r.read_ok.push(StoreId::Epic);
            }
            Err(e) => r.errors.push(format!("Epic : {e}")),
        }
    }
    if roots.gog_galaxy.is_some() || !roots.gog_entries.is_empty() {
        r.games.extend(gog::to_games(
            &roots.gog_entries,
            roots.gog_galaxy.as_deref(),
        ));
        r.read_ok.push(StoreId::Gog);
    }
    sort_library(&mut r.games);
    r
}

/// Trouve les vrais emplacements sur ce PC. `steam_choice` = dossier indiqué à la
/// main par l'utilisateur (écran « Steam est introuvable »), prioritaire s'il est valide.
pub fn detect_roots(steam_choice: Option<&std::path::Path>) -> Roots {
    let (gog_entries, gog_galaxy) = gog::detect();
    Roots {
        steam: steam_choice
            .and_then(steam::client_dir_from_choice)
            .or_else(steam::detect),
        epic_manifests: epic::detect(),
        gog_entries,
        gog_galaxy,
    }
}
