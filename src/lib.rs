//! Shared helpers for bevy-demo binaries.
//!
//! Re-exports common Bevy types and provides shared constants, systems,
//! resources, and components used across all demo binaries.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;

// --- Re-exports ---

pub use bevy::app::AppExit;
pub use bevy::math::ShapeSample;
pub use bevy::prelude::*;
pub use bevy::window::{PrimaryWindow, WindowPlugin, WindowPosition, WindowResolution};
pub use rand::rngs::SmallRng;
pub use rand::{Rng, SeedableRng};

// --- Constants ---

pub const WINDOW_WIDTH: f32 = 1606.0;
pub const WINDOW_HEIGHT: f32 = 1036.0;

// --- Helpers ---

/// Construct the background `Color` with transparent support.
///
/// When the `transparent` feature is enabled the alpha channel is preserved;
/// otherwise the colour is fully opaque and `_alpha` is ignored.
#[allow(unused_variables)]
pub const fn background_color(r: f32, g: f32, b: f32, alpha: f32) -> Color {
    #[cfg(feature = "transparent")]
    {
        Color::srgba(r, g, b, alpha)
    }
    #[cfg(not(feature = "transparent"))]
    {
        Color::srgb(r, g, b)
    }
}

/// Return the standard borderless window used by every demo.
pub fn default_window() -> Window {
    Window {
        decorations: false,
        #[cfg(feature = "transparent")]
        transparent: true,
        #[cfg(feature = "transparent")]
        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        ..default()
    }
}

// --- Systems ---

/// Quit the application when the Q key is held.
pub fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}

/// Move the window to a fixed development offset (160, 88).
/// Runs once via a `Local<bool>` guard.
#[cfg(feature = "window-offset")]
pub fn offset_window(mut windows: Query<&mut Window>, mut done: Local<bool>) {
    if *done {
        return;
    }
    for mut window in windows.iter_mut() {
        window.position = WindowPosition::At(IVec2::new(160, 88));
        info!("Window positioned at: (160, 88)");
        *done = true;
    }
}

// --- Resources ---

/// Wrapper around `SmallRng` used as a global random source.
#[derive(Resource)]
pub struct RandomSource(pub SmallRng);

// --- Components ---

/// 2D velocity component shared across several demos.
#[derive(Component)]
pub struct Velocity(pub Vec2);
