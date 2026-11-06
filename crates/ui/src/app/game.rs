//! Maquettes `Jeu-Possede` (fiche d'un jeu installé) et `Lancement`.

use std::time::Instant;

use iced::font::Weight;
use iced::widget::{column, container, row, space, stack};
use iced::{Border, Color, Element, Fill, Length};
use launcher_core::install_root;
use launcher_services::{game_tool, GameTool};

use super::{App, LaunchState, Msg, LAUNCH_SCREEN};
use crate::kit::{self, icon, linear, Icon, Ui, ON_HERO};
use crate::texts as t;
use crate::theme::{self, Mode};

impl App {
    pub(super) fn game_page(&self, ui: Ui, i: usize) -> Element<'_, Msg> {
        let Some(g) = self.games.get(i) else {
            return ui.empty(t::NO_GAME(), t::NO_GAME_HINT());
        };
        let session = match g.last_played {
            Some(at) => format!(
                "{} · {} {}",
                kit::playtime(g.playtime_minutes),
                t::LAST_SESSION(),
                kit::ago(at, Self::now_unix())
            ),
            None => t::NOT_PLAYED_YET().to_string(),
        };
        let hero_body = column![
            space().height(Fill),
            ui.tag(g.store.label()),
            ui.text(g.title.as_str(), 56, Weight::ExtraBold, theme::TEXT)
                .line_height(iced::widget::text::LineHeight::Relative(1.0)),
            row![
                ui.primary(t::PLAY().to_string(), true, Some(Msg::PlayGame(i))),
                container(ui.mono(session, 13, ON_HERO)).padding(iced::Padding::new(0.0).left(8.0)),
            ]
            .spacing(14)
            .align_y(iced::Center),
        ]
        .spacing(14)
        .padding(32);

