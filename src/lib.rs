use std::{marker::PhantomData, sync::Arc};

use bevy::{
    color::palettes::css::GREY,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

#[derive(Resource, Deref, DerefMut, Default)]
struct ActiveSmallMenus(Vec<Entity>);

#[derive(Resource, Default)]
struct SmallMenuSingleton;

#[derive(Event)]
pub struct SelectionCallback<T>(PhantomData<T>);

impl<T> Default for SelectionCallback<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Event)]
pub struct SelectionEvent<T> {
    pub selection: T,
    pub menu: Entity,
    pub entry: Entity,
}

#[derive(Event)]
pub struct ClosedMenu<T>(PhantomData<T>);

#[derive(Event)]
pub struct CloseSmallMenu;

#[derive(Event)]
pub enum CycleDirection {
    Left,
    Right,
}

#[derive(Event)]
pub struct NodeSelected<T>(PhantomData<T>);

#[derive(Event)]
pub struct ChangeNodeColors {
    pub new_idle_color: Color,
    pub new_selected_color: Color,
}

#[derive(Component, Deref)]
pub struct NodePayload<T>(T);

#[derive(Component)]
#[component(storage = "SparseSet")]
#[component(on_add = selected_node_hook)]
#[component(on_insert = selected_node_hook)]
pub struct SelectedNode;

#[derive(Component)]
pub struct SelectedNodeColor(pub Color);

impl Default for SelectedNodeColor {
    fn default() -> Self {
        Self(Color::WHITE)
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
#[component(on_add = idle_node_hook)]
#[component(on_insert = idle_node_hook)]
pub struct IdleNode;

#[derive(Component)]
pub struct IdleNodeColor(pub Color);

impl Default for IdleNodeColor {
    fn default() -> Self {
        Self(Color::Srgba(GREY))
    }
}

fn selected_node_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let Some(node_color) = world.entity(entity).get::<SelectedNodeColor>() else {
        return;
    };

    let active_color = node_color.0;
    color_tree(&mut world, entity, active_color);
}

fn idle_node_hook(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let Some(node_color) = world.entity(entity).get::<IdleNodeColor>() else {
        return;
    };
    let idle_color = node_color.0;
    color_tree(&mut world, entity, idle_color);
}

fn color_tree(world: &mut DeferredWorld, root: Entity, color: Color) {
    let mut stack = vec![root];

    while let Some(entity) = stack.pop() {
        if let Some(mut sprite) = world.entity_mut(entity).get_mut::<Sprite>() {
            sprite.color = color;
        }

        if let Some(mut text_color) = world.entity_mut(entity).get_mut::<TextColor>() {
            text_color.0 = color;
        }

        let Some(children) = world.entity(entity).get::<Children>() else {
            continue;
        };

        for child in children.iter() {
            stack.push(child);
        }
    }
}

#[derive(Default)]
pub struct MenuConfig {
    selected_node: usize,
    idle_color: Option<Color>,
    selected_color: Option<Color>,
}

pub struct SmallMenuPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> Default for SmallMenuPlugin<T> {
    fn default() -> Self {
        SmallMenuPlugin {
            _marker: PhantomData,
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct SmallMenu<T: Clone + 'static> {
    nodes: Vec<SmallMenuNode<T>>,
    config: MenuConfig,
}

pub struct SmallMenuNode<T> {
    pub payload: T,
    pub spawn_fn: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

impl<T> SmallMenuNode<T> {
    pub fn bundle<B>(payload: T, bundle: B) -> Self
    where
        B: Bundle + Clone + Send + Sync + 'static,
    {
        let spawn_fn = Arc::new(move |commands: &mut Commands, entry: Entity| {
            commands.entity(entry).insert(bundle.clone());
        });
        SmallMenuNode { payload, spawn_fn }
    }

    pub fn with_fn<F>(payload: T, f: F) -> Self
    where
        F: Fn(&mut Commands, Entity) + Send + Sync + 'static,
    {
        SmallMenuNode {
            payload,
            spawn_fn: Arc::new(f),
        }
    }
}

impl<T: Clone + 'static> SmallMenu<T> {
    pub fn new(nodes: Vec<SmallMenuNode<T>>) -> Self {
        Self {
            nodes,
            config: MenuConfig::default(),
        }
    }

    pub fn with_start_node(mut self, selected_node: usize) -> Self {
        self.config.selected_node = selected_node;
        self
    }

    pub fn with_colors(mut self, idle_color: Color, selected_color: Color) -> Self {
        self.config.idle_color = Some(idle_color);
        self.config.selected_color = Some(selected_color);
        self
    }
}

impl<T: Clone + Send + Sync + 'static> Plugin for SmallMenuPlugin<T> {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<SmallMenuSingleton>().is_none() {
            app.add_observer(close_small_menu)
                .add_observer(cycle_commands)
                .add_observer(cycle_commands)
                .add_observer(change_node_colors);
        }

