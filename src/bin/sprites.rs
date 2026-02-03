//! Sprite animation demo using procedural color cycling.
//!
//! Demonstrates:
//! - 2D sprite rendering with color changes
//! - Timer-based animation frame switching
//! - Arrow key movement
//! - No external assets required

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};

const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;
const SPRITE_SIZE: f32 = 64.0;
const MOVE_SPEED: f32 = 300.0;
const FRAME_DURATION: f32 = 0.15;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.03, 0.1, 0.1, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.03, 0.1, 0.1);

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

const ANIMATION_COLORS: [Color; 6] = [
    Color::srgb(1.0, 0.2, 0.2), // Red
    Color::srgb(1.0, 0.6, 0.2), // Orange
    Color::srgb(1.0, 1.0, 0.2), // Yellow
    Color::srgb(0.2, 1.0, 0.2), // Green
    Color::srgb(0.2, 0.6, 1.0), // Blue
    Color::srgb(0.8, 0.2, 1.0), // Purple
];

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
                animate_sprite,
                move_sprite,
                handle_input,
            ),
        )
        .run();
}

#[derive(Component)]
struct AnimatedSprite {
    current_frame: usize,
    timer: Timer,
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: ANIMATION_COLORS[0],
            custom_size: Some(Vec2::splat(SPRITE_SIZE)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        AnimatedSprite {
            current_frame: 0,
            timer: Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating),
        },
        Player,
    ));
}

fn animate_sprite(mut query: Query<(&mut Sprite, &mut AnimatedSprite)>, time: Res<Time>) {
    for (mut sprite, mut anim) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            anim.current_frame = (anim.current_frame + 1) % ANIMATION_COLORS.len();
            sprite.color = ANIMATION_COLORS[anim.current_frame];
        }
    }
}

fn move_sprite(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }

    let half_width = window.width() / 2.0 - SPRITE_SIZE / 2.0;
    let half_height = window.height() / 2.0 - SPRITE_SIZE / 2.0;

    for mut transform in query.iter_mut() {
        transform.translation += direction * MOVE_SPEED * time.delta_secs();
        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
        transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
    }
}

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}
