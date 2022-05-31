// based on https://github.com/mwbryant/rpg-bevy-tutorial/tree/tutorial6

use crate::AppState;
use bevy::{prelude::*, ui::FocusPolicy};

struct UiAssets {
    font: Handle<Font>,
    button: Handle<Image>,
    button_hovered: Handle<Image>,
    button_pressed: Handle<Image>,
}

struct UiCamera {
    camera_entity: Entity,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_menu)
            .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(despawn_menu))
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(success))
            .add_system(button_press_system);
    }
}

fn despawn_menu(
    mut commands: Commands,
    button_query: Query<Entity, With<Button>>,
    ui_camera: Res<UiCamera>,
) {
    for entity in button_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.entity(ui_camera.camera_entity).despawn_recursive();
}

fn success() {
    println!("Udało się zmienić state.");
}

fn setup_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let ui_assets = UiAssets {
        font: assets.load("QuattrocentoSans-Bold.ttf"),
        button: assets.load("button.png"),
        button_hovered: assets.load("button.png"),
        button_pressed: assets.load("button_pressed.png"),
    };

    let camera_entity = commands.spawn_bundle(UiCameraBundle::default()).id();

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Percent(20.0), Val::Percent(10.0)),
                margin: Rect::all(Val::Auto),
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    image: ui_assets.button.clone().into(),
                    ..Default::default()
                })
                .insert(FocusPolicy::Pass)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Start Game",
                            TextStyle {
                                font: ui_assets.font.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        focus_policy: FocusPolicy::Pass,
                        ..Default::default()
                    });
                });
        });

    commands.insert_resource(ui_assets);
    commands.insert_resource(UiCamera { camera_entity });
}

fn button_press_system(
    buttons: Query<(&Children, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<State<AppState>>,
    mut image_query: Query<&mut UiImage>,
    ui_assets: Res<UiAssets>,
) {
    for (children, interaction) in buttons.iter() {
        let child = children.iter().next().unwrap();
        let mut image = image_query.get_mut(*child).unwrap();
        match interaction {
            Interaction::Clicked => {
                image.0 = ui_assets.button_pressed.clone();
                state
                    .set(AppState::InGame)
                    .expect("Couldn't switch state to InGame");
            }
            Interaction::Hovered => {
                image.0 = ui_assets.button_hovered.clone();
            }
            Interaction::None => {
                image.0 = ui_assets.button.clone();
            }
        }
        println!("{:?}", state.current());
    }
}
