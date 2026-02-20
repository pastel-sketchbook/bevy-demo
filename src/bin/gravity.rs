//! Gravitational attraction simulation with a sun and orbiting planets.
//! Press Space to add a new random planet.

use bevy_demo::*;

const RANDOM_SEED: u64 = 42;

const BACKGROUND_COLOR: Color = background_color(0.01, 0.01, 0.03, 0.3);

const SUN_MASS: f32 = 1000.0;
const SUN_RADIUS: f32 = 30.0;
const PLANET_MASS: f32 = 1.0;
const PLANET_RADIUS: f32 = 8.0;
const GRAVITATIONAL_CONSTANT: f32 = 50.0;
const INITIAL_PLANETS: usize = 5;
const MIN_ORBIT_RADIUS: f32 = 100.0;
const MAX_ORBIT_RADIUS: f32 = 350.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                apply_gravity,
                update_positions,
                handle_input,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Mass(f32);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<RandomSource>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(SUN_RADIUS))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sun,
        Mass(SUN_MASS),
    ));

    for _ in 0..INITIAL_PLANETS {
        spawn_planet(&mut commands, &mut meshes, &mut materials, &mut rng.0);
    }
}

fn spawn_planet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rng: &mut SmallRng,
) {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let distance = rng.random_range(MIN_ORBIT_RADIUS..MAX_ORBIT_RADIUS);
    let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);

    let orbital_speed = (GRAVITATIONAL_CONSTANT * SUN_MASS / distance).sqrt();
    let velocity_dir = Vec2::new(-angle.sin(), angle.cos());
    let velocity = velocity_dir * orbital_speed;

    let color = Color::srgb(
        rng.random_range(0.3..1.0),
        rng.random_range(0.3..1.0),
        rng.random_range(0.3..1.0),
    );

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(position.x, position.y, 0.0),
        Planet,
        Mass(PLANET_MASS),
        Velocity(velocity),
    ));
}

fn apply_gravity(
    sun_query: Query<(&Transform, &Mass), With<Sun>>,
    mut planet_query: Query<(&Transform, &Mass, &mut Velocity), With<Planet>>,
    time: Res<Time>,
) {
    let Ok((sun_transform, sun_mass)) = sun_query.single() else {
        return;
    };
    let sun_pos = sun_transform.translation.truncate();

    for (planet_transform, planet_mass, mut velocity) in planet_query.iter_mut() {
        let planet_pos = planet_transform.translation.truncate();
        let direction = sun_pos - planet_pos;
        let distance_sq = direction.length_squared().max(100.0);
        let _distance = distance_sq.sqrt();

        let force_magnitude = GRAVITATIONAL_CONSTANT * sun_mass.0 * planet_mass.0 / distance_sq;
        let acceleration = direction.normalize() * (force_magnitude / planet_mass.0);

        velocity.0 += acceleration * time.delta_secs();
    }
}

fn update_positions(
    mut planet_query: Query<(&mut Transform, &Velocity), With<Planet>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in planet_query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<RandomSource>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        spawn_planet(&mut commands, &mut meshes, &mut materials, &mut rng.0);
    }

    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