        app.init_resource::<ActiveSmallMenus>()
            .init_resource::<SmallMenuSingleton>()
            .add_observer(selection_callback::<T>)
            .add_observer(open_small_menu::<T>)
            .add_observer(closed_menu_callback::<T>);
    }
}

fn open_small_menu<T: Clone + Send + Sync + 'static>(
    trigger: On<Add, SmallMenu<T>>,
    mut commands: Commands,
    mut active: ResMut<ActiveSmallMenus>,
    query: Query<&SmallMenu<T>>,
) -> Result {
    let menu_entity = trigger.entity;
    let menu = query.get(menu_entity)?;

    for (i, node) in menu.nodes.iter().enumerate() {
        let entry = commands.spawn(NodePayload(node.payload.clone())).id();

        (node.spawn_fn)(&mut commands, entry);
        let selected = i == menu.config.selected_node;

        // Use provided colors or defaults
        let selected_node_color = menu
            .config
            .selected_color
            .map_or_else(SelectedNodeColor::default, SelectedNodeColor);
        let idle_node_color = menu
            .config
            .idle_color
            .map_or_else(IdleNodeColor::default, IdleNodeColor);

        commands
            .entity(entry)
            .insert((selected_node_color, idle_node_color))
            .insert_if(SelectedNode, || selected)
            .insert_if(IdleNode, || !selected);

        commands.entity(menu_entity).add_child(entry);
    }

    active.push(menu_entity);
    Ok(())
}

fn cycle_commands(
    trigger: On<CycleDirection>,
    active_menu: Res<ActiveSmallMenus>,
    active: Query<Entity, With<SelectedNode>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) -> Result {
    let direction = match *trigger {
        CycleDirection::Left => -1,
        CycleDirection::Right => 1,
    };

    let Some(active_menu) = active_menu.last() else {
        return Ok(());
    };
    let Ok(descendants) = children_query.get(*active_menu) else {
        return Ok(());
    };

    let total_items = descendants.len() as i32;
    if total_items == 0 {
        return Ok(());
    }

    let mut current_index: i32 = 0;
    for (i, menu_node) in descendants.iter().enumerate() {
        if active.contains(menu_node) {
            current_index = i as i32;
            commands
                .entity(menu_node)
                .remove::<SelectedNode>()
                .insert(IdleNode);
            break;
        }
    }

    let next_index = (current_index + direction).rem_euclid(total_items);

    if let Some(next_entity) = descendants.get(next_index as usize) {
        commands
            .entity(*next_entity)
            .insert(SelectedNode)
            .remove::<IdleNode>();
    }
    Ok(())
}

fn close_small_menu(
    _: On<CloseSmallMenu>,
    mut commands: Commands,
    mut active_menus: ResMut<ActiveSmallMenus>,
) -> Result {
    let Some(menu_entity) = active_menus.pop() else {
        return Ok(());
    };

    commands.entity(menu_entity).despawn();
    Ok(())
}

fn closed_menu_callback<T: Clone + Send + Sync + 'static>(
    _: On<Remove, SmallMenu<T>>,
    mut commands: Commands,
) {
    commands.trigger(ClosedMenu::<T>(PhantomData));
}

fn selection_callback<T: Clone + Send + Sync + 'static>(
    _: On<SelectionCallback<T>>,
    mut commands: Commands,
    active: Res<ActiveSmallMenus>,
    children: Query<&Children>,
    menu_query: Query<&NodePayload<T>, With<SelectedNode>>,
) -> Result {
    let Some(menu) = active.last() else {
        return Ok(());
    };
    let Ok(children) = children.get(*menu) else {
        return Ok(());
    };

    for child in children.iter() {
        let Ok(selected) = menu_query.get(child) else {
            continue;
        };

        commands.trigger(SelectionEvent {
            selection: (**selected).clone(),
            menu: *menu,
            entry: child,
        });
    }
    Ok(())
}

