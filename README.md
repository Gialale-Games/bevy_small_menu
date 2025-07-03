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

## Usage
To use `bevy_small_menu`, add it as a plugin to your Bevy` App` for each enum type you want to use as a menu payload
```rust
 app.add_plugins((
  SmallMenuPlugin::<YourType>::default(),
  SmallMenuPlugin::<YourSecondType>::default(),
))
```

A menu can be initialized in two primary ways:

**Declarative**

Use `SmallMenuNode::bundle(payload: T, bundle: B)` when you want to declaratively define your menu nodes with Bevy bundles.
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
**Imperative**

Opt for `SmallMenuNode::with_fn(payload: T, setup_fn: Fn(Commands, Entity))` when you need more control and wish to imperatively manipulate the spawned node during its initial setup.

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

###  Todo's
- Declare styling for active and inactive nodes
- Add tests
