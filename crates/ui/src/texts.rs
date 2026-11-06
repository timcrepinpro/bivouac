//! Textes de l'interface. Ils viennent des fichiers de traduction
//! `crates/i18n/locales/fr.toml` et `en.toml` (PROMPT §2.7) : rien n'est écrit en dur
//! ici, chaque fonction renvoie le texte dans la langue choisie.
#![allow(non_snake_case)]

use launcher_core::StoreId;
use launcher_i18n::{fmt, tr};

macro_rules! texts {
    ($($name:ident = $key:literal;)*) => {
        $(pub fn $name() -> &'static str { tr($key) })*
    };
}

texts! {
    SEARCH = "search";
    ALL = "all";
    ALL_GAMES = "all_games";
    RECENTS = "recents";
    RESULTS = "results";
    NEVER_PLAYED = "never_played";
    PAGE_SOON = "page_soon";
    RESCAN = "rescan";
    PLAY = "play";
    PLAYTIME = "playtime";
    SIZE = "size";
    RECENT = "recent";
    INSTALLED = "installed";
    RESUME = "resume";
    SCANNING = "scanning";
    NO_GAME = "no_game";
    NO_GAME_HINT = "no_game_hint";
    NO_MATCH = "no_match";
    NOT_FOUND = "not_found";
    LAUNCHING = "launching";
    VIA = "via";
    SPIKE = "spike";
    SPIKE_TITLE = "spike_title";
    ACCENT = "accent";
    MODE = "mode";
    TRANSITION = "transition";
    HELP = "help";
    WAIT_SCAN = "wait_scan";
    FIX = "fix";
    CANCEL = "cancel";
    STILL_VISIBLE = "still_visible";
    PICK_UNAVAILABLE = "pick_unavailable";
    COLLECTIONS = "collections";
    DUPLICATES = "duplicates";
    STATS = "stats";
    USE_CODE_LINK = "use_code_link";
    LIBRARY = "library";
    BACK_LIBRARY = "back_library";
    PLATFORMS = "platforms";
    ALL_PLATFORMS = "all_platforms";
    INSTALLED_PL = "installed_pl";
    NEW_COLLECTION = "new_collection";
    SORT_BY = "sort_by";
    SORT_RECENT = "sort_recent";
    SORT_NAME = "sort_name";
    SORT_PLAYTIME = "sort_playtime";
    SORT_SIZE = "sort_size";
    GRID = "grid";
    LIST = "list";
    INSTALLATION = "installation";
    FOLDER = "folder";
    VERIFY = "verify";
    UNINSTALL = "uninstall";
    TOOL_SENT = "tool_sent";
    LAST_SESSION = "last_session";
    NOT_PLAYED_YET = "not_played_yet";
    ACHIEVEMENTS = "achievements";
    ACHIEVEMENTS_LATER = "achievements_later";
    LAUNCH_ASKING = "launch_asking";
    CLOSE = "close";
    GAME_STARTING = "game_starting";
    SETTINGS_ACCOUNTS = "settings_accounts";
    SETTINGS_FOLDERS = "settings_folders";
    SETTINGS_LOOK = "settings_look";
    ACCOUNTS_LEAD = "accounts_lead";
    DETECTED = "detected";
    NOT_DETECTED = "not_detected";
    LATER_STORE = "later_store";
    DOWNLOAD_APP = "download_app";
    CHOOSE_FOLDER = "choose_folder";
    INSTALL_FOLDERS = "install_folders";
    NO_FOLDER = "no_folder";
    DOWNLOADS_SETTINGS_LATER = "downloads_settings_later";
    THEME = "theme";
    DARK = "dark";
    LIGHT = "light";
    AUTO = "auto";
    LIGHT_LATER = "light_later";
    ACCENT_COLOR = "accent_color";
    ACCENT_HELP = "accent_help";
    DISPLAY = "display";
    HOME_PAGE = "home_page";
    PIXEL_AND_TRANSITIONS = "pixel_and_transitions";
    INTERFACE_STYLE = "interface_style";
    INTERFACE_STYLE_HELP = "interface_style_help";
    PAGE_TRANSITIONS = "page_transitions";
    PAGE_TRANSITIONS_HELP = "page_transitions_help";
    PREVIEW = "preview";
    TRANSITION_TYPE = "transition_type";
    FLUID = "fluid";
    PIXEL = "pixel";
    SPEED = "speed";
    SPEED_FAST = "speed_fast";
    SPEED_NORMAL = "speed_normal";
    SPEED_SLOW = "speed_slow";
    REDUCED_MOTION = "reduced_motion";
    TRY_IT = "try_it";
    SHORTCUTS = "shortcuts";
    STATS_LEAD = "stats_lead";
    TOTAL_PLAYTIME = "total_playtime";
    GAMES_INSTALLED = "games_installed";
    GAMES_PLAYED = "games_played";
    LAST_PLAYED = "last_played";
    STATS_ALL_TIME = "stats_all_time";
    WHEN_YOU_PLAY = "when_you_play";
    STEAM_ACCOUNT = "steam_account";
    BY_STORE = "by_store";
    MOST_PLAYED = "most_played";
    PER_DAY = "per_day";
    PER_DAY_LATER = "per_day_later";
    NO_PLAYTIME = "no_playtime";
    PROFILE_GAMES = "profile_games";
    PLAYTIME_BY_STORE = "playtime_by_store";
    MOST_PLAYED_SHORT = "most_played_short";
    NO_ACCOUNT = "no_account";
    NO_ACCOUNT_HELP = "no_account_help";
    DUPES_TITLE = "dupes_title";
    DUPES_LEAD = "dupes_lead";
    DUPES_COUNT = "dupes_count";
    DUPES_SPACE = "dupes_space";
    NO_DUPES = "no_dupes";
    NO_DUPES_HELP = "no_dupes_help";
    COPIES = "copies";
    MOST_PLAYED_COPY = "most_played_copy";
    ALL_MY_GAMES = "all_my_games";
    AUTOMATIC = "automatic";
    NEVER_LAUNCHED = "never_launched";
    NEVER_LAUNCHED_HELP = "never_launched_help";
    TAGS = "tags";
    COLLECTIONS_LATER = "collections_later";
    COLLECTIONS_LEAD = "collections_lead";
    SEARCH_BIG = "search_big";
    EVERYTHING = "everything";
    MY_LIBRARY = "my_library";
    IN_LIBRARY = "in_library";
    SEARCH_HINT = "search_hint";
    STORE_SEARCH_LATER = "store_search_later";
    CODE_TITLE = "code_title";
    CODE_LEAD = "code_lead";
    BACK_STORE = "back_store";
    WHICH_APP = "which_app";
    YOUR_CODE = "your_code";
    CODE_LABEL = "code_label";
    CODE_PLACEHOLDER = "code_placeholder";
    WHAT_HAPPENS = "what_happens";
    IG_TITLE = "ig_title";
    IG_BODY = "ig_body";
    DOWNLOADS = "downloads";
    NO_DOWNLOAD = "no_download";
    NO_DOWNLOAD_HELP = "no_download_help";
    DISK_SPACE = "disk_space";
    USED_BY_GAMES = "used_by_games";
    STORE = "store";
    STORE_LATER_TITLE = "store_later_title";
    STORE_LATER = "store_later";
    WISHLIST = "wishlist";
    WISHLIST_EMPTY = "wishlist_empty";
    WISHLIST_LATER = "wishlist_later";
    FRIENDS_TITLE = "friends_title";
    FRIENDS_LATER_TITLE = "friends_later_title";
    FRIENDS_LATER = "friends_later";
    ALL_MESSAGES = "all_messages";
    FIND_FRIEND = "find_friend";
    MESSAGES_EMPTY = "messages_empty";
    WELCOME_TITLE = "welcome_title";
    WELCOME_LEAD = "welcome_lead";
    CONTINUE = "continue";
    SKIP = "skip";
    WELCOME_FOOT = "welcome_foot";
    SEARCHING = "searching";
    OPEN_GAME = "open_game";
}

pub fn NAV() -> [&'static str; 4] {
    [
        tr("nav.library"),
        tr("nav.store"),
        tr("nav.downloads"),
        tr("nav.friends"),
    ]
}

pub fn CODE_STEPS() -> [(&'static str, &'static str); 4] {
    [
        (tr("code_step1_title"), tr("code_step1_body")),
        (tr("code_step2_title"), tr("code_step2_body")),
        (tr("code_step3_title"), tr("code_step3_body")),
        (tr("code_step4_title"), tr("code_step4_body")),
    ]
}

/// Texte avec le nom d'une boutique.
fn with_store(key: &'static str, store: StoreId) -> String {
    fmt(key, &[("store", store.name())])
}

pub fn client_missing_title(store: StoreId) -> String {
    with_store("client_missing_title", store)
}
pub fn client_missing_body(store: StoreId, game: Option<&str>) -> String {
    let who = match game {
        Some(title) => fmt(
            "client_missing_game",
            &[("game", title), ("store", store.name())],
        ),
        None => with_store("client_missing_games", store),
    };
    format!("{who} {}", tr("client_missing_why"))
}
pub fn not_in_usual_places(store: StoreId) -> String {
    with_store("not_in_usual_places", store)
}
pub fn others_work(stores: &[StoreId]) -> String {
    let names: Vec<&str> = stores.iter().map(|s| s.name()).collect();
    let list = match names.as_slice() {
        [] => String::new(),
        [one] => (*one).to_string(),
        [rest @ .., last] => fmt("list_and", &[("rest", &rest.join(", ")), ("last", last)]),
    };
    fmt("others_work", &[("list", &list)])
}
pub fn install(store: StoreId) -> String {
    with_store("install", store)
}
pub fn locate(store: StoreId) -> String {
    with_store("locate", store)
}
pub fn install_hint(store: StoreId) -> String {
    with_store("install_hint", store)
}
pub fn install_opened(store: StoreId) -> String {
    with_store("install_opened", store)
}
pub fn pick_title(store: StoreId) -> String {
    with_store("pick_title", store)
}
pub fn not_a_client_dir(store: StoreId) -> String {
    match store {
        StoreId::Steam => tr("not_a_client_dir_steam").into(),
        other => with_store("not_a_client_dir", other),
    }
}
pub fn client_saved(store: StoreId, dir: &std::path::Path) -> String {
    fmt(
        "client_saved",
        &[("store", store.name()), ("dir", &dir.display().to_string())],
    )
}
pub fn games_count(n: usize) -> String {
    let key = if n > 1 {
        "games_count.other"
    } else {
        "games_count.one"
    };
    fmt(key, &[("n", &n.to_string())])
}
pub fn redeem_opened(store: StoreId) -> String {
    with_store("redeem_opened", store)
}
pub fn games_n(n: usize) -> String {
    match n {
        0 => tr("games_n.zero").into(),
        1 => tr("games_n.one").into(),
        _ => fmt("games_n.other", &[("n", &n.to_string())]),
    }
}
pub fn continue_on(store: StoreId) -> String {
    with_store("continue_on", store)
}
pub fn code_note(store: StoreId) -> String {
    with_store("code_note", store)
}
pub fn played_hours(min: Option<u64>) -> String {
    match min {
        Some(m) if m > 0 => fmt("played_hours", &[("h", &((m + 30) / 60).to_string())]),
        _ => tr("never_played_lc").into(),
    }
}

// ----- Formats ----------------------------------------------------------------------

/// « 12 h ».
pub fn hours_short(h: u64) -> String {
    fmt("hours_short", &[("h", &h.to_string())])
}
/// « 12,5 Go » / « 12.5 GB ».
pub fn size_gb(gb: f64) -> String {
    let n = format!("{gb:.1}").replace('.', tr("decimal"));
    fmt("size_gb", &[("n", &n)])
}
pub fn size_mb(mb: u64) -> String {
    fmt("size_mb", &[("n", &mb.to_string())])
}
/// « il y a 2 h » / « 2 h ago ». `unit` : min, h, j (jours) ou mois.
pub fn ago(n: u64, unit: &str) -> String {
    let key = match unit {
        "min" => "ago_min",
        "h" => "ago_h",
        "j" => "ago_d",
        _ => "ago_mo",
    };
    fmt(key, &[("n", &n.to_string())])
}
pub fn step_found(store: &str) -> String {
    fmt("step_found", &[("store", store)])
}
pub fn step_sent(store: &str) -> String {
    fmt("step_sent", &[("store", store)])
}
pub fn launch_via(store: &str) -> String {
    fmt("launch_via", &[("store", store)])
}
pub fn tool_unsupported(store: StoreId) -> String {
    with_store("tool_unsupported", store)
}
pub fn CATALOG() -> &'static str {
    tr("catalog")
}
pub fn ALL_STORES() -> &'static str {
    tr("all_stores")
}
pub fn LANGUAGE() -> &'static str {
    tr("language")
}
pub fn LANGUAGE_HELP() -> &'static str {
    tr("language_help")
}
/// Nom d'un réglage (`accent.orange`, `mode.normal`, `transition.fondu`…).
pub fn setting(key: &'static str) -> &'static str {
    tr(key)
}
