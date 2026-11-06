//! Couleurs du thème sombre (PROMPT §11), accents et modes d'affichage.

use iced::font::Weight;
use iced::{Color, Font};

const fn hex(v: u32) -> Color {
    Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub const BG: Color = hex(0x0C0E14);
pub const SIDE: Color = hex(0x10131B);
pub const CARD: Color = hex(0x131722);
pub const CARD2: Color = hex(0x1B2030);
pub const FIELD: Color = hex(0x171B27);
pub const LINE: Color = hex(0x232A36);
pub const LINE2: Color = hex(0x2B3246);
pub const TEXT: Color = hex(0xF2F4F8);
pub const MUTED: Color = hex(0xA9B0C0);
pub const SOFT: Color = hex(0xD5DAE6);

/// Dégradés de remplacement tant qu'il n'y a pas de jaquette.
pub const COVERS: [(Color, Color); 8] = [
    (hex(0xE4572E), hex(0x4A1148)),
    (hex(0x1D4E89), hex(0x0B132B)),
    (hex(0xE76F51), hex(0x264653)),
    (hex(0x588157), hex(0x1B2E1F)),
    (hex(0x8338EC), hex(0x240046)),
    (hex(0xC77D2E), hex(0x3D2208)),
    (hex(0x2A9D8F), hex(0x10333B)),
    (hex(0xBC4749), hex(0x3A0F12)),
];

pub fn cover_colors(title: &str) -> (Color, Color) {
    let h = title
        .chars()
        .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32));
    COVERS[(h as usize) % COVERS.len()]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Orange,
    Menthe,
    CitronVert,
    Rose,
    Bleu,
}

impl Accent {
    pub const ALL: [Accent; 5] = [
        Accent::Orange,
        Accent::Menthe,
        Accent::CitronVert,
        Accent::Rose,
        Accent::Bleu,
    ];

    pub fn color(self) -> Color {
        match self {
            Accent::Orange => hex(0xFF8A1F),
            Accent::Menthe => hex(0x5EEAD4),
            Accent::CitronVert => hex(0xA3E635),
            Accent::Rose => hex(0xF472B6),
            Accent::Bleu => hex(0x93C5FD),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Accent::Orange => crate::texts::setting("accent.orange"),
            Accent::Menthe => crate::texts::setting("accent.menthe"),
            Accent::CitronVert => crate::texts::setting("accent.citron"),
            Accent::Rose => crate::texts::setting("accent.rose"),
            Accent::Bleu => crate::texts::setting("accent.bleu"),
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Accent::Orange => "orange",
            Accent::Menthe => "menthe",
            Accent::CitronVert => "citron",
            Accent::Rose => "rose",
            Accent::Bleu => "ciel",
        }
    }
    pub fn from_key(k: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|a| a.key() == k)
            .unwrap_or(Accent::Orange)
    }
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

/// Un rayon >= STEP_MARK est lu par le shader comme « coins en escalier »
/// (rayon = STEP_MARK + 4 × nombre de marches). Voir `vendor/iced_wgpu/src/shader/quad.wgsl`.
pub const STEP_MARK: f32 = 1000.0;

/// Normal, pixel à coins carrés, pixel à coins arrondis en escalier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    PixelCarre,
    PixelArrondi,
}

impl Mode {
    pub fn next(self) -> Self {
        match self {
            Mode::Normal => Mode::PixelCarre,
            Mode::PixelCarre => Mode::PixelArrondi,
            Mode::PixelArrondi => Mode::Normal,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Mode::Normal => crate::texts::setting("mode.normal"),
            Mode::PixelCarre => crate::texts::setting("mode.pixel_carre"),
            Mode::PixelArrondi => crate::texts::setting("mode.pixel_arrondi"),
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Mode::Normal => "normal",
            Mode::PixelCarre => "pixel-carre",
            Mode::PixelArrondi => "pixel-arrondi",
        }
    }
    pub fn from_key(k: &str) -> Self {
        [Mode::Normal, Mode::PixelCarre, Mode::PixelArrondi]
            .into_iter()
            .find(|m| m.key() == k)
            .unwrap_or(Mode::Normal)
    }
    pub fn is_pixel(self) -> bool {
        self != Mode::Normal
    }
    /// Taille d'un « gros pixel » pour la passe basse définition (`vendor/iced_wgpu`).
    /// Les maquettes pixel ont un texte net à pleine définition : la passe reste donc à 1
    /// (désactivée). `LAUNCHER_PIXEL_FACTOR=2` la réactive pour comparer.
    pub fn pixel_factor(self) -> u32 {
        if !self.is_pixel() {
            return 1;
        }
        std::env::var("LAUNCHER_PIXEL_FACTOR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    }
    /// Rayon d'un élément. En pixel arrondi : marches de 4 px (petit 1, moyen 2, grand 3),
    /// dessinées en escalier par le shader des rectangles (`vendor/iced_wgpu`, `STEP_MARK`).
    pub fn radius(self, normal: f32) -> f32 {
        match self {
            Mode::Normal => normal,
            Mode::PixelCarre => 0.0,
            Mode::PixelArrondi => {
                // Correspondance des maquettes PixelDoux : rayon ≤ 9 → 1 marche (onglets,
                // vignettes), ≤ 16 → 2 (boutons, cartes, pastilles), au-delà → 3.
                let steps = if normal <= 0.0 {
                    return 0.0;
                } else if normal <= 9.5 {
                    1.0
                } else if normal <= 16.0 {
                    2.0
                } else {
                    3.0
                };
                STEP_MARK + steps * 4.0
            }
        }
    }
    pub fn font(self, weight: iced::font::Weight) -> Font {
        if self.is_pixel() {
            // Graisses disponibles : 400, 600, 700 (800 devient 700).
            let weight = match weight {
                Weight::ExtraBold | Weight::Black | Weight::Bold => Weight::Bold,
                Weight::Semibold | Weight::Medium => Weight::Semibold,
                _ => Weight::Normal,
            };
            Font {
                weight,
                ..Font::with_name("Pixelify Sans")
            }
        } else {
            Font {
                weight,
                ..Font::with_name("Sora")
            }
        }
    }
    pub fn mono(self) -> Font {
        if self.is_pixel() {
            Font::with_name("Pixelify Sans")
        } else {
            // Seule graisse embarquée : 500 (maquettes : IBM Plex Mono 500).
            Font {
                weight: Weight::Medium,
                ..Font::with_name("IBM Plex Mono")
            }
        }
    }
}

/// Pixelify Sans est livrée en police variable ; on embarque trois graisses figées
/// (400, 600, 700) tirées de cette police, que le moteur de texte sait choisir.
pub const FONTS: [&[u8]; 8] = [
    include_bytes!("../assets/fonts/Sora-Regular.ttf"),
    include_bytes!("../assets/fonts/Sora-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Sora-Bold.ttf"),
    include_bytes!("../assets/fonts/Sora-ExtraBold.ttf"),
    include_bytes!("../assets/fonts/PixelifySans-Regular.ttf"),
    include_bytes!("../assets/fonts/PixelifySans-SemiBold.ttf"),
    include_bytes!("../assets/fonts/PixelifySans-Bold.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf"),
];
