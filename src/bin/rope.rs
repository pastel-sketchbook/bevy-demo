//! Verlet integration rope simulation rendered with Gizmos.
//! Drag with the mouse to move the anchor point. The rope hangs under gravity
//! with distance constraints solved iteratively. FixedUpdate at 120Hz.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{PrimaryWindow, WindowPlugin, WindowPosition, WindowResolution},
};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.05, 0.03, 0.08, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.03, 0.08);

const NODE_COUNT: usize = 40;
const SEGMENT_LENGTH: f32 = 12.0;
const GRAVITY: Vec2 = Vec2::new(0.0, -980.0);
const CONSTRAINT_ITERATIONS: usize = 15;
const FIXED_HZ: f64 = 120.0;
const ROPE_COLOR: Color = Color::srgb(0.8, 0.6, 0.9);
const NODE_COLOR: Color = Color::srgb(0.9, 0.7, 1.0);
const ANCHOR_COLOR: Color = Color::srgb(1.0, 0.4, 0.4);
const NODE_RADIUS: f32 = 3.0;
const ANCHOR_RADIUS: f32 = 8.0;
const DAMPING: f32 = 0.99;

// --- Resources ---

#[derive(Resource)]
struct Rope {
    positions: Vec<Vec2>,
    old_positions: Vec<Vec2>,
    pinned: Vec<bool>, // true = position is fixed (anchor)
}

impl Rope {
    fn new(anchor: Vec2) -> Self {
        let mut positions = Vec::with_capacity(NODE_COUNT);
        let mut old_positions = Vec::with_capacity(NODE_COUNT);
        let mut pinned = vec![false; NODE_COUNT];

        for i in 0..NODE_COUNT {
            let pos = anchor + Vec2::new(0.0, -(i as f32) * SEGMENT_LENGTH);
            positions.push(pos);
            old_positions.push(pos);
        }
        pinned[0] = true; // First node is the anchor

        Self {
            positions,
            old_positions,
            pinned,
        }
    }
}

#[derive(Resource)]
struct Dragging(bool);

// --- Main ---

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
        .insert_resource(Rope::new(Vec2::new(0.0, 300.0)))
        .insert_resource(Dragging(false))
        .insert_resource(Time::<Fixed>::from_hz(FIXED_HZ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (verlet_integrate, apply_constraints).chain())
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                mouse_drag,
                draw_rope,
                handle_quit,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn verlet_integrate(mut rope: ResMut<Rope>, time: Res<Time>) {
    let dt = time.delta_secs();
    for i in 0..NODE_COUNT {
        if rope.pinned[i] {
            continue;
        }
        let current = rope.positions[i];
        let old = rope.old_positions[i];
        let velocity = (current - old) * DAMPING;
        rope.old_positions[i] = current;
        rope.positions[i] = current + velocity + GRAVITY * dt * dt;
    }
}

fn apply_constraints(mut rope: ResMut<Rope>) {
    for _ in 0..CONSTRAINT_ITERATIONS {
        for i in 0..NODE_COUNT - 1 {
            let a = rope.positions[i];
            let b = rope.positions[i + 1];
            let delta = b - a;
            let distance = delta.length();
            if distance < f32::EPSILON {
                continue;
            }
            let diff = (distance - SEGMENT_LENGTH) / distance;
            let correction = delta * 0.5 * diff;

            if !rope.pinned[i] {
                rope.positions[i] += correction;
            }
            if !rope.pinned[i + 1] {
                rope.positions[i + 1] -= correction;
            }
        }
    }
}

fn mouse_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut rope: ResMut<Rope>,
    mut dragging: ResMut<Dragging>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        dragging.0 = true;
    }
    if buttons.just_released(MouseButton::Left) {
        dragging.0 = false;
    }

    if !dragging.0 {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Move anchor to mouse position
    rope.positions[0] = world_pos;
    rope.old_positions[0] = world_pos;
}

fn draw_rope(rope: Res<Rope>, mut gizmos: Gizmos) {
    // Draw line segments
    for i in 0..NODE_COUNT - 1 {
        gizmos.line_2d(rope.positions[i], rope.positions[i + 1], ROPE_COLOR);
    }

    // Draw nodes
    for i in 1..NODE_COUNT {
        gizmos.circle_2d(rope.positions[i], NODE_RADIUS, NODE_COLOR);
    }

    // Draw anchor (larger, different color)
    gizmos.circle_2d(rope.positions[0], ANCHOR_RADIUS, ANCHOR_COLOR);
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
