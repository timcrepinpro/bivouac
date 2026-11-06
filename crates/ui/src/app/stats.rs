//! Maquettes `Statistiques` et `Profil`. Seuls les totaux donnés par les boutiques
//! existent pour l'instant ; le détail par jour attend le suivi des sessions (Jalon 5).

use iced::font::Weight;
use iced::widget::{column, container, mouse_area, row, space, stack};
use iced::{Border, Color, Element, Fill, Length};
use launcher_core::{playtime_by_store, StoreId};

use super::{App, Msg};
use crate::kit::{self, linear, Ui};
use crate::texts as t;
use crate::theme;

/// Couleurs des boutiques de la maquette `Statistiques`.
fn store_color(s: StoreId) -> Color {
    match s {
        StoreId::Steam => Color::from_rgb8(0x3E, 0x8B, 0xEA),
        StoreId::Gog => Color::from_rgb8(0x1F, 0xA3, 0x7A),
        StoreId::Epic => Color::from_rgb8(0xE4, 0x57, 0x2E),
    }
}

/// « 20 h 12 » (sans minutes au-delà de 100 h).
fn hours(min: u64) -> String {
    match min {
        0 => "0 h".into(),
        m if m >= 6000 => format!("{} h", m / 60),
        m => format!("{} h {:02}", m / 60, m % 60),
    }
}

impl App {
    fn total_minutes(&self) -> u64 {
        self.games.iter().filter_map(|g| g.playtime_minutes).sum()
    }

