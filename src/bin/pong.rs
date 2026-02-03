//! Classic Pong game demonstrating 2D rendering with Mesh2d, collision detection, and scoring.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_SPEED: f32 = 400.0;
const PADDLE_OFFSET: f32 = 50.0;
const BALL_RADIUS: f32 = 10.0;
const BALL_SPEED: f32 = 300.0;
const PADDLE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BALL_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const RANDOM_SEED: u64 = 12345678901234;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.08, 0.04, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.08, 0.04); // Dark arcade green

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
        .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
        .insert_resource(Score::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                move_paddles,
                move_ball,
                ball_collision,
                update_score_text,
            ),
        )
        .run();
}

#[derive(Component)]
struct LeftPaddle;

#[derive(Component)]
struct RightPaddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct ScoreText;

#[derive(Resource, Default)]
struct Score {
    left: u32,
    right: u32,
}

#[derive(Resource)]
struct RandomSource(SmallRng);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<RandomSource>,
) {
    // Camera
    commands.spawn(Camera2d);

    // Left paddle
    let paddle_mesh = meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));
    let paddle_material = materials.add(PADDLE_COLOR);

    commands.spawn((
        Mesh2d(paddle_mesh.clone()),
        MeshMaterial2d(paddle_material.clone()),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + PADDLE_OFFSET, 0.0, 0.0),
        LeftPaddle,
    ));

    // Right paddle
    commands.spawn((
        Mesh2d(paddle_mesh),
        MeshMaterial2d(paddle_material),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - PADDLE_OFFSET, 0.0, 0.0),
        RightPaddle,
    ));

    // Ball
    let ball_mesh = meshes.add(Circle::new(BALL_RADIUS));
    let ball_material = materials.add(BALL_COLOR);
    let initial_velocity = random_ball_direction(&mut rng.0);

    commands.spawn((
        Mesh2d(ball_mesh),
        MeshMaterial2d(ball_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Ball,
        Velocity(initial_velocity),
    ));

    // Score text
    commands.spawn((
        Text2d::new("0 - 0"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 2.0 - 50.0, 0.0),
        ScoreText,
    ));
}

fn random_ball_direction(rng: &mut SmallRng) -> Vec2 {
    let angle = if rng.random_bool(0.5) {
        rng.random_range(-0.5..0.5)
    } else {
        rng.random_range(std::f32::consts::PI - 0.5..std::f32::consts::PI + 0.5)
    };
    Vec2::new(angle.cos(), angle.sin()) * BALL_SPEED
}

fn move_paddles(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window>,
    mut left_paddle: Query<&mut Transform, (With<LeftPaddle>, Without<RightPaddle>)>,
    mut right_paddle: Query<&mut Transform, (With<RightPaddle>, Without<LeftPaddle>)>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }

    let Ok(window) = window.single() else {
        return;
    };
    let half_height = window.height() / 2.0;
    let paddle_half = PADDLE_HEIGHT / 2.0;
    let max_y = half_height - paddle_half;

    // Left paddle (W/S)
    if let Ok(mut transform) = left_paddle.single_mut() {
        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.y += PADDLE_SPEED * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.y -= PADDLE_SPEED * time.delta_secs();
        }
        transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
    }

    // Right paddle (Up/Down)
    if let Ok(mut transform) = right_paddle.single_mut() {
        if keyboard.pressed(KeyCode::ArrowUp) {
            transform.translation.y += PADDLE_SPEED * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= PADDLE_SPEED * time.delta_secs();
        }
        transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
    }
}

fn move_ball(time: Res<Time>, mut ball: Query<(&mut Transform, &Velocity), With<Ball>>) {
    if let Ok((mut transform, velocity)) = ball.single_mut() {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn ball_collision(
    window: Query<&Window>,
    mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    left_paddle: Query<&Transform, (With<LeftPaddle>, Without<Ball>)>,
    right_paddle: Query<&Transform, (With<RightPaddle>, Without<Ball>)>,
    mut score: ResMut<Score>,
    mut rng: ResMut<RandomSource>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut ball_transform, mut velocity)) = ball.single_mut() else {
        return;
    };

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    // Top/bottom wall bounce
    if ball_transform.translation.y + BALL_RADIUS > half_height {
        ball_transform.translation.y = half_height - BALL_RADIUS;
        velocity.0.y = -velocity.0.y.abs();
    } else if ball_transform.translation.y - BALL_RADIUS < -half_height {
        ball_transform.translation.y = -half_height + BALL_RADIUS;
        velocity.0.y = velocity.0.y.abs();
    }

    // Paddle collision
    let paddle_half_w = PADDLE_WIDTH / 2.0;
    let paddle_half_h = PADDLE_HEIGHT / 2.0;

    // Left paddle
    if let Ok(paddle) = left_paddle.single() {
        if ball_transform.translation.x - BALL_RADIUS < paddle.translation.x + paddle_half_w
            && ball_transform.translation.x > paddle.translation.x
            && ball_transform.translation.y < paddle.translation.y + paddle_half_h
            && ball_transform.translation.y > paddle.translation.y - paddle_half_h
        {
            ball_transform.translation.x = paddle.translation.x + paddle_half_w + BALL_RADIUS;
            velocity.0.x = velocity.0.x.abs();
        }
    }

    // Right paddle
    if let Ok(paddle) = right_paddle.single() {
        if ball_transform.translation.x + BALL_RADIUS > paddle.translation.x - paddle_half_w
            && ball_transform.translation.x < paddle.translation.x
            && ball_transform.translation.y < paddle.translation.y + paddle_half_h
            && ball_transform.translation.y > paddle.translation.y - paddle_half_h
        {
            ball_transform.translation.x = paddle.translation.x - paddle_half_w - BALL_RADIUS;
            velocity.0.x = -velocity.0.x.abs();
        }
    }

    // Scoring
    if ball_transform.translation.x < -half_width {
        score.right += 1;
        ball_transform.translation = Vec3::ZERO;
        velocity.0 = random_ball_direction(&mut rng.0);
    } else if ball_transform.translation.x > half_width {
        score.left += 1;
        ball_transform.translation = Vec3::ZERO;
        velocity.0 = random_ball_direction(&mut rng.0);
    }
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text2d, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            *text = Text2d::new(format!("{} - {}", score.left, score.right));
        }
    }
}
