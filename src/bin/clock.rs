//! Analog clock rendered with Gizmos (immediate-mode drawing).
//! Uses `jiff` for America/Chicago timezone-aware wall-clock time.
//! Displays hour/minute/second hands and clock face tick marks.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};
use std::f32::consts::{FRAC_PI_2, TAU};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.02, 0.02, 0.04, 0.06);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.02, 0.02, 0.04);

const CLOCK_RADIUS: f32 = 400.0;
const TIMEZONE: &str = "America/Chicago";
const HOUR_HAND_LENGTH: f32 = 220.0;
const MINUTE_HAND_LENGTH: f32 = 320.0;
const SECOND_HAND_LENGTH: f32 = 350.0;
const HOUR_HAND_WIDTH: f32 = 24.0;
const MINUTE_HAND_WIDTH: f32 = 16.0;
const SECOND_HAND_WIDTH: f32 = 8.0;
const TICK_MAJOR_LENGTH: f32 = 30.0;
const TICK_MINOR_LENGTH: f32 = 15.0;
const TICK_MAJOR_WIDTH: f32 = 12.0;
const TICK_MINOR_WIDTH: f32 = 6.0;
const CENTER_DOT_RADIUS: f32 = 14.0;
const FACE_RING_WIDTH: f32 = 20.0;
const FACE_COLOR: Color = Color::srgba(0.76, 0.72, 0.87, 0.6); // pastel lavender
const HOUR_COLOR: Color = Color::srgb(0.98, 0.78, 0.65); // pastel peach
const MINUTE_COLOR: Color = Color::srgb(0.68, 0.90, 0.78); // pastel mint
const SECOND_COLOR: Color = Color::srgb(0.95, 0.68, 0.72); // pastel rose
const TICK_MAJOR_COLOR: Color = Color::srgb(0.68, 0.78, 0.95); // pastel cornflower
const TICK_MINOR_COLOR: Color = Color::srgb(0.80, 0.75, 0.92); // pastel lilac
const CENTER_COLOR: Color = Color::srgb(0.98, 0.93, 0.68); // pastel butter

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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                draw_clock,
                handle_quit,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Convert a clock angle (0 = 12 o'clock, clockwise) to a 2D direction vector.
/// Clock angles: 0 = up, PI/2 = right (3 o'clock), PI = down, 3PI/2 = left.
fn clock_direction(angle: f32) -> Vec2 {
    // Standard math: 0 = right, counter-clockwise.
    // Clock: 0 = up, clockwise. So math_angle = PI/2 - clock_angle.
    let math_angle = FRAC_PI_2 - angle;
    Vec2::new(math_angle.cos(), math_angle.sin())
}

fn draw_clock(mut gizmos: Gizmos) {
    // Get current wall-clock time in America/Chicago (handles CST/CDT automatically)
    let Ok(now) = jiff::Timestamp::now().in_tz(TIMEZONE) else {
        return;
    };
    let hours = (now.hour() % 12) as f64;
    let minutes = now.minute() as f64;
    let seconds = now.second() as f64 + now.subsec_nanosecond() as f64 / 1_000_000_000.0;

    // Smooth hand movement: seconds feed into minutes, minutes feed into hours
    let smooth_minutes = minutes + seconds / 60.0;
    let smooth_hours = hours + smooth_minutes / 60.0;

    // Angles (fraction of full circle, converted to radians)
    let second_angle = (seconds / 60.0) as f32 * TAU;
    let minute_angle = (smooth_minutes / 60.0) as f32 * TAU;
    let hour_angle = (smooth_hours / 12.0) as f32 * TAU;

    let center = Vec2::ZERO;

    // Draw clock face circle (bold ring)
    let steps = (FACE_RING_WIDTH / 1.0).ceil() as i32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let r = CLOCK_RADIUS - t * FACE_RING_WIDTH;
        gizmos.circle_2d(center, r, FACE_COLOR);
    }

    // Draw tick marks (60 ticks, major every 5)
    for i in 0..60 {
        let angle = (i as f32 / 60.0) * TAU;
        let dir = clock_direction(angle);
        let is_major = i % 5 == 0;
        let (length, width, color) = if is_major {
            (TICK_MAJOR_LENGTH, TICK_MAJOR_WIDTH, TICK_MAJOR_COLOR)
        } else {
            (TICK_MINOR_LENGTH, TICK_MINOR_WIDTH, TICK_MINOR_COLOR)
        };
        let outer = center + dir * CLOCK_RADIUS;
        let inner = center + dir * (CLOCK_RADIUS - length);
        draw_thick_line(&mut gizmos, inner, outer, width, color);
    }

    // Draw hour hand
    let hour_dir = clock_direction(hour_angle);
    draw_thick_line(
        &mut gizmos,
        center,
        center + hour_dir * HOUR_HAND_LENGTH,
        HOUR_HAND_WIDTH,
        HOUR_COLOR,
    );

    // Draw minute hand
    let minute_dir = clock_direction(minute_angle);
    draw_thick_line(
        &mut gizmos,
        center,
        center + minute_dir * MINUTE_HAND_LENGTH,
        MINUTE_HAND_WIDTH,
        MINUTE_COLOR,
    );

    // Draw second hand
    let second_dir = clock_direction(second_angle);
    draw_thick_line(
        &mut gizmos,
        center,
        center + second_dir * SECOND_HAND_LENGTH,
        SECOND_HAND_WIDTH,
        SECOND_COLOR,
    );
    // Second hand tail (short line opposite direction)
    draw_thick_line(
        &mut gizmos,
        center,
        center - second_dir * 60.0,
        SECOND_HAND_WIDTH,
        SECOND_COLOR,
    );

    // Center dot
    gizmos.circle_2d(center, CENTER_DOT_RADIUS, CENTER_COLOR);
}

/// Draw a thick line by drawing multiple parallel lines offset perpendicular.
fn draw_thick_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, width: f32, color: Color) {
    let dir = (end - start).normalize_or(Vec2::Y);
    let perp = Vec2::new(-dir.y, dir.x);
    let steps = (width / 1.5).ceil() as i32;
    let half = width / 2.0;
    for i in 0..=steps {
        let t = if steps == 0 {
            0.0
        } else {
            i as f32 / steps as f32
        };
        let offset = perp * (t * width - half);
        gizmos.line_2d(start + offset, end + offset, color);
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
