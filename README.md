> [!WARNING]
> This plugin is currently in an early development phase. The API is subject to change without prior notice as features are added and refined. Your feedback is welcome!
# bevy_small_menu
A type safe head-less menu plugin for [Bevy Engine](https://bevyengine.org/). This plugin transforms your UI Nodes or Sprites into interactive menus!

<img src="/assets/simple.gif" height="250" width="250"> |
<img src="/assets/character.gif" height="250" width="250">

## Getting Started
Dive into the examples to see bevy_small_menu in action::
- UI Nodes [Simple Selection example](examples/simple.rs)
- Sprites [Character Selection example](examples/character.rs)

### Initialization
To use `bevy_small_menu`, add it as a plugin to your Bevy `App` for each enum type you want to use as a menu payload
```rust
 app.add_plugins((
  SmallMenuPlugin::<YourType>::default(),
  SmallMenuPlugin::<YourSecondType>::default(),
))
```
A menu can be initialized in two primary ways:

**`SmallMenuNode::bundle`**

Use `SmallMenuNode::bundle(payload: T, bundle: B)` if your menu consist of [UI Nodes](https://docs.rs/bevy/latest/bevy/ui/struct.Node.html).
```rust
fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        SmallMenu::new(vec![
            SmallMenuNode::bundle(YourType::Variant1, Text::new("Variant 1")),
            SmallMenuNode::bundle(YourType::Variant2, Text::new("Variant 2")),
            SmallMenuNode::bundle(YourType::Variant3, Text::new("Variant 3")),
        ]),
    ));
}
```
**`SmallMenuNode::with_fn`**

Use `SmallMenuNode::with_fn(payload: T, setup_fn: Fn(Commands, Entity))` if your menu consist of [Sprites](https://docs.rs/bevy/latest/bevy/sprite/index.html). This allows manipulation of the spawned node during its initial setup.

```rust
fn setup(mut commands: Commands) {
    let menu_nodes: Vec<SmallMenuNode<YourType>> = entities
    .into_iter()
    .map(|(payload, sprite)| {
        let image = asset_server.load(sprite);
        offset += 100.;

        SmallMenuNode::with_fn(payload, move |commands, node| {
            let mut node = commands.entity(node);

            node.insert((
                Sprite::from_image(image.clone()),
                Transform::from_xyz(0. + offset, 0., 0.).with_scale(Vec3::splat(5.)),
            ));
        })
    })
    .collect();

    commands.spawn((
        SmallMenu::new(menu_nodes).with_start_node(1),
    ));
}
```
The `payload` is an enum variant that you declare when adding the plugin to your App.

### Usage

**Cycling through nodes**

Cycling though nodes of an active menu can be done with the triggers `CycleDirection::Right` and `CycleDirection::Left`.
```rust
if input.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
    commands.trigger(CycleDirection::Right);
}
if input.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
    commands.trigger(CycleDirection::Left);
}
```

**Getting the selected node**

Register an observer to handle selection events:
```rust
fn selction_event(
    trigger: Trigger<SelectionEvent<T>>,
)
```
Then, trigger a request to get the selected node using:
```rust
commands.trigger(SelectionCallback::<T>::default())
```


**Chaging colors**

Each node has a `SelectedNodeColor(pub Color)` and `IdleNodeColor(pub Color)` component.

These can be changed in the initialization:
```rust
SmallMenu::new(menu_nodes)
    .with_colors(Color::Srgba(GREY), Color::Srgba(GREEN)),
```

or during the runtime with `ChangeNodeColors`:

```rust
commands.trigger(ChangeNodeColors {
    new_idle_color: random_idle_color,
    new_selected_color: random_selected_color,
});
```

**Closing a menu**
The event will close the last initialized menu,
```rust
commands.trigger(CloseSmallMenu)
```

The plugin will then trigger the event `ClosedMenu` when a menu is closed.
```rust
_: Trigger<ClosedMenu<MainNodes>>,
```

### Bevy Version Compatibility
| bevy_behave | bevy |
| ----------- | ---- |
| 0.1         | 0.16 |
