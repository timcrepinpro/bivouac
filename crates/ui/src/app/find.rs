//! Maquettes `Recherche` et `Code`.

use iced::font::Weight;
use iced::widget::{button, column, container, row, space, text_input, Id};
use iced::{Border, Color, Element, Fill, Length};
use launcher_core::StoreId;

use super::settings::badge_label;
use super::{App, Msg, Page, SEARCH_ID};
use crate::kit::{icon, Icon, Ui};
use crate::texts as t;
use crate::theme;

const SHOWN: usize = 30;

fn domain(s: StoreId) -> &'static str {
    match s {
        StoreId::Steam => "steampowered.com",
        StoreId::Epic => "epicgames.com",
        StoreId::Gog => "gog.com",
    }
}

impl App {
    pub(super) fn search_page(&self, ui: Ui) -> Element<'_, Msg> {
        let accent = ui.accent;
        let input = text_input(t::SEARCH_BIG(), &self.query)
            .id(Id::new(SEARCH_ID))
            .on_input(Msg::Search)
            .size(20)
            .padding([18, 20])
            .font(ui.mode.font(Weight::Normal))
            .style(move |_, _| text_input::Style {
                background: theme::CARD.into(),
                border: Border {
                    radius: ui.r(16.0).into(),
                    width: 2.0,
                    color: accent,
                },
                icon: theme::MUTED,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: Color { a: 0.4, ..accent },
            });
        let pills = row![
            ui.pill(t::EVERYTHING(), true, Msg::FocusSearch),
            ui.pill(t::MY_LIBRARY(), false, Msg::FocusSearch),
        ]
        .spacing(8);

