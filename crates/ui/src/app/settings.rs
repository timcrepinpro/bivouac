//! Maquettes `Parametres` (comptes liés), `Parametres-Telechargements` et
//! `Parametres-Apparence`. Tout ce qui est affiché est lu sur ce PC.

use iced::font::Weight;
use iced::widget::{button, canvas, column, container, row, space};
use iced::{Border, Color, Element, Fill, Length};
use launcher_core::{install_folders, StoreId};

use super::{App, ClientStatus, Msg, Page, Tab};
use crate::kit::{self, Ui};
use crate::logo::Logo;
use crate::texts as t;
use crate::theme::{self, Accent, Mode};
use crate::transition::Kind;

/// Boutiques des maquettes qui ne sont pas encore branchées (Jalon 3).
const LATER: [(&str, &str); 3] = [
    ("EA", "EA app"),
    ("UB", "Ubisoft Connect"),
    ("BN", "Battle.net"),
];

pub(super) fn badge_label(s: StoreId) -> &'static str {
    match s {
        StoreId::Steam => "ST",
        StoreId::Epic => "EP",
        StoreId::Gog => "GOG",
    }
}

impl App {
    pub(super) fn settings(&self, ui: Ui, tab: Tab) -> Element<'_, Msg> {
        let side = column![
            ui.side_item(
                t::SETTINGS_ACCOUNTS().into(),
                None,
                tab == Tab::Accounts,
                Some(Msg::Go(Page::Settings(Tab::Accounts)))
            ),
            ui.side_item(
                t::SETTINGS_FOLDERS().into(),
                None,
                tab == Tab::Folders,
                Some(Msg::Go(Page::Settings(Tab::Folders)))
            ),
            ui.side_item(
                t::SETTINGS_LOOK().into(),
                None,
                tab == Tab::Appearance,
                Some(Msg::Go(Page::Settings(Tab::Appearance)))
            ),
        ]
        .spacing(4);
        let side = container(side)
            .width(260)
            .padding(12)
            .style(move |_| ui.panel(theme::CARD, 20.0, None));
        let main = match tab {
            Tab::Accounts => self.accounts_tab(ui),
            Tab::Folders => self.folders_tab(ui),
            Tab::Appearance => self.appearance_tab(ui),
        };
        row![column![side], container(main).width(Fill)]
            .spacing(24)
            .into()
    }

    fn accounts_tab(&self, ui: Ui) -> Element<'_, Msg> {
        let mut cards: Vec<Element<'_, Msg>> = Vec::new();
        for s in StoreId::ALL {
            let found = matches!(self.client(s), Some(ClientStatus::Installed { .. }));
            let status = if self.scanning && self.clients.is_empty() {
                ui.text(t::SEARCHING(), 13, Weight::Normal, theme::MUTED)
            } else if found {
                ui.text(
                    format!("{} · {}", t::DETECTED(), t::games_n(self.game_count(s))),
                    13,
                    Weight::Normal,
                    ui.accent,
                )
            } else {
                ui.text(t::NOT_DETECTED(), 13, Weight::Normal, theme::MUTED)
            };
            let action: Element<'_, Msg> = match (found, s) {
                (true, _) => ui.outline(
                    t::RESCAN().trim_start_matches("↻ "),
                    Some(Msg::Rescan),
                    false,
                ),
                (false, StoreId::Steam) => {
                    ui.small_primary(t::CHOOSE_FOLDER(), Some(Msg::PickClientDir(s)))
                }
                (false, _) => ui.small_primary(t::DOWNLOAD_APP(), Some(Msg::InstallClient(s))),
            };
            let content = row![
                ui.badge(badge_label(s), 46.0),
                column![ui.text(s.name(), 16, Weight::Bold, theme::TEXT), status]
                    .spacing(2)
                    .width(Fill),
                action,
            ]
            .spacing(14)
            .align_y(iced::Center);
            cards.push(if found {
                ui.card(content, 16)
            } else {
                ui.dashed_card(content, 16)
            });
        }
        for (b, name) in LATER {
            let content = row![
                ui.badge(b, 46.0),
                column![
                    ui.text(name, 16, Weight::Bold, theme::TEXT),
                    ui.text(t::LATER_STORE(), 13, Weight::Normal, theme::MUTED)
                ]
                .spacing(2)
                .width(Fill),
            ]
            .spacing(14)
            .align_y(iced::Center);
            cards.push(ui.dashed_card(content, 16));
        }
        let mut grid = column![].spacing(12);
        let mut it = cards.into_iter();
        while let Some(a) = it.next() {
            let b: Element<'_, Msg> = it.next().unwrap_or_else(|| space().width(Fill).into());
            grid = grid.push(
                row![
                    container(a).width(Length::FillPortion(1)),
                    container(b).width(Length::FillPortion(1))
                ]
                .spacing(12),
            );
        }
        column![
            ui.h1(t::SETTINGS_ACCOUNTS()),
            ui.lead(t::ACCOUNTS_LEAD()),
            space().height(4),
            grid
        ]
        .spacing(8)
        .into()
    }

    fn folders_tab(&self, ui: Ui) -> Element<'_, Msg> {
        let mut list = column![].spacing(12);
        let folders = install_folders(&self.games);
        if folders.is_empty() {
            list = list.push(ui.empty(t::NO_FOLDER(), t::NO_GAME_HINT()));
        }
        for (dir, n, bytes) in folders {
            list = list.push(
                ui.card(
                    column![
                        ui.mono(dir, 14, theme::TEXT),
                        ui.text(
                            format!("{} · {}", t::games_n(n), kit::size(Some(bytes))),
                            12,
                            Weight::Normal,
                            theme::MUTED
                        ),
                    ]
                    .spacing(4),
                    18,
                ),
            );
        }
        column![
            ui.h1(t::SETTINGS_FOLDERS()),
            space().height(6),
            ui.caps(t::INSTALL_FOLDERS()),
            list,
            space().height(10),
            ui.caps(t::DOWNLOADS()),
            ui.card(
                ui.text(
                    t::DOWNLOADS_SETTINGS_LATER(),
                    13,
                    Weight::Normal,
                    theme::MUTED
                ),
                18
            ),
        ]
        .spacing(10)
        .into()
    }

    fn appearance_tab(&self, ui: Ui) -> Element<'_, Msg> {
        // Thème : seul le sombre existe pour l'instant (thème clair : « Plus tard », §12).
        let theme_card = |label: &'static str, bg_left: Color, bg_right: Color, on: bool| {
            let accent = ui.accent;
            let preview = row![
                container(ui.text("Aa", 14, Weight::Bold, theme::TEXT))
                    .center_x(Fill)
                    .height(90)
                    .align_y(iced::Center)
                    .style(move |_| kit::bg(bg_left)),
                container(ui.text("Aa", 14, Weight::Bold, theme::BG))
                    .center_x(Fill)
                    .height(90)
                    .align_y(iced::Center)
                    .style(move |_| kit::bg(bg_right)),
            ];
            let dot_color = if on { accent } else { theme::MUTED };
            container(
                column![
                    preview,
                    row![
                        ui.dot(10.0, dot_color),
                        ui.text(
                            label,
                            15,
                            Weight::Semibold,
                            if on { theme::TEXT } else { theme::MUTED }
                        )
                    ]
                    .spacing(10)
                    .align_y(iced::Center)
                ]
                .spacing(14),
            )
            .padding(14)
            .width(Fill)
            .style(move |_| container::Style {
                background: Some(theme::CARD.into()),
                border: Border {
                    radius: ui.r(16.0).into(),
                    width: if on { 2.0 } else { 1.0 },
                    color: if on { accent } else { theme::LINE2 },
                },
                ..Default::default()
            })
        };
        let themes = row![
            theme_card(t::DARK(), theme::BG, theme::BG, true),
            theme_card(t::LIGHT(), theme::TEXT, theme::TEXT, false),
            theme_card(t::AUTO(), theme::BG, theme::TEXT, false),
        ]
        .spacing(14);

        let mut dots = row![].spacing(12).align_y(iced::Center);
        for a in Accent::ALL {
            let selected = a == self.accent;
            let round = match ui.mode {
                Mode::Normal => 18.0,
                Mode::PixelCarre => 0.0,
                Mode::PixelArrondi => theme::STEP_MARK + 8.0,
            };
            dots = dots.push(
                button(space().width(36).height(36))
                    .on_press(Msg::SetAccent(a))
                    .padding(0)
                    .style(move |_, _| button::Style {
                        background: Some(a.color().into()),
                        border: Border {
                            radius: round.into(),
                            width: if selected { 3.0 } else { 0.0 },
                            color: theme::TEXT,
                        },
                        ..Default::default()
                    }),
            );
        }
        let corner = if ui.mode == Mode::Normal {
            26.0 / 120.0
        } else {
            0.0
        };
        let accent_card = ui.card(
            row![
                canvas(Logo {
                    accent: ui.accent,
                    corner
                })
                .width(56)
                .height(56),
                column![
                    dots,
                    ui.text(t::ACCENT_HELP(), 13, Weight::Normal, theme::MUTED)
                ]
                .spacing(10),
            ]
            .spacing(20)
            .align_y(iced::Center),
            18,
        );

        let lang = launcher_i18n::current();
        let display = ui.card(
            column![
                ui.setting(
                    t::LANGUAGE(),
                    Some(t::LANGUAGE_HELP()),
                    ui.segmented(
                        launcher_i18n::Lang::ALL
                            .into_iter()
                            .map(|l| (l.name(), Msg::SetLang(l)))
                            .collect(),
                        launcher_i18n::Lang::ALL
                            .iter()
                            .position(|&l| l == lang)
                            .unwrap_or(0),
                        true,
                    ),
                ),
                ui.rule(),
                ui.setting(
                    t::HOME_PAGE(),
                    None,
                    ui.segmented(
                        vec![
                            (t::LIST(), Msg::SetGridView(false)),
                            (t::GRID(), Msg::SetGridView(true)),
                        ],
                        usize::from(self.grid_view),
                        true,
                    ),
                ),
            ],
            0,
        );

        let modes = [Mode::Normal, Mode::PixelCarre, Mode::PixelArrondi];
        let style = ui.segmented(
            modes.iter().map(|&m| (m.name(), Msg::SetMode(m))).collect(),
            modes.iter().position(|&m| m == self.mode).unwrap_or(0),
            true,
        );
        let transitions_on = self.transition_kind != Kind::Aucune;
        let pixel_card = ui.card(
            column![
                ui.setting(t::INTERFACE_STYLE(), Some(t::INTERFACE_STYLE_HELP()), style),
                ui.rule(),
                ui.setting(
                    t::PAGE_TRANSITIONS(),
                    Some(t::PAGE_TRANSITIONS_HELP()),
                    ui.toggle(transitions_on, Msg::ToggleTransitions)
                ),
            ],
            0,
        );

        // Aperçu : le jeu sélectionné, en vrai.
        let preview: Element<'_, Msg> = match self
            .selected
            .and_then(|i| self.games.get(i).map(|g| (i, g)))
        {
            Some((i, g)) => container(
                row![
                    ui.cover(g, 96.0, 120.0, 10.0, false),
                    column![
                        ui.text(g.title.as_str(), 20, Weight::Bold, theme::TEXT),
                        ui.text(
                            format!(
                                "{} · {}",
                                g.store.name(),
                                kit::short_playtime(g.playtime_minutes)
                            ),
                            12,
                            Weight::Normal,
                            theme::MUTED
                        ),
                        space().height(Fill),
                        ui.small_primary(t::PLAY(), Some(Msg::PlayGame(i))),
                    ]
                    .spacing(6)
                    .height(120),
                ]
                .spacing(14),
            )
            .padding(14)
            .width(Fill)
            .style(move |_| ui.panel(theme::CARD2, 14.0, None))
            .into(),
            None => ui
                .text(t::NO_GAME(), 13, Weight::Normal, theme::MUTED)
                .into(),
        };
        let pixel_family = self.transition_kind.is_pixel();
        let kinds = if pixel_family {
            Kind::PIXEL
        } else {
            Kind::FLUID
        };
        let mut kind_pills = row![].spacing(8);
        for k in kinds {
            kind_pills = kind_pills.push(ui.pill(
                k.name(),
                k == self.transition_kind,
                Msg::SetTransition(k),
            ));
        }
        let stage = container(ui.text(t::LIBRARY(), 16, Weight::Bold, theme::TEXT))
            .center_x(Fill)
            .height(110)
            .align_y(iced::Center)
            .style(move |_| ui.panel(theme::CARD2, 14.0, None));
        let speed = ui.segmented(
            vec![
                (t::SPEED_FAST(), Msg::SetSpeed(150)),
                (t::SPEED_NORMAL(), Msg::SetSpeed(300)),
                (t::SPEED_SLOW(), Msg::SetSpeed(500)),
            ],
            match self.transition_ms {
                0..=200 => 0,
                201..=400 => 1,
                _ => 2,
            },
            transitions_on,
        );
        let mut transition_col = column![
            ui.caps(t::TRANSITION_TYPE()),
            ui.segmented(
                vec![
                    (t::FLUID(), Msg::SetTransition(Kind::Fondu)),
                    (t::PIXEL(), Msg::SetTransition(Kind::Dissolution)),
                ],
                usize::from(pixel_family),
                transitions_on,
            ),
            kind_pills,
            stage,
            row![
                ui.light(
                    t::TRY_IT(),
                    (transitions_on && !self.reduce_motion).then_some(Msg::PreviewTransition)
                ),
                space().width(Fill),
                ui.text(t::SPEED(), 13, Weight::Normal, theme::MUTED),
                speed,
            ]
            .spacing(10)
            .align_y(iced::Center),
        ]
        .spacing(12);
        if self.reduce_motion {
            transition_col =
                transition_col.push(ui.text(t::REDUCED_MOTION(), 12, Weight::Normal, theme::MUTED));
        }
        let transition_card = ui.card(transition_col, 18);
        let previews = row![
            container(ui.card(column![ui.caps(t::PREVIEW()), preview].spacing(12), 18))
                .width(Length::FillPortion(1)),
            container(transition_card).width(Length::FillPortion(1)),
        ]
        .spacing(16);

        column![
            ui.h1(t::SETTINGS_LOOK()),
            space().height(4),
            ui.caps(t::THEME()),
            themes,
            ui.text(t::LIGHT_LATER(), 12, Weight::Normal, theme::MUTED),
            space().height(6),
            ui.caps(t::ACCENT_COLOR()),
            accent_card,
            space().height(6),
            ui.caps(t::DISPLAY()),
            display,
            space().height(6),
            ui.caps(t::PIXEL_AND_TRANSITIONS()),
            pixel_card,
            previews,
            ui.mono(t::SHORTCUTS(), 11, theme::MUTED),
        ]
        .spacing(10)
        .into()
    }
}
