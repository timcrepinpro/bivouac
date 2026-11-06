//! Briques des pages secondaires (Grille, Jeu, Paramètres, Statistiques…), prises
//! dans les maquettes du même nom. Complète `kit.rs`.

use iced::font::Weight;
use iced::widget::canvas::{self, Frame, LineDash, Path, Stroke};
use iced::widget::{button, column, container, image, row, space, stack, text_input};
use iced::{mouse, Border, Color, Element, Fill, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::kit::Ui;
use crate::theme::{self, Mode};

/// Contour pointillé `#4A5470` (cartes « non connecté », pastilles Collections…).
pub const DASH: Color = Color::from_rgb(
    0x4A as f32 / 255.0,
    0x54 as f32 / 255.0,
    0x70 as f32 / 255.0,
);
/// Avatar par défaut des maquettes (`#3B3166`).
pub const AVATAR: Color = Color::from_rgb(
    0x3B as f32 / 255.0,
    0x31 as f32 / 255.0,
    0x66 as f32 / 255.0,
);

impl Ui {
    /// Titre de page (32 px, 800).
    pub fn h1<'a>(self, s: impl iced::widget::text::IntoFragment<'a>) -> iced::widget::Text<'a> {
        self.text(s, 32, Weight::ExtraBold, theme::TEXT)
    }

    /// Titre de bloc (20 px, 700).
    pub fn h2<'a>(self, s: impl iced::widget::text::IntoFragment<'a>) -> iced::widget::Text<'a> {
        self.text(s, 20, Weight::Bold, theme::TEXT)
    }

    /// Sous-titre gris sous un titre de page.
    pub fn lead<'a>(self, s: impl iced::widget::text::IntoFragment<'a>) -> iced::widget::Text<'a> {
        self.text(s, 14, Weight::Normal, theme::MUTED)
    }

    /// Petites capitales des cartes (« INSTALLATION »), 12 px 700.
    pub fn caps<'a>(self, s: &str) -> iced::widget::Text<'a> {
        self.text(s.to_uppercase(), 12, Weight::Bold, theme::MUTED)
    }

    /// Carte `#131722`, rayon 16 (pixel : bordure de 2 px).
    pub fn card<'a, M: 'a>(self, content: impl Into<Element<'a, M>>, pad: u16) -> Element<'a, M> {
        container(content)
            .padding(pad)
            .width(Fill)
            .style(move |_| self.panel(theme::CARD, 16.0, None))
            .into()
    }

    /// Carte au contour pointillé (élément pas encore actif).
    pub fn dashed_card<'a, M: 'a>(
        self,
        content: impl Into<Element<'a, M>>,
        pad: u16,
    ) -> Element<'a, M> {
        let r = if self.mode == Mode::Normal { 16.0 } else { 0.0 };
        stack![
            container(content).padding(pad).width(Fill),
            canvas::Canvas::new(Dashed { radius: r })
                .width(Fill)
                .height(Fill),
        ]
        .into()
    }

    /// Ligne « libellé ……… valeur » des cartes d'informations.
    pub fn kv<'a, M: 'a>(self, label: &'a str, value: String) -> Element<'a, M> {
        // Valeur longue (chemin de dossier) : sous le libellé, pour ne pas déborder.
        if value.chars().count() > 26 {
            return column![
                self.text(label, 14, Weight::Normal, theme::TEXT),
                self.mono(value, 13, theme::SOFT),
            ]
            .spacing(4)
            .into();
        }
        row![
            self.text(label, 14, Weight::Normal, theme::TEXT),
            space().width(Fill),
            self.mono(value, 14, theme::TEXT),
        ]
        .align_y(iced::Center)
        .into()
    }

    /// Bouton contour (`border:1px solid #2B3246`, hauteur 38, 12 px 600).
    pub fn outline<'a, M: Clone + 'a>(
        self,
        label: impl Into<String>,
        msg: Option<M>,
        fill: bool,
    ) -> Element<'a, M> {
        let enabled = msg.is_some();
        let b = button(
            container(self.text(
                label.into(),
                12,
                Weight::Semibold,
                if enabled { theme::TEXT } else { theme::MUTED },
            ))
            .center_x(if fill { Fill } else { Length::Shrink })
            .height(Fill)
            .align_y(iced::Center),
        )
        .on_press_maybe(msg)
        .height(38)
        .padding([0, 14])
        .style(move |_, status| iced::widget::button::Style {
            background: (status == button::Status::Hovered).then(|| theme::CARD2.into()),
            text_color: theme::TEXT,
            border: Border {
                radius: self.r(10.0).into(),
                width: 1.0,
                color: theme::LINE2,
            },
            ..Default::default()
        });
        if fill {
            b.width(Fill).into()
        } else {
            b.into()
        }
    }

    /// Petit bouton d'accent (« Connecter », hauteur 38, 13 px 700).
    pub fn small_primary<'a, M: Clone + 'a>(
        self,
        label: impl Into<String>,
        msg: Option<M>,
    ) -> Element<'a, M> {
        let accent = self.accent;
        button(
            container(self.text(label.into(), 13, Weight::Bold, theme::BG))
                .height(Fill)
                .align_y(iced::Center),
        )
        .on_press_maybe(msg)
        .height(38)
        .padding([0, 16])
        .style(move |_, status| iced::widget::button::Style {
            background: Some(
                match status {
                    button::Status::Hovered => crate::kit::lighten(accent, 0.08),
                    button::Status::Disabled => theme::CARD2,
                    _ => accent,
                }
                .into(),
            ),
            text_color: theme::BG,
            border: Border {
                radius: self.r(10.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    /// Bouton clair (« Regrouper », « Envoyer »).
    pub fn light<'a, M: Clone + 'a>(
        self,
        label: impl Into<String>,
        msg: Option<M>,
    ) -> Element<'a, M> {
        button(
            container(self.text(label.into(), 13, Weight::Bold, theme::BG))
                .height(Fill)
                .align_y(iced::Center),
        )
        .on_press_maybe(msg)
        .height(38)
        .padding([0, 18])
        .style(move |_, _| iced::widget::button::Style {
            background: Some(theme::TEXT.into()),
            text_color: theme::BG,
            border: Border {
                radius: self.r(10.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    /// Lien texte (« ← Bibliothèque », « + Nouvelle collection »).
    pub fn link<'a, M: Clone + 'a>(
        self,
        label: impl Into<String>,
        size: u16,
        color: Color,
        msg: M,
    ) -> Element<'a, M> {
        button(self.text(label.into(), size, Weight::Semibold, color))
            .on_press(msg)
            .padding(0)
            .style(|_, _| iced::widget::button::Style::default())
            .into()
    }

    /// Pastille pointillée de la Bibliothèque (« Collections », « Doublons · 2 »).
    pub fn dashed_pill<'a, M: Clone + 'a>(self, label: String, msg: M) -> Element<'a, M> {
        let r = match self.mode {
            Mode::Normal => 15.0,
            _ => 0.0,
        };
        button(stack![
            container(self.text(label, 12, Weight::Semibold, theme::TEXT))
                .height(30)
                .padding([0, 12])
                .align_y(iced::Center),
            canvas::Canvas::new(Dashed { radius: r })
                .width(Fill)
                .height(30),
        ])
        .on_press(msg)
        .padding(0)
        .style(|_, _| iced::widget::button::Style::default())
        .into()
    }

    /// Barre de progression (`#2B3246`, remplissage accent).
    pub fn progress<'a, M: 'a>(self, frac: f32, h: f32, color: Option<Color>) -> Element<'a, M> {
        let frac = frac.clamp(0.0, 1.0);
        let fill = color.unwrap_or(self.accent);
        let r = self.r(h / 2.0);
        let mut bar = row![];
        if frac > 0.0 {
            bar = bar.push(
                container(space())
                    .width(Length::FillPortion((frac * 1000.0).round().max(1.0) as u16))
                    .height(h)
                    .style(move |_| self.bar(fill, r)),
            );
        }
        if frac < 1.0 {
            bar = bar.push(space().width(Length::FillPortion(
                ((1.0 - frac) * 1000.0).round().max(1.0) as u16,
            )));
        }
        container(bar)
            .width(Fill)
            .height(h)
            .style(move |_| self.bar(theme::LINE2, r))
            .into()
    }

    fn bar(self, color: Color, r: f32) -> container::Style {
        container::Style {
            background: Some(color.into()),
            border: Border {
                radius: r.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Interrupteur (48 × 26).
    pub fn toggle<'a, M: Clone + 'a>(self, on: bool, msg: M) -> Element<'a, M> {
        let accent = self.accent;
        let knob = container(space().width(20).height(20)).style(move |_| container::Style {
            background: Some(theme::TEXT.into()),
            border: Border {
                radius: match self.mode {
                    Mode::Normal => 10.0,
                    Mode::PixelCarre => 0.0,
                    Mode::PixelArrondi => theme::STEP_MARK + 4.0,
                }
                .into(),
                ..Default::default()
            },
            ..Default::default()
        });
        let track = if on {
            row![space().width(Fill), knob]
        } else {
            row![knob, space().width(Fill)]
        };
        button(track.padding(3))
            .on_press(msg)
            .width(48)
            .height(26)
            .padding(0)
            .style(move |_, _| iced::widget::button::Style {
                background: Some(if on { accent } else { theme::LINE2 }.into()),
                border: Border {
                    radius: self.r(13.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Choix exclusif sur fond sombre (« Grille | Liste », « Carrés | Arrondis »).
    /// `accent_on` : l'option choisie prend l'accent (sinon `#1B2030`).
    pub fn segmented<'a, M: Clone + 'a>(
        self,
        options: Vec<(&'a str, M)>,
        selected: usize,
        accent_on: bool,
    ) -> Element<'a, M> {
        let accent = self.accent;
        let mut r = row![].spacing(2);
        for (i, (label, msg)) in options.into_iter().enumerate() {
            let on = i == selected;
            let fg = match (on, accent_on) {
                (true, true) => theme::BG,
                (true, false) => theme::TEXT,
                _ => theme::MUTED,
            };
            r = r.push(
                button(
                    container(self.text(
                        label,
                        12,
                        if on { Weight::Bold } else { Weight::Semibold },
                        fg,
                    ))
                    .height(Fill)
                    .align_y(iced::Center),
                )
                .on_press(msg)
                .height(28)
                .padding([0, 12])
                .style(move |_, _| iced::widget::button::Style {
                    background: on.then(|| if accent_on { accent } else { theme::CARD2 }.into()),
                    border: Border {
                        radius: self.r(8.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }
        container(r)
            .padding(3)
            .style(move |_| container::Style {
                background: Some(theme::CARD.into()),
                border: Border {
                    radius: self.r(10.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Pastille de boutique « ST », « EP », « GOG » (46 × 46, `#1B2030`).
    pub fn badge<'a, M: 'a>(self, label: &'a str, size: f32) -> Element<'a, M> {
        container(self.text(label, 13, Weight::ExtraBold, theme::TEXT))
            .center(size)
            .style(move |_| container::Style {
                background: Some(theme::CARD2.into()),
                border: Border {
                    radius: self.r(12.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Avatar rond : image de la boutique si elle existe, sinon la couleur des maquettes.
    pub fn avatar<'a, M: 'a>(self, size: f32, picture: Option<&'a str>) -> Element<'a, M> {
        let radius = match self.mode {
            Mode::Normal => size / 2.0,
            Mode::PixelCarre => 0.0,
            Mode::PixelArrondi => theme::STEP_MARK + 12.0,
        };
        match picture {
            Some(p) => container(
                image(image::Handle::from_path(p))
                    .width(size)
                    .height(size)
                    .border_radius(if self.mode == Mode::Normal {
                        size / 2.0
                    } else {
                        0.0
                    }),
            )
            .width(size)
            .height(size)
            .into(),
            None => container(space().width(size).height(size))
                .style(move |_| container::Style {
                    background: Some(AVATAR.into()),
                    border: Border {
                        radius: radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into(),
        }
    }

    /// Carte d'état vide : pas de données réelles pour ce bloc (jamais de faux contenu).
    pub fn empty<'a, M: 'a>(self, title: &'a str, body: &'a str) -> Element<'a, M> {
        self.card(
            column![
                self.text(title, 16, Weight::Bold, theme::TEXT),
                self.text(body, 13, Weight::Normal, theme::MUTED)
                    .line_height(iced::widget::text::LineHeight::Relative(1.5)),
            ]
            .spacing(6),
            22,
        )
    }

    /// Ligne de réglage : titre, aide, contrôle à droite.
    pub fn setting<'a, M: 'a>(
        self,
        title: &'a str,
        help: Option<&'a str>,
        control: Element<'a, M>,
    ) -> Element<'a, M> {
        let mut left = column![self.text(title, 14, Weight::Bold, theme::TEXT)].spacing(3);
        if let Some(h) = help {
            left = left.push(self.text(h, 12, Weight::Normal, theme::MUTED));
        }
        row![left.width(Fill), control]
            .align_y(iced::Center)
            .padding([14, 18])
            .into()
    }

    /// Séparateur horizontal `#232A36`.
    pub fn rule<'a, M: 'a>(self) -> Element<'a, M> {
        container(space())
            .width(Fill)
            .height(1)
            .style(|_| crate::kit::bg(theme::LINE))
            .into()
    }

    /// Champ de saisie des maquettes (`#171B27`, contour accent quand il est actif).
    pub fn field<'a, M: Clone + 'a>(
        self,
        placeholder: &'a str,
        value: &'a str,
        on_input: impl Fn(String) -> M + 'a,
        size: u16,
        mono: bool,
    ) -> text_input::TextInput<'a, M> {
        let accent = self.accent;
        text_input(placeholder, value)
            .on_input(on_input)
            .size(f32::from(size))
            .padding([14, 18])
            .font(if mono {
                self.mode.mono()
            } else {
                self.mode.font(Weight::Normal)
            })
            .style(move |_, status| text_input::Style {
                background: theme::FIELD.into(),
                border: Border {
                    radius: self.r(12.0).into(),
                    width: 1.0,
                    color: if matches!(status, text_input::Status::Focused { .. }) {
                        accent
                    } else {
                        theme::LINE2
                    },
                },
                icon: theme::MUTED,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: Color { a: 0.4, ..accent },
            })
    }

    /// Ligne de liste latérale (« Tous · 184 »), active sur `#1B2030`.
    pub fn side_item<'a, M: Clone + 'a>(
        self,
        label: String,
        count: Option<usize>,
        active: bool,
        msg: Option<M>,
    ) -> Element<'a, M> {
        let mut r = row![self
            .text(
                label,
                14,
                if active { Weight::Bold } else { Weight::Normal },
                if active { theme::TEXT } else { theme::SOFT },
            )
            .width(Fill)]
        .align_y(iced::Center);
        if let Some(n) = count {
            r = r.push(self.text(n.to_string(), 14, Weight::Normal, theme::MUTED));
        }
        button(r)
            .on_press_maybe(msg)
            .width(Fill)
            .padding([10, 14])
            .style(move |_, status| iced::widget::button::Style {
                background: if active {
                    Some(theme::CARD2.into())
                } else if status == button::Status::Hovered {
                    Some(Color::from_rgb8(0x16, 0x1A, 0x26).into())
                } else {
                    None
                },
                text_color: theme::TEXT,
                border: Border {
                    radius: self.r(12.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Bloc de chiffre clé (« TEMPS DE JEU / 20 h 12 / … »).
    pub fn metric<'a, M: 'a>(
        self,
        label: &'a str,
        value: String,
        note: Option<String>,
    ) -> Element<'a, M> {
        let mut c = column![
            self.text(label.to_uppercase(), 12, Weight::Semibold, theme::SOFT),
            self.text(value, 30, Weight::ExtraBold, theme::TEXT),
        ]
        .spacing(4);
        if let Some(n) = note {
            c = c.push(self.text(n, 12, Weight::Normal, theme::MUTED));
        }
        container(c)
            .padding([18, 20])
            .width(Fill)
            .style(move |_| self.panel(theme::CARD, 16.0, None))
            .into()
    }
}

/// Contour pointillé dessiné par-dessus un élément.
struct Dashed {
    radius: f32,
}

impl<M> canvas::Program<M> for Dashed {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        let r = self.radius.min(bounds.height / 2.0);
        let path = Path::rounded_rectangle(
            Point::new(0.5, 0.5),
            Size::new(bounds.width - 1.0, bounds.height - 1.0),
            r.into(),
        );
        f.stroke(
            &path,
            Stroke {
                line_dash: LineDash {
                    segments: &[4.0, 3.0],
                    offset: 0,
                },
                ..Stroke::default().with_width(1.0).with_color(DASH)
            },
        );
        vec![f.into_geometry()]
    }
}
