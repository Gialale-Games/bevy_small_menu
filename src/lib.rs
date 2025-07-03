use std::{marker::PhantomData, sync::Arc};

use bevy::{
    color::palettes::css::GREY,
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

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

impl<T: Clone + Send + Sync + 'static> Plugin for SmallMenuPlugin<T> {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<SmallMenuSingleton>().is_none() {
            app.add_observer(close_small_menu);
        }

        app.init_resource::<ActiveSmallMenus>()
            .init_resource::<SmallMenuSingleton>()
            .add_event::<CycleDirection<T>>()
            .add_event::<SelectionCallback<T>>()
            .add_event::<CloseSmallMenu>()
            .add_observer(cycle_commands::<T>)
            .add_observer(selection_callback::<T>)
            .add_observer(open_small_menu::<T>)
            .add_observer(closed_menu_callback::<T>);
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ActiveSmallMenus(Vec<Entity>);

#[derive(Resource, Default)]
struct SmallMenuSingleton;

#[derive(Default)]
pub struct MenuConfig {
    selected_node: usize,
}

#[derive(Component)]
#[component(on_add = color_tree_white)]
#[component(on_remove = color_tree_grey)]
pub struct Selected;

#[derive(Component)]
#[require(Transform)]
pub struct SmallMenu<T: Clone + 'static> {
    nodes: Vec<SmallMenuNode<T>>,
    config: MenuConfig,
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

#[derive(Component, Deref)]
pub struct NodePayload<T>(T);

#[derive(Component)]
#[component(on_add = color_tree_grey)]
struct GrayOnSpawn;

fn color_tree_white(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    color_tree(&mut world, entity, Color::WHITE);
}

fn color_tree_grey(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    color_tree(&mut world, entity, Color::Srgba(GREY));
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

#[derive(Event)]
pub struct SelectionEvent<T> {
    pub selection: T,
    pub menu: Entity,
    pub entry: Entity,
}

fn open_small_menu<T: Clone + Send + Sync + 'static>(
    trigger: Trigger<OnAdd, SmallMenu<T>>,
    mut commands: Commands,
    mut active: ResMut<ActiveSmallMenus>,
    query: Query<&SmallMenu<T>>,
) {
    let menu_entity = trigger.target();
    let menu = query.get(menu_entity).unwrap();

    for (i, node) in menu.nodes.iter().enumerate() {
        let entry = commands.spawn(NodePayload(node.payload.clone())).id();

        (node.spawn_fn)(&mut commands, entry);

        commands
            .entity(entry)
            .insert(GrayOnSpawn)
            .insert_if(Selected, || i == menu.config.selected_node);

        commands.entity(menu_entity).add_child(entry);
    }

    active.push(menu_entity);
}

#[derive(Event)]
pub struct NodeSelected<T>(PhantomData<T>);

#[derive(Event)]
pub enum CycleDirection<T> {
    Left(PhantomData<T>),
    Right(PhantomData<T>),
}

impl<T> CycleDirection<T> {
    pub fn left() -> Self {
        CycleDirection::Left(PhantomData)
    }
    pub fn right() -> Self {
        CycleDirection::Right(PhantomData)
    }
}

fn cycle_commands<T: Clone + Send + Sync + 'static>(
    trigger: Trigger<CycleDirection<T>>,
    active_menu: Res<ActiveSmallMenus>,
    active: Query<Entity, With<Selected>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let direction = match *trigger {
        CycleDirection::Left(_) => -1,
        CycleDirection::Right(_) => 1,
    };

    let Some(active_menu) = active_menu.last() else {
        return;
    };
    let Ok(descendants) = children_query.get(*active_menu) else {
        return;
    };

    let total_items = descendants.len() as i32;
    if total_items == 0 {
        return;
    }

    let mut current_index: i32 = 0;
    for (i, menu_node) in descendants.iter().enumerate() {
        if active.contains(menu_node) {
            current_index = i as i32;
            commands.entity(menu_node).remove::<Selected>();
            break;
        }
    }

    let next_index = (current_index + direction).rem_euclid(total_items);

    if let Some(next_entity) = descendants.get(next_index as usize) {
        commands.entity(*next_entity).insert(Selected);
    }
}

#[derive(Event)]
pub struct CloseSmallMenu;

fn close_small_menu(
    _: Trigger<CloseSmallMenu>,
    mut commands: Commands,
    mut active_menus: ResMut<ActiveSmallMenus>,
) {
    let Some(menu_entity) = active_menus.pop() else {
        return;
    };

    commands.entity(menu_entity).despawn();
}

#[derive(Event)]
pub struct ClosedMenu<T>(PhantomData<T>);

fn closed_menu_callback<T: Clone + Send + Sync + 'static>(
    _: Trigger<OnRemove, SmallMenu<T>>,
    mut commands: Commands,
) {
    commands.trigger(ClosedMenu::<T>(PhantomData));
}

#[derive(Event)]
pub struct SelectionCallback<T>(PhantomData<T>);

impl<T> Default for SelectionCallback<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn selection_callback<T: Clone + Send + Sync + 'static>(
    _: Trigger<SelectionCallback<T>>,
    mut commands: Commands,
    active: Res<ActiveSmallMenus>,
    children: Query<&Children>,
    menu_query: Query<&NodePayload<T>, With<Selected>>,
) {
    let Some(menu) = active.last() else {
        return;
    };
    let Ok(children) = children.get(*menu) else {
        return;
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
}
