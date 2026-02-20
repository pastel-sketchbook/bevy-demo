//! 3D rotating cube with the letters "PASTEL" on each face.
//! Characters are rendered using an embedded 8x8 bitmap font drawn into textures.
//! The cube rotates randomly with periodic axis/speed changes.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{WindowPlugin, WindowPosition, WindowResolution},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.08, 0.06, 0.12, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.06, 0.12);

const RANDOM_SEED: u64 = 72849156348927651;
const CUBE_SIZE: f32 = 2.0;
const TEXTURE_SIZE: u32 = 256;
const ROTATION_CHANGE_SECS: f32 = 5.0;
const ROTATION_MIN_SPEED: f32 = 0.3;
const ROTATION_MAX_SPEED: f32 = 1.5;
// P=15, A=0, S=18, T=19, E=4, L=11
const FACE_LETTERS: [usize; 6] = [15, 0, 18, 19, 4, 11];
const LIGHT_INTENSITY: f32 = 15_000_000.0;
const CAMERA_X: f32 = -3.0;
const CAMERA_Y: f32 = 3.0;
const CAMERA_Z: f32 = 5.0;

// --- 8x8 Bitmap Font for A-Z ---
// Each letter is 8 rows, each row is a u8 bitmask (MSB = leftmost pixel).
const FONT: [[u8; 8]; 26] = [
    [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00], // A
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00], // B
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00], // C
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00], // D
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00], // E
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00], // F
    [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3E, 0x00], // G
    [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00], // H
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // I
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00], // J
    [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00], // K
    [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00], // L
    [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00], // M
    [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00], // N
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // O
    [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00], // P
    [0x3C, 0x66, 0x66, 0x66, 0x6E, 0x6C, 0x36, 0x00], // Q
    [0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x00], // R
    [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00], // S
    [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // T
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // U
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // V
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00], // W
    [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00], // X
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00], // Y
    [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00], // Z
];

// --- Pastel background colors for each face ---
const FACE_BG_COLORS: [[u8; 3]; 6] = [
    [180, 200, 220], // soft blue
    [210, 185, 200], // soft mauve
    [190, 215, 190], // soft sage
    [220, 200, 180], // soft sand
    [200, 190, 215], // soft lavender
    [215, 210, 190], // soft cream
];

// --- Components ---

#[derive(Component)]
struct CubeRoot;

#[derive(Component)]
#[allow(dead_code)]
struct CubeFace(usize);

#[derive(Component)]
struct RotationState {
    axis: Vec3,
    speed: f32,
}

// --- Resources ---

#[derive(Resource)]
struct RandomSource(SmallRng);

#[derive(Resource)]
struct RotationTimer(Timer);

// --- Helper Functions ---

/// Generate a random pastel color (high lightness, moderate saturation).
fn random_pastel(rng: &mut SmallRng) -> [u8; 3] {
    let hue: f32 = rng.random_range(0.0..360.0);
    let saturation: f32 = rng.random_range(0.4..0.7);
    let lightness: f32 = rng.random_range(0.55..0.75);
    hsl_to_rgb(hue, saturation, lightness)
}

/// Convert HSL to RGB (u8 triplet).
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    [
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    ]
}

