//! State machine demo: Menu → Playing → Paused → GameOver.
//! A simple "press Space to score" game with a 10-second countdown.
//! Demonstrates States, UI with Node/Text/Button/Interaction, generic cleanup.

#![allow(clippy::type_complexity)]

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{WindowPlugin, WindowPosition, WindowResolution},
};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.08, 0.05, 0.12, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.05, 0.12);

const GAME_DURATION: f32 = 10.0;
const BUTTON_COLOR: Color = Color::srgb(0.2, 0.2, 0.35);
const BUTTON_HOVER_COLOR: Color = Color::srgb(0.3, 0.3, 0.5);
const BUTTON_PRESS_COLOR: Color = Color::srgb(0.15, 0.15, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.95);
const ACCENT_COLOR: Color = Color::srgb(0.4, 0.7, 1.0);

// --- States ---

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}

// --- Components ---

/// Marker for entities that belong to a specific state (used by generic cleanup).
#[derive(Component)]
struct StateEntity;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct FinalScoreText;

#[derive(Component)]
enum MenuButton {
    Play,
    Quit,
}

#[derive(Component)]
enum GameOverButton {
    Restart,
    MainMenu,
}

// --- Resources ---

#[derive(Resource)]
struct GameScore(u32);

#[derive(Resource)]
struct GameTimer(Timer);

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
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(OnExit(GameState::Menu), cleanup::<StateEntity>)
        .add_systems(OnEnter(GameState::Playing), setup_playing)
        .add_systems(OnExit(GameState::Playing), cleanup::<StateEntity>)
        .add_systems(OnEnter(GameState::Paused), setup_paused)
        .add_systems(OnExit(GameState::Paused), cleanup::<StateEntity>)
        .add_systems(OnEnter(GameState::GameOver), setup_game_over)
        .add_systems(OnExit(GameState::GameOver), cleanup::<StateEntity>)
        .add_systems(
            Update,
            (
                #[cfg(feature = "window-offset")]
                offset_window,
                handle_quit,
            ),
        )
        .add_systems(
            Update,
            (menu_button_system, menu_button_colors).run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            Update,
            (playing_input, update_timer, update_playing_ui).run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, paused_input.run_if(in_state(GameState::Paused)))
        .add_systems(
            Update,
            (game_over_button_system, game_over_button_colors)
                .run_if(in_state(GameState::GameOver)),
        )
        .run();
}

/// Generic cleanup system: despawn all entities with component T.
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- Menu State ---

fn setup_menu(mut commands: Commands) {
    // Root UI node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            StateEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SPACE TAPPER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(ACCENT_COLOR),
            ));

            // Subtitle
            parent.spawn((
                Text::new("Press SPACE as fast as you can!"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Play button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    MenuButton::Play,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    MenuButton::Quit,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("QUIT"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

fn menu_button_system(
    query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Play => {
                    next_state.set(GameState::Playing);
                }
                MenuButton::Quit => {
                    app_exit.write(AppExit::Success);
                }
            }
        }
    }
}

fn menu_button_colors(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<Button>, With<MenuButton>)>,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BUTTON_PRESS_COLOR),
            Interaction::Hovered => BackgroundColor(BUTTON_HOVER_COLOR),
            Interaction::None => BackgroundColor(BUTTON_COLOR),
        };
    }
}

// --- Playing State ---

fn setup_playing(mut commands: Commands) {
    commands.insert_resource(GameScore(0));
    commands.insert_resource(GameTimer(Timer::from_seconds(
        GAME_DURATION,
        TimerMode::Once,
    )));

    // HUD
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            StateEntity,
        ))
        .with_children(|parent| {
            // Timer
            parent.spawn((
                Text::new(format!("{:.1}", GAME_DURATION)),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(ACCENT_COLOR),
                TimerText,
            ));

            // Score
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                ScoreText,
            ));

            // Instructions
            parent.spawn((
                Text::new("SPACE to score  |  ESC to pause"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));
        });
}

fn playing_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut score: ResMut<GameScore>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        score.0 += 1;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

fn update_timer(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        next_state.set(GameState::GameOver);
    }
}

fn update_playing_ui(
    score: Res<GameScore>,
    timer: Res<GameTimer>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<TimerText>)>,
    mut timer_query: Query<&mut Text, (With<TimerText>, Without<ScoreText>)>,
) {
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", score.0);
    }
    if let Ok(mut text) = timer_query.single_mut() {
        let remaining = timer.0.duration().as_secs_f32() - timer.0.elapsed_secs();
        **text = format!("{:.1}", remaining.max(0.0));
    }
}

// --- Paused State ---

fn setup_paused(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            StateEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(ACCENT_COLOR),
            ));
            parent.spawn((
                Text::new("Press ESC to resume"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn paused_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}

// --- GameOver State ---

fn setup_game_over(mut commands: Commands, score: Res<GameScore>) {
    let final_score = score.0;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            StateEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new(format!("Final Score: {}", final_score)),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                FinalScoreText,
            ));

            // Restart button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    GameOverButton::Restart,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("PLAY AGAIN"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });

            // Main menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    GameOverButton::MainMenu,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("MAIN MENU"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

fn game_over_button_system(
    query: Query<(&Interaction, &GameOverButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                GameOverButton::Restart => {
                    next_state.set(GameState::Playing);
                }
                GameOverButton::MainMenu => {
                    next_state.set(GameState::Menu);
                }
            }
        }
    }
}

fn game_over_button_colors(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<Button>, With<GameOverButton>)>,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BUTTON_PRESS_COLOR),
            Interaction::Hovered => BackgroundColor(BUTTON_HOVER_COLOR),
            Interaction::None => BackgroundColor(BUTTON_COLOR),
        };
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
