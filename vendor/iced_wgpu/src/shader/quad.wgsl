struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}

// --- Ajout launcher : coins « en escalier » du mode pixel arrondi -----------------
// Le launcher code un rayon >= STEP_MARK : rayon = STEP_MARK + 4 n, n = nombre de marches
// de 4 px (maquettes *-PixelDoux : petit 1, moyen 2, grand 3). Le vertex shader le
// transmet sous la forme vec4(-n, taille d'une marche en pixels physiques, 0, 0).
const STEP_MARK: f32 = 1000.0;

fn step_encode(radius: vec4<f32>, scale: f32) -> vec4<f32> {
    return vec4<f32>(-(radius.x - STEP_MARK) / 4.0, 4.0 * scale, 0.0, 0.0);
}

// Distances du point aux bords les plus proches, sur chaque axe.
fn step_edges(local: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    return min(local, size - local);
}

// Le point tombe-t-il dans une marche découpée d'un coin ?
fn step_cut(d: vec2<f32>, steps: f32, cell: f32) -> bool {
    if (d.x < 0.0 || d.y < 0.0) {
        return false;
    }
    return floor(d.x / cell) + floor(d.y / cell) < steps;
}

// Couverture (0..1) d'un rectangle à coins en escalier.
fn step_alpha(local: vec2<f32>, size: vec2<f32>, steps: f32, cell: f32) -> f32 {
    let d = step_edges(local, size);
    if (step_cut(d, steps, cell)) {
        return 0.0;
    }
    return clamp(0.5 + min(d.x, d.y), 0.0, 1.0);
}

// Le point est-il dans la bordure (épaisseur bw) qui suit l'escalier ?
fn step_in_border(local: vec2<f32>, size: vec2<f32>, steps: f32, cell: f32, bw: f32) -> bool {
    let d = step_edges(local, size);
    return min(d.x, d.y) < bw
        || step_cut(d - vec2<f32>(bw, 0.0), steps, cell)
        || step_cut(d - vec2<f32>(0.0, bw), steps, cell)
        || step_cut(d - vec2<f32>(bw, bw), steps, cell);
}
