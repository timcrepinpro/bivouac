//! Page Bibliothèque, fidèle à la maquette `Mix2` (et `Mix2-Pixel`, `Mix2-PixelDoux`).
//! Sections de la maquette sans données réelles pour l'instant (actualités du jeu,
//! offres des boutiques, amis en jeu, succès) : non affichées tant qu'elles sont vides.

use std::time::{SystemTime, UNIX_EPOCH};

use iced::font::Weight;
use iced::widget::{
    button, canvas, column, container, mouse_area, row, scrollable, space, stack, text_input, Id,
};
use iced::{Border, Color, Element, Fill, Length, Padding};
use launcher_core::StoreId;

use super::{App, ClientStatus, Entry, Msg, Page, HEAD_FIRST_H, LIST_ID, ROW_H, SEARCH_ID};
use crate::kit::{self, bg, linear, Icon, Ui, ON_HERO, RULE};
use crate::logo::Logo;
use crate::texts as t;
use crate::theme::{self, Mode};

impl App {
    pub(super) fn ui(&self) -> Ui {
        Ui {
            accent: self.accent.color(),
            mode: self.mode,
            hero: false,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Msg> {
        let ui = self.ui();
        let page: Element<'_, Msg> = match &self.page {
            Page::Library => row![
                self.sidebar(ui),
                container(space()).width(1).height(Fill).style(|_| bg(RULE)),
                column![self.hero(ui), self.recent(ui)].width(Fill)
            ]
            .into(),
            Page::ClientMissing { store, game, note } => {
                self.client_missing(ui, *store, *game, note.as_deref())
            }
            Page::Welcome => self.welcome(ui),
            Page::Grid => self.shell(ui, 0, self.grid(ui), false),
            Page::Game(i) => self.shell(ui, 0, self.game_page(ui, *i), true),
            Page::Launching { game, at, state } => {
                self.shell(ui, 0, self.launching(ui, *game, *at, state), false)
            }
            Page::Settings(tab) => self.shell(ui, 4, self.settings(ui, *tab), true),
            Page::Stats => self.shell(ui, 0, self.stats(ui), true),
            Page::Profile => self.shell(ui, 9, self.profile(ui), true),
            Page::Duplicates => self.shell(ui, 0, self.duplicates(ui), true),
            Page::Collections => self.shell(ui, 0, self.collections(ui), false),
            Page::Search => self.shell(ui, 9, self.search_page(ui), true),
            Page::Code => self.shell(ui, 1, self.code_page(ui), true),
            Page::Downloads => self.shell(ui, 2, self.downloads(ui), true),
            Page::Store => self.shell(ui, 1, self.store_page(ui), true),
            Page::Wishlist => self.shell(ui, 1, self.wishlist(ui), true),
            Page::Friends => self.shell(ui, 3, self.friends(ui, false), false),
            Page::Messages => self.shell(ui, 3, self.friends(ui, true), false),
        };
        let mut layers = stack![container(page)
            .width(Fill)
            .height(Fill)
            .style(|_| bg(theme::BG))];
        if let Some((msg, _)) = &self.toast {
            let toast = container(ui.text(msg.as_str(), 14, Weight::Normal, theme::TEXT))
                .padding([12, 18])
                .style(move |_| ui.panel(theme::CARD2, 12.0, Some(theme::LINE2)));
            layers = layers.push(
                container(toast)
                    .width(Fill)
                    .height(Fill)
                    .align_x(iced::Center)
                    .align_bottom(Fill)
                    .padding(28),
            );
        }
        if self.show_fps {
            let fps = container(ui.mono(format!("{:.0} images/s", self.fps), 13, ui.accent))
                .padding([6, 10])
                .style(move |_| ui.panel(theme::CARD, 8.0, None));
            layers = layers.push(
                container(fps)
                    .width(Fill)
                    .align_right(Fill)
                    .padding(Padding::new(0.0).top(96.0).right(40.0)),
            );
        }
        layers.into()
    }

    // ----- Barre latérale -------------------------------------------------------

    fn sidebar(&self, ui: Ui) -> Element<'_, Msg> {
        let search = text_input(t::SEARCH(), &self.query)
            .id(Id::new(SEARCH_ID))
            .on_input(Msg::Search)
            .padding([12, 14])
            .size(13)
            .font(ui.mode.font(Weight::Normal))
            .style(move |_, _| text_input::Style {
                background: theme::FIELD.into(),
                border: Border {
                    radius: ui.r(12.0).into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                icon: theme::MUTED,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: Color {
                    a: 0.4,
                    ..ui.accent
                },
            });
        let mut filters = row![].spacing(6);
        for (label, val) in [
            (t::ALL(), None),
            ("Steam", Some(StoreId::Steam)),
            ("Epic", Some(StoreId::Epic)),
            ("GOG", Some(StoreId::Gog)),
        ] {
            filters = filters.push(ui.pill(label, self.store == val, Msg::Filter(val)));
        }
        let dupes = launcher_core::duplicates(&self.games).len();
        // Pastilles pointillées des maquettes : retour à la ligne si la place manque.
        let filters = filters
            .push(ui.dashed_pill(t::COLLECTIONS().to_string(), Msg::Go(Page::Collections)))
            .push(ui.dashed_pill(
                format!("{} · {dupes}", t::DUPLICATES()),
                Msg::Go(Page::Duplicates),
            ))
            .push(ui.dashed_pill(t::STATS().to_string(), Msg::Go(Page::Stats)))
            .wrap()
            .vertical_spacing(6);

        // Bandeau « Steam est introuvable » (PROMPT §7 : un écran ou un bandeau par état).
        let mut top = column![search, filters].spacing(12);
        if !self.scanning
            && self.opts.demo.is_none()
            && matches!(self.client(StoreId::Steam), Some(ClientStatus::NotFound))
        {
            top = top.push(
                button(
                    row![
                        ui.text(
                            t::client_missing_title(StoreId::Steam),
                            12,
                            Weight::Semibold,
                            theme::TEXT
                        )
                        .width(Fill),
                        ui.text(t::FIX(), 12, Weight::Bold, ui.accent),
                    ]
                    .align_y(iced::Center),
                )
                .on_press(Msg::ShowClientMissing(StoreId::Steam))
                .width(Fill)
                .padding([10, 12])
                .style(move |_, _| button::Style {
                    background: Some(theme::CARD2.into()),
                    text_color: theme::TEXT,
                    border: Border {
                        radius: ui.r(10.0).into(),
                        width: 1.0,
                        color: theme::LINE2,
                    },
                    ..Default::default()
                }),
            );
        }

        // Liste virtualisée : seules les lignes visibles sont construites.
        let n = self.entries.len();
        let total = self.list_height();
        let first = self
            .tops
            .partition_point(|&y| y + ROW_H <= self.scroll_y)
            .min(n);
        let last = self
            .tops
            .partition_point(|&y| y < self.scroll_y + self.list_h + ROW_H)
            .min(n)
            .max(first);
        let before = self.tops.get(first).copied().unwrap_or(0.0);
        let after = total - self.tops.get(last).copied().unwrap_or(total);
        let mut rows = column![space().height(before)];
        for (k, e) in self.entries[first..last].iter().enumerate() {
            rows = rows.push(match e {
                Entry::Header(label) => {
                    let head = self.list_header(ui, label, first + k == 0);
                    if label.ends_with('→') {
                        mouse_area(head).on_press(Msg::SetGridView(true)).into()
                    } else {
                        head
                    }
                }
                Entry::Game(i) => self.list_row(ui, *i),
            });
        }
        rows = rows.push(space().height(after.max(0.0)));
        let list: Element<'_, Msg> = if self.visible.is_empty() && !self.scanning {
            container(ui.text(
                if self.games.is_empty() {
                    t::NO_GAME()
                } else {
                    t::NO_MATCH()
                },
                13,
                Weight::Normal,
                theme::MUTED,
            ))
            .padding([8, 16])
            .height(Fill)
            .into()
        } else {
            scrollable(container(rows).padding([0, 10]))
                .id(Id::new(LIST_ID))
                .on_scroll(Msg::Scrolled)
                .height(Fill)
                .into()
        };

        let rescan = button(ui.text(t::RESCAN(), 13, Weight::Normal, theme::SOFT))
            .on_press(Msg::Rescan)
            .padding(0)
            .style(|_, _| button::Style::default());
        let code = button(ui.text(t::USE_CODE_LINK(), 13, Weight::Normal, theme::SOFT))
            .on_press(Msg::Go(Page::Code))
            .padding(0)
            .style(|_, _| button::Style::default());
        let footer = container(column![rescan, code].spacing(10))
            .width(Fill)
            .padding([14, 16])
            .style(|_| container::Style {
                border: Border {
                    width: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            });

        container(
            column![
                top.padding(Padding::new(16.0).top(20.0).bottom(14.0)),
                list,
                container(space()).width(Fill).height(1).style(|_| bg(RULE)),
                footer,
            ]
            .height(Fill),
        )
        .width(288)
        .height(Fill)
        .style(|_| bg(theme::SIDE))
        .into()
    }

    fn list_header<'a>(&self, ui: Ui, label: &'a str, first: bool) -> Element<'a, Msg> {
        let pad = if first {
            Padding::new(0.0).top(4.0).bottom(6.0).left(6.0)
        } else {
            Padding::new(0.0).top(16.0).bottom(6.0).left(6.0)
        };
        container(ui.section(label.to_string()))
            .padding(pad)
            .height(if first { HEAD_FIRST_H } else { super::HEAD_H })
            .align_y(iced::Bottom)
            .into()
    }

    fn list_row(&self, ui: Ui, i: usize) -> Element<'_, Msg> {
        let g = &self.games[i];
        let active = self.selected == Some(i);
        let content = row![
            ui.cover(g, 34.0, 34.0, 8.0, false),
            ui.text(
                g.title.as_str(),
                13,
                if active {
                    Weight::Semibold
                } else {
                    Weight::Normal
                },
                theme::TEXT
            )
            .width(Fill),
            ui.mono(g.store.label(), 10, theme::MUTED),
        ]
        .spacing(12)
        .align_y(iced::Center);
        let pixel = ui.mode.is_pixel();
        container(
            button(content)
                .on_press(Msg::Select(i))
                .width(Fill)
                .height(ROW_H - 2.0)
                .padding([8, 10])
                .style(move |_, status| button::Style {
                    background: match (active, status) {
                        (true, _) => Some(theme::CARD2.into()),
                        (false, button::Status::Hovered) => {
                            Some(Color::from_rgb8(0x16, 0x1A, 0x26).into())
                        }
                        _ => None,
                    },
                    text_color: theme::TEXT,
                    border: Border {
                        radius: ui.r(10.0).into(),
                        width: if pixel && active { 2.0 } else { 0.0 },
                        color: theme::LINE2,
                    },
                    ..Default::default()
                }),
        )
        .height(ROW_H)
        .into()
    }

    // ----- En-tête et bannière --------------------------------------------------------

    /// En-tête commun : logo, navigation, état, recherche, profil.
    /// `active` : onglet allumé (0 à 3), 4 = Paramètres, autre = aucun.
    pub(super) fn header(&self, ui: Ui, active: usize) -> Element<'_, Msg> {
        let corner = if ui.mode == Mode::Normal {
            26.0 / 120.0
        } else {
            0.0
        };
        row![
            container(
                canvas(Logo {
                    accent: ui.accent,
                    corner
                })
                .width(42)
                .height(42)
            )
            .padding(Padding::new(0.0).right(4.0)),
            ui.nav(active, Msg::Nav, Msg::Nav(4), active == 4),
            space().width(Fill),
            ui.status_chip(if self.scanning {
                t::SCANNING().to_string()
            } else {
                t::games_count(self.games.len())
            }),
            ui.icon_button(Icon::Search, Msg::FocusSearch),
            mouse_area(ui.avatar(42.0, self.steam_avatar())).on_press(Msg::Go(Page::Profile)),
        ]
        .spacing(8)
        .align_y(iced::Center)
        .into()
    }

    fn hero(&self, ui: Ui) -> Element<'_, Msg> {
        let game = self.selected.and_then(|i| self.games.get(i));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (kicker, title) = match game {
            Some(g) => (
                match g.last_played {
                    Some(at) => format!("{} · {}", t::RESUME(), kit::ago(at, now)),
                    None => t::INSTALLED().to_string(),
                },
                g.title.clone(),
            ),
            None => (t::NO_GAME_HINT().to_string(), t::NO_GAME().to_string()),
        };
        let mut actions = row![ui.primary(t::PLAY().to_string(), true, game.map(|_| Msg::Play))]
            .spacing(14)
            .align_y(iced::Center);
        if let Some(g) = game {
            actions = actions
                .push(ui.tag(g.store.label()))
                .push(space().width(Fill))
                .push(ui.stat(t::PLAYTIME(), kit::playtime(g.playtime_minutes)))
                .push(ui.stat(t::SIZE(), kit::size(g.size_bytes)));
        }

        let hero_ui = Ui { hero: true, ..ui };
        let title_el: Element<'_, Msg> = match self.selected {
            Some(i) => mouse_area(
                ui.text(title, 60, Weight::ExtraBold, theme::TEXT)
                    .line_height(iced::widget::text::LineHeight::Relative(1.0)),
            )
            .on_press(Msg::OpenGame(i))
            .into(),
            None => ui
                .text(title, 60, Weight::ExtraBold, theme::TEXT)
                .line_height(iced::widget::text::LineHeight::Relative(1.0))
                .into(),
        };
        let body = column![
            self.header(hero_ui, 0),
            space().height(Fill),
            ui.text(kicker.to_uppercase(), 12, Weight::Semibold, ON_HERO),
            title_el,
            container(actions).padding(Padding::new(0.0).top(6.0)),
        ]
        .spacing(14)
        .padding([32, 40]);

        // Fond de la maquette : dégradé de la jaquette (3 arrêts), voile à gauche, fondu en bas.
        let (c1, c2) = game
            .map(|g| theme::cover_colors(&g.title))
            .unwrap_or((theme::CARD2, theme::BG));
        let deep = Color::from_rgb8(0x13, 0x14, 0x3A);
        let mut layers =
            stack![container(space())
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(linear(
                        135.0,
                        &[(0.0, kit::darken(c1, 0.2)), (0.55, c2), (1.0, deep)]
                    )),
                    ..Default::default()
                })];
        if let Some(path) = game.and_then(|g| g.covers.hero.as_ref()) {
            layers = layers.push(
                iced::widget::image(iced::widget::image::Handle::from_path(path))
                    .content_fit(iced::ContentFit::Cover)
                    .width(Fill)
                    .height(Fill),
            );
        }
        let clear = Color {
            a: 0.0,
            ..theme::BG
        };
        layers =
            layers
                .push(container(space()).width(Fill).height(Fill).style(move |_| {
                    container::Style {
                        background: Some(linear(
                            120.0,
                            &[
                                (
                                    0.0,
                                    Color {
                                        a: 0.85,
                                        ..theme::BG
                                    },
                                ),
                                (
                                    0.65,
                                    Color {
                                        a: 0.1,
                                        ..theme::BG
                                    },
                                ),
                            ],
                        )),
                        ..Default::default()
                    }
                }))
                .push(container(space()).width(Fill).height(Fill).style(move |_| {
                    container::Style {
                        background: Some(linear(0.0, &[(0.0, theme::BG), (0.5, clear)])),
                        ..Default::default()
                    }
                }));
        container(layers.push(body)).width(Fill).height(380).into()
    }

    fn recent(&self, ui: Ui) -> Element<'_, Msg> {
        let mut picks: Vec<usize> = (0..self.games.len())
            .filter(|&i| self.games[i].last_played.is_some())
            .take(5)
            .collect();
        if picks.is_empty() {
            picks = (0..self.games.len().min(5)).collect();
        }
        if picks.is_empty() {
            return space().into();
        }
        let mut cards = row![].spacing(16);
        for &i in &picks {
            let g = &self.games[i];
            let card = column![
                ui.cover(g, f32::INFINITY, 126.0, 12.0, true),
                row![
                    ui.text(g.store.name(), 12, Weight::Normal, theme::MUTED),
                    space().width(Fill),
                    ui.text(
                        kit::short_playtime(g.playtime_minutes),
                        12,
                        Weight::Normal,
                        theme::MUTED
                    )
                ],
            ]
            .spacing(6)
            .width(Length::FillPortion(1));
            cards = cards.push(mouse_area(card).on_press(Msg::OpenGame(i)));
        }
        // Garde la grille de 5 colonnes de la maquette même avec moins de jeux.
        for _ in picks.len()..5 {
            cards = cards.push(space().width(Length::FillPortion(1)));
        }
        column![ui.text(t::RECENT(), 17, Weight::Bold, theme::TEXT), cards]
            .spacing(12)
            .padding(Padding::new(40.0).top(22.0))
            .into()
    }

    /// Cadre des pages secondaires (maquettes `Grille`, `Jeu-Possede`…) : en-tête de 72 px,
    /// contenu avec 40 px de marge. `scroll` : la page défile si elle dépasse.
    pub(super) fn shell<'a>(
        &'a self,
        ui: Ui,
        active: usize,
        content: Element<'a, Msg>,
        scroll: bool,
    ) -> Element<'a, Msg> {
        let head = container(self.header(ui, active))
            .height(72)
            .padding([0, 40])
            .align_y(iced::Center);
        let body: Element<'a, Msg> = if scroll {
            scrollable(container(content).padding(Padding::new(40.0).top(12.0).bottom(28.0)))
                .height(Fill)
                .into()
        } else {
            container(content)
                .padding(Padding::new(40.0).top(12.0).bottom(28.0))
                .height(Fill)
                .into()
        };
        column![head, body].into()
    }

    /// Avatar du compte Steam en cache sur ce PC, s'il existe.
    pub(super) fn steam_avatar(&self) -> Option<&str> {
        self.accounts
            .iter()
            .find(|a| a.store == StoreId::Steam)
            .and_then(|a| a.avatar.as_deref())
    }

    pub(super) fn game_count(&self, store: StoreId) -> usize {
        self.games.iter().filter(|g| g.store == store).count()
    }
}
