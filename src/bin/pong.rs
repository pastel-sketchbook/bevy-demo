//! Classic Pong game demonstrating 2D rendering with Mesh2d, collision detection, and scoring.

#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy_demo::*;

// --- Constants ---

const BACKGROUND_COLOR: Color = background_color(0.0, 0.08, 0.04, 0.3);
const PADDLE_WIDTH: f32 = 15.0;
const PADDLE_HEIGHT: f32 = 160.0;
const PADDLE_SPEED: f32 = 400.0;
const PADDLE_OFFSET: f32 = 50.0;
const BALL_RADIUS: f32 = 10.0;
const MIN_BALL_SPEED: f32 = 200.0;
const MAX_BALL_SPEED: f32 = 500.0;
const PADDLE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BALL_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const RANDOM_SEED: u64 = 12345678901234;
const AI_REACTION_SPEED: f32 = 0.85; // AI tracks ball at 85% of paddle speed

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
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
struct BallRadius(f32);

#[derive(Component)]
struct BezierPath {
    start: Vec2,
    control: Vec2,
    end: Vec2,
    t: f32,
    duration: f32, // Time to traverse the full path in seconds
}

impl BezierPath {
    /// Evaluate the quadratic bezier curve at parameter t (0.0 to 1.0)
    fn evaluate(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;
        // Quadratic bezier: B(t) = (1-t)^2*P0 + 2(1-t)tP1 + t^2*P2
        self.start * (inv_t * inv_t) + self.control * (2.0 * inv_t * t) + self.end * (t * t)
    }

