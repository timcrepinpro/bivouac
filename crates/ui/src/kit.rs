//! Briques visuelles partagées par toutes les pages, réglées sur les maquettes
//! (`launcher-design/maquettes-html`). Chaque brique connaît les trois modes :
//! normal, pixel carré (`*-Pixel`) et pixel arrondi (`*-PixelDoux`).
//!
//! Règles des maquettes pixel reprises ici :
//! - panneaux `#131722` / `#1B2030` : bordure intérieure de 2 px `#2B3246` ;
//! - bouton principal : ombre dure décalée de 4 px vers le bas, sans flou ;
//! - jaquettes : bordure intérieure de 3 px sombre et trame en damier de 4 px ;
//! - coins en escalier de 4 px (shader `vendor/iced_wgpu`, voir `theme::STEP_MARK`).

use iced::border::Radius;
use iced::font::Weight;
use iced::gradient::Linear;
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{button, column, container, image, row, space, stack, text};
use iced::{
    mouse, Background, Border, Color, ContentFit, Degrees, Element, Fill, Gradient, Length, Point,
    Rectangle, Renderer, Shadow, Size, Theme, Vector,
};
use launcher_core::InstalledGame;

use crate::theme::{self, Mode};

/// Bordure discrète des séparateurs de la barre latérale (`#1D2230`).
pub const RULE: Color = Color::from_rgb(
    0x1D as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x30 as f32 / 255.0,
);
/// Texte clair des éléments posés sur la bannière (`#E8EAF2`).
pub const ON_HERO: Color = Color::from_rgb(
    0xE8 as f32 / 255.0,
    0xEA as f32 / 255.0,
    0xF2 as f32 / 255.0,
);
/// Étiquettes des statistiques (`#C9CEDB`).
pub const STAT_LABEL: Color = Color::from_rgb(
    0xC9 as f32 / 255.0,
    0xCE as f32 / 255.0,
    0xDB as f32 / 255.0,
);

/// Contexte de style d'une vue.
#[derive(Clone, Copy)]
pub struct Ui {
    pub accent: Color,
    pub mode: Mode,
    /// Posé sur la bannière (Bibliothèque) : fonds « verre fumé » sombres ;
    /// sinon fonds blancs à 7 % comme sur les autres maquettes.
    pub hero: bool,
}

