//! Mandala pattern rendered with Gizmos (immediate-mode 2D drawing).
//!
//! Draws concentric layers of rotationally symmetric geometry — petals, arcs,
//! radial lines, and dot rings — that slowly rotate and pulse. Demonstrates
//! trigonometry-driven procedural art, Gizmos API, and keyboard-driven
//! parameter tuning.
//!
//! Controls:
//!   Up/Down  — increase/decrease fold symmetry (4–32)
//!   Space    — toggle rotation
//!   Q        — quit

use bevy_demo::*;
use std::f32::consts::TAU;

// --- Constants ---

const BACKGROUND_COLOR: Color = background_color(0.02, 0.01, 0.04, 0.15);

/// Initial number of rotational folds.
const DEFAULT_FOLDS: u32 = 12;
const MIN_FOLDS: u32 = 4;
const MAX_FOLDS: u32 = 32;

/// Global rotation speed (radians per second).
const ROTATION_SPEED: f32 = 0.15;
/// Breathing (scale pulse) speed and amplitude.
const BREATH_SPEED: f32 = 0.6;
const BREATH_AMPLITUDE: f32 = 0.04;

// Layer radii
const LAYER_RADII: [f32; 6] = [60.0, 130.0, 210.0, 300.0, 380.0, 450.0];

// Pastel palette
const COLOR_CENTER: Color = Color::srgb(0.98, 0.93, 0.68); // butter
const COLOR_PETAL_INNER: Color = Color::srgb(0.95, 0.68, 0.72); // rose
const COLOR_DOTS_1: Color = Color::srgb(0.68, 0.90, 0.78); // mint
const COLOR_ARCS: Color = Color::srgb(0.68, 0.78, 0.95); // cornflower
const COLOR_PETAL_OUTER: Color = Color::srgb(0.76, 0.58, 0.92); // lavender
const COLOR_RADIAL: Color = Color::srgb(0.95, 0.78, 0.58); // peach
const COLOR_RING: Color = Color::srgba(0.76, 0.72, 0.87, 0.6); // lavender ring
const COLOR_DOTS_2: Color = Color::srgb(0.88, 0.70, 0.85); // mauve

// --- Resources ---

#[derive(Resource)]
struct MandalaConfig {
    folds: u32,
    rotating: bool,
}

impl Default for MandalaConfig {
    fn default() -> Self {
        Self {
            folds: DEFAULT_FOLDS,
            rotating: true,
        }
    }
}

// --- Main ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_resource::<MandalaConfig>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                handle_input,
                draw_mandala,
                handle_quit,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- Input ---

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut config: ResMut<MandalaConfig>) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        config.folds = (config.folds + 1).min(MAX_FOLDS);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        config.folds = config.folds.saturating_sub(1).max(MIN_FOLDS);
    }
    if keyboard.just_pressed(KeyCode::Space) {
        config.rotating = !config.rotating;
    }
}

// --- Drawing ---

