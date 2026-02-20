//! Bouncing shapes demo - demonstrates velocity-based movement, boundary collision, shape morphing.

use bevy_demo::*;

const SHAPE_COUNT: usize = 20;
const MIN_SIZE: f32 = 15.0;
const MAX_SIZE: f32 = 40.0;
const MIN_SPEED: f32 = 100.0;
const MAX_SPEED: f32 = 300.0;
const RANDOM_SEED: u64 = 42;
const SHAPE_TYPES: usize = 5;

const BACKGROUND_COLOR: Color = background_color(0.08, 0.05, 0.15, 0.3);

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
                move_shapes,
                bounce_off_walls,
                handle_input,
            ),
        )
        .run();
}

#[derive(Component)]
struct Shape;

#[derive(Component)]
struct Size(f32);

#[derive(Resource)]
struct ShapeMeshes {
    circle: Handle<Mesh>,
    square: Handle<Mesh>,
    triangle: Handle<Mesh>,
    hexagon: Handle<Mesh>,
    pentagon: Handle<Mesh>,
}

fn create_shape_meshes(meshes: &mut Assets<Mesh>) -> ShapeMeshes {
    ShapeMeshes {
        circle: meshes.add(Circle::new(1.0)),
        square: meshes.add(Rectangle::new(2.0, 2.0)),
        triangle: meshes.add(RegularPolygon::new(1.0, 3)),
        hexagon: meshes.add(RegularPolygon::new(1.0, 6)),
        pentagon: meshes.add(RegularPolygon::new(1.0, 5)),
    }
}

fn random_mesh(shape_meshes: &ShapeMeshes, rng: &mut SmallRng) -> Handle<Mesh> {
    match rng.random_range(0..SHAPE_TYPES) {
        0 => shape_meshes.circle.clone(),
        1 => shape_meshes.square.clone(),
        2 => shape_meshes.triangle.clone(),
        3 => shape_meshes.hexagon.clone(),
        _ => shape_meshes.pentagon.clone(),
    }
}

fn random_color(rng: &mut SmallRng) -> Color {
    Color::srgb(
        rng.random_range(0.3..1.0),
        rng.random_range(0.3..1.0),
        rng.random_range(0.3..1.0),
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut rng: ResMut<RandomSource>,
) {
    commands.spawn(Camera2d);

    let shape_meshes = create_shape_meshes(&mut meshes);
    let window = window.single().unwrap();
    let half_width = window.width() / 2.0 - MAX_SIZE;
    let half_height = window.height() / 2.0 - MAX_SIZE;

    for _ in 0..SHAPE_COUNT {
        spawn_shape(
            &mut commands,
            &mut materials,
            &shape_meshes,
            &mut rng.0,
            half_width,
            half_height,
        );
    }

    commands.insert_resource(shape_meshes);
}

fn spawn_shape(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    shape_meshes: &ShapeMeshes,
    rng: &mut SmallRng,
    half_width: f32,
    half_height: f32,
) {
    let x = rng.random_range(-half_width..half_width);
    let y = rng.random_range(-half_height..half_height);
    let speed = rng.random_range(MIN_SPEED..MAX_SPEED);
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
    let size = rng.random_range(MIN_SIZE..MAX_SIZE);

    let mesh = random_mesh(shape_meshes, rng);
    let material = materials.add(random_color(rng));

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(size)),
        Shape,
        Velocity(velocity),
        Size(size),
    ));
}

fn move_shapes(mut query: Query<(&mut Transform, &Velocity), With<Shape>>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn bounce_off_walls(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut Size,
            &mut Mesh2d,
        ),
        With<Shape>,
    >,
    window: Query<&Window, With<PrimaryWindow>>,
    mut rng: ResMut<RandomSource>,
    shape_meshes: Res<ShapeMeshes>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let half_width = window.width() / 2.0 - MAX_SIZE;
    let half_height = window.height() / 2.0 - MAX_SIZE;

    for (entity, mut transform, mut velocity, mut size, mut mesh) in query.iter_mut() {
        let mut bounced = false;

        if transform.translation.x <= -half_width {
            transform.translation.x = -half_width;
            velocity.0.x = velocity.0.x.abs();
            bounced = true;
        } else if transform.translation.x >= half_width {
            transform.translation.x = half_width;
            velocity.0.x = -velocity.0.x.abs();
            bounced = true;
        }

        if transform.translation.y <= -half_height {
            transform.translation.y = -half_height;
            velocity.0.y = velocity.0.y.abs();
            bounced = true;
        } else if transform.translation.y >= half_height {
            transform.translation.y = half_height;
            velocity.0.y = -velocity.0.y.abs();
            bounced = true;
        }

        if bounced {
            let new_size = rng.0.random_range(MIN_SIZE..MAX_SIZE);
            size.0 = new_size;
            transform.scale = Vec3::splat(new_size);

            mesh.0 = random_mesh(&shape_meshes, &mut rng.0);

            let new_material = materials.add(random_color(&mut rng.0));
            commands.entity(entity).insert(MeshMaterial2d(new_material));
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut rng: ResMut<RandomSource>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    shape_meshes: Res<ShapeMeshes>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        let Ok(window) = window.single() else {
            return;
        };
        let half_width = window.width() / 2.0 - MAX_SIZE;
        let half_height = window.height() / 2.0 - MAX_SIZE;
        spawn_shape(
            &mut commands,
            &mut materials,
            &shape_meshes,
            &mut rng.0,
            half_width,
            half_height,
        );
    }
}
