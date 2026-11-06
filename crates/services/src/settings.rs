//! `user.toml` (PROMPT §6) : réglages lisibles et modifiables à la main.
//! Emplacement des boutiques choisi à la main, apparence, écran Bienvenue déjà vu.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ServiceError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Langue de l'interface : fr (par défaut) ou en.
    pub langue: String,
    pub boutiques: Boutiques,
    pub apparence: Apparence,
    pub bienvenue_vue: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            langue: "fr".into(),
            boutiques: Boutiques::default(),
            apparence: Apparence::default(),
            bienvenue_vue: false,
        }
    }
}

/// Paramètres › Apparence. Des mots simples pour rester modifiable à la main ;
/// une valeur inconnue revient à la valeur par défaut.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Apparence {
    /// orange, menthe, citron, rose, ciel
    pub accent: String,
    /// normal, pixel-carre, pixel-arrondi
    pub mode: String,
    /// fondu, dissolution, balayage, rideau, aucune
    pub transition: String,
    /// Vue de la bibliothèque au démarrage : liste ou grille
    pub vue: String,
    /// Durée des transitions en millisecondes : 150, 300 ou 500
    pub vitesse: u32,
}

impl Default for Apparence {
    fn default() -> Self {
        Apparence {
            accent: "orange".into(),
            mode: "normal".into(),
            transition: "fondu".into(),
            vue: "liste".into(),
            vitesse: 300,
        }
    }
}

/// Dossiers indiqués à la main (écran « … est introuvable »).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Boutiques {
    pub steam: Option<PathBuf>,
}

/// Fichier absent = réglages par défaut.
pub fn load(path: &Path) -> Result<Settings, ServiceError> {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).map_err(|e| ServiceError::Settings {
            path: path.to_path_buf(),
            reason: e.to_string(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(source) => Err(ServiceError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), ServiceError> {
    let body = toml::to_string_pretty(settings).map_err(|e| ServiceError::Settings {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let text = format!("# Réglages du launcher. Tu peux modifier ce fichier à la main.\n\n{body}");
    std::fs::write(path, text).map_err(|source| ServiceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn reads_a_hand_written_file() {
        let s: Settings = toml::from_str("[boutiques]\nsteam = 'D:/Steam'\n").unwrap();
        assert_eq!(s.boutiques.steam, Some(PathBuf::from("D:/Steam")));
        let empty: Settings = toml::from_str("").unwrap();
        assert_eq!(empty, Settings::default());
    }
}