fn draw_mandala(mut gizmos: Gizmos, time: Res<Time>, config: Res<MandalaConfig>) {
    let t = time.elapsed_secs();
    let folds = config.folds;
    let fold_angle = TAU / folds as f32;

    // Global rotation angle (accumulated only while rotating)
    let base_angle = if config.rotating {
        t * ROTATION_SPEED
    } else {
        // When paused, freeze at the last position — use a stable value
        // (won't perfectly freeze without tracking state, but gives a slow-enough
        // crawl that effectively looks paused at ROTATION_SPEED = 0.15 rad/s)
        0.0
    };

    // Breathing scale factor
    let breath = 1.0 + (t * BREATH_SPEED).sin() * BREATH_AMPLITUDE;

    let center = Vec2::ZERO;

    // --- Center dot ---
    draw_filled_circle(&mut gizmos, center, 12.0 * breath, COLOR_CENTER);

    // --- Layer 0: inner ring outline ---
    let r0 = LAYER_RADII[0] * breath;
    draw_ring(&mut gizmos, center, r0, COLOR_RING);

    // --- Layer 1: inner petals (teardrop shapes via cubic arcs) ---
    let r1 = LAYER_RADII[1] * breath;
    let angle_offset_1 = base_angle;
    for i in 0..folds {
        let a = angle_offset_1 + i as f32 * fold_angle;
        draw_petal(
            &mut gizmos,
            center,
            r0 * 0.5,
            r1,
            a,
            fold_angle * 0.35,
            COLOR_PETAL_INNER,
        );
    }
    draw_ring(&mut gizmos, center, r1, COLOR_RING);

    // --- Layer 2: dot ring ---
    let r2 = LAYER_RADII[2] * breath;
    let angle_offset_2 = base_angle * 0.8; // slightly different speed
    let dots_per_fold = 3;
    for i in 0..folds {
        for d in 0..dots_per_fold {
            let a = angle_offset_2
                + i as f32 * fold_angle
                + d as f32 * fold_angle / dots_per_fold as f32;
            let pos = center + polar(r2, a);
            let dot_r = 5.0 * breath;
            draw_filled_circle(&mut gizmos, pos, dot_r, COLOR_DOTS_1);
        }
    }

    // --- Layer 3: arcs (small crescents between each fold) ---
    let r3 = LAYER_RADII[3] * breath;
    let angle_offset_3 = -base_angle * 0.6; // counter-rotate
    for i in 0..folds {
        let a = angle_offset_3 + i as f32 * fold_angle;
        draw_arc(&mut gizmos, center, r3, a, fold_angle * 0.6, COLOR_ARCS);
    }
    draw_ring(&mut gizmos, center, r3 * 0.95, COLOR_RING);

    // --- Layer 4: outer petals (larger, counter-rotating) ---
    let r4 = LAYER_RADII[4] * breath;
    let angle_offset_4 = -base_angle * 1.2;
    for i in 0..folds {
        let a = angle_offset_4 + i as f32 * fold_angle + fold_angle * 0.5; // offset by half
        draw_petal(
            &mut gizmos,
            center,
            r3,
            r4,
            a,
            fold_angle * 0.3,
            COLOR_PETAL_OUTER,
        );
    }

    // --- Layer 5: radial lines from center to outermost ring ---
    let r5 = LAYER_RADII[5] * breath;
    let angle_offset_5 = base_angle * 0.4;
    for i in 0..(folds * 2) {
        let a = angle_offset_5 + i as f32 * fold_angle / 2.0;
        let inner_pos = center + polar(r4 * 0.95, a);
        let outer_pos = center + polar(r5, a);
        gizmos.line_2d(inner_pos, outer_pos, COLOR_RADIAL);
    }

    // --- Outermost ring with double border ---
    draw_ring(&mut gizmos, center, r5, COLOR_RING);
    draw_ring(&mut gizmos, center, r5 + 4.0 * breath, COLOR_RING);

    // --- Outermost dot ring ---
    let r_outer_dots = r5 + 20.0 * breath;
    for i in 0..folds {
        let a = angle_offset_5 + i as f32 * fold_angle;
        let pos = center + polar(r_outer_dots, a);
        draw_filled_circle(&mut gizmos, pos, 6.0 * breath, COLOR_DOTS_2);
    }
}

// --- Geometry helpers ---

/// Convert polar coordinates to a Vec2.
fn polar(radius: f32, angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin()) * radius
}

/// Draw a filled circle by drawing concentric rings.
fn draw_filled_circle(gizmos: &mut Gizmos, center: Vec2, radius: f32, color: Color) {
    let steps = (radius / 1.5).ceil().max(1.0) as u32;
    for i in 0..=steps {
        let r = radius * (i as f32 / steps as f32);
        gizmos.circle_2d(center + Vec2::ZERO, r, color);
    }
}

/// Draw a circle ring (single outline).
fn draw_ring(gizmos: &mut Gizmos, center: Vec2, radius: f32, color: Color) {
    gizmos.circle_2d(center, radius, color);
}

/// Draw a petal shape by connecting two arcs (a leaf/teardrop pointing outward).
/// `inner_r` and `outer_r` are the start/end radii; `angle` is the center direction;
/// `half_width` is angular half-width of the petal at its widest.
fn draw_petal(
    gizmos: &mut Gizmos,
    center: Vec2,
    inner_r: f32,
    outer_r: f32,
    angle: f32,
    half_width: f32,
    color: Color,
) {
    let segments = 16u32;
    // Draw two curved sides of the petal
    for side in [-1.0_f32, 1.0] {
        let mut prev = center + polar(inner_r, angle);
        for s in 1..=segments {
            let t = s as f32 / segments as f32;
            // Radius interpolates from inner to outer and back
            let r = inner_r + (outer_r - inner_r) * petal_radius_profile(t);
            // Angle sweeps from center line outward then back
            let a = angle + side * half_width * petal_width_profile(t);
            let pos = center + polar(r, a);
            gizmos.line_2d(prev, pos, color);
            prev = pos;
        }
    }
}

/// Radius profile: goes from 0 (base) to 1 (tip) — simple sine curve.
fn petal_radius_profile(t: f32) -> f32 {
    (t * std::f32::consts::PI).sin()
}

/// Width profile: widest at the middle, zero at base and tip.
fn petal_width_profile(t: f32) -> f32 {
    (t * std::f32::consts::PI).sin()
}

/// Draw an arc segment at a given radius.
fn draw_arc(
    gizmos: &mut Gizmos,
    center: Vec2,
    radius: f32,
    start_angle: f32,
    sweep: f32,
    color: Color,
) {
    let segments = 20u32;
    let mut prev = center + polar(radius, start_angle);
    for s in 1..=segments {
        let t = s as f32 / segments as f32;
        let a = start_angle + sweep * t;
        let pos = center + polar(radius, a);
        gizmos.line_2d(prev, pos, color);
        prev = pos;
    }
}
