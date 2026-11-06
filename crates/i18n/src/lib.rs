//! Traductions de l'interface (PROMPT §2.7) : français par défaut, anglais en second.
//!
//! Les textes sont dans `locales/fr.toml` et `locales/en.toml`, embarqués dans le
//! binaire. La langue se change en direct : chaque texte est relu à chaque image.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

const FR: &str = include_str!("../locales/fr.toml");
const EN: &str = include_str!("../locales/en.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    /// Valeur enregistrée dans `user.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
        }
    }

    /// Valeur inconnue = français.
    pub fn from_key(key: &str) -> Lang {
        match key.trim().to_ascii_lowercase().as_str() {
            "en" | "english" | "anglais" => Lang::En,
            _ => Lang::Fr,
        }
    }

    /// Nom de la langue, écrit dans cette langue.
    pub fn name(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

pub fn current() -> Lang {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Lang::En,
        _ => Lang::Fr,
    }
}

fn parse(src: &str) -> HashMap<String, String> {
    // Les fichiers sont embarqués et vérifiés par les tests : une erreur ici ne peut
    // venir que d'une modification non testée, on retombe alors sur les clés.
    toml::from_str(src).unwrap_or_default()
}

fn table(lang: Lang) -> &'static HashMap<String, String> {
    static FR_T: OnceLock<HashMap<String, String>> = OnceLock::new();
    static EN_T: OnceLock<HashMap<String, String>> = OnceLock::new();
    match lang {
        Lang::Fr => FR_T.get_or_init(|| parse(FR)),
        Lang::En => EN_T.get_or_init(|| parse(EN)),
    }
}

/// Texte de `key` dans la langue choisie, puis en français, sinon la clé elle-même.
pub fn tr(key: &'static str) -> &'static str {
    table(current())
        .get(key)
        .or_else(|| table(Lang::Fr).get(key))
        .map(String::as_str)
        .unwrap_or(key)
}

/// Comme [`tr`], en remplaçant les `{nom}` par leurs valeurs.
pub fn fmt(key: &'static str, args: &[(&str, &str)]) -> String {
    let mut out = tr(key).to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::collections::BTreeSet;

    fn strict(src: &str) -> HashMap<String, String> {
        toml::from_str(src).unwrap()
    }

    fn placeholders(s: &str) -> BTreeSet<String> {
        s.split('{')
            .skip(1)
            .filter_map(|p| p.split_once('}').map(|(n, _)| n.to_string()))
            .collect()
    }

    #[test]
    fn both_files_have_the_same_keys_and_placeholders() {
        let fr = strict(FR);
        let en = strict(EN);
        let fk: BTreeSet<_> = fr.keys().collect();
        let ek: BTreeSet<_> = en.keys().collect();
        assert_eq!(fk, ek, "clés différentes entre fr.toml et en.toml");
        for (k, v) in &fr {
            assert_eq!(placeholders(v), placeholders(&en[k]), "clé {k}");
            assert!(!en[k].trim().is_empty(), "clé {k} vide en anglais");
        }
    }

    #[test]
    fn switches_language_live() {
        set(Lang::En);
        assert_eq!(tr("play"), "Play");
        assert_eq!(fmt("install", &[("store", "Steam")]), "Install Steam");
        set(Lang::Fr);
        assert_eq!(tr("play"), "Jouer");
        assert_eq!(fmt("install", &[("store", "Steam")]), "Installer Steam");
        assert_eq!(tr("clé-inconnue"), "clé-inconnue");
        assert_eq!(Lang::from_key("EN"), Lang::En);
        assert_eq!(Lang::from_key("de"), Lang::Fr);
    }
}
