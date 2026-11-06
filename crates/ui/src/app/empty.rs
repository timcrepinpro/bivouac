//! Maquettes dont les données n'existent pas encore (`Boutique`, `Souhaits`,
//! `Telechargements`, `Amis`, `Messages`) : même mise en page, blocs vides honnêtes,
//! jamais de faux prix ni de faux amis. Et `Bienvenue`, au premier démarrage.

use iced::font::Weight;
use iced::widget::{canvas, column, container, row, space, stack};
use iced::{Border, Color, Element, Fill, Length};
use launcher_core::StoreId;

use super::settings::badge_label;
use super::{App, ClientStatus, Msg, Page};
use crate::kit::{self, linear, Ui};
use crate::logo::Logo;
use crate::texts as t;
use crate::theme::{self, Mode};

impl App {
    pub(super) fn downloads(&self, ui: Ui) -> Element<'_, Msg> {
        let total: u64 = self.games.iter().filter_map(|g| g.size_bytes).sum();
        let mut bar = row![].spacing(0);
        let mut lines = column![].spacing(8);
        let colors = [
            ui.accent,
            Color::from_rgb8(0xA5, 0xB4, 0xFC),
            Color::from_rgb8(0xFD, 0xBA, 0x74),
        ];
        for (n, s) in StoreId::ALL.into_iter().enumerate() {
            let bytes: u64 = self
                .games
                .iter()
                .filter(|g| g.store == s)
                .filter_map(|g| g.size_bytes)
                .sum();
            if bytes > 0 && total > 0 {
                let c = colors[n];
                bar = bar.push(
                    container(space())
                        .width(Length::FillPortion(((bytes * 1000 / total) as u16).max(1)))
                        .height(8)
                        .style(move |_| kit::bg(c)),
                );
            }
            lines = lines.push(
                row![
                    ui.text(s.name(), 13, Weight::Normal, theme::TEXT),
                    space().width(Fill),
                    ui.mono(kit::size(Some(bytes)), 13, theme::TEXT),
                ]
                .align_y(iced::Center),
            );
        }
        let disk = ui.card(
            column![
                ui.caps(t::DISK_SPACE()),
                row![
                    ui.text(t::USED_BY_GAMES(), 15, Weight::Bold, theme::TEXT),
                    space().width(Fill),
                    ui.mono(kit::size(Some(total)), 14, theme::SOFT),
                ],
                container(bar)
                    .width(Fill)
                    .height(8)
                    .style(move |_| kit::bg(theme::LINE2)),
                lines,
            ]
            .spacing(14),
            22,
        );
        column![
            ui.h1(t::DOWNLOADS()),
            row![
                container(ui.empty(t::NO_DOWNLOAD(), t::NO_DOWNLOAD_HELP()))
                    .width(Length::FillPortion(37)),
                container(disk).width(Length::FillPortion(18)),
            ]
            .spacing(24),
        ]
        .spacing(20)
        .into()
    }

    pub(super) fn store_page(&self, ui: Ui) -> Element<'_, Msg> {
        let r = ui.r(24.0);
        let accent = ui.accent;
        let kicker = container(ui.text("JALON 6", 12, Weight::ExtraBold, theme::BG))
            .padding([4, 9])
            .style(move |_| ui.panel(accent, 6.0, None));
        let hero = container(stack![
            container(space())
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(linear(
                        135.0,
                        &[
                            (0.0, Color::from_rgb8(0x0E, 0x3B, 0x3A)),
                            (0.55, Color::from_rgb8(0x13, 0x25, 0x4A)),
                            (1.0, Color::from_rgb8(0x2A, 0x1D, 0x5C)),
                        ]
                    )),
                    border: Border {
                        radius: r.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            column![
                space().height(Fill),
                kicker,
                ui.text(t::STORE_LATER_TITLE(), 46, Weight::ExtraBold, theme::TEXT),
                container(ui.text(t::STORE_LATER(), 15, Weight::Normal, kit::ON_HERO)).width(520),
                row![
                    ui.primary(
                        t::CODE_TITLE().to_string(),
                        false,
                        Some(Msg::Go(Page::Code))
                    ),
                    ui.secondary(t::WISHLIST().to_string(), Msg::Go(Page::Wishlist)),
                ]
                .spacing(12),
            ]
            .spacing(14)
            .padding(32),
        ])
        .width(Fill)
        .height(330);

        let mut pills = row![ui.h2(t::CATALOG())].spacing(8).align_y(iced::Center);
        for (n, label) in [t::ALL_STORES(), "Steam", "Epic", "GOG", "Instant Gaming"]
            .into_iter()
            .enumerate()
        {
            pills = pills.push(ui.pill(label, n == 0, Msg::Go(Page::Store)));
        }
        pills = pills.push(space().width(Fill)).push(ui.outline(
            t::CODE_TITLE(),
            Some(Msg::Go(Page::Code)),
            false,
        ));
        column![
            hero,
            pills,
            ui.empty(t::STORE_LATER_TITLE(), t::STORE_LATER()),
        ]
        .spacing(20)
        .into()
    }

    pub(super) fn wishlist(&self, ui: Ui) -> Element<'_, Msg> {
        column![
            row![
                ui.h1(t::WISHLIST()),
                ui.text(t::games_n(0), 14, Weight::Normal, theme::MUTED)
            ]
            .spacing(14)
            .align_y(iced::Center),
            ui.empty(t::WISHLIST_EMPTY(), t::WISHLIST_LATER()),
        ]
        .spacing(20)
        .into()
    }

    pub(super) fn friends(&self, ui: Ui, messages: bool) -> Element<'_, Msg> {
        let side = container(
            column![
                ui.field(t::FIND_FRIEND(), "", |_| Msg::Go(Page::Friends), 13, false)
                    .on_input_maybe(None::<fn(String) -> Msg>),
                ui.side_item(
                    t::ALL_MESSAGES().into(),
                    Some(0),
                    messages,
                    Some(Msg::Go(Page::Messages))
                ),
                container(ui.text("EN JEU · 0", 12, Weight::Bold, theme::MUTED)).padding([8, 4]),
                container(ui.text("EN LIGNE · 0", 12, Weight::Bold, theme::MUTED)).padding([8, 4]),
            ]
            .spacing(10),
        )
        .padding(20)
        .width(360)
        .height(Fill)
        .style(move |_| ui.panel(theme::CARD, 20.0, None));
        let title = if messages {
            t::MESSAGES_EMPTY()
        } else {
            t::FRIENDS_LATER_TITLE()
        };
        let main = container(
            column![
                ui.text(title, 20, Weight::Bold, theme::TEXT),
                ui.text(t::FRIENDS_LATER(), 14, Weight::Normal, theme::MUTED),
            ]
            .spacing(8)
            .align_x(iced::Center),
        )
        .center(Fill)
        .style(move |_| ui.panel(theme::CARD, 20.0, None));
        row![side, main].spacing(24).height(Fill).into()
    }

    pub(super) fn welcome(&self, ui: Ui) -> Element<'_, Msg> {
        let corner = if ui.mode == Mode::Normal {
            26.0 / 120.0
        } else {
            0.0
        };
        let top = row![
            canvas(Logo {
                accent: ui.accent,
                corner
            })
            .width(42)
            .height(42),
            ui.text("LAUNCHER", 17, Weight::Bold, theme::TEXT),
            space().width(Fill),
            ui.link(t::SKIP(), 13, theme::SOFT, Msg::FinishWelcome),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .padding([14, 40]);

        let mut cards: Vec<Element<'_, Msg>> = Vec::new();
        for s in StoreId::ALL {
            let found = matches!(self.client(s), Some(ClientStatus::Installed { .. }));
            let (status, color) = if self.scanning && self.clients.is_empty() {
                (t::SEARCHING().to_string(), theme::MUTED)
            } else if found {
                (
                    format!("{} · {}", t::DETECTED(), t::games_n(self.game_count(s))),
                    ui.accent,
                )
            } else {
                (t::NOT_DETECTED().to_string(), theme::MUTED)
            };
            let action: Element<'_, Msg> = match (found, s) {
                (true, _) => space().into(),
                (false, StoreId::Steam) => {
                    ui.small_primary(t::CHOOSE_FOLDER(), Some(Msg::PickClientDir(s)))
                }
                (false, _) => ui.small_primary(t::DOWNLOAD_APP(), Some(Msg::InstallClient(s))),
            };
            cards.push(
                ui.card(
                    row![
                        ui.badge(badge_label(s), 46.0),
                        column![
                            ui.text(s.name(), 16, Weight::Bold, theme::TEXT),
                            ui.text(status, 13, Weight::Normal, color),
                        ]
                        .spacing(2)
                        .width(Fill),
                        action,
                    ]
                    .spacing(14)
                    .align_y(iced::Center),
                    16,
                ),
            );
        }
        for (b, name) in [
            ("EA", "EA app"),
            ("UB", "Ubisoft Connect"),
            ("BN", "Battle.net"),
        ] {
            cards.push(
                ui.dashed_card(
                    row![
                        ui.badge(b, 46.0),
                        column![
                            ui.text(name, 16, Weight::Bold, theme::TEXT),
                            ui.text(t::LATER_STORE(), 13, Weight::Normal, theme::MUTED),
                        ]
                        .spacing(2),
                    ]
                    .spacing(14)
                    .align_y(iced::Center),
                    16,
                ),
            );
        }
        let mut grid = column![].spacing(12);
        let mut it = cards.into_iter();
        while let Some(a) = it.next() {
            let b: Element<'_, Msg> = it.next().unwrap_or_else(|| space().into());
            grid = grid.push(
                row![
                    container(a).width(Length::FillPortion(1)),
                    container(b).width(Length::FillPortion(1))
                ]
                .spacing(12),
            );
        }
        let body = column![
            ui.progress(1.0, 4.0, None),
            space().height(14),
            ui.text(t::WELCOME_TITLE(), 44, Weight::ExtraBold, theme::TEXT),
            ui.text(t::WELCOME_LEAD(), 15, Weight::Normal, theme::SOFT),
            space().height(10),
            grid,
            space().height(14),
            row![
                ui.primary(t::CONTINUE().to_string(), false, Some(Msg::FinishWelcome)),
                ui.text(t::WELCOME_FOOT(), 13, Weight::Normal, theme::SOFT),
            ]
            .spacing(14)
            .align_y(iced::Center),
        ]
        .spacing(8)
        .width(820);
        column![top, container(body).center_x(Fill).padding([20, 0])].into()
    }
}
