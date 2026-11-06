//! Boucle de l'application : état, messages, navigation entre les pages.
//! Chaque page a son fichier dans `app/` et suit la maquette du même nom.

mod empty;
mod find;
mod game;
mod grid;
mod library;
mod missing;
mod settings;
mod smart;
mod stats;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::font::Weight;
use iced::keyboard::{self, key::Named, Key};
use iced::widget::scrollable::{AbsoluteOffset, Viewport};
use iced::widget::Id;
use iced::{window, Subscription, Task};
use launcher_core::search::normalize;
use launcher_core::{search, Account, InstalledGame, StoreId};
use launcher_services::{
    Apparence, ClientChoice, ClientStatus, GameTool, Paths, PlayError, Refresh,
};

use crate::bench::{fake_games, Report};
use crate::texts as t;
use crate::theme::{self, Accent, Mode};
use crate::transition::Kind;

/// Ligne de jeu de la liste : 50 px + 2 px d'écart (maquette Mix2).
const ROW_H: f32 = 52.0;
/// Titres de section de la liste (« RÉCENTS », « TOUS LES JEUX · N → »).
const HEAD_FIRST_H: f32 = 27.0;
const HEAD_H: f32 = 39.0;
/// Nombre de jeux dans la section « Récents ».
const RECENTS: usize = 4;
const SEARCH_ID: &str = "recherche";
const LIST_ID: &str = "liste-jeux";
/// Durée de l'écran Lancement une fois la demande transmise à la boutique.
const LAUNCH_SCREEN: Duration = Duration::from_secs(6);

/// Options de lancement (ligne de commande).
#[derive(Debug, Clone)]
pub struct Options {
    /// Instant du tout début de `main`, pour mesurer le démarrage.
    pub started: Instant,
    /// Mode mesure : 2 000 jeux, défilement automatique, rapport écrit puis sortie.
    pub bench: bool,
    /// Ne pas scanner le PC : uniquement des jeux fictifs.
    pub demo: Option<usize>,
}

struct Transition {
    start: Instant,
    kind: Kind,
    target: Target,
    swapped: bool,
}

/// Ce qui change au milieu d'une transition : le jeu affiché ou la page entière.
enum Target {
    Game(usize),
    Page(Box<Page>),
}

/// Lancement d'un jeu (maquette `Lancement`) : ce que le launcher sait vraiment.
#[derive(Debug, Clone, PartialEq)]
enum LaunchState {
    /// Demande en cours auprès de la boutique.
    Asking,
    /// La boutique a reçu la demande ; le jeu démarre de son côté.
    Sent,
    Failed(String),
}

/// Onglets de Paramètres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Accounts,
    Folders,
    Appearance,
}

/// Tri de la Grille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sort {
    Recent,
    Name,
    Playtime,
    Size,
}

impl Sort {
    const ALL: [Sort; 4] = [Sort::Recent, Sort::Name, Sort::Playtime, Sort::Size];
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Sort::Recent => t::SORT_RECENT(),
            Sort::Name => t::SORT_NAME(),
            Sort::Playtime => t::SORT_PLAYTIME(),
            Sort::Size => t::SORT_SIZE(),
        })
    }
}

struct Bench {
    phase: BenchPhase,
    last: Option<Instant>,
    offset: f32,
    report: Report,
}

#[derive(PartialEq)]
enum BenchPhase {
    Scroll,
    Transition,
    Done,
}

/// Page affichée.
#[derive(Debug, Clone, PartialEq)]
enum Page {
    /// Maquette `Bienvenue`, au premier démarrage.
    Welcome,
    /// Maquette `Mix2` (vue liste).
    Library,
    /// Maquette `Grille`.
    Grid,
    /// Maquette `Jeu-Possede`.
    Game(usize),
    /// Maquette `Lancement`.
    Launching {
        game: usize,
        at: Instant,
        state: LaunchState,
    },
    Settings(Tab),
    Stats,
    Profile,
    Duplicates,
    Collections,
    Search,
    Code,
    Downloads,
    Store,
    Wishlist,
    Friends,
    Messages,
    /// Maquette `Lancement-App` : la boutique d'un jeu est introuvable.
    ClientMissing {
        store: StoreId,
        game: Option<usize>,
        /// Message sous les boutons (dossier refusé, etc.).
        note: Option<String>,
    },
}

