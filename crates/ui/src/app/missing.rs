//! Maquette `Lancement-App` : l'application d'une boutique est introuvable.
//! Trois actions (PROMPT §2.2) : installer, indiquer l'emplacement, annuler.

use iced::font::Weight;
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{column, container, row, space};
use iced::{mouse, Color, Element, Fill, Point, Rectangle, Renderer, Theme};
use launcher_core::StoreId;

use super::{App, ClientStatus, Msg};
use crate::kit::Ui;
use crate::texts as t;
use crate::theme::{self, Mode};

const RED: Color = Color::from_rgb(0.937, 0.267, 0.267);

impl App {
    pub(super) fn client_missing<'a>(
        &'a self,
        ui: Ui,
        store: StoreId,
        game: Option<usize>,
        note: Option<&'a str>,
    ) -> Element<'a, Msg> {
        let game = game.and_then(|i| self.games.get(i));

        let cover: Element<'a, Msg> = match game {
            Some(g) => ui.cover(g, 176.0, 224.0, 16.0, false),
            None => {
                let (c1, c2) = theme::cover_colors(store.name());
                ui.art(c1, c2, None, None, 176.0, 224.0, 16.0)
            }
        };

        // Les autres boutiques trouvées sur ce PC.
        let others: Vec<StoreId> = StoreId::ALL
            .into_iter()
            .filter(|s| *s != store)
            .filter(|s| matches!(self.client(*s), Some(ClientStatus::Installed { .. })))
            .collect();
        let mut checks = column![check(ui, false, t::not_in_usual_places(store))].spacing(14);
        if !others.is_empty() {
            checks = checks.push(check(ui, true, t::others_work(&others)));
        }
        checks = checks.push(check(ui, true, t::STILL_VISIBLE().to_string()));

        let install = ui.primary(t::install(store), false, Some(Msg::InstallClient(store)));
        let locate = ui.secondary(t::locate(store), Msg::PickClientDir(store));
        let cancel = ui.ghost(t::CANCEL(), Msg::BackToLibrary);

        let mut body = column![
            cover,
            space().height(10),
            ui.text(
                t::client_missing_title(store),
                40,
                Weight::ExtraBold,
                theme::TEXT
            ),
            container(
                ui.text(
                    t::client_missing_body(store, game.map(|g| g.title.as_str())),
                    16,
                    Weight::Normal,
                    theme::MUTED
                )
                .align_x(iced::Center)
            )
            .max_width(600),
            space().height(4),
            checks,
            space().height(8),
            row![install, locate, cancel]
                .spacing(12)
                .align_y(iced::Center),
            ui.text(t::install_hint(store), 13, Weight::Normal, theme::MUTED),
        ]
        .spacing(14)
        .align_x(iced::Center);
        if let Some(n) = note {
            body = body.push(
                container(ui.text(n, 13, Weight::Semibold, theme::TEXT))
                    .padding([10, 14])
                    .style(move |_| ui.panel(theme::CARD2, 10.0, Some(theme::LINE2))),
            );
        }

        column![
            container(self.header(ui, 0)).padding([16, 40]),
            container(body)
                .width(Fill)
                .height(Fill)
                .align_x(iced::Center)
                .align_y(iced::Center)
                .padding([0, 16]),
        ]
        .into()
    }
}

/// Ligne de la liste : pastille ✓ (accent) ou ✕ (rouge) puis le texte.
fn check<'a>(ui: Ui, ok: bool, label: String) -> Element<'a, Msg> {
    row![
        iced::widget::canvas(Mark {
            ok,
            color: if ok { ui.accent } else { RED },
            square: ui.mode == Mode::PixelCarre,
        })
        .width(22)
        .height(22),
        ui.text(label, 16, Weight::Normal, theme::TEXT),
    ]
    .spacing(12)
    .align_y(iced::Center)
    .into()
}

/// Pastille dessinée (pas de glyphe : les polices n'ont pas toutes ✓ et ✕).
struct Mark {
    ok: bool,
    color: Color,
    square: bool,
}

impl<M> canvas::Program<M> for Mark {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        let s = bounds.width.min(bounds.height);
        let c = Point::new(s / 2.0, s / 2.0);
        let disc = if self.square {
            Path::rectangle(Point::ORIGIN, iced::Size::new(s, s))
        } else {
            Path::circle(c, s / 2.0)
        };
        f.fill(&disc, self.color);
        let stroke = Stroke::default()
            .with_width(s * 0.12)
            .with_color(theme::BG)
            .with_line_cap(canvas::LineCap::Round);
        let p = |x: f32, y: f32| Point::new(x * s, y * s);
        let mark = if self.ok {
            Path::new(|b| {
                b.move_to(p(0.30, 0.52));
                b.line_to(p(0.44, 0.66));
                b.line_to(p(0.71, 0.36));
            })
        } else {
            Path::new(|b| {
                b.move_to(p(0.35, 0.35));
                b.line_to(p(0.65, 0.65));
                b.move_to(p(0.65, 0.35));
                b.line_to(p(0.35, 0.65));
            })
        };
        f.stroke(&mark, stroke);
        vec![f.into_geometry()]
    }
}
