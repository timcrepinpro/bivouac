//! Services : la seule porte de l'interface vers les boutiques, la base et le système
//! (PROMPT §5 : « ui ne parle jamais directement à une boutique »).

pub mod settings;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use launcher_core::{Account, InstalledGame, LaunchAction, StoreId};
use launcher_db::Db;
pub use launcher_stores::ClientStatus;
pub use settings::{Apparence, Settings};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("dossier de données introuvable (variable APPDATA absente)")]
    NoDataDir,
    #[error("impossible de créer {path} : {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Db(#[from] launcher_db::DbError),
    #[error("réglages illisibles ({path}) : {reason}")]
    Settings { path: PathBuf, reason: String },
}

/// Pourquoi un jeu n'a pas pu démarrer.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayError {
    /// L'application de la boutique est absente : écran `Lancement-App`.
    ClientMissing(StoreId),
    /// La boutique est là mais le système a refusé d'ouvrir le lien ou le programme.
    Launch(String),
}

/// Résultat d'une mise à jour de la bibliothèque.
#[derive(Debug, Clone, Default)]
pub struct Refresh {
    pub games: Vec<InstalledGame>,
    pub clients: Vec<(StoreId, ClientStatus)>,
    pub errors: Vec<String>,
    pub accounts: Vec<Account>,
    /// Durée du scan des boutiques seul (pour les mesures).
    pub scan_took: Duration,
}

/// Emplacement des fichiers du launcher (base, réglages). Remplaçable pour les tests.
#[derive(Debug, Clone)]
pub struct Paths {
    pub dir: PathBuf,
}

impl Paths {
    pub fn system() -> Result<Self, ServiceError> {
        launcher_platform::data_dir()
            .map(|dir| Paths { dir })
            .ok_or(ServiceError::NoDataDir)
    }
    fn db(&self) -> PathBuf {
        self.dir.join("library.db")
    }
    fn settings(&self) -> PathBuf {
        self.dir.join("user.toml")
    }
    fn ensure(&self) -> Result<(), ServiceError> {
        std::fs::create_dir_all(&self.dir).map_err(|source| ServiceError::Io {
            path: self.dir.clone(),
            source,
        })
    }
    fn open_db(&self) -> Result<Db, ServiceError> {
        self.ensure()?;
        Ok(Db::open(&self.db())?)
    }
}

/// Bibliothèque déjà connue, lue dans la base : affichée tout de suite au démarrage,
/// avant même que le scan des boutiques soit fini.
pub fn cached_library(paths: &Paths) -> Result<Vec<InstalledGame>, ServiceError> {
    Ok(paths.open_db()?.installed_games()?)
}

/// Scanne les boutiques, met la base à jour et renvoie la bibliothèque complète.
/// Une boutique introuvable ne touche pas à ses jeux déjà connus : ils restent visibles.
pub fn refresh_library(paths: &Paths) -> Result<Refresh, ServiceError> {
    let settings = settings::load(&paths.settings())?;
    let roots = launcher_stores::detect_roots(settings.boutiques.steam.as_deref());
    refresh_with(paths, &roots)
}

/// Comme [`refresh_library`], avec des emplacements donnés (tests, fixtures).
pub fn refresh_with(
    paths: &Paths,
    roots: &launcher_stores::Roots,
) -> Result<Refresh, ServiceError> {
    let t0 = Instant::now();
    let scan = launcher_stores::scan(roots);
    let scan_took = t0.elapsed();
    let mut db = paths.open_db()?;
    for store in &scan.read_ok {
        db.sync_store(*store, &scan.games)?;
    }
    Ok(Refresh {
        games: db.installed_games()?,
        clients: scan.clients,
        errors: scan.errors,
        accounts: scan.accounts,
        scan_took,
    })
}

/// Lance un jeu par sa boutique, si l'application de la boutique est là.
pub fn play(game: &InstalledGame, clients: &[(StoreId, ClientStatus)]) -> Result<(), PlayError> {
    let found = clients
        .iter()
        .any(|(s, c)| *s == game.store && matches!(c, ClientStatus::Installed { .. }));
    if !found {
        return Err(PlayError::ClientMissing(game.store));
    }
    launcher_platform::launch(&game.launch).map_err(|e| PlayError::Launch(e.to_string()))
}

