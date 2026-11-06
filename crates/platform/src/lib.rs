//! Couche système, isolée ici pour porter plus tard vers Linux et macOS (PROMPT §2.6).

use std::path::PathBuf;
use std::process::Command;

use launcher_core::LaunchAction;

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("impossible d'ouvrir {uri} : {source}")]
    Uri { uri: String, source: std::io::Error },
    #[error("impossible de lancer {path} : {source}")]
    Exe {
        path: String,
        source: std::io::Error,
    },
}

/// Démarre un jeu via sa boutique. Ne bloque pas : la boutique prend le relais.
pub fn launch(action: &LaunchAction) -> Result<(), LaunchError> {
    match action {
        LaunchAction::Uri { uri } => open::that_detached(uri).map_err(|source| LaunchError::Uri {
            uri: uri.clone(),
            source,
        }),
        LaunchAction::Exe { path, args, cwd } => {
            let mut cmd = Command::new(path);
            cmd.args(args);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.spawn().map(|_| ()).map_err(|source| LaunchError::Exe {
                path: path.clone(),
                source,
            })
        }
    }
}

/// L'utilisateur a-t-il coupé les animations du système ?
/// Windows : Paramètres › Accessibilité › Effets visuels › « Effets d'animation ».
#[cfg(windows)]
pub fn reduce_motion() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION,
    };
    let mut on: i32 = 1;
    // SAFETY : l'appel écrit un BOOL (i32) dans `on`, qui vit jusqu'à la fin de l'appel.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut on as *mut i32).cast(),
            0,
        )
    };
    ok != 0 && on == 0
}

/// Ailleurs : pas encore lu, les animations restent actives.
#[cfg(not(windows))]
pub fn reduce_motion() -> bool {
    false
}

/// `C:\ProgramData` (ou sa valeur réelle).
pub fn program_data() -> PathBuf {
    std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
}

/// Dossier des données du launcher (PROMPT §6) : `%APPDATA%\launcher` sous Windows,
/// `$XDG_DATA_HOME/launcher` (ou `~/.local/share/launcher`) ailleurs.
pub fn data_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("launcher"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|h| h.join(".local/share")))
            .map(|p| p.join("launcher"))
    }
}

pub fn home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Lecture du registre Windows. Sur les autres systèmes, tout renvoie `None` ou vide.
pub mod registry {
    #[derive(Debug, Clone, Copy)]
    pub enum Hive {
        CurrentUser,
        LocalMachine,
    }

    #[cfg(windows)]
    fn open(hive: Hive, key: &str) -> Option<winreg::RegKey> {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        let root = winreg::RegKey::predef(match hive {
            Hive::CurrentUser => HKEY_CURRENT_USER,
            Hive::LocalMachine => HKEY_LOCAL_MACHINE,
        });
        root.open_subkey(key).ok()
    }

    /// Valeur texte d'une clé.
    pub fn string(hive: Hive, key: &str, value: &str) -> Option<String> {
        #[cfg(windows)]
        {
            open(hive, key)?
                .get_value::<String, _>(value)
                .ok()
                .filter(|s| !s.is_empty())
        }
        #[cfg(not(windows))]
        {
            let _ = (hive, key, value);
            None
        }
    }

    /// Noms des sous-clés.
    pub fn subkeys(hive: Hive, key: &str) -> Vec<String> {
        #[cfg(windows)]
        {
            open(hive, key)
                .map(|k| k.enum_keys().flatten().collect())
                .unwrap_or_default()
        }
        #[cfg(not(windows))]
        {
            let _ = (hive, key);
            Vec::new()
        }
    }
}