/// Draw a single character into an image at a given pixel position.
/// The character is upscaled from 8x8 to `scale`x`scale` pixels per font pixel.
fn draw_char(data: &mut [u8], ch: usize, offset_x: u32, offset_y: u32, scale: u32, fg: [u8; 3]) {
    let bitmap = &FONT[ch];
    let width = TEXTURE_SIZE;
    for row in 0..8u32 {
        let bits = bitmap[row as usize];
        for col in 0..8u32 {
            let is_set = (bits >> (7 - col)) & 1 == 1;
            if is_set {
                // Fill the scaled block
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = offset_x + col * scale + sx;
                        let py = offset_y + row * scale + sy;
                        if px < TEXTURE_SIZE && py < TEXTURE_SIZE {
                            let idx = ((py * width + px) * 4) as usize;
                            data[idx] = fg[0];
                            data[idx + 1] = fg[1];
                            data[idx + 2] = fg[2];
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
}

/// Fill an image with a background color and draw a single large character centered.
fn render_face(image: &mut Image, ch: usize, bg: [u8; 3], fg: [u8; 3]) {
    let data = image.data.as_mut().expect("Image has no CPU data");
    // Fill background
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let idx = ((y * TEXTURE_SIZE + x) * 4) as usize;
            data[idx] = bg[0];
            data[idx + 1] = bg[1];
            data[idx + 2] = bg[2];
            data[idx + 3] = 255;
        }
    }
    // Character scale: 8 font pixels -> fills most of the texture
    // With scale=28, char is 224x224 pixels, centered in 256x256
    let scale = (TEXTURE_SIZE / 8) - 4; // 28 for 256px texture
    let offset = (TEXTURE_SIZE - 8 * scale) / 2;
    draw_char(data, ch, offset, offset, scale, fg);
}

/// Create a new render-target image.
fn create_face_image() -> Image {
    let size = Extent3d {
        width: TEXTURE_SIZE,
        height: TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    Image::new_fill(
        size,
        TextureDimension::D2,
        &[128, 128, 128, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

/// Generate a random normalized axis.
fn random_axis(rng: &mut SmallRng) -> Vec3 {
    Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    )
    .normalize_or(Vec3::Y)
}

// --- Face geometry ---
// Returns (translation, rotation) for each cube face relative to center.
// Plane3d::default() lies in the XZ plane with normal facing +Y.
// Each rotation reorients from +Y normal to the desired face normal.
fn face_transforms() -> [(Vec3, Quat); 6] {
    let half = CUBE_SIZE / 2.0;
    [
        // Front (+Z): rotate +Y normal toward +Z
        (
            Vec3::new(0.0, 0.0, half),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
        // Back (-Z): rotate +Y normal toward -Z
        (
            Vec3::new(0.0, 0.0, -half),
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        ),
        // Right (+X): rotate +Y normal toward +X
        (
            Vec3::new(half, 0.0, 0.0),
            Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        ),
        // Left (-X): rotate +Y normal toward -X
        (
            Vec3::new(-half, 0.0, 0.0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        ),
        // Top (+Y): already facing +Y, no rotation needed
        (Vec3::new(0.0, half, 0.0), Quat::IDENTITY),
        // Bottom (-Y): flip +Y to -Y
        (
            Vec3::new(0.0, -half, 0.0),
            Quat::from_rotation_x(std::f32::consts::PI),
        ),
    ]
}

// --- Main ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                decorations: false,
                #[cfg(feature = "transparent")]
                transparent: true,
                #[cfg(feature = "transparent")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
        .insert_resource(RotationTimer(Timer::from_seconds(
            ROTATION_CHANGE_SECS,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                rotate_cube,
                randomize_rotation,
                handle_quit,
            ),
        )
        .run();
}

#[cfg(feature = "window-offset")]
fn offset_window(mut windows: Query<&mut Window>, mut done: Local<bool>) {
    if *done {
        return;
    }
    for mut window in windows.iter_mut() {
        window.position = WindowPosition::At(IVec2::new(160, 88));
        info!("Window positioned at: (160, 88)");
        *done = true;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut rng_res: ResMut<RandomSource>,
) {
    let rng = &mut rng_res.0;
    let face_mesh = meshes.add(Plane3d::default().mesh().size(CUBE_SIZE, CUBE_SIZE));
    let transforms = face_transforms();

    // Spawn the cube root entity
    let cube_root = commands
        .spawn((
            Transform::default(),
            Visibility::Visible,
            CubeRoot,
            RotationState {
                axis: random_axis(rng),
                speed: rng.random_range(ROTATION_MIN_SPEED..ROTATION_MAX_SPEED),
            },
        ))
        .id();

    // Spawn 6 face entities as children
    for i in 0..6 {
        let mut image = create_face_image();
        let letter = FACE_LETTERS[i];
        let fg = random_pastel(rng);
        let bg = FACE_BG_COLORS[i];
        render_face(&mut image, letter, bg, fg);

        let image_handle = images.add(image);

        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle),
            unlit: true,
            ..default()
        });

        let (translation, rotation) = transforms[i];
        let face_entity = commands
            .spawn((
                Transform::from_translation(translation).with_rotation(rotation),
                Visibility::Visible,
                Mesh3d(face_mesh.clone()),
                MeshMaterial3d(material),
                CubeFace(i),
            ))
            .id();

        commands.entity(cube_root).add_child(face_entity);
    }

    // Light
    commands.spawn((
        PointLight {
            intensity: LIGHT_INTENSITY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_X, CAMERA_Y, CAMERA_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_cube(
    mut query: Query<(&mut Transform, &RotationState), With<CubeRoot>>,
    time: Res<Time>,
) {
    for (mut transform, rotation) in query.iter_mut() {
        let angle = rotation.speed * time.delta_secs();
        transform.rotate(Quat::from_axis_angle(rotation.axis, angle));
    }
}

fn randomize_rotation(
    mut timer: ResMut<RotationTimer>,
    time: Res<Time>,
    mut query: Query<&mut RotationState, With<CubeRoot>>,
    mut rng_res: ResMut<RandomSource>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let rng = &mut rng_res.0;
    for mut rotation in query.iter_mut() {
        rotation.axis = random_axis(rng);
        rotation.speed = rng.random_range(ROTATION_MIN_SPEED..ROTATION_MAX_SPEED);
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
