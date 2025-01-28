use bevy::{
    math::{prelude::*, NormedVectorSpace},
    prelude::*,
    window::{WindowPlugin, WindowResolution},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Remove window frame by setting borderless to true
                decorations: false,
                resolution: WindowResolution::new(1610.0, 1042.0),
                ..default()
            }),
            ..default()
        }),))
        // set the global default clear color
        // Changed ClearColor to an early morning light greenish color
        .insert_resource(ClearColor(Color::srgba(0.88, 0.93, 0.90, 1.0)))
        .insert_resource(RandomSource(ChaCha8Rng::seed_from_u64(68941654987813521))) // Insert RandomSource resource here, before setup system
        .add_systems(Startup, setup)
        .add_systems(Update, move_firefly)
        .run();
}

#[derive(Component)]
struct Firefly;

#[derive(Component)]
struct FireflySpeed(f32);

#[derive(Component)]
struct FireflyPosition(Vec3);

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rng_res: ResMut<RandomSource>, // Get the RandomSource resource
) {
    let mut rng = rand::thread_rng();
    let num_fireflies = rng.gen_range(30..=70);
    let legal_region = Cuboid::from_size(Vec3::splat(10.0)); // Define the legal region here

    for _ in 0..num_fireflies {
        let size_random = rng.gen_range(0.05..0.3);
        // Updated speed range to 0.05..2.0 for slower movement
        let speed_random = rng.gen_range(0.05..2.0);
        let red_random = rng.gen_range(0.1..0.9);
        let green_random = rng.gen_range(0.1..0.9);
        let blue_random = rng.gen_range(0.1..0.9);
        let alpha_random = rng.gen_range(0.7..1.0);
        let random_position = legal_region.sample_interior(&mut rng_res.0); // Generate random position

        // firefly
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(size_random))),
            MeshMaterial3d(materials.add(Color::srgba(
                red_random,
                green_random,
                blue_random,
                alpha_random,
            ))),
            Firefly,
            FireflySpeed(speed_random),
            FireflyPosition(random_position), // Set the random initial position
            Transform::from_translation(random_position), // Set the transform position to the random initial position
        ));
    }

    // A light:
    commands.spawn((
        PointLight {
            intensity: 15_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // A camera:
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn move_firefly(
    mut fireflies: Query<(&mut Transform, &FireflySpeed, &mut FireflyPosition), With<Firefly>>,
    time: Res<Time>,
    mut rng: ResMut<RandomSource>,
) {
    let legal_region = Cuboid::from_size(Vec3::splat(10.0));
    for (mut target, target_speed, mut target_pos) in fireflies.iter_mut() {
        match Dir3::new(target_pos.0 - target.translation) {
            Ok(dir) => {
                let delta_time = time.delta_secs();
                let abs_delta = (target_pos.0 - target.translation).norm();

                let magnitude = f32::min(abs_delta, delta_time * target_speed.0);
                target.translation += dir * magnitude;
            }

            Err(_) => {
                target_pos.0 = legal_region.sample_interior(&mut rng.0);
            }
        }
    }
}
