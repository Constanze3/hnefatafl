use crate::game::tafl::*;
use crate::game::GameState;
use bevy::utils::Duration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                UiSet
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(TaflState::Playing)),
                UiDynamicSet
                    .run_if(in_state(GameState::InGame).and_then(game_ui_is_visible))
                    .run_if(in_state(TaflState::Playing))
                    .after(indicate_turn),
            ),
        );

        app.add_event::<SetupGameUiEvent>()
            .add_event::<IndicateTurnEvent>()
            .add_event::<OnGameTimerFinishedEvent>()
            .add_systems(OnEnter(GameState::InGame), spawn_game_ui)
            .add_systems(OnExit(GameState::InGame), despawn_game_ui)
            .add_systems(
                Update,
                (
                    (setup_game_ui, indicate_turn).in_set(UiSet),
                    (rotate_loading_circle, update_game_timer).in_set(UiDynamicSet),
                ),
            )
            .insert_resource(TurnIndicators::default());
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct UiSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct UiDynamicSet;

fn game_ui_is_visible(q_game_ui: Query<&Visibility, With<GameUi>>) -> bool {
    let visibility = q_game_ui.single();
    return visibility == Visibility::Visible;
}

#[derive(Component)]
pub struct LoadingCircle {
    pub side: Side,
}

#[derive(Component)]
pub struct GameTimer {
    pub side: Side,
    pub active: bool,
    pub timer: Timer,
}

#[derive(Event)]
pub struct IndicateTurnEvent {
    pub side: Option<Side>,
}