fn change_node_colors(
    trigger: On<ChangeNodeColors>,
    mut node_colors: Query<(
        Entity,
        Option<&SelectedNode>,
        &mut SelectedNodeColor,
        &mut IdleNodeColor,
    )>,
    mut commands: Commands,
) -> Result {
    for (entity, has_selected, mut selected_color, mut idle_color) in node_colors.iter_mut() {
        selected_color.0 = trigger.new_selected_color;
        idle_color.0 = trigger.new_idle_color;

        // Re-apply the SelectedNode/IdleNode component to trigger the hook and update sprite/text colors
        // This ensures the visual update propagates immediately.
        match has_selected {
            Some(_) => {
                commands.entity(entity).insert(SelectedNode);
            }
            None => {
                commands.entity(entity).insert(IdleNode);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::palettes::css::{BLUE, GREEN, RED, YELLOW};

    #[derive(Clone, Debug, PartialEq)]
    enum TestPayload {
        Payload1,
        Payload2,
        Payload3,
    }

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SmallMenuPlugin::<TestPayload>::default());
        app
    }

    fn create_test_menu_nodes() -> Vec<SmallMenuNode<TestPayload>> {
        vec![
            SmallMenuNode::bundle(
                TestPayload::Payload1,
                (Sprite::default(), TextColor(Color::WHITE)),
            ),
            SmallMenuNode::bundle(
                TestPayload::Payload2,
                (Sprite::default(), TextColor(Color::WHITE)),
            ),
            SmallMenuNode::bundle(
                TestPayload::Payload3,
                (Sprite::default(), TextColor(Color::WHITE)),
            ),
        ]
    }

    #[test]
    fn test_menu_creation() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let active_menus = app.world().resource::<ActiveSmallMenus>();
        assert_eq!(active_menus.len(), 1);
        assert_eq!(active_menus[0], menu_entity);

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        assert_eq!(children.len(), 3);

        let mut selected_count = 0;
        let mut idle_count = 0;

        for child in children.iter() {
            if app.world().entity(child).contains::<SelectedNode>() {
                selected_count += 1;
            }
            if app.world().entity(child).contains::<IdleNode>() {
                idle_count += 1;
            }
        }

        assert_eq!(selected_count, 1);
        assert_eq!(idle_count, 2);
    }

    #[test]
    fn test_menu_with_custom_start_node() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes).with_start_node(2);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        let third_child = children[2];

        assert!(app.world().entity(third_child).contains::<SelectedNode>());
        assert!(!app.world().entity(third_child).contains::<IdleNode>());
    }

    #[test]
    fn test_menu_with_custom_colors() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes).with_colors(Color::Srgba(RED), Color::Srgba(BLUE));

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        let first_child = children[0];

        let selected_color = app
            .world()
            .entity(first_child)
            .get::<SelectedNodeColor>()
            .expect("first child should have SelectedNodeColor");
        let idle_color = app
            .world()
            .entity(first_child)
            .get::<IdleNodeColor>()
            .expect("first child should have IdleNodeColor");

        assert_eq!(selected_color.0, Color::Srgba(BLUE));
        assert_eq!(idle_color.0, Color::Srgba(RED));
    }

    #[test]
    fn test_cycle_right() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        app.world_mut().trigger(CycleDirection::Right);
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");

        assert!(app.world().entity(children[0]).contains::<IdleNode>());
        assert!(app.world().entity(children[1]).contains::<SelectedNode>());
        assert!(app.world().entity(children[2]).contains::<IdleNode>());
    }

    #[test]
    fn test_cycle_left() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        app.world_mut().trigger(CycleDirection::Left);
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");

        assert!(app.world().entity(children[0]).contains::<IdleNode>());
        assert!(app.world().entity(children[1]).contains::<IdleNode>());
        assert!(app.world().entity(children[2]).contains::<SelectedNode>());
    }

    #[test]
    fn test_cycle_wrap_around() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        app.world_mut().trigger(CycleDirection::Right);
        app.update();
        app.world_mut().trigger(CycleDirection::Right);
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        assert!(app.world().entity(children[2]).contains::<SelectedNode>());

        app.world_mut().trigger(CycleDirection::Right);
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        assert!(app.world().entity(children[0]).contains::<SelectedNode>());
        assert!(app.world().entity(children[1]).contains::<IdleNode>());
        assert!(app.world().entity(children[2]).contains::<IdleNode>());
    }

    #[test]
    fn test_selection_callback() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        app.add_observer(|trigger: On<SelectionEvent<TestPayload>>| {
            let event = trigger.event();
            assert_eq!(event.selection, TestPayload::Payload1)
        });

        app.world_mut().spawn(menu);
        app.update();

        app.world_mut()
            .trigger(SelectionCallback::<TestPayload>::default());
        app.update();

        app.world_mut().trigger(CycleDirection::Right);
        app.update();

        app.add_observer(|trigger: On<SelectionEvent<TestPayload>>| {
            let event = trigger.event();
            assert_eq!(event.selection, TestPayload::Payload2)
        });
    }

    #[test]
    fn test_close_menu() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let active_menus = app.world().resource::<ActiveSmallMenus>();
        assert_eq!(active_menus.len(), 1);
        assert!(app.world().get_entity(menu_entity).is_ok());

        app.world_mut().trigger(CloseSmallMenu);
        app.update();

        let active_menus = app.world().resource::<ActiveSmallMenus>();
        assert_eq!(active_menus.len(), 0);
        assert!(app.world().get_entity(menu_entity).is_err());
    }

    #[test]
    fn test_multiple_menus() {
        let mut app = setup_test_app();
        let nodes1 = create_test_menu_nodes();
        let nodes2 = create_test_menu_nodes();

        let menu1 = SmallMenu::new(nodes1);
        let menu2 = SmallMenu::new(nodes2);

        let menu_entity1 = app.world_mut().spawn(menu1).id();
        app.update();
        let menu_entity2 = app.world_mut().spawn(menu2).id();
        app.update();

        let active_menus = app.world().resource::<ActiveSmallMenus>();
        assert_eq!(active_menus.len(), 2);
        assert_eq!(active_menus[0], menu_entity1);
        assert_eq!(active_menus[1], menu_entity2);

        app.world_mut().trigger(CloseSmallMenu);
        app.update();

        let active_menus = app.world().resource::<ActiveSmallMenus>();
        assert_eq!(active_menus.len(), 1);
        assert_eq!(active_menus[0], menu_entity1);
    }

    #[test]
    fn test_change_node_colors() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes);

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        app.world_mut().trigger(ChangeNodeColors {
            new_idle_color: Color::Srgba(RED),
            new_selected_color: Color::Srgba(BLUE),
        });
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");

        for child in children.iter() {
            let selected_color = app
                .world()
                .entity(child)
                .get::<SelectedNodeColor>()
                .expect("child should have SelectedNodeColor");
            let idle_color = app
                .world()
                .entity(child)
                .get::<IdleNodeColor>()
                .expect("child should have IdleNodeColor");

            assert_eq!(selected_color.0, Color::Srgba(BLUE));
            assert_eq!(idle_color.0, Color::Srgba(RED));
        }
    }

    #[test]
    fn test_color_hooks_update_sprites() {
        let mut app = setup_test_app();
        let nodes = create_test_menu_nodes();
        let menu = SmallMenu::new(nodes).with_colors(Color::Srgba(RED), Color::Srgba(BLUE));

        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        let selected_child = children[0];
        let idle_child = children[1];

        let selected_sprite = app
            .world()
            .entity(selected_child)
            .get::<Sprite>()
            .expect("selected child should have Sprite");
        let idle_sprite = app
            .world()
            .entity(idle_child)
            .get::<Sprite>()
            .expect("idle child should have Sprite");

        assert_eq!(selected_sprite.color, Color::Srgba(BLUE));
        assert_eq!(idle_sprite.color, Color::Srgba(RED));

        let selected_text = app
            .world()
            .entity(selected_child)
            .get::<TextColor>()
            .expect("selected child should have TextColor");
        let idle_text = app
            .world()
            .entity(idle_child)
            .get::<TextColor>()
            .expect("idle child should have TextColor");

        assert_eq!(selected_text.0, Color::Srgba(BLUE));
        assert_eq!(idle_text.0, Color::Srgba(RED));
    }

    #[test]
    fn test_custom_spawn_function() {
        let mut app = setup_test_app();

        let custom_node = SmallMenuNode::with_fn(TestPayload::Payload2, |commands, entity| {
            commands.entity(entity).insert((
                Sprite {
                    color: Color::Srgba(GREEN),
                    ..default()
                },
                TextColor(Color::Srgba(YELLOW)),
                Name::new("CustomNode"),
            ));
        });

        let menu = SmallMenu::new(vec![custom_node]);
        let menu_entity = app.world_mut().spawn(menu).id();
        app.update();

        let children = app
            .world()
            .entity(menu_entity)
            .get::<Children>()
            .expect("menu entity should have children");
        let child = children[0];

        assert!(app.world().entity(child).contains::<Name>());
        let name = app
            .world()
            .entity(child)
            .get::<Name>()
            .expect("child should have Name");
        assert_eq!(name.as_str(), "CustomNode");
    }
}
