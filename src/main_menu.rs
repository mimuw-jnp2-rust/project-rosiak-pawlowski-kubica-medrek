// Based on https://github.com/bevyengine/bevy/blob/main/examples/games/game_menu.rs

use crate::AppState;
use bevy::{app::AppExit, prelude::*};

// Color constants.
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag component used to tag entities added on the main menu screen.
#[derive(Component)]
struct OnMenuScreen;

// Tag component used to mark which setting is currently selected.
#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click.
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

// UI camera entity saved in resources to despawn it easily.
struct UiCamera {
    camera_entity: Entity,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(main_menu_setup))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(menu_action)
                    .with_system(button_system),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::MainMenu).with_system(despawn_screen::<OnMenuScreen>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::PlayerDied).with_system(player_died_menu_setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::PlayerDied)
                    .with_system(menu_action)
                    .with_system(button_system),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::PlayerDied)
                    .with_system(despawn_screen::<OnMenuScreen>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::GameEnded).with_system(game_ended_menu_setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::GameEnded)
                    .with_system(menu_action)
                    .with_system(button_system),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::GameEnded).with_system(despawn_screen::<OnMenuScreen>),
            );
    }
}
// This system handles changing all buttons color based on mouse interaction.
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in interaction_query.iter_mut() {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>, text: &str) {
    // We spawn the camera and save it in resources.
    let camera_entity = commands.spawn_bundle(UiCameraBundle::default()).id();
    commands.insert_resource(UiCamera { camera_entity });
    let font = asset_server.load("QuattrocentoSans-Bold.ttf");

    // Common style for all buttons on the screen.
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: Rect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::GRAY.into(),
            ..default()
        })
        .insert(OnMenuScreen)
        .with_children(|parent| {
            // Display the game name.
            parent.spawn_bundle(TextBundle {
                style: Style {
                    margin: Rect::all(Val::Px(50.0)),
                    ..default()
                },
                text: Text::with_section(
                    text,
                    TextStyle {
                        font: font.clone(),
                        font_size: 80.0,
                        color: TEXT_COLOR,
                    },
                    Default::default(),
                ),
                ..default()
            });

            // Display "new game" button.
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::Play)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "New Game",
                            button_text_style.clone(),
                            Default::default(),
                        ),
                        ..default()
                    });
                });

            // Display "quit" button.
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::Quit)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section("Quit", button_text_style, Default::default()),
                        ..default()
                    });
                });
        });
}

// Different menu setups based on current state of the game.

fn main_menu_setup(commands: Commands, asset_server: Res<AssetServer>) {
    menu_setup(commands, asset_server, "Super game");
}

fn player_died_menu_setup(commands: Commands, asset_server: Res<AssetServer>) {
    menu_setup(commands, asset_server, "Try again?");
}

fn game_ended_menu_setup(commands: Commands, asset_server: Res<AssetServer>) {
    menu_setup(commands, asset_server, "Congrats! You won!");
}

// Handle action on buttons.
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<State<AppState>>,
) {
    for (interaction, menu_button_action) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    game_state.set(AppState::InGame).unwrap();
                }
            }
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component.
fn despawn_screen<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut commands: Commands,
    ui_camera: Res<UiCamera>,
) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.entity(ui_camera.camera_entity).despawn_recursive();
}