pub fn indicate_turn(
    mut event: EventReader<IndicateTurnEvent>,
    mut q_loading_circle: Query<(&LoadingCircle, &mut Visibility)>,
    mut q_game_timer: Query<&mut GameTimer>,
) {
    for ev in event.read() {
        if let Some(side) = ev.side {
            for (loading_circle, mut visibility) in &mut q_loading_circle {
                if loading_circle.side == side {
                    *visibility = Visibility::Inherited;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }

            for mut game_timer in &mut q_game_timer {
                if game_timer.side == side {
                    game_timer.active = true;
                } else {
                    game_timer.active = false;
                }
            }
        } else {
            for (_, mut visibility) in &mut q_loading_circle {
                *visibility = Visibility::Hidden;
            }

            for mut game_timer in &mut q_game_timer {
                game_timer.active = false;
            }
        }
    }
}

#[derive(Event)]
pub struct SetupGameUiEvent {
    pub side_with_initial_turn: Side,
    pub timer_duration: Duration,
}

// Sets up the game ui and turns it visible
pub fn setup_game_ui(
    mut event: EventReader<SetupGameUiEvent>,
    mut q_game_timer: Query<(&mut GameTimer, &mut Text)>,
    mut q_loading_circle: Query<(&LoadingCircle, &mut Visibility), Without<GameUi>>,
    mut q_game_ui: Query<&mut Visibility, With<GameUi>>,
) {
    for ev in event.read() {
        for (mut game_timer, mut text) in &mut q_game_timer {
            game_timer.timer = Timer::new(ev.timer_duration, TimerMode::Once);
            set_timer_text(&game_timer.timer, &mut text);
        }

        for (loading_circle, mut loading_circle_visibility) in &mut q_loading_circle {
            if loading_circle.side == ev.side_with_initial_turn {
                *loading_circle_visibility = Visibility::Inherited;
            } else {
                *loading_circle_visibility = Visibility::Hidden;
            }
        }

        let mut game_ui_visibility = q_game_ui.single_mut();
        *game_ui_visibility = Visibility::Visible;
    }
}

#[derive(Event)]
pub struct OnGameTimerFinishedEvent {
    pub side: Side,
}

pub fn update_game_timer(
    mut q_game_timer: Query<(&mut GameTimer, &mut Text)>,
    time: Res<Time>,
    mut on_game_timer_finished_event: EventWriter<OnGameTimerFinishedEvent>,
    mut indicate_turn_event: EventWriter<IndicateTurnEvent>,
) {
    let mut finished_sides = vec![];

    for (mut game_timer, mut text) in &mut q_game_timer {
        if !game_timer.active {
            continue;
        }

        game_timer.timer.tick(time.delta());
        set_timer_text(&game_timer.timer, &mut text);

        if game_timer.timer.finished() {
            if finished_sides.contains(&game_timer.side) {
                continue;
            }

            finished_sides.push(game_timer.side);
            on_game_timer_finished_event.send(OnGameTimerFinishedEvent {
                side: game_timer.side,
            });
            indicate_turn_event.send(IndicateTurnEvent { side: None });
        }
    }
}

fn set_timer_text(timer: &Timer, text: &mut Text) {
    let remaining = timer.remaining().as_secs();
    let mins = remaining / 60;
    let secs = remaining % 60;

    text.sections[0].value = format!("{:0>2}:{:0>2}", mins, secs);
}

pub struct TurnIndicator {
    pub icon: String,
    pub background: String,
    pub loading_circle: String,
    pub side: Side,
}

#[derive(Resource)]
pub struct TurnIndicators {
    pub left: TurnIndicator,
    pub right: TurnIndicator,
}

impl Default for TurnIndicators {
    fn default() -> Self {
        Self {
            left: TurnIndicator {
                icon: "turn_indication/red_helmet.png".to_string(),
                background: "turn_indication/black_circle.png".to_string(),
                loading_circle: "turn_indication/red_loading_circle.png".to_string(),
                side: Side::Attacker,
            },
            right: TurnIndicator {
                icon: "turn_indication/blue_helmet.png".to_string(),
                background: "turn_indication/white_circle.png".to_string(),
                loading_circle: "turn_indication/blue_loading_circle.png".to_string(),
                side: Side::Defender,
            },
        }
    }
}

#[derive(Component)]
pub struct GameUi;

pub fn spawn_game_ui(
    turn_indicators: Res<TurnIndicators>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            GameUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|parent| {
            for (i, turn_indicator) in vec![&turn_indicators.left, &turn_indicators.right]
                .into_iter()
                .enumerate()
            {
                let side_dependent_style = if i == 0 {
                    Style {
                        left: Val::Percent(10.),
                        ..default()
                    }
                } else {
                    Style {
                        right: Val::Percent(10.),
                        ..default()
                    }
                };

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            flex_wrap: FlexWrap::Wrap,
                            min_width: Val::Px(150.),
                            ..side_dependent_style
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Visual
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    margin: UiRect::bottom(Val::Px(80.)),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn(ImageBundle {
                                        style: Style {
                                            position_type: PositionType::Absolute,
                                            width: Val::Px(1080. / 10.),
                                            height: Val::Px(1080. / 10.),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        image: UiImage::new(
                                            asset_server.load(turn_indicator.background.clone()),
                                        ),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent.spawn(ImageBundle {
                                            style: Style {
                                                width: Val::Px(776. / 10.),
                                                height: Val::Px(355. / 10.),
                                                ..default()
                                            },
                                            image: UiImage::new(
                                                asset_server.load(turn_indicator.icon.clone()),
                                            ),
                                            ..default()
                                        });
                                    });

                                parent.spawn((
                                    LoadingCircle {
                                        side: turn_indicator.side,
                                    },
                                    ImageBundle {
                                        style: Style {
                                            position_type: PositionType::Absolute,
                                            width: Val::Px(1314. / 10.),
                                            height: Val::Px(1314. / 10.),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        image: UiImage::new(
                                            asset_server
                                                .load(turn_indicator.loading_circle.clone()),
                                        ),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    },
                                ));
                            });

                        // Timer
                        parent.spawn((
                            TextBundle::from_section(
                                "00:00",
                                TextStyle {
                                    font: asset_server.load("fonts/NotoSansMono-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ),
                            GameTimer {
                                side: turn_indicator.side,
                                active: false,
                                timer: Timer::default(),
                            },
                        ));
                    });
            }
        });
}

pub fn despawn_game_ui(q_game_ui: Query<Entity, With<GameUi>>, mut commands: Commands) {
    let game_ui_entity = q_game_ui.single();
    commands.entity(game_ui_entity).despawn_recursive();
}

pub fn rotate_loading_circle(
    mut q_loading_circle: Query<(&mut Transform, &Visibility), With<LoadingCircle>>,
    time: Res<Time>,
) {
    for (mut transform, visibility) in &mut q_loading_circle {
        if visibility != Visibility::Hidden {
            transform.rotate_z(f32::to_radians(70.) * time.delta_seconds());
        }
    }
}
