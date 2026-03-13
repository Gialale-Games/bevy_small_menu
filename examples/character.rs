use bevy::{
    color::palettes::css::{BLACK, GREEN, GREY, YELLOW},
    prelude::*,
    window::WindowResolution,
};
use bevy_small_menu::{
    ChangeNodeColors, CycleDirection, NodePayload, SelectedNode, SelectionCallback, SelectionEvent,
    SmallMenu, SmallMenuNode, SmallMenuPlugin,
};

#[derive(Clone, Copy)]
enum Character {
    Bandit,
    King,
    Prince,
}

#[derive(Event, Deref)]
struct DrawArrow(Entity);

#[derive(Component)]
struct Arrow;

#[derive(Component)]
struct DisplaySelected;

fn main() {
    let primary_window = Window {
        title: "Bevy SmallMenu Character Selection".to_string(),
        resolution: WindowResolution::new(1280, 720),
        resizable: false,
        ..default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(primary_window),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(SmallMenuPlugin::<Character>::default())
        .add_observer(get_selected_node)
        .add_observer(draw_selection_arrow)
        .add_systems(Startup, setup)
        .add_systems(Update, (input_triggers, draw_flag_on_selected))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut offset = -200.;
    let menu_nodes: Vec<SmallMenuNode<Character>> = [
        (Character::Prince, "prince.png".to_string()),
        (Character::King, "king.png".to_string()),
        (Character::Bandit, "bandit.png".to_string()),
    ]
    .into_iter()
    .map(|(payload, sprite)| {
        let image = asset_server.load(sprite);
        offset += 100.;

        SmallMenuNode::with_fn(payload, move |commands, entry| {
            let mut entry = commands.entity(entry);

            entry.insert((
                Sprite::from_image(image.clone()),
                Transform::from_xyz(0. + offset, 0., 0.).with_scale(Vec3::splat(5.)),
            ));
        })
    })
    .collect();

    commands.spawn((
        SmallMenu::new(menu_nodes)
            .with_start_node(1)
            .with_colors(Color::Srgba(GREY), Color::Srgba(GREEN)),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(150.),
            left: Val::Percent(42.),
            ..Default::default()
        },
        Text::new("Player selected: "),
        DisplaySelected,
    ));
}

fn input_triggers(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        commands.trigger(CycleDirection::Right);
    }
    if input.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        commands.trigger(CycleDirection::Left);
    }
    if input.just_pressed(KeyCode::Enter) {
        commands.trigger(SelectionCallback::<Character>::default());
    }

    if input.just_pressed(KeyCode::KeyR) {
        let random_idle_color = Color::Srgba(BLACK);
        let random_selected_color = Color::Srgba(YELLOW);

        commands.trigger(ChangeNodeColors {
            new_idle_color: random_idle_color,
            new_selected_color: random_selected_color,
        });
    }
}

fn get_selected_node(
    trigger: On<SelectionEvent<Character>>,
    mut display_selected: Query<&mut Text, With<DisplaySelected>>,
) {
    let mut display = display_selected.single_mut().unwrap();
    display.0 = "Player selected: ".to_string();
    match trigger.selection {
        Character::Bandit => display.0.push_str("Bandit"),
        Character::King => display.0.push_str("King"),
        Character::Prince => display.0.push_str("Prince"),
    }
}

fn draw_selection_arrow(
    trigger: On<DrawArrow>,
    mut current_hover: Query<&mut Transform, With<Arrow>>,
    menu_entries_add: Query<&GlobalTransform, With<NodePayload<Character>>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok(entry_position) = menu_entries_add.get(**trigger) else {
        return;
    };

    match current_hover.single_mut() {
        Ok(mut flag_position) => {
            flag_position.translation.x = entry_position.translation().x;
        }
        Err(_) => {
            let posisiton = entry_position.translation();
            commands.spawn((
                Sprite::from_image(asset_server.load("arrow.png")),
                Transform::from_xyz(posisiton.x, posisiton.y + 100., 0.)
                    .with_scale(Vec3::splat(5.)),
                Arrow,
            ));
        }
    }
}

fn draw_flag_on_selected(
    menu_entries_add: Query<Entity, (Added<SelectedNode>, With<NodePayload<Character>>)>,
    mut commands: Commands,
) {
    let Ok(entry) = menu_entries_add.single() else {
        return;
    };

    commands.trigger(DrawArrow(entry));
}