    /// Indices des jeux les plus joués (temps connu uniquement).
    fn most_played(&self, n: usize) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.games.len())
            .filter(|&i| self.games[i].playtime_minutes.unwrap_or(0) > 0)
            .collect();
        v.sort_by_key(|&i| std::cmp::Reverse(self.games[i].playtime_minutes.unwrap_or(0)));
        v.truncate(n);
        v
    }

    pub(super) fn stats(&self, ui: Ui) -> Element<'_, Msg> {
        let total = self.total_minutes();
        let played = self
            .games
            .iter()
            .filter(|g| g.playtime_minutes.unwrap_or(0) > 0 || g.last_played.is_some())
            .count();
        let last = self
            .games
            .iter()
            .filter(|g| g.last_played.is_some())
            .max_by_key(|g| g.last_played);
        let metrics = row![
            ui.metric(
                t::TOTAL_PLAYTIME(),
                hours(total),
                Some(t::STATS_ALL_TIME().into())
            ),
            ui.metric(t::GAMES_INSTALLED(), self.games.len().to_string(), None),
            ui.metric(t::GAMES_PLAYED(), played.to_string(), None),
            ui.metric(
                t::LAST_PLAYED(),
                last.and_then(|g| g.last_played)
                    .map(|at| kit::ago(at, Self::now_unix()))
                    .unwrap_or_else(|| "—".into()),
                last.map(|g| g.title.clone()),
            ),
        ]
        .spacing(16);

        let per_day = ui.card(
            column![
                ui.text(t::PER_DAY(), 16, Weight::Bold, theme::TEXT),
                ui.text(t::PER_DAY_LATER(), 13, Weight::Normal, theme::MUTED)
                    .line_height(iced::widget::text::LineHeight::Relative(1.5)),
            ]
            .spacing(10),
            20,
        );

        let by_store = playtime_by_store(&self.games);
        let mut bar = row![].spacing(2);
        let mut lines = column![].spacing(0);
        for (s, m) in &by_store {
            let share = if total > 0 {
                *m as f32 / total as f32
            } else {
                0.0
            };
            let c = store_color(*s);
            if share > 0.0 {
                let label = if share >= 0.08 {
                    format!("{:.0} %", share * 100.0)
                } else {
                    String::new()
                };
                bar = bar.push(
                    container(ui.text(label, 12, Weight::Bold, theme::BG))
                        .center_x(Length::FillPortion((share * 1000.0).max(1.0) as u16))
                        .height(34)
                        .align_y(iced::Center)
                        .style(move |_| kit::bg(c)),
                );
            }
            lines = lines.push(ui.rule()).push(
                row![
                    ui.dot(10.0, c),
                    ui.text(s.name(), 14, Weight::Semibold, theme::TEXT),
                    space().width(Fill),
                    ui.text(
                        format!("{} · {:.0} %", hours(*m), share * 100.0),
                        13,
                        Weight::Normal,
                        theme::MUTED
                    ),
                ]
                .spacing(10)
                .align_y(iced::Center)
                .padding([11, 0]),
            );
        }
        let store_card = ui.card(
            column![
                ui.text(t::BY_STORE(), 16, Weight::Bold, theme::TEXT),
                if total > 0 {
                    Element::from(bar)
                } else {
                    ui.text(t::NO_PLAYTIME(), 13, Weight::Normal, theme::MUTED)
                        .into()
                },
                lines,
            ]
            .spacing(14),
            20,
        );

        let top = self.most_played(5);
        let max = top
            .first()
            .and_then(|&i| self.games[i].playtime_minutes)
            .unwrap_or(1)
            .max(1);
        let mut top_list =
            column![ui.text(t::MOST_PLAYED(), 16, Weight::Bold, theme::TEXT)].spacing(14);
        if top.is_empty() {
            top_list = top_list.push(ui.text(t::NO_PLAYTIME(), 13, Weight::Normal, theme::MUTED));
        }
        for i in top {
            let g = &self.games[i];
            let m = g.playtime_minutes.unwrap_or(0);
            top_list = top_list.push(
                mouse_area(
                    row![
                        ui.cover(g, 32.0, 40.0, 8.0, false),
                        column![
                            row![
                                ui.text(g.title.as_str(), 14, Weight::Bold, theme::TEXT),
                                space().width(Fill),
                                ui.text(hours(m), 14, Weight::Normal, theme::MUTED),
                            ],
                            ui.progress(m as f32 / max as f32, 6.0, None),
                        ]
                        .spacing(8),
                    ]
                    .spacing(14)
                    .align_y(iced::Center),
                )
                .on_press(Msg::OpenGame(i)),
            );
        }
        let when = ui.card(
            column![
                ui.text(t::WHEN_YOU_PLAY(), 16, Weight::Bold, theme::TEXT),
                ui.text(t::PER_DAY_LATER(), 13, Weight::Normal, theme::MUTED)
                    .line_height(iced::widget::text::LineHeight::Relative(1.5)),
            ]
            .spacing(10),
            20,
        );

        column![
            ui.h1(t::STATS()),
            ui.lead(t::STATS_LEAD()),
            space().height(4),
            metrics,
            row![
                container(per_day).width(Length::FillPortion(16)),
                container(store_card).width(Length::FillPortion(9)),
            ]
            .spacing(16),
            row![
                container(ui.card(top_list, 20)).width(Length::FillPortion(1)),
                container(when).width(Length::FillPortion(1)),
            ]
            .spacing(16),
        ]
        .spacing(16)
        .into()
    }

    pub(super) fn profile(&self, ui: Ui) -> Element<'_, Msg> {
        let account = self.accounts.iter().find(|a| a.store == StoreId::Steam);
        let (name, sub) = match account {
            Some(a) => (a.name.clone(), t::STEAM_ACCOUNT().to_string()),
            None => (
                t::NO_ACCOUNT().to_string(),
                t::NO_ACCOUNT_HELP().to_string(),
            ),
        };
        let r = ui.r(24.0);
        let banner_bg =
            container(space())
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(linear(
                        90.0,
                        &[
                            (0.0, Color::from_rgb8(0x1B, 0x12, 0x38)),
                            (0.65, Color::from_rgb8(0x0F, 0x1A, 0x3A)),
                            (1.0, Color::from_rgb8(0x0E, 0x6B, 0x5C)),
                        ],
                    )),
                    border: Border {
                        radius: r.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
        let banner = container(stack![
            banner_bg,
            row![
                ui.avatar(96.0, account.and_then(|a| a.avatar.as_deref())),
                column![
                    ui.text(name, 36, Weight::ExtraBold, theme::TEXT),
                    ui.text(sub, 14, Weight::Normal, theme::SOFT),
                ]
                .spacing(4),
            ]
            .spacing(22)
            .align_y(iced::Center)
            .padding([28, 28]),
        ])
        .width(Fill)
        .height(152);

        let total = self.total_minutes();
        let metrics = row![
            ui.metric(t::PROFILE_GAMES(), self.games.len().to_string(), None),
            ui.metric(t::TOTAL_PLAYTIME(), hours(total), None),
        ]
        .spacing(16);

        let by_store = playtime_by_store(&self.games);
        let max = by_store.iter().map(|(_, m)| *m).max().unwrap_or(1).max(1);
        let mut per = column![ui.caps(t::PLAYTIME_BY_STORE())].spacing(14);
        for (s, m) in by_store {
            let color = match s {
                StoreId::Steam => ui.accent,
                StoreId::Epic => Color::from_rgb8(0xA5, 0xB4, 0xFC),
                StoreId::Gog => Color::from_rgb8(0xFD, 0xBA, 0x74),
            };
            per = per.push(
                column![
                    row![
                        ui.text(s.name(), 14, Weight::Normal, theme::TEXT),
                        space().width(Fill),
                        ui.mono(hours(m), 14, theme::TEXT),
                    ],
                    ui.progress(m as f32 / max as f32, 8.0, Some(color)),
                ]
                .spacing(8),
            );
        }
        let mut top = column![ui.caps(t::MOST_PLAYED_SHORT())].spacing(12);
        let picks = self.most_played(3);
        if picks.is_empty() {
            top = top.push(ui.text(t::NO_PLAYTIME(), 13, Weight::Normal, theme::MUTED));
        }
        for i in picks {
            let g = &self.games[i];
            top = top.push(
                mouse_area(
                    row![
                        ui.cover(g, 40.0, 40.0, 10.0, false),
                        ui.text(g.title.as_str(), 14, Weight::Normal, theme::TEXT)
                            .width(Fill),
                        ui.mono(hours(g.playtime_minutes.unwrap_or(0)), 13, theme::MUTED),
                    ]
                    .spacing(12)
                    .align_y(iced::Center),
                )
                .on_press(Msg::OpenGame(i)),
            );
        }
        column![
            banner,
            metrics,
            row![
                container(ui.card(per, 22)).width(Length::FillPortion(1)),
                container(ui.card(top, 22)).width(Length::FillPortion(1)),
            ]
            .spacing(22),
        ]
        .spacing(22)
        .into()
    }
}
