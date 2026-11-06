//! Maquettes `Doublons` et `Collections` (Jalon 4 : bibliothèque intelligente).
//! Ici, seulement ce qui se calcule déjà sur de vraies données : les jeux installés
//! sur plusieurs boutiques et la collection automatique « Jamais lancés ».

use iced::font::Weight;
use iced::widget::{button, column, container, mouse_area, row, scrollable, space};
use iced::{Border, Element, Fill, Length};
use launcher_core::duplicates;

use super::{App, Msg};
use crate::kit::{self, Ui};
use crate::texts as t;
use crate::theme;

impl App {
    pub(super) fn duplicates(&self, ui: Ui) -> Element<'_, Msg> {
        let groups = duplicates(&self.games);
        // Espace occupé en double : toutes les copies sauf la plus jouée.
        let extra: u64 = groups
            .iter()
            .flat_map(|g| g.iter().skip(1))
            .filter_map(|&i| self.games[i].size_bytes)
            .sum();
        let metrics = row![
            ui.metric(t::DUPES_COUNT(), groups.len().to_string(), None),
            ui.metric(
                t::DUPES_SPACE(),
                kit::size(Some(extra)).replace('—', &t::size_gb(0.0)),
                None
            ),
        ]
        .spacing(16);

        let mut list = column![].spacing(14);
        if groups.is_empty() {
            list = list.push(ui.empty(t::NO_DUPES(), t::NO_DUPES_HELP()));
        }
        for group in &groups {
            let first = &self.games[group[0]];
            let stores: Vec<&str> = group.iter().map(|&i| self.games[i].store.name()).collect();
            let head = row![
                ui.cover(first, 64.0, 84.0, 10.0, false),
                column![
                    ui.text(first.title.as_str(), 20, Weight::ExtraBold, theme::TEXT),
                    ui.text(
                        format!("{} {} · {}", group.len(), t::COPIES(), stores.join(", ")),
                        13,
                        Weight::Normal,
                        theme::MUTED
                    ),
                ]
                .spacing(4),
            ]
            .spacing(16)
            .width(330)
            .align_y(iced::Center);
            let mut copies = row![].spacing(10);
            for (n, &i) in group.iter().enumerate() {
                let g = &self.games[i];
                let main = n == 0;
                let accent = ui.accent;
                copies = copies.push(
                    mouse_area(
                        container(
                            row![
                                ui.dot(14.0, if main { accent } else { theme::LINE2 }),
                                column![
                                    ui.text(g.store.name(), 14, Weight::Bold, theme::TEXT),
                                    ui.text(
                                        format!(
                                            "{} · {} · {}",
                                            t::INSTALLED(),
                                            kit::size(g.size_bytes),
                                            kit::short_playtime(g.playtime_minutes)
                                        ),
                                        13,
                                        Weight::Normal,
                                        theme::MUTED
                                    ),
                                    ui.text(
                                        if main { t::MOST_PLAYED_COPY() } else { "" },
                                        12,
                                        Weight::Normal,
                                        theme::MUTED
                                    ),
                                ]
                                .spacing(2),
                            ]
                            .spacing(12),
                        )
                        .padding([12, 14])
                        .width(Fill)
                        .style(move |_| container::Style {
                            background: Some(if main { theme::CARD2 } else { theme::CARD }.into()),
                            border: Border {
                                radius: ui.r(12.0).into(),
                                width: 1.0,
                                color: if main { accent } else { theme::LINE2 },
                            },
                            ..Default::default()
                        }),
                    )
                    .on_press(Msg::OpenGame(i)),
                );
            }
            list = list.push(
                ui.card(
                    row![head, copies.width(Fill)]
                        .spacing(16)
                        .align_y(iced::Center),
                    18,
                ),
            );
        }

