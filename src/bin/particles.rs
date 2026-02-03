//! 2D Particle System Demo
//!
//! Demonstrates continuous particle spawning with velocity, lifetime, and fade-out.

use bevy::{
    app::AppExit,
    prelude::*,
    window::{Window, WindowPlugin, WindowPosition, WindowResolution},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;
const RANDOM_SEED: u64 = 12345678901234567;
const SPAWN_RATE: f32 = 50.0; // particles per second
const PARTICLE_LIFETIME: f32 = 2.0;
const PARTICLE_SPEED: f32 = 150.0;
const PARTICLE_SIZE: f32 = 8.0;
const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.02, 0.08); // Dark magenta

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
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
        .insert_resource(SpawnTimer(Timer::from_seconds(
            1.0 / SPAWN_RATE,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                spawn_particles,
                update_particles,
                despawn_particles,
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

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Lifetime {
    remaining: f32,
    total: f32,
}

#[derive(Resource)]
struct RandomSource(SmallRng);

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct ParticleMesh(Handle<Mesh>);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(Camera2d);

    let mesh = meshes.add(Circle::new(PARTICLE_SIZE));
    commands.insert_resource(ParticleMesh(mesh));
}

fn spawn_particles(
    mut commands: Commands,
    mut rng: ResMut<RandomSource>,
    mut timer: ResMut<SpawnTimer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    particle_mesh: Res<ParticleMesh>,
    time: Res<Time>,
    window: Query<&Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };

    timer.0.tick(time.delta());

    for _ in 0..timer.0.times_finished_this_tick() {
        let angle = rng.0.random_range(0.0..std::f32::consts::TAU);
        let speed = rng
            .0
            .random_range(PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 1.5);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        let hue = rng.0.random_range(0.0..360.0);
        let color = Color::hsl(hue, 0.8, 0.6);

        let lifetime = rng
            .0
            .random_range(PARTICLE_LIFETIME * 0.5..PARTICLE_LIFETIME * 1.5);

        let material = materials.add(ColorMaterial::from_color(color));

        let _ = window; // acknowledge we have access to window for spawn position

        commands.spawn((
            Particle,
            Velocity(velocity),
            Lifetime {
                remaining: lifetime,
                total: lifetime,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Mesh2d(particle_mesh.0.clone()),
            MeshMaterial2d(material),
        ));
    }
}

fn update_particles(
    mut query: Query<(
        &mut Transform,
        &Velocity,
        &mut Lifetime,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, velocity, mut lifetime, material_handle) in query.iter_mut() {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

        lifetime.remaining -= dt;

        let alpha = (lifetime.remaining / lifetime.total).clamp(0.0, 1.0);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            if let Color::Hsla(Hsla {
                hue,
                saturation,
                lightness,
                ..
            }) = material.color
            {
                material.color = Color::hsla(hue, saturation, lightness, alpha);
            }
        }
    }
}

fn despawn_particles(mut commands: Commands, query: Query<(Entity, &Lifetime), With<Particle>>) {
    for (entity, lifetime) in query.iter() {
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