/// Une ligne de la liste latérale.
#[derive(Debug, Clone, PartialEq)]
enum Entry {
    Header(String),
    Game(usize),
}

struct App {
    opts: Options,
    /// Lignes de la liste (titres + jeux) et leur position verticale.
    entries: Vec<Entry>,
    tops: Vec<f32>,
    paths: Paths,
    page: Page,
    /// Page à retrouver avec « ← » (fiche d'un jeu, lancement).
    back: Page,
    /// Vue de la bibliothèque choisie (liste `Mix2` ou `Grille`).
    grid_view: bool,
    sort: Sort,
    accounts: Vec<Account>,
    code_store: StoreId,
    code: String,
    /// Collections : collection automatique « Jamais lancés » affichée.
    coll_never: bool,
    games: Vec<InstalledGame>,
    /// Titres normalisés, calculés une fois par chargement (recherche floue).
    norms: Vec<String>,
    visible: Vec<usize>,
    clients: Vec<(StoreId, ClientStatus)>,
    scanning: bool,
    query: String,
    store: Option<StoreId>,
    selected: Option<usize>,
    accent: Accent,
    mode: Mode,
    transition_kind: Kind,
    /// Durée d'une transition (150, 300 ou 500 ms).
    transition_ms: u64,
    /// Préférence système « réduire les animations » : transitions coupées.
    reduce_motion: bool,
    transition: Option<Transition>,
    scroll_y: f32,
    list_h: f32,
    toast: Option<(String, Instant)>,
    show_fps: bool,
    fps_last: Option<Instant>,
    fps: f32,
    first_frame_seen: bool,
    bench: Option<Bench>,
}

#[derive(Debug, Clone)]
enum Msg {
    /// Bibliothèque lue dans la base : affichage immédiat au démarrage.
    Cached(Result<Vec<InstalledGame>, String>),
    /// Scan des boutiques terminé et base à jour (durée totale).
    Refreshed(Result<Refresh, String>, Duration),
    Search(String),
    Filter(Option<StoreId>),
    Select(usize),
    Play,
    Launched(Result<(), PlayError>),
    Rescan,
    ShowClientMissing(StoreId),
    InstallClient(StoreId),
    InstallOpened(StoreId, Result<(), String>),
    PickClientDir(StoreId),
    #[cfg_attr(not(windows), allow(dead_code))]
    ClientDirPicked(StoreId, Option<PathBuf>),
    ClientDirSaved(Result<ClientChoice, String>),
    BackToLibrary,
    Scrolled(Viewport),
    Frame(Instant),
    Key(keyboard::Event),
    SetAccent(Accent),
    SetMode(Mode),
    SetTransition(Kind),
    ToggleTransitions,
    /// « Voir l'aperçu » : rejoue la transition sur la page actuelle.
    PreviewTransition,
    SetSpeed(u64),
    SetLang(launcher_i18n::Lang),
    SetGridView(bool),
    SetSort(Sort),
    /// Réglages enregistrés dans `user.toml` (ou l'erreur à afficher).
    Saved(Result<(), String>),
    /// Onglet de la navigation principale.
    Nav(usize),
    Go(Page),
    /// Icône loupe de l'en-tête : page Recherche, champ actif.
    FocusSearch,
    OpenGame(usize),
    PlayGame(usize),
    Tool(usize, GameTool),
    ToolDone(Result<(), String>),
    CodeStore(StoreId),
    CodeInput(String),
    Redeem,
    Redeemed(Result<(), String>),
    FinishWelcome,
    CollNever(bool),
}

pub fn run(opts: Options) -> iced::Result {
    let mut app = iced::application(move || App::boot(opts.clone()), App::update, App::view)
        .title("Launcher")
        .subscription(App::subscription)
        .window_size((1440.0, 900.0))
        .default_font(Mode::Normal.font(Weight::Normal))
        .antialiasing(true);
    for f in theme::FONTS {
        app = app.font(f);
    }
    app.run()
}

