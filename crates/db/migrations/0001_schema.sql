-- Schéma de départ (PROMPT §6). Un `game` = un jeu logique ; un `copy` = un exemplaire
-- sur une boutique. Les tables des jalons suivants sont créées dès maintenant pour que
-- les migrations restent linéaires.
CREATE TABLE games (
    id               INTEGER PRIMARY KEY,
    canonical_title  TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    created_at       INTEGER NOT NULL
);
CREATE INDEX games_normalized ON games(normalized_title);

CREATE TABLE copies (
    id               INTEGER PRIMARY KEY,
    game_id          INTEGER NOT NULL REFERENCES games(id),
    store            TEXT NOT NULL,
    external_id      TEXT NOT NULL,
    title            TEXT NOT NULL,
    installed        INTEGER NOT NULL DEFAULT 0,
    install_path     TEXT,
    size_bytes       INTEGER,
    playtime_minutes INTEGER,
    -- Ajouts du Jalon 1 : de quoi réafficher la bibliothèque sans rescanner.
    last_played      INTEGER,
    covers_json      TEXT NOT NULL DEFAULT '{}',
    launch_json      TEXT NOT NULL,
    seen_at          INTEGER NOT NULL,
    UNIQUE(store, external_id)
);

CREATE TABLE sessions (
    id         INTEGER PRIMARY KEY,
    copy_id    INTEGER NOT NULL REFERENCES copies(id),
    started_at INTEGER NOT NULL,
    ended_at   INTEGER,
    source     TEXT NOT NULL
);
CREATE TABLE tags (id INTEGER PRIMARY KEY, key TEXT UNIQUE NOT NULL, label TEXT NOT NULL, color TEXT);
CREATE TABLE game_tags (game_id INTEGER NOT NULL, tag_id INTEGER NOT NULL, PRIMARY KEY (game_id, tag_id));
CREATE TABLE collections (id INTEGER PRIMARY KEY, key TEXT UNIQUE NOT NULL, name TEXT NOT NULL, color TEXT, kind TEXT NOT NULL);
CREATE TABLE collection_games (collection_id INTEGER NOT NULL, game_id INTEGER NOT NULL, PRIMARY KEY (collection_id, game_id));
CREATE TABLE smart_rules (collection_id INTEGER NOT NULL, rule_json TEXT NOT NULL);
CREATE TABLE merge_overrides (game_id INTEGER PRIMARY KEY, preferred_store TEXT, merged INTEGER NOT NULL DEFAULT 0);
CREATE TABLE price_alerts (id INTEGER PRIMARY KEY, game_id INTEGER NOT NULL, target_cents INTEGER NOT NULL, currency TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1);
CREATE TABLE price_history (game_id INTEGER NOT NULL, shop TEXT NOT NULL, price_cents INTEGER NOT NULL, seen_at INTEGER NOT NULL);