/// Page officielle de téléchargement de l'application d'une boutique.
pub fn install_page(store: StoreId) -> &'static str {
    match store {
        StoreId::Steam => "https://store.steampowered.com/about/",
        StoreId::Epic => "https://store.epicgames.com/download",
        StoreId::Gog => "https://www.gog.com/galaxy",
    }
}

/// Ouvre la page de téléchargement dans le navigateur.
pub fn open_install_page(store: StoreId) -> Result<(), String> {
    let uri = install_page(store).to_string();
    launcher_platform::launch(&launcher_core::LaunchAction::Uri { uri }).map_err(|e| e.to_string())
}

/// Ouvre une adresse (page web, `steam://…`) avec le programme du système.
pub fn open_uri(uri: &str) -> Result<(), String> {
    launcher_platform::launch(&LaunchAction::Uri {
        uri: uri.to_string(),
    })
    .map_err(|e| e.to_string())
}

/// Outils d'un jeu installé, confiés à sa boutique (maquette `Jeu-Possede`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTool {
    Verify,
    Uninstall,
}

/// L'action de la boutique pour cet outil, si la boutique la propose.
/// Steam seulement pour l'instant : Epic et GOG n'ont pas de lien documenté.
pub fn game_tool(game: &InstalledGame, tool: GameTool) -> Option<LaunchAction> {
    match (game.store, tool) {
        (StoreId::Steam, GameTool::Verify) => {
            Some(launcher_stores::steam::verify(&game.external_id))
        }
        (StoreId::Steam, GameTool::Uninstall) => {
            Some(launcher_stores::steam::uninstall(&game.external_id))
        }
        _ => None,
    }
}

/// Échec d'une action de la fiche jeu. L'interface écrit le message dans la langue choisie.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolError {
    /// La boutique ne propose pas cette action.
    Unsupported,
    /// L'ouverture de l'adresse a échoué (message du système).
    Launch(String),
}

pub fn run_game_tool(game: &InstalledGame, tool: GameTool) -> Result<(), ToolError> {
    match game_tool(game, tool) {
        Some(action) => {
            launcher_platform::launch(&action).map_err(|e| ToolError::Launch(e.to_string()))
        }
        None => Err(ToolError::Unsupported),
    }
}

/// Page officielle d'activation d'un code (maquette `Code`). Le code est ajouté à
/// l'adresse quand la boutique l'accepte (Steam, GOG) ; il est aussi copié par l'interface.
pub fn redeem_url(store: StoreId, code: &str) -> String {
    let code: String = code
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    match store {
        StoreId::Steam => {
            format!("https://store.steampowered.com/account/registerkey?key={code}")
        }
        StoreId::Gog => format!("https://www.gog.com/redeem/{code}"),
        StoreId::Epic => "https://www.epicgames.com/store/redeem".to_string(),
    }
}

/// Préférence système « réduire les animations » (transitions coupées).
pub fn reduce_motion() -> bool {
    launcher_platform::reduce_motion()
}

/// Réglages actuels (fichier absent = valeurs par défaut).
pub fn load_settings(paths: &Paths) -> Result<Settings, ServiceError> {
    settings::load(&paths.settings())
}

/// Enregistre l'apparence dans `user.toml` sans toucher au reste.
pub fn save_appearance(paths: &Paths, apparence: Apparence) -> Result<(), ServiceError> {
    paths.ensure()?;
    let path = paths.settings();
    let mut s = settings::load(&path)?;
    s.apparence = apparence;
    settings::save(&path, &s)
}

/// Enregistre la langue de l'interface (`fr` ou `en`) dans `user.toml`.
pub fn save_language(paths: &Paths, langue: &str) -> Result<(), ServiceError> {
    paths.ensure()?;
    let path = paths.settings();
    let mut s = settings::load(&path)?;
    s.langue = langue.into();
    settings::save(&path, &s)
}

/// L'écran Bienvenue a été vu : il ne revient plus.
pub fn mark_welcomed(paths: &Paths) -> Result<(), ServiceError> {
    paths.ensure()?;
    let path = paths.settings();
    let mut s = settings::load(&path)?;
    s.bienvenue_vue = true;
    settings::save(&path, &s)
}