impl App {
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        iced_wgpu::pixel::set_factor(self.mode.pixel_factor());
    }

    /// Enregistre l'apparence dans `user.toml`, sans bloquer l'interface.
    fn save_appearance(&self) -> Task<Msg> {
        if self.opts.demo.is_some() {
            return Task::none();
        }
        let a = Apparence {
            accent: self.accent.key().into(),
            mode: self.mode.key().into(),
            transition: self.transition_kind.key().into(),
            vitesse: self.transition_ms as u32,
            vue: if self.grid_view { "grille" } else { "liste" }.into(),
        };
        let paths = self.paths.clone();
        Task::perform(
            async move { launcher_services::save_appearance(&paths, a).map_err(|e| e.to_string()) },
            Msg::Saved,
        )
    }

    /// Page de la bibliothèque choisie (liste ou grille).
    fn home(&self) -> Page {
        if self.grid_view {
            Page::Grid
        } else {
            Page::Library
        }
    }

    /// Transition à jouer : aucune si le système demande moins d'animations.
    fn effective_kind(&self) -> Kind {
        if self.reduce_motion {
            Kind::Aucune
        } else {
            self.transition_kind
        }
    }

    /// Change de page, avec la transition choisie.
    fn go(&mut self, page: Page) {
        if page == self.page {
            return;
        }
        if matches!(page, Page::Game(_) | Page::Launching { .. })
            && !matches!(self.page, Page::Game(_) | Page::Launching { .. })
        {
            self.back = self.page.clone();
        }
        match self.effective_kind() {
            Kind::Aucune => self.page = page,
            kind => {
                self.transition = Some(Transition {
                    start: Instant::now(),
                    kind,
                    target: Target::Page(Box::new(page)),
                    swapped: false,
                })
            }
        }
    }

    fn now_unix() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn boot(opts: Options) -> (Self, Task<Msg>) {
        let bench = opts.bench.then(|| Bench {
            phase: BenchPhase::Scroll,
            last: None,
            offset: 0.0,
            report: Report::default(),
        });
        let demo = opts.demo;
        // Sans APPDATA (cas anormal), on garde le launcher utilisable dans le dossier temporaire.
        let paths = Paths::system().unwrap_or_else(|_| Paths {
            dir: std::env::temp_dir().join("launcher"),
        });
        let first = load_task(demo, paths.clone());
        // Petit fichier texte : lu tout de suite pour démarrer avec la bonne apparence.
        let settings = if demo.is_some() {
            launcher_services::Settings::default()
        } else {
            launcher_services::load_settings(&paths).unwrap_or_default()
        };
        launcher_i18n::set(launcher_i18n::Lang::from_key(&settings.langue));
        let a = &settings.apparence;
        let grid_view = a.vue == "grille";
        let start = if demo.is_none() && !settings.bienvenue_vue && !opts.bench {
            Page::Welcome
        } else if grid_view {
            Page::Grid
        } else {
            Page::Library
        };
        // Captures des pages pendant le développement (jamais dans l'exe livré).
        #[cfg(debug_assertions)]
        let start = std::env::var("LAUNCHER_PAGE")
            .ok()
            .and_then(|p| debug_page(&p))
            .unwrap_or(start);
        let mode = Mode::from_key(&a.mode);
        iced_wgpu::pixel::set_factor(mode.pixel_factor());
        let app = App {
            opts,
            entries: Vec::new(),
            tops: vec![0.0],
            paths,
            page: start,
            back: Page::Library,
            grid_view,
            sort: Sort::Recent,
            accounts: Vec::new(),
            code_store: StoreId::Steam,
            code: String::new(),
            coll_never: false,
            games: Vec::new(),
            norms: Vec::new(),
            visible: Vec::new(),
            clients: Vec::new(),
            scanning: true,
            query: String::new(),
            store: None,
            selected: None,
            accent: Accent::from_key(&a.accent),
            mode,
            transition_kind: Kind::from_key(&a.transition),
            transition_ms: match a.vitesse {
                0..=200 => 150,
                201..=400 => 300,
                _ => 500,
            },
            reduce_motion: launcher_services::reduce_motion(),
            transition: None,
            scroll_y: 0.0,
            list_h: 700.0,
            toast: None,
            show_fps: false,
            fps_last: None,
            fps: 0.0,
            first_frame_seen: false,
            bench,
        };
        (app, first)
    }

    /// Recherche floue : les meilleurs résultats d'abord, sinon l'ordre de la bibliothèque.
    fn refilter(&mut self) {
        let q = normalize(&self.query);
        let mut hits: Vec<(i32, usize)> = (0..self.games.len())
            .filter(|&i| self.store.is_none_or(|s| s == self.games[i].store))
            .filter_map(|i| search::score(&q, &self.norms[i]).map(|sc| (sc, i)))
            .collect();
        if !q.is_empty() {
            hits.sort_by_key(|h| std::cmp::Reverse(h.0));
        }
        let all: Vec<usize> = hits.into_iter().map(|(_, i)| i).collect();

        // Sans recherche ni filtre : « Récents » (derniers joués) puis « Tous les jeux ».
        let mut entries = Vec::new();
        let mut visible = Vec::new();
        let browsing = q.is_empty() && self.store.is_none();
        let recents: Vec<usize> = if browsing {
            all.iter()
                .copied()
                .filter(|&i| self.games[i].last_played.is_some())
                .take(RECENTS)
                .collect()
        } else {
            Vec::new()
        };
        if !recents.is_empty() {
            entries.push(Entry::Header(t::RECENTS().to_string()));
            for &i in &recents {
                entries.push(Entry::Game(i));
                visible.push(i);
            }
        }
        let label = if q.is_empty() {
            format!("{} · {} →", t::ALL_GAMES(), all.len())
        } else {
            format!("{} · {}", t::RESULTS(), all.len())
        };
        entries.push(Entry::Header(label));
        for &i in &all {
            if !recents.contains(&i) {
                entries.push(Entry::Game(i));
                visible.push(i);
            }
        }
        let mut tops = Vec::with_capacity(entries.len() + 1);
        let mut y = 0.0;
        for (n, e) in entries.iter().enumerate() {
            tops.push(y);
            y += match e {
                Entry::Header(_) if n == 0 => HEAD_FIRST_H,
                Entry::Header(_) => HEAD_H,
                Entry::Game(_) => ROW_H,
            };
        }
        tops.push(y);
        self.entries = entries;
        self.tops = tops;
        self.visible = visible;
    }

    /// Position verticale de la ligne d'un jeu dans la liste.
    fn row_top(&self, game: usize) -> Option<f32> {
        self.entries
            .iter()
            .position(|e| *e == Entry::Game(game))
            .and_then(|n| self.tops.get(n).copied())
    }

    fn list_height(&self) -> f32 {
        self.tops.last().copied().unwrap_or(0.0)
    }

    /// Remplace la liste en gardant le jeu sélectionné s'il est toujours là.
    fn set_games(&mut self, games: Vec<InstalledGame>) {
        let keep = self
            .selected
            .and_then(|i| self.games.get(i))
            .map(InstalledGame::key);
        self.norms = games.iter().map(|g| normalize(&g.title)).collect();
        self.games = games;
        self.refilter();
        self.selected = keep
            .and_then(|k| self.games.iter().position(|g| g.key() == k))
            .or_else(|| self.visible.first().copied());
    }

    fn client(&self, store: StoreId) -> Option<&ClientStatus> {
        self.clients
            .iter()
            .find(|(s, _)| *s == store)
            .map(|(_, c)| c)
    }

    fn select(&mut self, i: usize) {
        if self.selected == Some(i) {
            return;
        }
        match self.effective_kind() {
            Kind::Aucune => self.selected = Some(i),
            kind => {
                self.transition = Some(Transition {
                    start: Instant::now(),
                    kind,
                    target: Target::Game(i),
                    swapped: false,
                })
            }
        }
    }

    /// Jeux de la Grille : filtre de boutique puis tri choisi.
    fn grid_games(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.games.len())
            .filter(|&i| self.store.is_none_or(|s| s == self.games[i].store))
            .collect();
        let g = &self.games;
        match self.sort {
            Sort::Recent => {}
            Sort::Name => v.sort_by_key(|&i| g[i].title.to_lowercase()),
            Sort::Playtime => {
                v.sort_by_key(|&i| std::cmp::Reverse(g[i].playtime_minutes.unwrap_or(0)))
            }
            Sort::Size => v.sort_by_key(|&i| std::cmp::Reverse(g[i].size_bytes.unwrap_or(0))),
        }
        v
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Cached(Ok(games)) => {
                // Le scan peut avoir déjà fini (base vide, PC rapide) : on ne l'écrase pas.
                if self.scanning {
                    self.set_games(games);
                }
            }
            Msg::Cached(Err(e)) => self.toast = Some((e, Instant::now())),
            Msg::Refreshed(res, took) => {
                self.scanning = false;
                let res = match res {
                    Ok(r) => r,
                    Err(e) => {
                        self.toast = Some((e, Instant::now()));
                        return Task::none();
                    }
                };
                let mut games = res.games;
                self.clients = res.clients;
                self.accounts = res.accounts;
                if self.bench.is_some() {
                    let missing = 2000usize.saturating_sub(games.len());
                    games.extend(fake_games(missing));
                }
                self.set_games(games);
                if self.bench.is_some() {
                    let t0 = Instant::now();
                    self.query = "ombre".into();
                    self.refilter();
                    let filter = t0.elapsed();
                    self.query.clear();
                    self.refilter();
                    let n = self.games.len();
                    if let Some(b) = &mut self.bench {
                        b.report.scan = Some(res.scan_took.min(took));
                        b.report.games = n;
                        b.report.filter = Some(filter);
                    }
                }
                if !res.errors.is_empty() {
                    self.toast = Some((res.errors.join(" · "), Instant::now()));
                }
            }
            Msg::Search(q) => {
                self.query = q;
                self.refilter();
                self.scroll_y = 0.0;
                return iced::widget::operation::scroll_to(
                    LIST_ID,
                    AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Msg::Filter(s) => {
                self.store = s;
                self.refilter();
            }
            Msg::Select(i) => self.select(i),
            Msg::Play => {
                if self.scanning && self.clients.is_empty() {
                    self.toast = Some((t::WAIT_SCAN().to_string(), Instant::now()));
                    return Task::none();
                }
                if let Some((i, g)) = self
                    .selected
                    .and_then(|i| self.games.get(i).map(|g| (i, g)))
                {
                    let game = g.clone();
                    self.go(Page::Launching {
                        game: i,
                        at: Instant::now(),
                        state: LaunchState::Asking,
                    });
                    let clients = self.clients.clone();
                    return Task::perform(
                        async move { launcher_services::play(&game, &clients) },
                        Msg::Launched,
                    );
                }
            }
            Msg::Launched(Err(PlayError::ClientMissing(store))) => {
                self.toast = None;
                self.transition = None;
                self.page = Page::ClientMissing {
                    store,
                    game: self.selected,
                    note: None,
                };
            }
            Msg::Launched(res) => {
                let new = match res {
                    Ok(()) => LaunchState::Sent,
                    Err(PlayError::Launch(e)) => LaunchState::Failed(e),
                    Err(PlayError::ClientMissing(_)) => LaunchState::Sent,
                };
                let page = match &mut self.transition {
                    Some(Transition {
                        target: Target::Page(p),
                        ..
                    }) => p.as_mut(),
                    _ => &mut self.page,
                };
                if let Page::Launching { state, .. } = page {
                    *state = new;
                }
            }
            Msg::Rescan => {
                self.scanning = true;
                return refresh_task(self.opts.demo, self.paths.clone());
            }
            Msg::ShowClientMissing(store) => {
                self.page = Page::ClientMissing {
                    store,
                    game: None,
                    note: None,
                }
            }
            Msg::InstallClient(store) => {
                return Task::perform(
                    async move { launcher_services::open_install_page(store) },
                    move |r| Msg::InstallOpened(store, r),
                );
            }
            Msg::InstallOpened(store, r) => {
                let note = match r {
                    Ok(()) => t::install_opened(store),
                    Err(e) => e,
                };
                if let Page::ClientMissing { note: n, .. } = &mut self.page {
                    *n = Some(note);
                }
            }
            Msg::PickClientDir(store) => return pick_folder(store),
            Msg::ClientDirPicked(_, None) => {}
            Msg::ClientDirPicked(store, Some(dir)) => {
                let paths = self.paths.clone();
                return Task::perform(
                    async move {
                        launcher_services::choose_client_dir(&paths, store, &dir)
                            .map_err(|e| e.to_string())
                    },
                    Msg::ClientDirSaved,
                );
            }
            Msg::ClientDirSaved(Ok(ClientChoice::Saved(dir))) => {
                let store = match &self.page {
                    Page::ClientMissing { store, .. } => *store,
                    _ => StoreId::Steam,
                };
                if matches!(self.page, Page::ClientMissing { .. }) {
                    self.page = self.home();
                }
                self.toast = Some((t::client_saved(store, &dir), Instant::now()));
                self.scanning = true;
                return refresh_task(self.opts.demo, self.paths.clone());
            }
            Msg::ClientDirSaved(Ok(ClientChoice::NotAClientDir)) => {
                if let Page::ClientMissing { store, note, .. } = &mut self.page {
                    *note = Some(t::not_a_client_dir(*store));
                }
            }
            Msg::ClientDirSaved(Err(e)) => {
                if let Page::ClientMissing { note, .. } = &mut self.page {
                    *note = Some(e);
                }
            }
            Msg::BackToLibrary => {
                let back = match &self.page {
                    Page::Game(_) | Page::Launching { .. } | Page::ClientMissing { .. } => {
                        self.back.clone()
                    }
                    _ => self.home(),
                };
                self.go(back);
            }
            Msg::FocusSearch => {
                self.go(Page::Search);
                return iced::widget::operation::focus(Id::new(SEARCH_ID));
            }
            Msg::Nav(i) => {
                let page = match i {
                    0 => self.home(),
                    1 => Page::Store,
                    2 => Page::Downloads,
                    3 => Page::Friends,
                    _ => Page::Settings(Tab::Accounts),
                };
                self.go(page);
            }
            Msg::Go(page) => self.go(page),
            Msg::OpenGame(i) => {
                self.selected = Some(i);
                self.go(Page::Game(i));
            }
            Msg::PlayGame(i) => {
                self.selected = Some(i);
                return self.update(Msg::Play);
            }
            Msg::Tool(i, tool) => {
                if let Some(g) = self.games.get(i) {
                    let game = g.clone();
                    return Task::perform(
                        async move {
                            launcher_services::run_game_tool(&game, tool).map_err(|e| match e {
                                launcher_services::ToolError::Unsupported => {
                                    t::tool_unsupported(game.store)
                                }
                                launcher_services::ToolError::Launch(m) => m,
                            })
                        },
                        Msg::ToolDone,
                    );
                }
            }
            Msg::ToolDone(Ok(())) => {
                self.toast = Some((t::TOOL_SENT().to_string(), Instant::now()));
            }
            Msg::ToolDone(Err(e)) | Msg::Redeemed(Err(e)) | Msg::Saved(Err(e)) => {
                self.toast = Some((e, Instant::now()))
            }
            Msg::Saved(Ok(())) => {}
            Msg::CodeStore(s) => self.code_store = s,
            Msg::CodeInput(c) => self.code = c,
            Msg::Redeem => {
                let url = launcher_services::redeem_url(self.code_store, &self.code);
                let copy = iced::clipboard::write(self.code.trim().to_string());
                return Task::batch([
                    copy,
                    Task::perform(
                        async move { launcher_services::open_uri(&url) },
                        Msg::Redeemed,
                    ),
                ]);
            }
            Msg::Redeemed(Ok(())) => {
                self.toast = Some((t::redeem_opened(self.code_store), Instant::now()));
            }
            Msg::FinishWelcome => {
                let home = self.home();
                self.go(home);
                if self.opts.demo.is_none() {
                    let paths = self.paths.clone();
                    return Task::perform(
                        async move {
                            launcher_services::mark_welcomed(&paths).map_err(|e| e.to_string())
                        },
                        Msg::Saved,
                    );
                }
            }
            Msg::SetLang(lang) => {
                launcher_i18n::set(lang);
                if self.opts.demo.is_some() {
                    return Task::none();
                }
                let paths = self.paths.clone();
                return Task::perform(
                    async move {
                        launcher_services::save_language(&paths, lang.key())
                            .map_err(|e| e.to_string())
                    },
                    Msg::Saved,
                );
            }
            Msg::SetSpeed(ms) => {
                self.transition_ms = ms;
                return self.save_appearance();
            }
            Msg::PreviewTransition => {
                if self.transition_kind != Kind::Aucune {
                    self.transition = Some(Transition {
                        start: Instant::now(),
                        kind: self.transition_kind,
                        target: Target::Page(Box::new(self.page.clone())),
                        swapped: false,
                    });
                }
            }
            Msg::SetGridView(on) => {
                self.grid_view = on;
                let home = self.home();
                self.go(home);
                return self.save_appearance();
            }
            Msg::SetSort(s) => self.sort = s,
            Msg::CollNever(on) => self.coll_never = on,
            Msg::Scrolled(v) => {
                self.scroll_y = v.absolute_offset().y;
                self.list_h = v.bounds().height;
            }
            Msg::Frame(now) => return self.on_frame(now),
            Msg::Key(keyboard::Event::KeyPressed {
                key: Key::Named(k), ..
            }) => match k {
                Named::F1 => self.go(Page::Settings(Tab::Appearance)),
                Named::F2 => self.show_fps = !self.show_fps,
                Named::F3 => {
                    self.set_mode(self.mode.next());
                    return self.save_appearance();
                }
                Named::F4 => {
                    self.transition_kind = self.transition_kind.next();
                    return self.save_appearance();
                }
                Named::F5 => {
                    self.accent = self.accent.next();
                    return self.save_appearance();
                }
                Named::Escape if self.page != self.home() => {
                    return self.update(Msg::BackToLibrary)
                }
                Named::Enter if self.page == Page::Library => return self.update(Msg::Play),
                Named::ArrowDown | Named::ArrowUp if self.page == Page::Library => {
                    let pos = self
                        .selected
                        .and_then(|s| self.visible.iter().position(|&v| v == s));
                    let next = match (k, pos) {
                        (Named::ArrowDown, Some(p)) => {
                            (p + 1).min(self.visible.len().saturating_sub(1))
                        }
                        (Named::ArrowUp, Some(p)) => p.saturating_sub(1),
                        _ => 0,
                    };
                    if let Some(&i) = self.visible.get(next) {
                        self.selected = Some(i);
                        // Garde la ligne choisie à l'écran.
                        let y = self.row_top(i).unwrap_or(0.0);
                        if y < self.scroll_y || y + ROW_H > self.scroll_y + self.list_h {
                            let target = (y - self.list_h / 2.0).max(0.0);
                            return iced::widget::operation::scroll_to(
                                LIST_ID,
                                AbsoluteOffset { x: 0.0, y: target },
                            );
                        }
                    }
                }
                _ => {}
            },
            Msg::Key(_) => {}
            Msg::SetAccent(a) => {
                self.accent = a;
                return self.save_appearance();
            }
            Msg::SetMode(m) => {
                self.set_mode(m);
                return self.save_appearance();
            }
            Msg::SetTransition(k) => {
                self.transition_kind = k;
                return self.save_appearance();
            }
            Msg::ToggleTransitions => {
                self.transition_kind = if self.transition_kind == Kind::Aucune {
                    Kind::Fondu
                } else {
                    Kind::Aucune
                };
                return self.save_appearance();
            }
        }
        Task::none()
    }

    fn on_frame(&mut self, now: Instant) -> Task<Msg> {
        if !self.first_frame_seen {
            self.first_frame_seen = true;
            if let Some(b) = &mut self.bench {
                b.report.startup = Some(self.opts.started.elapsed());
            }
        }
        if let Some(last) = self.fps_last {
            let dt = now.saturating_duration_since(last).as_secs_f32();
            if dt > 0.0 {
                self.fps = self.fps * 0.9 + (1.0 / dt) * 0.1;
            }
        }
        self.fps_last = Some(now);

        let duration = Duration::from_millis(self.transition_ms).as_secs_f32();
        if let Some(tr) = &mut self.transition {
            let p = now.saturating_duration_since(tr.start).as_secs_f32() / duration;
            // Le shader d'écran dessine l'effet ; il s'arrête à la fin.
            iced_wgpu::pixel::set_effect(
                if p >= 1.0 { 0 } else { tr.kind.shader_id() },
                p,
                0x0C0E14,
            );
            if p >= 0.5 && !tr.swapped {
                tr.swapped = true;
                match &tr.target {
                    Target::Game(i) => self.selected = Some(*i),
                    Target::Page(page) => self.page = page.as_ref().clone(),
                }
            }
            if p >= 1.0 {
                self.transition = None;
            }
        }
        // Lancement : l'écran reste le temps que la boutique démarre le jeu, puis se ferme.
        if let Page::Launching {
            at,
            state: LaunchState::Sent,
            ..
        } = &self.page
        {
            if at.elapsed() > LAUNCH_SCREEN && self.transition.is_none() {
                let back = self.back.clone();
                self.go(back);
            }
        }
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed() > Duration::from_secs(4))
        {
            self.toast = None;
        }
        self.bench_step(now)
    }

    fn bench_step(&mut self, now: Instant) -> Task<Msg> {
        if self.scanning {
            return Task::none();
        }
        let total = self.list_height();
        let list_h = self.list_h;
        let Some(b) = &mut self.bench else {
            return Task::none();
        };
        if let Some(last) = b.last {
            let dt = now.saturating_duration_since(last);
            match b.phase {
                BenchPhase::Scroll => b.report.frames.push(dt),
                BenchPhase::Transition => b.report.transition_frames.push(dt),
                BenchPhase::Done => {}
            }
        }
        b.last = Some(now);
        match b.phase {
            BenchPhase::Scroll => {
                b.offset += 24.0;
                if b.offset > total - list_h || b.report.frames.len() > 1200 {
                    b.phase = BenchPhase::Transition;
                } else {
                    return iced::widget::operation::scroll_to(
                        LIST_ID,
                        AbsoluteOffset {
                            x: 0.0,
                            y: b.offset,
                        },
                    );
                }
            }
            BenchPhase::Transition => {
                if self.transition.is_none() {
                    if b.report.transition_frames.len() > 90 {
                        b.phase = BenchPhase::Done;
                    } else {
                        let next = self
                            .visible
                            .get(b.report.transition_frames.len() % 7 + 1)
                            .copied();
                        if let Some(i) = next {
                            self.transition = Some(Transition {
                                start: now,
                                kind: Kind::Dissolution,
                                target: Target::Game(i),
                                swapped: false,
                            });
                        }
                    }
                }
            }
            BenchPhase::Done => {
                let text = b.report.render();
                println!("{text}");
                let path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("mesures-jalon0.txt")))
                    .unwrap_or_else(|| "mesures-jalon0.txt".into());
                if let Err(e) = std::fs::write(&path, text) {
                    eprintln!("Impossible d'écrire {} : {e}", path.display());
                }
                return iced::exit();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        let animating = self.transition.is_some()
            || matches!(self.page, Page::Launching { .. })
            || self.bench.is_some()
            || self.show_fps
            || self.toast.is_some()
            || !self.first_frame_seen;
        let keys = keyboard::listen().map(Msg::Key);
        if animating {
            Subscription::batch([keys, window::frames().map(Msg::Frame)])
        } else {
            keys
        }
    }
}

