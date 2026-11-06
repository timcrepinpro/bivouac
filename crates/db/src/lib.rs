//! `library.db` (PROMPT §6) : SQLite, migrations versionnées par `PRAGMA user_version`.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use launcher_core::search::normalize;
use launcher_core::{sort_library, Covers, InstalledGame, LaunchAction, StoreId};
use rusqlite::{params, Connection, OptionalExtension};

/// Migrations dans l'ordre. On n'en modifie jamais une déjà publiée : on en ajoute une.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/0001_schema.sql")];

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("base de données : {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("donnée illisible dans la base : {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "la base vient d'une version plus récente du launcher (schéma {found}, connu {known})"
    )]
    TooNew { found: i64, known: i64 },
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, DbError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Version actuelle du schéma.
    pub fn schema_version(&self) -> Result<i64, DbError> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }

    fn migrate(&mut self) -> Result<(), DbError> {
        let known = MIGRATIONS.len() as i64;
        let found = self.schema_version()?;
        if found > known {
            return Err(DbError::TooNew { found, known });
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(found as usize) {
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", i as i64 + 1)?;
            tx.commit()?;
        }
        Ok(())
    }

    /// Enregistre le résultat d'un scan réussi d'une boutique : ajoute ou met à jour
    /// chaque exemplaire, et marque « désinstallés » ceux qui ont disparu.
    /// À n'appeler que si la boutique a bien été lue (sinon on garderait l'ancien état).
    pub fn sync_store(&mut self, store: StoreId, games: &[InstalledGame]) -> Result<(), DbError> {
        let now = now();
        let tx = self.conn.transaction()?;
        for g in games.iter().filter(|g| g.store == store) {
            let norm = normalize(&g.title);
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT game_id FROM copies WHERE store = ?1 AND external_id = ?2",
                    params![store.key(), g.external_id],
                    |r| r.get(0),
                )
                .optional()?;
            let game_id = match existing {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO games (canonical_title, normalized_title, created_at) VALUES (?1, ?2, ?3)",
                        params![g.title, norm, now],
                    )?;
                    tx.last_insert_rowid()
                }
            };
            tx.execute(
                "INSERT INTO copies (game_id, store, external_id, title, installed, install_path,
                    size_bytes, playtime_minutes, last_played, covers_json, launch_json, seen_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(store, external_id) DO UPDATE SET
                    title = excluded.title, installed = 1, install_path = excluded.install_path,
                    size_bytes = excluded.size_bytes,
                    playtime_minutes = COALESCE(excluded.playtime_minutes, copies.playtime_minutes),
                    last_played = COALESCE(excluded.last_played, copies.last_played),
                    covers_json = excluded.covers_json, launch_json = excluded.launch_json,
                    seen_at = excluded.seen_at",
                params![
                    game_id,
                    store.key(),
                    g.external_id,
                    g.title,
                    g.install_path,
                    g.size_bytes.map(|v| v as i64),
                    g.playtime_minutes.map(|v| v as i64),
                    g.last_played.map(|v| v as i64),
                    serde_json::to_string(&g.covers)?,
                    serde_json::to_string(&g.launch)?,
                    now,
                ],
            )?;
        }
        tx.execute(
            "UPDATE copies SET installed = 0 WHERE store = ?1 AND seen_at < ?2",
            params![store.key(), now],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Les jeux installés connus, triés comme la bibliothèque.
    pub fn installed_games(&self) -> Result<Vec<InstalledGame>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT store, external_id, title, install_path, size_bytes, playtime_minutes,
                    last_played, covers_json, launch_json
             FROM copies WHERE installed = 1",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (store, external_id, title, install_path, size, play, last, covers, launch) = row?;
            // Une boutique inconnue (base d'une version future) est ignorée, pas fatale.
            let Some(store) = StoreId::ALL.into_iter().find(|s| s.key() == store) else {
                continue;
            };
            out.push(InstalledGame {
                store,
                external_id,
                title,
                install_path,
                size_bytes: size.map(|v| v as u64),
                playtime_minutes: play.map(|v| v as u64),
                last_played: last.map(|v| v as u64),
                covers: serde_json::from_str::<Covers>(&covers)?,
                launch: serde_json::from_str::<LaunchAction>(&launch)?,
            });
        }
        sort_library(&mut out);
        Ok(out)
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(store: StoreId, id: &str, title: &str) -> InstalledGame {
        InstalledGame {
            store,
            external_id: id.into(),
            title: title.into(),
            install_path: Some(format!("C:/jeux/{id}")),
            size_bytes: Some(1000),
            playtime_minutes: Some(60),
            last_played: None,
            covers: Covers::default(),
            launch: LaunchAction::Uri {
                uri: format!("steam://rungameid/{id}"),
            },
        }
    }

    #[test]
    fn migrates_to_latest_and_is_idempotent() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.schema_version().unwrap(), MIGRATIONS.len() as i64);
        let mut db = db;
        db.migrate().unwrap();
        assert_eq!(db.schema_version().unwrap(), MIGRATIONS.len() as i64);
    }

    #[test]
    fn refuses_a_newer_schema() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        assert!(matches!(
            Db::init(conn),
            Err(DbError::TooNew { found: 99, .. })
        ));
    }

    #[test]
    fn sync_round_trips_and_marks_removed_games() {
        let mut db = Db::open_in_memory().unwrap();
        let a = game(StoreId::Steam, "1", "Hades");
        let b = game(StoreId::Steam, "2", "Celeste");
        db.sync_store(StoreId::Steam, &[a.clone(), b.clone()])
            .unwrap();
        let all = db.installed_games().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&a));

        // Celeste a été désinstallé : il disparaît de la liste, sa ligne reste.
        db.sync_store(StoreId::Steam, std::slice::from_ref(&a))
            .unwrap();
        assert_eq!(db.installed_games().unwrap(), vec![a]);
        let rows: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM copies", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2);
    }

    #[test]
    fn a_store_sync_leaves_other_stores_alone() {
        let mut db = Db::open_in_memory().unwrap();
        let s = game(StoreId::Steam, "1", "Hades");
        let e = game(StoreId::Epic, "Fortune", "Hades");
        db.sync_store(StoreId::Steam, std::slice::from_ref(&s))
            .unwrap();
        db.sync_store(StoreId::Epic, std::slice::from_ref(&e))
            .unwrap();
        db.sync_store(StoreId::Steam, &[s]).unwrap();
        assert_eq!(db.installed_games().unwrap().len(), 2);
    }

    #[test]
    fn keeps_known_playtime_when_a_scan_has_none() {
        let mut db = Db::open_in_memory().unwrap();
        let a = game(StoreId::Steam, "1", "Hades");
        db.sync_store(StoreId::Steam, std::slice::from_ref(&a))
            .unwrap();
        let mut b = a.clone();
        b.playtime_minutes = None;
        db.sync_store(StoreId::Steam, &[b]).unwrap();
        assert_eq!(db.installed_games().unwrap()[0].playtime_minutes, Some(60));
    }

    #[test]
    fn reopens_from_disk() {
        let dir = std::env::temp_dir().join(format!("launcher-db-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("library.db");
        {
            let mut db = Db::open(&path).unwrap();
            db.sync_store(StoreId::Steam, &[game(StoreId::Steam, "1", "Hades")])
                .unwrap();
        }
        let db = Db::open(&path).unwrap();
        assert_eq!(db.installed_games().unwrap().len(), 1);
        drop(db);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