/// Résultat du choix manuel d'un dossier de boutique.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientChoice {
    /// Dossier valide, enregistré dans `user.toml`.
    Saved(PathBuf),
    /// Le dossier ne contient pas la boutique.
    NotAClientDir,
}

/// L'utilisateur indique où est l'application d'une boutique (« Indiquer où est Steam »).
pub fn choose_client_dir(
    paths: &Paths,
    store: StoreId,
    choice: &Path,
) -> Result<ClientChoice, ServiceError> {
    let dir = match store {
        StoreId::Steam => launcher_stores::steam::client_dir_from_choice(choice),
        // Epic et GOG : au Jalon 3.
        StoreId::Epic | StoreId::Gog => None,
    };
    let Some(dir) = dir else {
        return Ok(ClientChoice::NotAClientDir);
    };
    paths.ensure()?;
    let path = paths.settings();
    let mut s: Settings = settings::load(&path)?;
    s.boutiques.steam = Some(dir.clone());
    settings::save(&path, &s)?;
    Ok(ClientChoice::Saved(dir))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use launcher_stores::Roots;

    fn temp(name: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("launcher-svc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths { dir }
    }

    fn fake_steam(paths: &Paths, appid: &str, name: &str) -> PathBuf {
        let steam = paths.dir.join("steam");
        let apps = steam.join("steamapps");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::write(
            apps.join(format!("appmanifest_{appid}.acf")),
            format!(
                "\"AppState\"\n{{\n\"appid\" \"{appid}\"\n\"name\" \"{name}\"\n\"installdir\" \"{name}\"\n\"SizeOnDisk\" \"10\"\n}}\n"
            ),
        )
        .unwrap();
        steam
    }

    #[test]
    fn keeps_games_visible_when_steam_disappears() {
        let paths = temp("disappear");
        let steam = fake_steam(&paths, "10", "Hades");
        let found = Roots {
            steam: Some(steam.clone()),
            ..Default::default()
        };
        let r = refresh_with(&paths, &found).unwrap();
        assert_eq!(r.games.len(), 1);

        // Steam désinstallé : le jeu reste listé, mais le lancer demande l'écran Lancement-App.
        let r = refresh_with(&paths, &Roots::default()).unwrap();
        assert_eq!(r.games.len(), 1);
        assert_eq!(cached_library(&paths).unwrap().len(), 1);
        assert_eq!(
            play(&r.games[0], &r.clients),
            Err(PlayError::ClientMissing(StoreId::Steam))
        );
        let _ = std::fs::remove_dir_all(&paths.dir);
    }

    #[test]
    fn redeem_links_carry_a_clean_code() {
        assert_eq!(
            redeem_url(StoreId::Steam, " AAAAA-BBBBB-CCCCC&x=1 "),
            "https://store.steampowered.com/account/registerkey?key=AAAAA-BBBBB-CCCCCx1"
        );
        assert_eq!(
            redeem_url(StoreId::Gog, "ab12"),
            "https://www.gog.com/redeem/ab12"
        );
    }

    #[test]
    fn appearance_survives_a_steam_choice() {
        let paths = temp("apparence");
        let a = Apparence {
            accent: "menthe".into(),
            mode: "pixel-arrondi".into(),
            transition: "rideau".into(),
            vue: "grille".into(),
            vitesse: 150,
        };
        save_appearance(&paths, a.clone()).unwrap();
        mark_welcomed(&paths).unwrap();
        let s = load_settings(&paths).unwrap();
        assert_eq!(s.apparence, a);
        assert!(s.bienvenue_vue);
        let _ = std::fs::remove_dir_all(&paths.dir);
    }

    #[test]
    fn saves_a_hand_picked_steam_folder() {
        let paths = temp("choice");
        let steam = fake_steam(&paths, "10", "Hades");
        let other = paths.dir.join("vide");
        std::fs::create_dir_all(&other).unwrap();
        assert_eq!(
            choose_client_dir(&paths, StoreId::Steam, &other).unwrap(),
            ClientChoice::NotAClientDir
        );
        assert_eq!(
            choose_client_dir(&paths, StoreId::Steam, &steam.join("steamapps")).unwrap(),
            ClientChoice::Saved(steam.clone())
        );
        let s = settings::load(&paths.settings()).unwrap();
        assert_eq!(s.boutiques.steam, Some(steam));
        let _ = std::fs::remove_dir_all(&paths.dir);
    }
}
