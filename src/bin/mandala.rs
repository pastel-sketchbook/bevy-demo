//! GPU fractal mandala — Julia set + kaleidoscopic folding rendered as 8
//! transparent layers composited across screen space.
//!
//! Mathematical mapping: **C × R³ × Z → R⁴**
//!   - C  = complex Julia parameter (per layer)
//!   - R³ = screen position (x, y) + per-layer rotation
//!   - Z  = integer layer index (0–7)
//!   - R⁴ = RGBA output per pixel
//!
//! Each layer is a full-screen quad with its own `Material2d` instance
//! containing a Julia set parameter, hue offset, alpha, rotation speed,
//! and zoom. A WGSL fragment shader performs kaleidoscopic folding followed
//! by Julia iteration with smooth escape-time pastel colouring.
//!
//! Controls:
//!   Up/Down     — increase / decrease fold symmetry (4–32)
//!   Left/Right  — zoom in / out
//!   Space       — toggle animation
//!   Q           — quit

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::{Shader, ShaderRef};
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d};
use bevy_demo::*;

/// Shader handle registered via `load_internal_asset!` so the binary is
/// self-contained and does not depend on an external `assets/` directory.
const SHADER_HANDLE: Handle<Shader> = uuid_handle!("e7a3d4b1-9c2f-4a6e-b8d5-1f3c7e9a0b2d");

// --- Constants ---------------------------------------------------------------

const BACKGROUND_COLOR: Color = background_color(0.02, 0.01, 0.04, 1.0);

const DEFAULT_FOLDS: f32 = 12.0;
const MIN_FOLDS: f32 = 4.0;
const MAX_FOLDS: f32 = 32.0;
const DEFAULT_ZOOM: f32 = 1.0;
const ZOOM_SPEED: f32 = 0.02;
const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 5.0;

/// Number of transparent layers stacked on screen.
const NUM_LAYERS: usize = 8;

// --- Per-layer preset table --------------------------------------------------

/// (base Julia c, hue_offset, alpha, rotation_speed_multiplier, zoom_offset)
const LAYER_PRESETS: [(Vec2, f32, f32, f32, f32); NUM_LAYERS] = [
    (Vec2::new(-0.400, 0.600), 0.000, 0.55, 1.00, 0.00), // hero
    (Vec2::new(0.285, 0.010), 0.125, 0.25, -0.70, 0.15),
    (Vec2::new(-0.800, 0.156), 0.250, 0.45, 0.50, -0.10), // hero
    (Vec2::new(-0.702, -0.384), 0.375, 0.20, -0.40, 0.20),
    (Vec2::new(0.355, 0.355), 0.500, 0.20, 0.30, -0.15),
    (Vec2::new(-0.100, 0.651), 0.625, 0.30, -0.25, 0.12),
    (Vec2::new(-0.750, 0.110), 0.750, 0.15, 0.20, -0.20),
    (Vec2::new(0.000, -0.800), 0.875, 0.15, -0.15, 0.08),
];

// --- GPU uniform (matches WGSL struct) ---------------------------------------

#[derive(Clone, Copy, ShaderType)]
struct MandalaParams {
    c: Vec2,
    folds: f32,
    time: f32,
    hue_offset: f32,
    alpha: f32,
    zoom: f32,
    rotation: f32,
    layer_depth: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

// --- Material2d implementation -----------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct MandalaMaterial {
    #[uniform(0)]
    params: MandalaParams,
}

impl Material2d for MandalaMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// --- Resources ---------------------------------------------------------------

#[derive(Resource)]
struct MandalaConfig {
    folds: f32,
    zoom: f32,
    animating: bool,
}

impl Default for MandalaConfig {
    fn default() -> Self {
        Self {
            folds: DEFAULT_FOLDS,
            zoom: DEFAULT_ZOOM,
            animating: true,
        }
    }
}

