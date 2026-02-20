//! HDR bloom demo with emissive neon shapes. Demonstrates the Bloom
//! post-processing component, Hdr marker, and emissive StandardMaterials with
//! intensity values greater than 1.0. Shapes pulse their emissive glow.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.01, 0.01, 0.02, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.01, 0.01, 0.02);

const LIGHT_INTENSITY: f32 = 500_000.0;
const PULSE_SPEED: f32 = 2.0;
const EMISSIVE_BASE: f32 = 5.0;
const EMISSIVE_AMPLITUDE: f32 = 8.0;
const ORBIT_SPEED: f32 = 0.3;
const CAMERA_DISTANCE: f32 = 12.0;
const CAMERA_HEIGHT: f32 = 5.0;

// --- Components ---

#[derive(Component)]
struct EmissiveShape {
    base_color: LinearRgba,
    phase: f32,
}

#[derive(Component)]
struct OrbitObject {
    radius: f32,
    speed: f32,
    phase: f32,
    height: f32,
}

// --- Main ---

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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                pulse_emissive,
                orbit_objects,
                handle_quit,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // HDR Camera with bloom
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Transform::from_xyz(0.0, CAMERA_HEIGHT, CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        Bloom {
            intensity: 0.3,
            ..default()
        },
    ));

    // Dim ambient light only
    commands.spawn((
        PointLight {
            intensity: LIGHT_INTENSITY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 6.0, 0.0),
    ));

    // Ground plane (semi-transparent silver surface)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.72, 0.73, 0.75, 0.5),
            alpha_mode: AlphaMode::Blend,
            metallic: 0.6,
            perceptual_roughness: 0.35,
            reflectance: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // Pastel-toned shapes with bloom glow
    let neon_shapes: Vec<(Mesh, LinearRgba, Vec3, f32, f32, f32, f32)> = vec![
        // (mesh, color, position, phase, orbit_radius, orbit_speed, height)
        (
            Mesh::from(Torus::new(0.2, 0.6)),
            LinearRgba::new(0.95, 0.55, 0.65, 1.0), // Pastel rose
            Vec3::new(0.0, 1.5, 0.0),
            0.0,
            3.0,
            0.5,
            1.5,
        ),
        (
            Mesh::from(Sphere::new(0.5)),
            LinearRgba::new(0.55, 0.72, 0.95, 1.0), // Pastel sky
            Vec3::new(2.0, 2.0, 0.0),
            1.0,
            4.0,
            0.3,
            2.0,
        ),
        (
            Mesh::from(Capsule3d::new(0.25, 0.8)),
            LinearRgba::new(0.55, 0.90, 0.65, 1.0), // Pastel mint
            Vec3::new(-2.0, 1.0, 2.0),
            2.0,
            3.5,
            0.4,
            1.0,
        ),
        (
            Mesh::from(Cuboid::new(0.8, 0.8, 0.8)),
            LinearRgba::new(0.98, 0.78, 0.58, 1.0), // Pastel peach
            Vec3::new(0.0, 1.5, -3.0),
            3.0,
            5.0,
            0.25,
            1.5,
        ),
        (
            Mesh::from(Cylinder::new(0.3, 1.0)),
            LinearRgba::new(0.76, 0.58, 0.92, 1.0), // Pastel lavender
            Vec3::new(3.0, 1.0, 3.0),
            4.0,
            2.5,
            0.6,
            1.0,
        ),
        (
            Mesh::from(Torus::new(0.15, 0.4)),
            LinearRgba::new(0.98, 0.93, 0.58, 1.0), // Pastel butter
            Vec3::new(-3.0, 2.5, -2.0),
            5.0,
            4.5,
            0.35,
            2.5,
        ),
    ];

    for (mesh, color, _position, phase, orbit_radius, orbit_speed, height) in neon_shapes {
        let emissive = color * EMISSIVE_BASE;
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.65, 0.65, 0.68),
            emissive,
            ..default()
        });

        // Compute initial position from orbit
        let x = orbit_radius * phase.cos();
        let z = orbit_radius * phase.sin();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_xyz(x, height, z),
            EmissiveShape {
                base_color: color,
                phase,
            },
            OrbitObject {
                radius: orbit_radius,
                speed: orbit_speed * ORBIT_SPEED,
                phase,
                height,
            },
        ));
    }
}

fn pulse_emissive(
    time: Res<Time>,
    query: Query<(&EmissiveShape, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for (shape, material_handle) in query.iter() {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        // Pulse between base and amplified emissive intensity
        let pulse = ((t * PULSE_SPEED + shape.phase).sin() + 1.0) / 2.0;
        let intensity = EMISSIVE_BASE + pulse * EMISSIVE_AMPLITUDE;
        material.emissive = shape.base_color * intensity;
    }
}

fn orbit_objects(time: Res<Time>, mut query: Query<(&mut Transform, &OrbitObject)>) {
    let t = time.elapsed_secs();
    for (mut transform, orbit) in query.iter_mut() {
        let angle = orbit.phase + t * orbit.speed;
        transform.translation.x = orbit.radius * angle.cos();
        transform.translation.z = orbit.radius * angle.sin();
        transform.translation.y = orbit.height;
        // Gentle bobbing
        transform.translation.y += (t * 0.8 + orbit.phase).sin() * 0.3;
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
