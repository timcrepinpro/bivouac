//! Transitions entre pages (PROMPT §11). Elles sont dessinées par le shader d'écran
//! (`vendor/iced_wgpu/src/pixel.rs`) : fondu, glissement et zoom pour la famille
//! « Fluide », grille de 16 × 6 blocs pour la famille « Pixel ».
//! `order` et `covered` décrivent ici la règle des blocs que le shader applique :
//! elles servent de référence et de tests.

pub const COLS: usize = 16;
pub const ROWS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Fondu,
    Glissement,
    Zoom,
    Dissolution,
    Balayage,
    Rideau,
    Aucune,
}

impl Kind {
    pub const FLUID: [Kind; 3] = [Kind::Fondu, Kind::Glissement, Kind::Zoom];
    pub const PIXEL: [Kind; 3] = [Kind::Dissolution, Kind::Balayage, Kind::Rideau];
    const ALL: [Kind; 7] = [
        Kind::Fondu,
        Kind::Glissement,
        Kind::Zoom,
        Kind::Dissolution,
        Kind::Balayage,
        Kind::Rideau,
        Kind::Aucune,
    ];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|k| *k == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
    pub fn is_pixel(self) -> bool {
        Self::PIXEL.contains(&self)
    }
    /// Numéro de l'effet pour le shader d'écran (0 = aucun).
    pub fn shader_id(self) -> u32 {
        match self {
            Kind::Aucune => 0,
            Kind::Fondu => 1,
            Kind::Glissement => 2,
            Kind::Zoom => 3,
            Kind::Dissolution => 4,
            Kind::Balayage => 5,
            Kind::Rideau => 6,
        }
    }
    /// Mot enregistré dans `user.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Fondu => "fondu",
            Kind::Glissement => "glissement",
            Kind::Zoom => "zoom",
            Kind::Dissolution => "dissolution",
            Kind::Balayage => "balayage",
            Kind::Rideau => "rideau",
            Kind::Aucune => "aucune",
        }
    }
    pub fn from_key(k: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|x| x.key() == k)
            .unwrap_or(Kind::Fondu)
    }
    pub fn name(self) -> &'static str {
        use crate::texts::setting;
        match self {
            Kind::Fondu => setting("transition.fondu"),
            Kind::Glissement => setting("transition.glissement"),
            Kind::Zoom => setting("transition.zoom"),
            Kind::Dissolution => setting("transition.dissolution"),
            Kind::Balayage => setting("transition.balayage"),
            Kind::Rideau => setting("transition.rideau"),
            Kind::Aucune => setting("transition.aucune"),
        }
    }
}

/// Moment d'apparition d'un bloc, entre 0 et 1.
#[cfg_attr(not(test), allow(dead_code))]
pub fn order(kind: Kind, col: usize, row: usize) -> f32 {
    match kind {
        Kind::Balayage => col as f32 / COLS as f32,
        Kind::Rideau => row as f32 / ROWS as f32,
        _ => {
            // Aléatoire fixe : hachage entier du numéro de bloc.
            let mut x = (row * COLS + col) as u32 ^ 0x9E37_79B9;
            x ^= x >> 16;
            x = x.wrapping_mul(0x7FEB_352D);
            x ^= x >> 15;
            (x % 1000) as f32 / 1000.0
        }
    }
}

/// Le bloc est-il plein à l'avancement `p` (0 → 1) ? Il se remplit pendant la
/// première moitié et se vide pendant la seconde.
#[cfg_attr(not(test), allow(dead_code))]
pub fn covered(kind: Kind, col: usize, row: usize, p: f32) -> bool {
    let o = order(kind, col, row);
    if p < 0.5 {
        o < p * 2.0
    } else {
        o >= (p - 0.5) * 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_covered_at_middle_and_clear_at_ends() {
        for kind in [Kind::Dissolution, Kind::Balayage, Kind::Rideau] {
            let count = |p| {
                (0..ROWS)
                    .flat_map(|r| (0..COLS).map(move |c| (c, r)))
                    .filter(|&(c, r)| covered(kind, c, r, p))
                    .count()
            };
            assert_eq!(count(0.0), 0);
            assert_eq!(count(0.4999), COLS * ROWS - count_late(kind));
            assert_eq!(count(0.5), COLS * ROWS);
            assert_eq!(count(1.0), 0);
        }
    }

    // Blocs dont l'ordre est ≥ 0,9998 (aucun, sauf hasard) : juste pour garder le test exact.
    fn count_late(kind: Kind) -> usize {
        (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (c, r)))
            .filter(|&(c, r)| order(kind, c, r) >= 0.9998)
            .count()
    }

    #[test]
    fn sweep_goes_left_to_right() {
        assert!(covered(Kind::Balayage, 0, 3, 0.1));
        assert!(!covered(Kind::Balayage, 15, 3, 0.1));
    }
}