        column![
            ui.h1(t::DUPES_TITLE()),
            ui.lead(t::DUPES_LEAD()),
            space().height(4),
            metrics,
            list,
        ]
        .spacing(16)
        .into()
    }

    pub(super) fn collections(&self, ui: Ui) -> Element<'_, Msg> {
        let never: Vec<usize> = (0..self.games.len())
            .filter(|&i| {
                self.games[i].playtime_minutes.unwrap_or(0) == 0
                    && self.games[i].last_played.is_none()
            })
            .collect();
        let shown: Vec<usize> = if self.coll_never {
            never.clone()
        } else {
            (0..self.games.len()).collect()
        };

        // Colonne de gauche : collections.
        let entry = |label: &str, help: Option<&str>, n: usize, on: bool, msg: Msg| {
            let accent = ui.accent;
            let mut text_col = column![ui.text(
                label.to_string(),
                14,
                if on { Weight::Bold } else { Weight::Semibold },
                theme::TEXT
            )];
            if let Some(h) = help {
                text_col = text_col.push(ui.text(h.to_string(), 11, Weight::Normal, theme::MUTED));
            }
            button(
                row![
                    ui.dot(8.0, if on { theme::TEXT } else { theme::MUTED }),
                    text_col.width(Fill),
                    ui.text(n.to_string(), 12, Weight::Normal, theme::MUTED),
                ]
                .spacing(12)
                .align_y(iced::Center),
            )
            .on_press(msg)
            .width(Fill)
            .padding([10, 16])
            .style(move |_, _| button::Style {
                background: on.then(|| theme::CARD2.into()),
                border: Border {
                    radius: ui.r(10.0).into(),
                    width: 0.0,
                    color: accent,
                },
                ..Default::default()
            })
        };
        let side = column![
            container(ui.text(
                t::COLLECTIONS().to_uppercase(),
                12,
                Weight::Bold,
                theme::MUTED
            ))
            .padding([6, 4]),
            entry(
                t::ALL_MY_GAMES(),
                None,
                self.games.len(),
                !self.coll_never,
                Msg::CollNever(false)
            ),
            container(ui.text(
                t::AUTOMATIC().to_uppercase(),
                12,
                Weight::Bold,
                theme::MUTED
            ))
            .padding(iced::Padding::new(4.0).top(16.0)),
            entry(
                t::NEVER_LAUNCHED(),
                Some(t::NEVER_LAUNCHED_HELP()),
                never.len(),
                self.coll_never,
                Msg::CollNever(true)
            ),
            container(ui.text(t::TAGS().to_uppercase(), 12, Weight::Bold, theme::MUTED))
                .padding(iced::Padding::new(4.0).top(16.0)),
            container(ui.text(t::COLLECTIONS_LATER(), 12, Weight::Normal, theme::MUTED))
                .padding([0, 4]),
        ]
        .spacing(4)
        .width(276);

        // Centre : les jeux de la collection, en cartes de deux colonnes.
        let mut grid = column![].spacing(12);
        for pair in shown.chunks(2) {
            let mut r = row![].spacing(12);
            for &i in pair {
                let g = &self.games[i];
                let sub = match g.playtime_minutes {
                    Some(m) if m > 0 => {
                        format!("{} · {}", g.store.name(), kit::short_playtime(Some(m)))
                    }
                    _ => format!("{} · {}", g.store.name(), t::NEVER_PLAYED().to_lowercase()),
                };
                let on = self.selected == Some(i);
                let card = container(
                    row![
                        ui.cover(g, 56.0, 74.0, 10.0, false),
                        column![
                            ui.text(g.title.as_str(), 16, Weight::Bold, theme::TEXT),
                            ui.text(sub, 13, Weight::Normal, theme::MUTED),
                        ]
                        .spacing(4),
                    ]
                    .spacing(14)
                    .align_y(iced::Center),
                )
                .padding(12)
                .width(Fill)
                .style(move |_| ui.panel(if on { theme::CARD2 } else { theme::CARD }, 14.0, None));
                r = r.push(
                    container(mouse_area(card).on_press(Msg::Select(i)))
                        .width(Length::FillPortion(1)),
                );
            }
            if pair.len() == 1 {
                r = r.push(space().width(Length::FillPortion(1)));
            }
            grid = grid.push(r);
        }
        let title = if self.coll_never {
            t::NEVER_LAUNCHED()
        } else {
            t::ALL_MY_GAMES()
        };
        let center = column![
            ui.h1(title),
            ui.lead(format!(
                "{} · {}",
                t::games_n(shown.len()),
                t::COLLECTIONS_LEAD()
            )),
            scrollable(grid).height(Fill),
        ]
        .spacing(10)
        .width(Fill);

        // Droite : le jeu sélectionné.
        let detail: Element<'_, Msg> = match self
            .selected
            .and_then(|i| self.games.get(i).map(|g| (i, g)))
        {
            Some((i, g)) => column![
                row![
                    ui.cover(g, 64.0, 84.0, 10.0, false),
                    column![
                        ui.text(g.title.as_str(), 20, Weight::Bold, theme::TEXT),
                        ui.text(
                            format!(
                                "{} · {}",
                                g.store.name(),
                                kit::short_playtime(g.playtime_minutes)
                            ),
                            13,
                            Weight::Normal,
                            theme::MUTED
                        ),
                    ]
                    .spacing(4),
                ]
                .spacing(14)
                .align_y(iced::Center),
                ui.caps(t::TAGS()),
                ui.text(t::COLLECTIONS_LATER(), 13, Weight::Normal, theme::MUTED),
                ui.outline(t::OPEN_GAME(), Some(Msg::OpenGame(i)), true),
            ]
            .spacing(14)
            .into(),
            None => ui
                .text(t::NO_GAME(), 13, Weight::Normal, theme::MUTED)
                .into(),
        };
        let aside = container(detail)
            .padding(20)
            .width(320)
            .style(move |_| ui.panel(theme::CARD, 20.0, None));

        row![side, center, column![aside]].spacing(24).into()
    }
}