        let (c1, c2) = theme::cover_colors(&g.title);
        let deep = Color::from_rgb8(0x13, 0x14, 0x3A);
        let r = ui.r(24.0);
        let mut hero =
            stack![container(space())
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(linear(
                        135.0,
                        &[(0.0, kit::darken(c1, 0.2)), (0.55, c2), (1.0, deep)]
                    )),
                    border: Border {
                        radius: r.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })];
        if let Some(p) = g.covers.hero.as_deref() {
            hero = hero.push(
                iced::widget::image(iced::widget::image::Handle::from_path(p))
                    .content_fit(iced::ContentFit::Cover)
                    .border_radius(if ui.mode == Mode::Normal { 24.0 } else { 0.0 })
                    .width(Fill)
                    .height(Fill),
            );
        }
        let fade = Color {
            a: 0.85,
            ..theme::BG
        };
        let clear = Color {
            a: 0.0,
            ..theme::BG
        };
        hero = hero.push(container(space()).width(Fill).height(Fill).style(move |_| {
            container::Style {
                background: Some(linear(0.0, &[(0.0, fade), (0.55, clear)])),
                border: Border {
                    radius: r.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
        let pixel_frame = ui.mode.is_pixel();
        if pixel_frame {
            hero = hero.push(container(space()).width(Fill).height(Fill).style(move |_| {
                container::Style {
                    border: Border {
                        radius: r.into(),
                        width: 3.0,
                        color: Color {
                            a: 0.9,
                            ..theme::BG
                        },
                    },
                    ..Default::default()
                }
            }));
        }
        let hero = container(hero.push(hero_body)).width(Fill).height(340);

        // Colonne de droite : ce que le launcher sait de l'installation.
        let mut install = column![ui.caps(t::INSTALLATION())].spacing(10);
        install = install.push(ui.kv(t::SIZE(), kit::size(g.size_bytes)));
        if let Some(p) = &g.install_path {
            install = install.push(ui.kv(t::FOLDER(), install_root(p)));
        }
        let verify = game_tool(g, GameTool::Verify).map(|_| Msg::Tool(i, GameTool::Verify));
        let uninstall =
            game_tool(g, GameTool::Uninstall).map(|_| Msg::Tool(i, GameTool::Uninstall));
        if verify.is_some() || uninstall.is_some() {
            install = install.push(
                container(
                    row![
                        ui.outline(t::VERIFY(), verify, true),
                        ui.outline(t::UNINSTALL(), uninstall, true)
                    ]
                    .spacing(8),
                )
                .padding(iced::Padding::new(0.0).top(6.0)),
            );
        }
        let aside = column![ui.card(install, 18)].spacing(14);

        let main = column![ui.empty(t::ACHIEVEMENTS(), t::ACHIEVEMENTS_LATER())].spacing(14);

        column![
            ui.link(t::BACK_LIBRARY(), 13, theme::MUTED, Msg::BackToLibrary),
            hero,
            row![
                container(main).width(Length::FillPortion(2)),
                container(aside).width(Length::FillPortion(1)),
            ]
            .spacing(28),
        ]
        .spacing(24)
        .into()
    }

    pub(super) fn launching(
        &self,
        ui: Ui,
        i: usize,
        at: Instant,
        state: &LaunchState,
    ) -> Element<'_, Msg> {
        let Some(g) = self.games.get(i) else {
            return space().into();
        };
        let store = g.store.name();
        let sent = *state == LaunchState::Sent;
        let failed = matches!(state, LaunchState::Failed(_));
        let mut steps = column![
            step(ui, sent || failed, t::step_found(store)),
            step(
                ui,
                sent,
                if sent {
                    t::step_sent(store)
                } else {
                    t::LAUNCH_ASKING().to_string()
                }
            ),
            step(ui, false, t::GAME_STARTING().to_string()),
        ]
        .spacing(14);
        if let LaunchState::Failed(e) = state {
            steps = steps.push(ui.text(e.clone(), 13, Weight::Normal, theme::MUTED));
        }
        let frac = match state {
            LaunchState::Asking => 0.15,
            LaunchState::Sent => {
                0.4 + 0.6 * (at.elapsed().as_secs_f32() / LAUNCH_SCREEN.as_secs_f32()).min(1.0)
            }
            LaunchState::Failed(_) => 0.4,
        };

        let accent = ui.accent;
        let r = ui.r(16.0);
        let cover = stack![
            ui.cover(g, 206.0, 266.0, 16.0, false),
            container(space())
                .width(206)
                .height(266)
                .style(move |_| container::Style {
                    border: Border {
                        radius: r.into(),
                        width: 3.0,
                        color: accent,
                    },
                    ..Default::default()
                }),
        ];
        let content = column![
            cover,
            space().height(10),
            ui.text(g.title.as_str(), 40, Weight::ExtraBold, theme::TEXT),
            ui.text(t::launch_via(store), 14, Weight::Normal, theme::MUTED),
            space().height(8),
            container(steps).width(560),
            container(ui.progress(frac, 8.0, None)).width(560),
            space().height(6),
            ui.outline(t::CLOSE(), Some(Msg::BackToLibrary), false),
        ]
        .spacing(8)
        .align_x(iced::Center);
        container(content)
            .width(Fill)
            .height(Fill)
            .align_x(iced::Center)
            .padding(iced::Padding::new(0.0).top(80.0))
            .into()
    }
}

/// Étape du lancement : rond coché (fait) ou cercle vide (en cours).
fn step<'a>(ui: Ui, done: bool, label: String) -> Element<'a, Msg> {
    let accent = ui.accent;
    let radius = match ui.mode {
        Mode::Normal => 11.0,
        Mode::PixelCarre => 0.0,
        Mode::PixelArrondi => theme::STEP_MARK + 8.0,
    };
    let mark: Element<'a, Msg> = if done {
        container(icon(Icon::Check, theme::BG, 14.0))
            .center(22)
            .style(move |_| container::Style {
                background: Some(accent.into()),
                border: Border {
                    radius: radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    } else {
        container(space())
            .width(22)
            .height(22)
            .style(move |_| container::Style {
                border: Border {
                    radius: radius.into(),
                    width: 2.0,
                    color: accent,
                },
                ..Default::default()
            })
            .into()
    };
    row![mark, ui.text(label, 15, Weight::Normal, theme::TEXT)]
        .spacing(12)
        .align_y(iced::Center)
        .into()
}