/// Accumulates time only while animation is active so pausing truly freezes.
#[derive(Resource, Default)]
struct AnimationTime {
    elapsed: f32,
}

/// Keeps handles to each layer's material so we can update uniforms each frame.
#[derive(Resource)]
struct LayerHandles(Vec<Handle<MandalaMaterial>>);

// --- Main --------------------------------------------------------------------

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }),
        Material2dPlugin::<MandalaMaterial>::default(),
    ));

    // Embed the WGSL shader at compile time so the binary is self-contained.
    load_internal_asset!(
        app,
        SHADER_HANDLE,
        "../../assets/shaders/mandala.wgsl",
        Shader::from_wgsl
    );

    app.insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_resource::<MandalaConfig>()
        .init_resource::<AnimationTime>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                handle_input,
                update_materials,
                handle_quit,
            ),
        )
        .run();
}

// --- Setup -------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MandalaMaterial>>,
) {
    commands.spawn(Camera2d);

    // Full-screen rectangle mesh (Bevy 2D coordinates)
    let mesh_handle = meshes.add(Rectangle::new(WINDOW_WIDTH, WINDOW_HEIGHT));

    let mut handles = Vec::with_capacity(NUM_LAYERS);

    for (i, &(c, hue_offset, alpha, _rot_speed, _zoom_off)) in LAYER_PRESETS.iter().enumerate() {
        let mat = MandalaMaterial {
            params: MandalaParams {
                c,
                folds: DEFAULT_FOLDS,
                time: 0.0,
                hue_offset,
                alpha,
                zoom: DEFAULT_ZOOM,
                rotation: 0.0,
                layer_depth: i as f32 / (NUM_LAYERS - 1) as f32,
                _pad1: 0.0,
                _pad2: 0.0,
                _pad3: 0.0,
            },
        };

        let mat_handle = materials.add(mat);
        handles.push(mat_handle.clone());

        // Stack layers at increasing z depths so they composite back-to-front.
        commands.spawn((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(mat_handle),
            Transform::from_xyz(0.0, 0.0, i as f32),
        ));
    }

    commands.insert_resource(LayerHandles(handles));
}

// --- Input -------------------------------------------------------------------

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut config: ResMut<MandalaConfig>) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        config.folds = (config.folds + 1.0).min(MAX_FOLDS);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        config.folds = (config.folds - 1.0).max(MIN_FOLDS);
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        config.zoom = (config.zoom + ZOOM_SPEED).min(MAX_ZOOM);
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        config.zoom = (config.zoom - ZOOM_SPEED).max(MIN_ZOOM);
    }
    if keyboard.just_pressed(KeyCode::Space) {
        config.animating = !config.animating;
    }
}

// --- Per-frame material update -----------------------------------------------

fn update_materials(
    config: Res<MandalaConfig>,
    time: Res<Time>,
    mut anim_time: ResMut<AnimationTime>,
    layer_handles: Res<LayerHandles>,
    mut materials: ResMut<Assets<MandalaMaterial>>,
) {
    if config.animating {
        anim_time.elapsed += time.delta_secs();
    }
    let t = anim_time.elapsed;

    for (i, handle) in layer_handles.0.iter().enumerate() {
        let Some(mat) = materials.get_mut(handle) else {
            continue;
        };

        let (c, hue_offset, alpha, rot_speed, zoom_offset) = (
            LAYER_PRESETS[i].0,
            LAYER_PRESETS[i].1,
            LAYER_PRESETS[i].2,
            LAYER_PRESETS[i].3,
            LAYER_PRESETS[i].4,
        );

        mat.params.c = c;
        mat.params.folds = config.folds;
        mat.params.time = t;
        mat.params.hue_offset = hue_offset;
        mat.params.alpha = alpha;
        mat.params.zoom = config.zoom + zoom_offset;
        mat.params.rotation = t * rot_speed * 0.3;
        mat.params.layer_depth = i as f32 / (NUM_LAYERS - 1) as f32;
    }
}