    /// Create a new bezier path from current position toward target side
    fn new_path(start: Vec2, going_right: bool, rng: &mut SmallRng) -> Self {
        let half_w = WINDOW_WIDTH / 2.0;
        let half_h = WINDOW_HEIGHT / 2.0;

        // End point is on the opposite side, random Y position
        let end_x = if going_right {
            half_w + BALL_RADIUS * 2.0 // Past right edge
        } else {
            -half_w - BALL_RADIUS * 2.0 // Past left edge
        };
        let end_y = rng.random_range(-half_h * 0.8..half_h * 0.8);
        let end = Vec2::new(end_x, end_y);

        // Control point creates the curve - positioned between start and end
        // with random vertical offset for variety
        let mid_x = (start.x + end_x) / 2.0;
        let curve_strength = rng.random_range(-half_h * 0.6..half_h * 0.6);
        let control = Vec2::new(mid_x, start.y + curve_strength);

        // Duration based on distance (faster = shorter duration)
        let distance = start.distance(end);
        let speed = rng.random_range(MIN_BALL_SPEED..MAX_BALL_SPEED);
        let duration = distance / speed;

        Self {
            start,
            control,
            end,
            t: 0.0,
            duration,
        }
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource, Default)]
struct Score {
    left: u32,
    right: u32,
}

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
    let going_right = rng.0.random_bool(0.5);
    let initial_path = BezierPath::new_path(Vec2::ZERO, going_right, &mut rng.0);

    commands.spawn((
        Mesh2d(ball_mesh),
        MeshMaterial2d(ball_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Ball,
        BallRadius(BALL_RADIUS),
        initial_path,
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

fn move_paddles(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window>,
    ball: Query<&Transform, With<Ball>>,
    mut left_paddle: Query<&mut Transform, (With<LeftPaddle>, Without<RightPaddle>, Without<Ball>)>,
    mut right_paddle: Query<
        &mut Transform,
        (With<RightPaddle>, Without<LeftPaddle>, Without<Ball>),
    >,
    mut app_exit: MessageWriter<AppExit>,
) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }

    let Ok(window) = window.single() else {
        return;
    };
    let Ok(ball_transform) = ball.single() else {
        return;
    };

    let half_height = window.height() / 2.0;
    let paddle_half = PADDLE_HEIGHT / 2.0;
    let max_y = half_height - paddle_half;
    let ball_y = ball_transform.translation.y;
    let ai_speed = PADDLE_SPEED * AI_REACTION_SPEED;

    // Left paddle - AI controlled
    if let Ok(mut transform) = left_paddle.single_mut() {
        let diff = ball_y - transform.translation.y;
        let move_amount = diff.signum() * ai_speed.min(diff.abs()) * time.delta_secs();
        transform.translation.y += move_amount;
        transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
    }

    // Right paddle - AI controlled
    if let Ok(mut transform) = right_paddle.single_mut() {
        let diff = ball_y - transform.translation.y;
        let move_amount = diff.signum() * ai_speed.min(diff.abs()) * time.delta_secs();
        transform.translation.y += move_amount;
        transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
    }
}

fn move_ball(time: Res<Time>, mut ball: Query<(&mut Transform, &mut BezierPath), With<Ball>>) {
    if let Ok((mut transform, mut path)) = ball.single_mut() {
        // Advance t based on time and duration
        path.t += time.delta_secs() / path.duration;

        // Evaluate bezier curve at current t
        let pos = path.evaluate(path.t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn ball_collision(
    window: Query<&Window>,
    mut ball: Query<
        (
            &mut Transform,
            &mut BezierPath,
            &mut BallRadius,
            &mut Mesh2d,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Ball>,
    >,
    left_paddle: Query<&Transform, (With<LeftPaddle>, Without<Ball>)>,
    right_paddle: Query<&Transform, (With<RightPaddle>, Without<Ball>)>,
    mut score: ResMut<Score>,
    mut rng: ResMut<RandomSource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut ball_transform, mut path, mut radius, mut mesh, mut material)) = ball.single_mut()
    else {
        return;
    };

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let ball_r = radius.0;

    // Top/bottom wall bounce - create new path reflecting off wall
    if ball_transform.translation.y + ball_r > half_height {
        ball_transform.translation.y = half_height - ball_r;
        let current_pos = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);
        let going_right = path.end.x > path.start.x;
        *path = BezierPath::new_path(current_pos, going_right, &mut rng.0);
    } else if ball_transform.translation.y - ball_r < -half_height {
        ball_transform.translation.y = -half_height + ball_r;
        let current_pos = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);
        let going_right = path.end.x > path.start.x;
        *path = BezierPath::new_path(current_pos, going_right, &mut rng.0);
    }

    // Paddle collision
    let paddle_half_w = PADDLE_WIDTH / 2.0;
    let paddle_half_h = PADDLE_HEIGHT / 2.0;

    // Left paddle - ball bounces right
    if let Ok(paddle) = left_paddle.single()
        && ball_transform.translation.x - ball_r < paddle.translation.x + paddle_half_w
        && ball_transform.translation.x > paddle.translation.x
        && ball_transform.translation.y < paddle.translation.y + paddle_half_h
        && ball_transform.translation.y > paddle.translation.y - paddle_half_h
    {
        ball_transform.translation.x = paddle.translation.x + paddle_half_w + ball_r;
        let current_pos = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);
        *path = BezierPath::new_path(current_pos, true, &mut rng.0); // Go right
    }

    // Right paddle - ball bounces left
    if let Ok(paddle) = right_paddle.single()
        && ball_transform.translation.x + ball_r > paddle.translation.x - paddle_half_w
        && ball_transform.translation.x < paddle.translation.x
        && ball_transform.translation.y < paddle.translation.y + paddle_half_h
        && ball_transform.translation.y > paddle.translation.y - paddle_half_h
    {
        ball_transform.translation.x = paddle.translation.x - paddle_half_w - ball_r;
        let current_pos = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);
        *path = BezierPath::new_path(current_pos, false, &mut rng.0); // Go left
    }

    // Scoring - ball went past edge, randomize size and color
    let scored = if ball_transform.translation.x < -half_width {
        score.right += 1;
        true
    } else if ball_transform.translation.x > half_width {
        score.left += 1;
        true
    } else {
        false
    };

    if scored {
        ball_transform.translation = Vec3::ZERO;
        let going_right = rng.0.random_bool(0.5);
        *path = BezierPath::new_path(Vec2::ZERO, going_right, &mut rng.0);

        // Randomize ball size (8 to 20 pixels)
        let new_radius = rng.0.random_range(8.0..20.0);
        radius.0 = new_radius;
        mesh.0 = meshes.add(Circle::new(new_radius));

        // Randomize ball color (bright colors)
        let r = rng.0.random_range(0.5..1.0);
        let g = rng.0.random_range(0.5..1.0);
        let b = rng.0.random_range(0.5..1.0);
        material.0 = materials.add(Color::srgb(r, g, b));
    }
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text2d, With<ScoreText>>) {
    if score.is_changed()
        && let Ok(mut text) = query.single_mut()
    {
        *text = Text2d::new(format!("{} - {}", score.left, score.right));
    }
}