        let mut col = column![input, pills].spacing(18);
        if self.query.trim().is_empty() {
            col = col.push(ui.text(t::SEARCH_HINT(), 14, Weight::Normal, theme::MUTED));
        } else {
            col = col.push(ui.text(
                format!(
                    "{} · {}",
                    t::IN_LIBRARY().to_uppercase(),
                    self.visible.len()
                ),
                13,
                Weight::Bold,
                theme::MUTED,
            ));
            if self.visible.is_empty() {
                col = col.push(ui.text(t::NO_MATCH(), 14, Weight::Normal, theme::MUTED));
            }
            for &i in self.visible.iter().take(SHOWN) {
                let g = &self.games[i];
                col = col.push(
                    button(
                        container(
                            row![
                                ui.cover(g, 56.0, 56.0, 12.0, false),
                                column![
                                    ui.text(g.title.as_str(), 16, Weight::Bold, theme::TEXT),
                                    ui.text(
                                        format!(
                                            "{} · {}",
                                            g.store.name(),
                                            t::played_hours(g.playtime_minutes)
                                        ),
                                        13,
                                        Weight::Normal,
                                        theme::MUTED
                                    ),
                                ]
                                .spacing(4)
                                .width(Fill),
                                ui.small_primary(t::PLAY(), Some(Msg::PlayGame(i))),
                            ]
                            .spacing(14)
                            .align_y(iced::Center),
                        )
                        .padding(16)
                        .width(Fill)
                        .style(move |_| ui.panel(theme::CARD, 16.0, None)),
                    )
                    .on_press(Msg::OpenGame(i))
                    .padding(0)
                    .style(|_, _| button::Style::default()),
                );
            }
        }
        col = col
            .push(space().height(6))
            .push(ui.text(t::STORE().to_uppercase(), 13, Weight::Bold, theme::MUTED))
            .push(ui.text(t::STORE_SEARCH_LATER(), 13, Weight::Normal, theme::MUTED));
        col.into()
    }

    pub(super) fn code_page(&self, ui: Ui) -> Element<'_, Msg> {
        let accent = ui.accent;
        let number = |n: &'static str| {
            container(ui.text(n, 14, Weight::ExtraBold, theme::BG))
                .center(28)
                .style(move |_| container::Style {
                    background: Some(accent.into()),
                    border: Border {
                        radius: if ui.mode == theme::Mode::Normal {
                            14.0.into()
                        } else {
                            ui.r(14.0).into()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
        };
        let tile = |label: &'static str,
                    name: &'static str,
                    sub: &'static str,
                    on: bool,
                    msg: Option<Msg>| {
            let enabled = msg.is_some();
            button(
                row![
                    ui.badge(label, 40.0),
                    column![
                        ui.text(
                            name,
                            15,
                            Weight::Bold,
                            if enabled { theme::TEXT } else { theme::MUTED }
                        ),
                        ui.mono(sub, 10, theme::MUTED),
                    ]
                    .spacing(2),
                ]
                .spacing(12)
                .align_y(iced::Center),
            )
            .on_press_maybe(msg)
            .width(Length::FillPortion(1))
            .padding([12, 14])
            .style(move |_, _| button::Style {
                background: Some(theme::CARD2.into()),
                border: Border {
                    radius: ui.r(14.0).into(),
                    width: if on { 2.0 } else { 0.0 },
                    color: accent,
                },
                ..Default::default()
            })
        };
        let mut first = row![].spacing(12);
        for s in StoreId::ALL {
            first = first.push(tile(
                badge_label(s),
                s.name(),
                domain(s),
                self.code_store == s,
                Some(Msg::CodeStore(s)),
            ));
        }
        let later = row![
            tile("EA", "EA app", t::LATER_STORE(), false, None),
            tile("UB", "Ubisoft Connect", t::LATER_STORE(), false, None),
            tile("BN", "Battle.net", t::LATER_STORE(), false, None),
        ]
        .spacing(12);

        let go = !self.code.trim().is_empty();
        let fg = if go { theme::BG } else { theme::MUTED };
        let cta = button(
            container(
                row![
                    ui.text(t::continue_on(self.code_store), 17, Weight::ExtraBold, fg),
                    icon(Icon::External, fg, 18.0),
                ]
                .spacing(10)
                .align_y(iced::Center),
            )
            .center(Fill),
        )
        .on_press_maybe(go.then_some(Msg::Redeem))
        .width(Fill)
        .height(56)
        .padding(0)
        .style(move |_, status| button::Style {
            background: Some(
                if status == button::Status::Disabled {
                    theme::CARD2
                } else {
                    accent
                }
                .into(),
            ),
            border: Border {
                radius: ui.r(14.0).into(),
                ..Default::default()
            },
            ..Default::default()
        });
        let cta = container(cta).center_x(Fill);

        let left = ui.card(
            column![
                row![
                    number("1"),
                    ui.text(t::WHICH_APP(), 18, Weight::Bold, theme::TEXT)
                ]
                .spacing(12)
                .align_y(iced::Center),
                first,
                later,
                space().height(6),
                row![
                    number("2"),
                    ui.text(t::YOUR_CODE(), 18, Weight::Bold, theme::TEXT)
                ]
                .spacing(12)
                .align_y(iced::Center),
                ui.text(t::CODE_LABEL(), 13, Weight::Normal, theme::SOFT),
                ui.field(t::CODE_PLACEHOLDER(), &self.code, Msg::CodeInput, 17, true),
                space().height(4),
                cta,
                container(ui.text(
                    t::code_note(self.code_store),
                    12,
                    Weight::Normal,
                    theme::MUTED
                ))
                .center_x(Fill),
            ]
            .spacing(14),
            26,
        );

        let mut steps = column![ui.caps(t::WHAT_HAPPENS())].spacing(16);
        for (n, (title, body)) in t::CODE_STEPS().iter().enumerate() {
            steps = steps.push(
                row![
                    ui.mono(format!("{:02}", n + 1), 13, accent).width(34),
                    column![
                        ui.text(*title, 15, Weight::Bold, theme::TEXT),
                        ui.text(*body, 13, Weight::Normal, theme::SOFT),
                    ]
                    .spacing(4),
                ]
                .align_y(iced::Alignment::Start),
            );
        }
        let right = column![
            ui.card(steps, 24),
            container(
                column![
                    ui.text(t::IG_TITLE(), 15, Weight::Bold, theme::TEXT),
                    ui.text(t::IG_BODY(), 13, Weight::Normal, theme::SOFT),
                ]
                .spacing(8)
            )
            .padding(18)
            .width(Fill)
            .style(move |_| ui.panel(theme::CARD2, 16.0, None)),
        ]
        .spacing(16);

        column![
            ui.link(t::BACK_STORE(), 13, theme::MUTED, Msg::Go(Page::Store)),
            ui.h1(t::CODE_TITLE()),
            ui.lead(t::CODE_LEAD()),
            space().height(6),
            row![
                container(left).width(Length::FillPortion(16)),
                container(right).width(Length::FillPortion(10)),
            ]
            .spacing(24),
        ]
        .spacing(10)
        .into()
    }
}
