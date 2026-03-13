use bevy::{input::common_conditions::input_pressed, prelude::*};
use bevy_small_menu::{
    CloseSmallMenu, ClosedMenu, CycleDirection, SelectedNode, SelectionCallback, SelectionEvent,
    SmallMenu, SmallMenuNode, SmallMenuPlugin,
};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerState {
    #[default]
    World,
    Menu,
}

#[derive(Clone, Copy)]
enum MainNodes {
    FastFood,
    Fruits,
    Vegetable,
}

#[derive(Clone, Copy)]
enum FastFood {
    Pizza,
    Kebab,
}

#[derive(Clone, Copy)]
enum Fruits {
    Apple,
    Banana,
    Ananas,
}

#[derive(Clone, Copy)]
enum Vegetable {
    Salat,
}

#[derive(Component)]
struct DisplaySelected;

fn main() {
    let primary_window = Window {
        title: "Bevy SmallMenu Simple".to_string(),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }))
        .add_plugins((
            SmallMenuPlugin::<MainNodes>::default(),
            SmallMenuPlugin::<FastFood>::default(),
            SmallMenuPlugin::<Fruits>::default(),
            SmallMenuPlugin::<Vegetable>::default(),
        ))
        .insert_state(PlayerState::World)
        .add_observer(get_selected_node)
        .add_observer(get_closed_main_menu)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            draw_main_nodes.run_if(input_pressed(KeyCode::Enter).and(in_state(PlayerState::World))),
        )
        .add_systems(OnExit(PlayerState::World), clean_setup)
        .add_systems(
            Update,
            (input_triggers, display_selected).run_if(in_state(PlayerState::Menu)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Text2d::new("Press Enter to Start"));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(150.),
            left: Val::Percent(40.),
            ..Default::default()
        },
        Text::new(""),
        DisplaySelected,
    ));
}

fn clean_setup(ui: Query<Entity, With<Text2d>>, mut commands: Commands) {
    for entity in ui {
        commands.entity(entity).despawn();
    }
}

fn draw_main_nodes(mut commands: Commands, mut next_state: ResMut<NextState<PlayerState>>) {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            position_type: PositionType::Absolute,
            top: Val::Percent(50.),
            left: Val::Percent(40.),
            column_gap: Val::Px(10.),
            ..Default::default()
        },
        SmallMenu::new(vec![
            SmallMenuNode::bundle(MainNodes::FastFood, Text::new("Fast Food")),
            SmallMenuNode::bundle(MainNodes::Fruits, Text::new("Fruits")),
            SmallMenuNode::bundle(MainNodes::Vegetable, Text::new("Vegetable")),
        ]),
    ));
    next_state.set(PlayerState::Menu);
}

fn input_triggers(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        commands.trigger(CycleDirection::Right);
    }
    if input.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        commands.trigger(CycleDirection::Left);
    }
    if input.just_pressed(KeyCode::Enter) {
        commands.trigger(SelectionCallback::<MainNodes>::default());
    }
    if input.just_pressed(KeyCode::Escape) {
        commands.trigger(CloseSmallMenu);
    }
}

fn get_selected_node(trigger: On<SelectionEvent<MainNodes>>, mut commands: Commands) {
    match trigger.selection {
        MainNodes::FastFood => {
            commands.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(45.),
                    left: Val::Percent(40.),
                    column_gap: Val::Px(10.),
                    ..Default::default()
                },
                SmallMenu::new(vec![
                    SmallMenuNode::bundle(FastFood::Pizza, Text::new("Pizza")),
                    SmallMenuNode::bundle(FastFood::Kebab, Text::new("Kebab")),
                ]),
            ));
        }
        MainNodes::Fruits => {
            commands.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(45.),
                    left: Val::Percent(42.),
                    column_gap: Val::Px(10.),
                    ..Default::default()
                },
                SmallMenu::new(vec![
                    SmallMenuNode::bundle(Fruits::Apple, Text::new("Apple")),
                    SmallMenuNode::bundle(Fruits::Ananas, Text::new("Ananas")),
                    SmallMenuNode::bundle(Fruits::Banana, Text::new("Banana")),
                ]),
            ));
        }
        MainNodes::Vegetable => {
            commands.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(45.),
                    left: Val::Percent(48.),
                    column_gap: Val::Px(10.),
                    ..Default::default()
                },
                SmallMenu::new(vec![SmallMenuNode::bundle(
                    Vegetable::Salat,
                    Text::new("Salat"),
                )]),
            ));
        }
    }
}

fn display_selected(
    selected: Query<&Text, With<SelectedNode>>,
    mut display_selected: Query<&mut Text, (With<DisplaySelected>, Without<SelectedNode>)>,
) {
    let mut selection = String::from("Current food: ");

    for text in &selected {
        selection.push_str(text.0.as_str());
        selection.push(' ');
    }

    let mut display = display_selected.single_mut().unwrap();
    display.0 = selection;
}

fn get_closed_main_menu(
    _: On<ClosedMenu<MainNodes>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    commands.spawn(Text2d::new("Press Enter to Start"));
    next_state.set(PlayerState::World);
}