impl Ui {
    pub fn text<'a>(
        self,
        s: impl text::IntoFragment<'a>,
        size: u16,
        weight: Weight,
        color: Color,
    ) -> text::Text<'a> {
        text(s)
            .size(f32::from(size))
            .font(self.mode.font(weight))
            .color(color)
    }

    pub fn mono<'a>(
        self,
        s: impl text::IntoFragment<'a>,
        size: u16,
        color: Color,
    ) -> text::Text<'a> {
        text(s)
            .size(f32::from(size))
            .font(self.mode.mono())
            .color(color)
    }

    pub fn r(self, normal: f32) -> f32 {
        self.mode.radius(normal)
    }

    /// Fond de panneau. En mode pixel, les panneaux `#131722` et `#1B2030` reçoivent
    /// la bordure intérieure de 2 px des maquettes.
    pub fn panel(self, color: Color, radius: f32, border: Option<Color>) -> container::Style {
        let pixel_frame = self.mode.is_pixel()
            && (color == theme::CARD || color == theme::CARD2)
            && border.is_none();
        container::Style {
            background: Some(color.into()),
            border: Border {
                radius: self.r(radius).into(),
                width: if pixel_frame {
                    2.0
                } else if border.is_some() {
                    1.0
                } else {
                    0.0
                },
                color: if pixel_frame {
                    theme::LINE2
                } else {
                    border.unwrap_or(Color::TRANSPARENT)
                },
            },
            ..Default::default()
        }
    }

    /// Fond « verre fumé » des éléments posés sur la bannière (`rgba(12,14,20,a)`).
    pub fn glass(self, alpha: f32, radius: f32) -> container::Style {
        container::Style {
            background: Some(
                Color {
                    a: alpha,
                    ..theme::BG
                }
                .into(),
            ),
            border: Border {
                radius: self.r(radius).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Fond des pièces de l'en-tête (navigation, pastilles d'état).
    pub fn chrome(self) -> container::Style {
        if self.hero {
            return self.glass(0.55, 12.0);
        }
        container::Style {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.07).into()),
            border: Border {
                radius: self.r(12.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Pastille de filtre (hauteur 30, rayon 15).
    pub fn pill<'a, M: Clone + 'a>(self, label: &'a str, on: bool, msg: M) -> Element<'a, M> {
        button(
            container(self.text(
                label,
                12,
                Weight::Semibold,
                if on { theme::BG } else { theme::SOFT },
            ))
            .height(Fill)
            .align_y(iced::Center),
        )
        .on_press(msg)
        .height(30)
        .padding([0, 12])
        .style(move |_, _| button::Style {
            background: on.then_some(theme::TEXT.into()),
            text_color: if on { theme::BG } else { theme::SOFT },
            border: Border {
                radius: self.r(15.0).into(),
                width: if on { 0.0 } else { 1.0 },
                color: theme::LINE2,
            },
            ..Default::default()
        })
        .into()
    }

    /// Barre de navigation principale (maquette : `rgba(12,14,20,.55)`, onglet actif clair).
    pub fn nav<'a, M: Clone + 'a>(
        self,
        active: usize,
        on_tab: impl Fn(usize) -> M,
        on_settings: M,
        settings_active: bool,
    ) -> Element<'a, M> {
        let mut nav = row![].spacing(4);
        for (i, label) in crate::texts::NAV().iter().enumerate() {
            let on = i == active;
            nav = nav.push(
                button(
                    container(self.text(
                        *label,
                        13,
                        if on { Weight::Bold } else { Weight::Semibold },
                        if on { theme::BG } else { ON_HERO },
                    ))
                    .height(Fill)
                    .align_y(iced::Center),
                )
                .on_press(on_tab(i))
                .height(34)
                .padding([0, 16])
                .style(move |_, status| button::Style {
                    background: if on {
                        Some(theme::TEXT.into())
                    } else if status == button::Status::Hovered {
                        Some(
                            Color {
                                a: 0.08,
                                ..theme::TEXT
                            }
                            .into(),
                        )
                    } else {
                        None
                    },
                    border: Border {
                        radius: self.r(9.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }
        nav = nav.push(
            button(
                container(icon(
                    Icon::Settings,
                    if settings_active { theme::BG } else { ON_HERO },
                    18.0,
                ))
                .center(Fill),
            )
            .on_press(on_settings)
            .width(38)
            .height(34)
            .padding(0)
            .style(move |_, status| button::Style {
                background: if settings_active {
                    Some(theme::TEXT.into())
                } else if status == button::Status::Hovered {
                    Some(
                        Color {
                            a: 0.08,
                            ..theme::TEXT
                        }
                        .into(),
                    )
                } else {
                    None
                },
                border: Border {
                    radius: self.r(9.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        container(nav)
            .padding(4)
            .style(move |_| self.chrome())
            .into()
    }

    /// Pastille d'état (point d'accent + texte) sur fond verre, hauteur 42.
    pub fn status_chip<'a, M: 'a>(self, label: String) -> Element<'a, M> {
        container(
            row![
                self.dot(8.0, self.accent),
                self.text(label, 13, Weight::Normal, ON_HERO)
            ]
            .spacing(8)
            .align_y(iced::Center),
        )
        .height(42)
        .padding([0, 14])
        .align_y(iced::Center)
        .style(move |_| self.chrome())
        .into()
    }

    /// Point de couleur (carré en pixel carré, cercle en escalier en pixel arrondi).
    pub fn dot<'a, M: 'a>(self, size: f32, color: Color) -> Element<'a, M> {
        let radius = match self.mode {
            Mode::Normal => size / 2.0,
            Mode::PixelCarre => 0.0,
            // Une marche de 4 px effacerait un point de 8 px : il reste carré.
            Mode::PixelArrondi if size < 12.0 => 0.0,
            Mode::PixelArrondi => theme::STEP_MARK + 4.0,
        };
        container(space().width(size).height(size))
            .style(move |_| container::Style {
                background: Some(color.into()),
                border: Border {
                    radius: radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Bouton carré 42 × 42 avec une icône (recherche…).
    pub fn icon_button<'a, M: Clone + 'a>(self, which: Icon, msg: M) -> Element<'a, M> {
        button(container(icon(which, ON_HERO, 18.0)).center(Fill))
            .on_press(msg)
            .width(42)
            .height(42)
            .padding(0)
            .style(move |_, status| button::Style {
                background: Some(
                    Color {
                        a: if status == button::Status::Hovered {
                            0.16
                        } else {
                            0.1
                        },
                        ..Color::WHITE
                    }
                    .into(),
                ),
                border: Border {
                    radius: self.r(12.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Bouton principal (« Jouer ») : accent, hauteur 52, texte 16/800, ombre dure en pixel.
    pub fn primary<'a, M: Clone + 'a>(
        self,
        label: String,
        with_play: bool,
        msg: Option<M>,
    ) -> Element<'a, M> {
        let mut content = row![].spacing(10).align_y(iced::Center);
        if with_play {
            content = content.push(icon(Icon::Play, theme::BG, 18.0));
        }
        content = content.push(self.text(label, 16, Weight::ExtraBold, theme::BG));
        let accent = self.accent;
        let pixel = self.mode.is_pixel();
        button(container(content).height(Fill).align_y(iced::Center))
            .height(52)
            .padding([0, 34])
            .on_press_maybe(msg)
            .style(move |_, status| button::Style {
                background: Some(
                    match status {
                        button::Status::Hovered => lighten(accent, 0.08),
                        button::Status::Disabled => theme::CARD2,
                        _ => accent,
                    }
                    .into(),
                ),
                text_color: theme::BG,
                border: Border {
                    radius: self.r(14.0).into(),
                    ..Default::default()
                },
                shadow: if pixel {
                    Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 0.0,
                    }
                } else {
                    Shadow::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Bouton secondaire : contour, texte clair.
    pub fn secondary<'a, M: Clone + 'a>(self, label: String, msg: M) -> Element<'a, M> {
        button(
            container(self.text(label, 15, Weight::Bold, theme::TEXT))
                .height(Fill)
                .align_y(iced::Center),
        )
        .on_press(msg)
        .height(52)
        .padding([0, 24])
        .style(move |_, status| button::Style {
            background: (status == button::Status::Hovered).then(|| theme::CARD2.into()),
            text_color: theme::TEXT,
            border: Border {
                radius: self.r(12.0).into(),
                width: 1.0,
                color: theme::MUTED,
            },
            ..Default::default()
        })
        .into()
    }

    /// Bouton texte seul (« Annuler »).
    pub fn ghost<'a, M: Clone + 'a>(self, label: &'a str, msg: M) -> Element<'a, M> {
        button(
            container(self.text(label, 15, Weight::Bold, theme::MUTED))
                .height(Fill)
                .align_y(iced::Center),
        )
        .on_press(msg)
        .height(52)
        .padding([0, 20])
        .style(|_, _| button::Style::default())
        .into()
    }

    /// Étiquette de boutique sur fond clair (« STEAM »).
    pub fn tag<'a, M: 'a>(self, label: &'a str) -> Element<'a, M> {
        container(self.text(label, 12, Weight::Semibold, theme::BG))
            .padding([5, 9])
            .style(move |_| self.panel(theme::TEXT, 6.0, None))
            .into()
    }

    /// Bloc de statistique de la bannière (« TEMPS DE JEU / 42 h 10 »).
    pub fn stat<'a, M: 'a>(self, label: &'a str, value: String) -> Element<'a, M> {
        container(
            column![
                self.text(label.to_uppercase(), 11, Weight::Normal, STAT_LABEL),
                self.mono(value, 16, theme::TEXT)
            ]
            .spacing(4),
        )
        .padding([10, 16])
        .width(Length::Shrink)
        .style(move |_| self.glass(0.6, 12.0))
        .into()
    }

    /// Titre de section en petites capitales grises (« RÉCENTS »).
    pub fn section<'a>(self, label: String) -> text::Text<'a> {
        self.text(label.to_uppercase(), 11, Weight::Semibold, theme::MUTED)
    }

    /// Jaquette d'un jeu : image de la boutique si elle existe, sinon dégradé à 150°.
    /// En mode pixel : bordure sombre de 3 px et trame en damier.
    pub fn cover<'a, M: 'a>(
        self,
        g: &'a InstalledGame,
        w: f32,
        h: f32,
        radius: f32,
        with_title: bool,
    ) -> Element<'a, M> {
        let (c1, c2) = theme::cover_colors(&g.title);
        let path = if with_title {
            g.covers.hero.as_deref()
        } else {
            g.covers.portrait.as_deref()
        };
        let title = with_title.then_some(g.title.as_str());
        self.art(c1, c2, path, title, w, h, radius)
    }

    /// Visuel générique (jaquette, vignette) : dégradé, image facultative, titre facultatif.
    #[allow(clippy::too_many_arguments)]
    pub fn art<'a, M: 'a>(
        self,
        c1: Color,
        c2: Color,
        image_path: Option<&'a str>,
        title: Option<&'a str>,
        w: f32,
        h: f32,
        radius: f32,
    ) -> Element<'a, M> {
        let pixel = self.mode.is_pixel();
        let r = self.r(radius);
        let width = if w.is_finite() {
            Length::Fixed(w)
        } else {
            Fill
        };
        let mut layers =
            stack![container(space())
                .width(width)
                .height(h)
                .style(move |_| container::Style {
                    background: Some(linear(150.0, &[(0.0, c1), (1.0, c2)])),
                    border: Border {
                        radius: r.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })];
        if let Some(p) = image_path {
            // Les images gardent des coins arrondis classiques (le shader d'image n'a pas
            // l'escalier) : en pixel elles sont carrées, sous la trame.
            let img_r = if pixel { 0.0 } else { r };
            layers = layers.push(
                container(
                    image(image::Handle::from_path(p))
                        .content_fit(ContentFit::Cover)
                        .border_radius(Radius::from(img_r)),
                )
                .width(width)
                .height(h),
            );
        }
        if pixel {
            let steps = match self.mode {
                Mode::PixelArrondi => ((r - theme::STEP_MARK) / 4.0).max(0.0),
                _ => 0.0,
            };
            layers =
                layers
                    .push(canvas::Canvas::new(Dither { steps }).width(width).height(h))
                    .push(container(space()).width(width).height(h).style(move |_| {
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
        if let Some(t) = title {
            layers = layers.push(
                container(self.text(t, 15, Weight::Bold, theme::TEXT))
                    .width(width)
                    .height(h)
                    .padding([10, 12])
                    .align_bottom(Fill),
            );
        }
        container(layers).width(width).height(h).into()
    }
}

pub fn bg(color: Color) -> container::Style {
    container::Style {
        background: Some(color.into()),
        ..Default::default()
    }
}

/// Dégradé linéaire à la manière de CSS (`linear-gradient(<deg>, …)`).
pub fn linear(deg: f32, stops: &[(f32, Color)]) -> Background {
    let mut l = Linear::new(Degrees(deg));
    for (offset, color) in stops {
        l = l.add_stop(*offset, *color);
    }
    Background::Gradient(Gradient::Linear(l))
}

pub fn lighten(c: Color, by: f32) -> Color {
    Color {
        r: (c.r + by).min(1.0),
        g: (c.g + by).min(1.0),
        b: (c.b + by).min(1.0),
        a: c.a,
    }
}

/// Assombrit une couleur (mélange vers le noir).
pub fn darken(c: Color, by: f32) -> Color {
    Color {
        r: c.r * (1.0 - by),
        g: c.g * (1.0 - by),
        b: c.b * (1.0 - by),
        a: c.a,
    }
}

/// Icônes des maquettes (tracés SVG 24 × 24 refaits à la main).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Icon {
    Play,
    Search,
    Settings,
    Check,
    External,
}

pub fn icon<'a, M: 'a>(which: Icon, color: Color, size: f32) -> Element<'a, M> {
    canvas::Canvas::new(IconProgram { which, color })
        .width(size)
        .height(size)
        .into()
}

struct IconProgram {
    which: Icon,
    color: Color,
}

impl<M> canvas::Program<M> for IconProgram {
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
        let k = bounds.width.min(bounds.height) / 24.0;
        let p = |x: f32, y: f32| Point::new(x * k, y * k);
        let stroke = Stroke::default()
            .with_width(2.0 * k)
            .with_color(self.color)
            .with_line_cap(canvas::LineCap::Round)
            .with_line_join(canvas::LineJoin::Round);
        match self.which {
            Icon::Play => {
                let tri = Path::new(|b| {
                    b.move_to(p(6.0, 4.0));
                    b.line_to(p(20.0, 12.0));
                    b.line_to(p(6.0, 20.0));
                    b.close();
                });
                f.fill(&tri, self.color);
            }
            Icon::Search => {
                f.stroke(&Path::circle(p(11.0, 11.0), 7.0 * k), stroke);
                f.stroke(&Path::line(p(20.0, 20.0), p(16.0, 16.0)), stroke);
            }
            Icon::Check => {
                let stroke = stroke.with_width(3.0 * k);
                f.stroke(
                    &Path::new(|b| {
                        b.move_to(p(5.0, 12.5));
                        b.line_to(p(10.0, 17.5));
                        b.line_to(p(19.0, 7.0));
                    }),
                    stroke,
                );
            }
            Icon::External => {
                f.stroke(
                    &Path::new(|b| {
                        b.move_to(p(14.0, 4.0));
                        b.line_to(p(20.0, 4.0));
                        b.line_to(p(20.0, 10.0));
                        b.move_to(p(20.0, 4.0));
                        b.line_to(p(11.0, 13.0));
                        b.move_to(p(18.0, 14.0));
                        b.line_to(p(18.0, 20.0));
                        b.line_to(p(4.0, 20.0));
                        b.line_to(p(4.0, 6.0));
                        b.line_to(p(10.0, 6.0));
                    }),
                    stroke,
                );
            }
            Icon::Settings => {
                f.stroke(&Path::circle(p(12.0, 12.0), 3.0 * k), stroke);
                let rays = [
                    ((12.0, 2.0), (12.0, 5.0)),
                    ((12.0, 19.0), (12.0, 22.0)),
                    ((2.0, 12.0), (5.0, 12.0)),
                    ((19.0, 12.0), (22.0, 12.0)),
                    ((5.0, 5.0), (7.0, 7.0)),
                    ((17.0, 17.0), (19.0, 19.0)),
                    ((19.0, 5.0), (17.0, 7.0)),
                    ((7.0, 17.0), (5.0, 19.0)),
                ];
                for ((x1, y1), (x2, y2)) in rays {
                    f.stroke(&Path::line(p(x1, y1), p(x2, y2)), stroke);
                }
            }
        }
        vec![f.into_geometry()]
    }
}

/// Trame des jaquettes en mode pixel : damier de carrés de 4 px à 22 % de noir
/// (maquettes : `conic-gradient(… 25%, transparent 0 50%, … 75%, transparent 0) 0 0/8px 8px`).
struct Dither {
    /// Marches des coins (pixel arrondi) : pas de trame dans les marches découpées.
    steps: f32,
}

impl<M> canvas::Program<M> for Dither {
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
        let (w, h) = (bounds.width, bounds.height);
        let cut = |x: f32, y: f32| {
            if self.steps <= 0.0 {
                return false;
            }
            let dx = x.min(w - x);
            let dy = y.min(h - y);
            (dx / 4.0).floor() + (dy / 4.0).floor() < self.steps
        };
        let path = Path::new(|b| {
            let mut y = 0.0;
            let mut row_i = 0u32;
            while y < h {
                let mut x = if row_i.is_multiple_of(2) { 4.0 } else { 0.0 };
                while x < w {
                    if !cut(x + 2.0, y + 2.0) {
                        b.rectangle(
                            Point::new(x, y),
                            Size::new(4.0_f32.min(w - x), 4.0_f32.min(h - y)),
                        );
                    }
                    x += 8.0;
                }
                y += 4.0;
                row_i += 1;
            }
        });
        f.fill(&path, Color::from_rgba(0.0, 0.0, 0.0, 0.22));
        vec![f.into_geometry()]
    }
}

pub fn playtime(min: Option<u64>) -> String {
    match min {
        Some(m) if m >= 60 => format!("{} h {:02}", m / 60, m % 60),
        Some(m) if m > 0 => format!("{m} min"),
        _ => "—".into(),
    }
}

pub fn short_playtime(min: Option<u64>) -> String {
    match min {
        Some(m) if m > 0 => crate::texts::hours_short((m + 30) / 60),
        _ => crate::texts::NEVER_PLAYED().into(),
    }
}

pub fn size(bytes: Option<u64>) -> String {
    match bytes {
        Some(b) if b >= 1_000_000_000 => crate::texts::size_gb(b as f64 / 1e9),
        Some(b) if b > 0 => crate::texts::size_mb(b / 1_000_000),
        _ => "—".into(),
    }
}

/// « il y a 2 h », « il y a 3 j »… à partir d'un horodatage Unix.
pub fn ago(unix: u64, now: u64) -> String {
    let s = now.saturating_sub(unix);
    match s {
        0..=3599 => crate::texts::ago((s / 60).max(1), "min"),
        3600..=86_399 => crate::texts::ago(s / 3600, "h"),
        86_400..=2_591_999 => crate::texts::ago(s / 86_400, "j"),
        _ => crate::texts::ago(s / 2_592_000, "mois"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_relative_time() {
        assert_eq!(ago(1000, 1000), "il y a 1 min");
        assert_eq!(ago(0, 7200), "il y a 2 h");
        assert_eq!(ago(0, 3 * 86_400), "il y a 3 j");
    }
}
