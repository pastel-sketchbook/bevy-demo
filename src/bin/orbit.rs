//! 3D camera orbit controller demo. Left-drag to orbit around the scene,
//! scroll to zoom in/out. Several auto-rotating 3D primitives are displayed.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy_demo::*;

// --- Constants ---

const BACKGROUND_COLOR: Color = background_color(0.06, 0.06, 0.1, 0.3);

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_SENSITIVITY: f32 = 0.5;
const MIN_DISTANCE: f32 = 3.0;
const MAX_DISTANCE: f32 = 30.0;
const INITIAL_DISTANCE: f32 = 12.0;
const INITIAL_YAW: f32 = 0.5;
const INITIAL_PITCH: f32 = 0.4;
const AUTO_ROTATE_SPEED: f32 = 0.8;
const LIGHT_INTENSITY: f32 = 15_000_000.0;

// --- Components ---

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

#[derive(Component)]
struct AutoRotate {
    axis: Vec3,
    speed: f32,
}

// --- Main ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                orbit_camera,
                auto_rotate,
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
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, INITIAL_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            yaw: INITIAL_YAW,
            pitch: INITIAL_PITCH,
            distance: INITIAL_DISTANCE,
        },
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: LIGHT_INTENSITY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Ground plane (semi-transparent silver surface)
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.73, 0.75, 0.5),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.6,
        perceptual_roughness: 0.35,
        reflectance: 0.8,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    // Shapes with pastel colors and auto-rotation
    let shapes: Vec<(Mesh, Color, Vec3, Vec3, f32)> = vec![
        (
            Mesh::from(Torus::new(0.3, 0.8)),
            Color::srgb(0.95, 0.68, 0.72), // Pastel rose
            Vec3::new(-3.0, 1.0, 0.0),
            Vec3::Y,
            1.0,
        ),
        (
            Mesh::from(Capsule3d::new(0.4, 1.2)),
            Color::srgb(0.68, 0.90, 0.78), // Pastel mint
            Vec3::new(0.0, 1.0, -3.0),
            Vec3::new(1.0, 0.5, 0.0).normalize(),
            0.7,
        ),
        (
            Mesh::from(Cuboid::new(1.2, 1.2, 1.2)),
            Color::srgb(0.68, 0.78, 0.95), // Pastel cornflower
            Vec3::new(3.0, 1.0, 0.0),
            Vec3::new(0.5, 1.0, 0.3).normalize(),
            1.2,
        ),
        (
            Mesh::from(Sphere::new(0.7)),
            Color::srgb(0.98, 0.88, 0.65), // Pastel peach
            Vec3::new(0.0, 1.0, 3.0),
            Vec3::Y,
            0.5,
        ),
        (
            Mesh::from(Cylinder::new(0.5, 1.5)),
            Color::srgb(0.80, 0.68, 0.92), // Pastel lavender
            Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(0.3, 1.0, 0.7).normalize(),
            0.9,
        ),
    ];

    for (mesh, color, position, axis, speed) in shapes {
        let material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            AutoRotate {
                axis,
                speed: speed * AUTO_ROTATE_SPEED,
            },
        ));
    }
}

fn orbit_camera(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = query.single_mut() else {
        return;
    };

    // Orbit with left mouse drag
    if mouse_button.pressed(MouseButton::Left) {
        orbit.yaw -= mouse_motion.delta.x * ORBIT_SENSITIVITY;
        orbit.pitch -= mouse_motion.delta.y * ORBIT_SENSITIVITY;
        // Clamp pitch to avoid flipping
        orbit.pitch = orbit.pitch.clamp(-1.4, 1.4);
    }

    // Zoom with scroll wheel
    orbit.distance -= mouse_scroll.delta.y * ZOOM_SENSITIVITY;
    orbit.distance = orbit.distance.clamp(MIN_DISTANCE, MAX_DISTANCE);

    // Compute camera position from spherical coordinates
    let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.distance * orbit.pitch.sin();
    let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();

    transform.translation = Vec3::new(x, y.max(0.5), z);
    *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
}

fn auto_rotate(mut query: Query<(&mut Transform, &AutoRotate)>, time: Res<Time>) {
    for (mut transform, rotate) in query.iter_mut() {
        let angle = rotate.speed * time.delta_secs();
        transform.rotate(Quat::from_axis_angle(rotate.axis, angle));
    }
}
