use bevy::{
    app::AppExit,
    math::prelude::*,
    prelude::*,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;
const BACKGROUND_COLOR: Color = Color::srgba(0.07, 0.14, 0.04, 1.0);
const RANDOM_SEED: u64 = 68941654987813521;
const MIN_FIREFLIES: usize = 30;
const MAX_FIREFLIES: usize = 70;
const MIN_SIZE: f32 = 0.05;
const MAX_SIZE: f32 = 0.3;
const MIN_SPEED: f32 = 0.05;
const MAX_SPEED: f32 = 2.0;
const MIN_COLOR: f32 = 0.1;
const MAX_COLOR: f32 = 0.9;
const MIN_ALPHA: f32 = 0.7;
const MAX_ALPHA: f32 = 1.0;
const WORLD_SIZE: f32 = 10.0;
const LIGHT_INTENSITY: f32 = 15_000_000.0;
const LIGHT_X: f32 = 4.0;
const LIGHT_Y: f32 = 8.0;
const LIGHT_Z: f32 = 4.0;
const CAMERA_X: f32 = -2.0;
const CAMERA_Y: f32 = 3.0;
const CAMERA_Z: f32 = 5.0;
const COLOR_INCREMENT: f32 = 0.02;
const MIN_BLINK_SPEED: f32 = 0.5;
const MAX_BLINK_SPEED: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                decorations: false,
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        // Changed ClearColor to an very dark greenish color
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                move_firefly,
                firefly_blink,
                handle_keyboard_input,
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

#[derive(Component)]
struct Firefly;

#[derive(Component)]
struct FireflySpeed(f32);

#[derive(Component)]
struct FireflyPosition(Vec3);

#[derive(Resource)]
struct RandomSource(SmallRng);

#[derive(Component)]
struct Blink {
    speed: f32,
    timer: Timer,
    max_alpha: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rng_res: ResMut<RandomSource>,
) {
    let mut rng = rand::rng();
    let num_fireflies = rng.random_range(MIN_FIREFLIES..=MAX_FIREFLIES);
    // Define the legal region here
    let legal_region = Cuboid::from_size(Vec3::splat(WORLD_SIZE));

    for _ in 0..num_fireflies {
        let size_random = rng.random_range(MIN_SIZE..MAX_SIZE);
        // Updated speed range to 0.05..2.0 for slower movement
        let speed_random = rng.random_range(MIN_SPEED..MAX_SPEED);
        let red_random = rng.random_range(MIN_COLOR..MAX_COLOR);
        let green_random = rng.random_range(MIN_COLOR..MAX_COLOR);
        let blue_random = rng.random_range(MIN_COLOR..MAX_COLOR);
        let alpha_random = rng.random_range(MIN_ALPHA..MAX_ALPHA);
        let random_position = legal_region.sample_interior(&mut rng_res.0);
        let blink_speed_random = rng.random_range(MIN_BLINK_SPEED..MAX_BLINK_SPEED);

        let firefly_color = Color::srgba(red_random, green_random, blue_random, alpha_random);
        let firefly_mesh = meshes.add(Sphere::new(size_random));
        let firefly_material = materials.add(firefly_color);

        // firefly
        commands.spawn((
            Transform::from_translation(random_position), // Use Transform component directly
            Visibility::Visible, // Use Visibility component directly, set to Visible
            Mesh3d(firefly_mesh), // Add Mesh3d component
            MeshMaterial3d(firefly_material), // Add MeshMaterial3d component
            Firefly,
            FireflySpeed(speed_random),
            FireflyPosition(random_position), // Set the random initial position
            Blink {
                // Added Blink component with random speed and max_alpha
                speed: blink_speed_random,
                timer: Timer::from_seconds(0.0, TimerMode::Repeating),
                max_alpha: alpha_random,
            },
        ));
    }

    // A light:
    commands.spawn((
        PointLight {
            intensity: LIGHT_INTENSITY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(LIGHT_X, LIGHT_Y, LIGHT_Z),
    ));

    // A camera:
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_X, CAMERA_Y, CAMERA_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn move_firefly(
    mut fireflies: Query<(&mut Transform, &FireflySpeed, &mut FireflyPosition), With<Firefly>>,
    time: Res<Time>,
    mut rng: ResMut<RandomSource>,
) {
    let legal_region = Cuboid::from_size(Vec3::splat(WORLD_SIZE));
    for (mut transform, target_speed, mut target_pos) in fireflies.iter_mut() {
        let target_direction = target_pos.0 - transform.translation;
        if target_direction == Vec3::ZERO {
            target_pos.0 = legal_region.sample_interior(&mut rng.0);
            continue; // Skip to the next firefly to avoid division by zero in normalize()
        }
        // Use Vec3::normalize() directly
        let move_direction = target_direction.normalize();
        let delta_time = time.delta_secs();
        let abs_delta = target_direction.length(); // Calculate length directly
        let magnitude = f32::min(abs_delta, delta_time * target_speed.0);
        transform.translation += move_direction * magnitude;
    }
}

fn firefly_blink(
    mut query: Query<(&MeshMaterial3d<StandardMaterial>, &mut Blink)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (material_component, mut blink) in query.iter_mut() {
        blink.timer.tick(time.delta());
        let fraction = blink.timer.elapsed_secs() * blink.speed;
        // Use a sine wave to create a smooth blink effect, range [0.5, 1.0] to avoid completely invisible
        let alpha_multiplier =
            (f32::sin(fraction * std::f32::consts::PI * 2.0) * 0.25 + 0.75).clamp(0.5, 1.0);

        // Get the Handle<StandardMaterial> from MeshMaterial3d
        let material_handle = material_component.0.clone(); // Clone the handle *before* if let

        // Access the Assets<StandardMaterial> resource and get the Material
        if let Some(material) = materials.get_mut(&material_handle) {
            // Use materials.get_mut(handle)
            // Corrected way to set alpha: Pattern match and reconstruct Color
            if let Color::Srgba(Srgba {
                red, green, blue, ..
            }) = material.base_color
            {
                material.base_color = Color::srgba(
                    red,
                    green,
                    blue,
                    blink.max_alpha * alpha_multiplier, // Set new alpha
                );
            }
        }
    }
}

// New system to handle keyboard input to control background color or quit the application
fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut clear_color: ResMut<ClearColor>,
    mut app_exit_events: MessageWriter<AppExit>,
) {
    let increment = COLOR_INCREMENT; // Use constant

    if keyboard_input.pressed(KeyCode::KeyL) {
        // Make background lighter
        if let ClearColor(Color::Srgba(Srgba {
            red,
            green,
            blue,
            alpha,
        })) = *clear_color
        {
            // Dereference clear_color here with *
            clear_color.0 = Color::srgba(
                (red + increment).min(1.0),
                (green + increment).min(1.0),
                (blue + increment).min(1.0),
                alpha,
            );
        }
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        // Make background darker
        if let ClearColor(Color::Srgba(Srgba {
            red,
            green,
            blue,
            alpha,
        })) = *clear_color
        {
            // Dereference clear_color here with *
            clear_color.0 = Color::srgba(
                (red - increment).max(0.0),
                (green - increment).max(0.0),
                (blue - increment).max(0.0),
                alpha,
            );
        }
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        // Quit the application when Q is pressed
        app_exit_events.write(AppExit::Success);
    }
}
