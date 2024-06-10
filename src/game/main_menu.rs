use crate::game::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(Update, play_button.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuUi;

fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            MainMenuUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        bottom: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Hnefatafl",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 100.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });

            // Button
            parent
                .spawn((
                    PlayButton,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.),
                            height: Val::Px(50.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });
}

fn despawn_main_menu(q_main_menu: Query<Entity, With<MainMenuUi>>, mut commands: Commands) {
    let main_menu_entity = q_main_menu.single();
    commands.entity(main_menu_entity).despawn_recursive();
}

#[derive(Component)]
struct PlayButton;

fn play_button(
    mut q_button: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color) in &mut q_button {
        match *interaction {
            Interaction::Pressed => {
                *background_color = Color::rgb_u8(157, 79, 79).into();
                next_game_state.set(GameState::InGame);
            }
            Interaction::Hovered => {
                *background_color = Color::rgb_u8(157, 79, 79).into();
            }
            Interaction::None => {
                *background_color = Color::rgb_u8(78, 112, 165).into();
            }
        }
    }
}
