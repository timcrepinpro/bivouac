//! Maquette `Grille` : la bibliothèque en vignettes, avec collections et plateformes.
//! « Favoris » et « À finir » viendront avec les collections (Jalon 4).

use iced::font::Weight;
use iced::widget::{column, container, mouse_area, pick_list, row, scrollable, space};
use iced::{Border, Element, Fill, Length};
use launcher_core::StoreId;

use super::{App, Msg, Page, Sort};
use crate::kit::{self, Ui};
use crate::texts as t;
use crate::theme;

const COLS: usize = 5;

impl App {
    pub(super) fn grid(&self, ui: Ui) -> Element<'_, Msg> {
        row![self.grid_side(ui), self.grid_main(ui)]
            .spacing(24)
            .into()
    }

    fn grid_side(&self, ui: Ui) -> Element<'_, Msg> {
        let head = |label: &str| {
            container(ui.text(label.to_uppercase(), 11, Weight::Bold, theme::MUTED))
                .padding([8, 14])
        };
        let n = self.games.len();
        let mut c = column![
            head(t::COLLECTIONS()),
            ui.side_item(
                t::ALL().into(),
                Some(n),
                self.store.is_none(),
                Some(Msg::Filter(None))
            ),
            ui.side_item(
                t::INSTALLED_PL().into(),
                Some(n),
                false,
                Some(Msg::Filter(None))
            ),
            head(t::PLATFORMS()),
        ]
        .spacing(4);
        for s in StoreId::ALL {
            c = c.push(ui.side_item(
                s.name().into(),
                Some(self.game_count(s)),
                self.store == Some(s),
                Some(Msg::Filter(Some(s))),
            ));
        }
        c = c.push(
            container(ui.link(
                t::NEW_COLLECTION(),
                14,
                ui.accent,
                Msg::Go(Page::Collections),
            ))
            .padding([12, 14]),
        );
        container(c)
            .width(220)
            .padding(12)
            .style(move |_| ui.panel(theme::CARD, 20.0, None))
            .into()
    }

    fn grid_main(&self, ui: Ui) -> Element<'_, Msg> {
        let sort = pick_list(Sort::ALL, Some(self.sort), Msg::SetSort)
            .text_size(13)
            .font(ui.mode.font(Weight::Normal))
            .padding([7, 10])
            .style(move |_, _| pick_list::Style {
                text_color: theme::TEXT,
                placeholder_color: theme::MUTED,
                handle_color: theme::TEXT,
                background: theme::CARD.into(),
                border: Border {
                    radius: ui.r(10.0).into(),
                    width: 1.0,
                    color: theme::LINE2,
                },
            })
            .menu_style(move |_| iced::overlay::menu::Style {
                background: theme::CARD2.into(),
                border: Border {
                    radius: ui.r(10.0).into(),
                    width: 1.0,
                    color: theme::LINE2,
                },
                text_color: theme::TEXT,
                selected_text_color: theme::BG,
                selected_background: ui.accent.into(),
                shadow: iced::Shadow::default(),
            });
        let bar = row![
            container(ui.h1(t::LIBRARY())).padding(iced::Padding::new(0.0).right(12.0)),
            ui.pill(t::ALL_PLATFORMS(), self.store.is_none(), Msg::Filter(None)),
            space().width(Fill),
            ui.text(t::SORT_BY(), 13, Weight::Normal, theme::MUTED),
            sort,
            ui.segmented(
                vec![
                    (t::GRID(), Msg::SetGridView(true)),
                    (t::LIST(), Msg::SetGridView(false))
                ],
                0,
                false
            ),
        ]
        .spacing(10)
        .align_y(iced::Center);

        let picks = self.grid_games();
        let body: Element<'_, Msg> = if picks.is_empty() {
            ui.empty(
                if self.scanning {
                    t::SCANNING()
                } else {
                    t::NO_GAME()
                },
                t::NO_GAME_HINT(),
            )
        } else {
            let mut rows = column![].spacing(18);
            for chunk in picks.chunks(COLS) {
                let mut r = row![].spacing(16);
                for &i in chunk {
                    r = r.push(self.grid_card(ui, i));
                }
                for _ in chunk.len()..COLS {
                    r = r.push(space().width(Length::FillPortion(1)));
                }
                rows = rows.push(r);
            }
            scrollable(rows).height(Fill).into()
        };
        column![bar, body].spacing(16).width(Fill).into()
    }

    fn grid_card(&self, ui: Ui, i: usize) -> Element<'_, Msg> {
        let g = &self.games[i];
        let card = column![
            ui.cover(g, f32::INFINITY, 158.0, 12.0, true),
            row![
                ui.text(g.store.name(), 12, Weight::Normal, theme::MUTED),
                space().width(Fill),
                ui.text(
                    match g.playtime_minutes {
                        Some(m) if m > 0 => kit::short_playtime(Some(m)),
                        _ => "—".into(),
                    },
                    12,
                    Weight::Normal,
                    theme::MUTED
                ),
            ],
        ]
        .spacing(6);
        container(mouse_area(card).on_press(Msg::OpenGame(i)))
            .width(Length::FillPortion(1))
            .into()
    }
}
