//! Boids flocking simulation demonstrating separation, alignment, and cohesion.

use bevy_demo::*;

const NUM_BOIDS: usize = 60;
const BOID_SPEED: f32 = 150.0;
const MAX_SPEED: f32 = 200.0;
const PERCEPTION_RADIUS: f32 = 50.0;
const SEPARATION_WEIGHT: f32 = 1.5;
const ALIGNMENT_WEIGHT: f32 = 1.0;
const COHESION_WEIGHT: f32 = 1.0;
const RANDOM_SEED: u64 = 12345678901234;

const BACKGROUND_COLOR: Color = background_color(0.02, 0.05, 0.12, 0.3);

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
                flocking,
                apply_velocity,
                wrap_position,
                handle_quit,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Boid;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<RandomSource>,
) {
    commands.spawn(Camera2d);

    let triangle = Triangle2d::new(
        Vec2::new(0.0, 10.0),
        Vec2::new(-5.0, -5.0),
        Vec2::new(5.0, -5.0),
    );
    let mesh = meshes.add(triangle);
    let material = materials.add(Color::srgb(0.8, 0.9, 1.0));

    for _ in 0..NUM_BOIDS {
        let x = rng.0.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let y = rng
            .0
            .random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
        let angle = rng.0.random_range(0.0..std::f32::consts::TAU);
        let velocity = Vec2::from_angle(angle) * BOID_SPEED;

        commands.spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(x, y, 0.0),
            Boid,
            Velocity(velocity),
        ));
    }
}

fn flocking(mut query: Query<(Entity, &Transform, &mut Velocity), With<Boid>>) {
    let boids: Vec<(Entity, Vec2, Vec2)> = query
        .iter()
        .map(|(e, t, v)| (e, t.translation.truncate(), v.0))
        .collect();

    for (entity, _, mut velocity) in query.iter_mut() {
        let (pos, vel) = boids
            .iter()
            .find(|(e, _, _)| *e == entity)
            .map(|(_, p, v)| (*p, *v))
            .unwrap();

        let mut separation = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec2::ZERO;
        let mut count = 0;

        for (other_entity, other_pos, other_vel) in &boids {
            if *other_entity == entity {
                continue;
            }

            let diff = pos - *other_pos;
            let dist = diff.length();

            if dist < PERCEPTION_RADIUS && dist > 0.0 {
                separation += diff / dist;
                alignment += *other_vel;
                cohesion += *other_pos;
                count += 1;
            }
        }

        if count > 0 {
            let count_f = count as f32;
            separation /= count_f;
            alignment = (alignment / count_f - vel).clamp_length_max(BOID_SPEED);
            cohesion = (cohesion / count_f - pos).clamp_length_max(BOID_SPEED);

            velocity.0 += separation * SEPARATION_WEIGHT
                + alignment * ALIGNMENT_WEIGHT
                + cohesion * COHESION_WEIGHT;
            velocity.0 = velocity.0.clamp_length_max(MAX_SPEED);
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity), With<Boid>>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        if velocity.0.length_squared() > 0.0 {
            let angle = velocity.0.y.atan2(velocity.0.x) - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn wrap_position(mut query: Query<&mut Transform, With<Boid>>, window: Query<&Window>) {
    let Ok(window) = window.single() else {
        return;
    };

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;

    for mut transform in query.iter_mut() {
        let pos = &mut transform.translation;
        if pos.x > half_w {
            pos.x = -half_w;
        } else if pos.x < -half_w {
            pos.x = half_w;
        }
        if pos.y > half_h {
            pos.y = -half_h;
        } else if pos.y < -half_h {
            pos.y = half_h;
        }
    }
}
