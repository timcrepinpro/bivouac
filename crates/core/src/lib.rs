//! Domaine pur : aucune entrée/sortie ici (règle du PROMPT §5).

pub mod search;

use serde::{Deserialize, Serialize};

/// Boutique d'origine d'un jeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreId {
    Steam,
    Epic,
    Gog,
}

impl StoreId {
    pub const ALL: [StoreId; 3] = [StoreId::Steam, StoreId::Epic, StoreId::Gog];

    pub fn key(self) -> &'static str {
        match self {
            StoreId::Steam => "steam",
            StoreId::Epic => "epic",
            StoreId::Gog => "gog",
        }
    }
    /// Étiquette technique affichée en IBM Plex Mono.
    pub fn label(self) -> &'static str {
        match self {
            StoreId::Steam => "STEAM",
            StoreId::Epic => "EPIC",
            StoreId::Gog => "GOG",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            StoreId::Steam => "Steam",
            StoreId::Epic => "Epic",
            StoreId::Gog => "GOG",
        }
    }
}

/// Comment démarrer un jeu : toujours via la boutique d'origine (PROMPT §2.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LaunchAction {
    /// Adresse de protocole (`steam://…`, `com.epicgames.launcher://…`).
    Uri { uri: String },
    /// Programme à démarrer (GOG Galaxy, ou l'exe du jeu en dernier recours).
    Exe {
        path: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
}

/// Images locales d'un jeu, quand la boutique en garde en cache.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Covers {
    pub portrait: Option<String>,
    pub hero: Option<String>,
}

/// Un exemplaire installé d'un jeu, sur une boutique (le `copy` du schéma §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledGame {
    pub store: StoreId,
    /// Identifiant natif : appid Steam, AppName Epic, gameID GOG.
    pub external_id: String,
    pub title: String,
    pub install_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub playtime_minutes: Option<u64>,
    /// Dernière partie, en secondes Unix.
    pub last_played: Option<u64>,
    pub covers: Covers,
    pub launch: LaunchAction,
}

impl InstalledGame {
    /// Identifiant unique dans le launcher : `boutique:id`.
    pub fn key(&self) -> String {
        format!("{}:{}", self.store.key(), self.external_id)
    }
}

/// Compte d'une boutique lu sur ce PC (nom affiché, image locale).
#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub store: StoreId,
    pub name: String,
    pub avatar: Option<String>,
}

/// Dossier racine d'une installation : la bibliothèque Steam (avant `steamapps`),
/// sinon le dossier qui contient le jeu.
pub fn install_root(path: &str) -> String {
    let norm = path.replace('/', "\\");
    let lower = norm.to_lowercase();
    if let Some(i) = lower.find("\\steamapps\\") {
        return norm[..i].to_string();
    }
    let trimmed = norm.trim_end_matches('\\');
    match trimmed.rfind('\\') {
        Some(i) if i > 0 => trimmed[..i].to_string(),
        _ => trimmed.to_string(),
    }
}

/// Dossiers d'installation : (dossier, nombre de jeux, octets), le plus rempli d'abord.
pub fn install_folders(games: &[InstalledGame]) -> Vec<(String, usize, u64)> {
    let mut out: Vec<(String, usize, u64)> = Vec::new();
    for g in games {
        let Some(p) = &g.install_path else { continue };
        let root = install_root(p);
        match out
            .iter_mut()
            .find(|(r, _, _)| r.eq_ignore_ascii_case(&root))
        {
            Some(e) => {
                e.1 += 1;
                e.2 += g.size_bytes.unwrap_or(0);
            }
            None => out.push((root, 1, g.size_bytes.unwrap_or(0))),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// Jeux présents sur plusieurs boutiques (même titre une fois normalisé, PROMPT §8).
/// Chaque groupe contient les indices des exemplaires, le plus joué d'abord.
pub fn duplicates(games: &[InstalledGame]) -> Vec<Vec<usize>> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, g) in games.iter().enumerate() {
        let key: String = search::normalize(&g.title)
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if key.is_empty() {
            continue;
        }
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(i),
            None => groups.push((key, vec![i])),
        }
    }
    groups
        .into_iter()
        .map(|(_, mut v)| {
            v.sort_by_key(|&i| std::cmp::Reverse(games[i].playtime_minutes.unwrap_or(0)));
            v
        })
        .filter(|v| v.iter().any(|&i| games[i].store != games[v[0]].store))
        .collect()
}