/// Démarrage : la base d'abord (affichage immédiat), puis le scan des boutiques.
fn load_task(demo: Option<usize>, paths: Paths) -> Task<Msg> {
    if demo.is_some() {
        return refresh_task(demo, paths);
    }
    let cached = paths.clone();
    Task::perform(
        async move { launcher_services::cached_library(&cached).map_err(|e| e.to_string()) },
        Msg::Cached,
    )
    .chain(refresh_task(None, paths))
}

fn refresh_task(demo: Option<usize>, paths: Paths) -> Task<Msg> {
    Task::perform(
        async move {
            let t0 = Instant::now();
            let res = match demo {
                Some(n) => Ok(Refresh {
                    games: fake_games(n),
                    ..Default::default()
                }),
                None => launcher_services::refresh_library(&paths).map_err(|e| e.to_string()),
            };
            (res, t0.elapsed())
        },
        |(res, took)| Msg::Refreshed(res, took),
    )
}

/// Boîte de dialogue « choisir un dossier » du système.
#[cfg(windows)]
fn pick_folder(store: StoreId) -> Task<Msg> {
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .set_title(t::pick_title(store))
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |dir| Msg::ClientDirPicked(store, dir),
    )
}

/// Hors Windows, pas encore de boîte de dialogue : on l'indique sur l'écran.
#[cfg(not(windows))]
fn pick_folder(_store: StoreId) -> Task<Msg> {
    Task::done(Msg::ClientDirSaved(Err(t::PICK_UNAVAILABLE().to_string())))
}

#[cfg(debug_assertions)]
fn debug_page(name: &str) -> Option<Page> {
    Some(match name {
        "bienvenue" => Page::Welcome,
        "liste" => Page::Library,
        "grille" => Page::Grid,
        "jeu" => Page::Game(0),
        "lancement" => Page::Launching {
            game: 0,
            at: Instant::now(),
            state: LaunchState::Sent,
        },
        "comptes" => Page::Settings(Tab::Accounts),
        "dossiers" => Page::Settings(Tab::Folders),
        "apparence" => Page::Settings(Tab::Appearance),
        "stats" => Page::Stats,
        "profil" => Page::Profile,
        "doublons" => Page::Duplicates,
        "collections" => Page::Collections,
        "recherche" => Page::Search,
        "code" => Page::Code,
        "telechargements" => Page::Downloads,
        "boutique" => Page::Store,
        "souhaits" => Page::Wishlist,
        "amis" => Page::Friends,
        "messages" => Page::Messages,
        _ => return None,
    })
}