/// Temps de jeu total par boutique, en minutes (boutiques sans temps comprises).
pub fn playtime_by_store(games: &[InstalledGame]) -> Vec<(StoreId, u64)> {
    StoreId::ALL
        .iter()
        .map(|&s| {
            let total = games
                .iter()
                .filter(|g| g.store == s)
                .filter_map(|g| g.playtime_minutes)
                .sum();
            (s, total)
        })
        .collect()
}

/// Trie : joués récemment d'abord, puis par titre.
pub fn sort_library(games: &mut [InstalledGame]) {
    games.sort_by(|a, b| {
        b.last_played
            .unwrap_or(0)
            .cmp(&a.last_played.unwrap_or(0))
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
}

/// Filtre de la bibliothèque : boutique + recherche floue (voir [`search`]).
/// `query` doit déjà être passé par [`search::normalize`]. Renvoie la note du jeu.
pub fn matches(game: &InstalledGame, store: Option<StoreId>, query: &str) -> Option<i32> {
    if store.is_some_and(|s| s != game.store) {
        return None;
    }
    search::score(query, &search::normalize(&game.title))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(title: &str, store: StoreId, last: Option<u64>) -> InstalledGame {
        InstalledGame {
            store,
            external_id: title.into(),
            title: title.into(),
            install_path: None,
            size_bytes: None,
            playtime_minutes: None,
            last_played: last,
            covers: Covers::default(),
            launch: LaunchAction::Uri { uri: String::new() },
        }
    }

    #[test]
    fn finds_install_roots() {
        assert_eq!(
            install_root(r"E:\Bibliothèque Steam\steamapps\common\Hades"),
            r"E:\Bibliothèque Steam"
        );
        assert_eq!(install_root(r"D:\Jeux\Celeste\"), r"D:\Jeux");
        let mut a = g("Hades", StoreId::Steam, None);
        a.install_path = Some(r"D:\SteamLibrary\steamapps\common\Hades".into());
        a.size_bytes = Some(10);
        let mut b = g("Celeste", StoreId::Steam, None);
        b.install_path = Some(r"d:\steamlibrary\steamapps\common\Celeste".into());
        b.size_bytes = Some(5);
        let mut c = g("Control", StoreId::Epic, None);
        c.install_path = Some(r"C:\Epic\Control".into());
        assert_eq!(
            install_folders(&[a, b, c]),
            vec![
                (r"D:\SteamLibrary".to_string(), 2, 15),
                (r"C:\Epic".to_string(), 1, 0)
            ]
        );
    }

    #[test]
    fn groups_duplicates_across_stores_only() {
        let mut steam = g("Hollow Knight", StoreId::Steam, None);
        steam.playtime_minutes = Some(10);
        let mut gog = g("Hollow Knight™", StoreId::Gog, None);
        gog.playtime_minutes = Some(60);
        let v = vec![
            steam,
            g("Celeste", StoreId::Epic, None),
            gog,
            g("Celeste", StoreId::Epic, None),
        ];
        // Même boutique deux fois : pas un doublon. Le plus joué passe en premier.
        assert_eq!(duplicates(&v), vec![vec![2, 0]]);
    }

    #[test]
    fn sums_playtime_by_store() {
        let mut a = g("a", StoreId::Steam, None);
        a.playtime_minutes = Some(30);
        let mut b = g("b", StoreId::Steam, None);
        b.playtime_minutes = Some(15);
        let t = playtime_by_store(&[a, b, g("c", StoreId::Gog, None)]);
        assert_eq!(t[0], (StoreId::Steam, 45));
        assert_eq!(t[2], (StoreId::Gog, 0));
    }

    #[test]
    fn sorts_recent_then_alpha() {
        let mut v = vec![
            g("b", StoreId::Gog, None),
            g("a", StoreId::Gog, None),
            g("z", StoreId::Steam, Some(5)),
        ];
        sort_library(&mut v);
        let t: Vec<_> = v.iter().map(|x| x.title.as_str()).collect();
        assert_eq!(t, ["z", "a", "b"]);
    }

    #[test]
    fn filters_by_store_and_text() {
        let x = g("Hollow Knight", StoreId::Steam, None);
        assert!(matches(&x, None, "knight").is_some());
        assert!(matches(&x, Some(StoreId::Steam), "").is_some());
        assert!(matches(&x, Some(StoreId::Epic), "").is_none());
        assert!(matches(&x, None, "hades").is_none());
    }
}
