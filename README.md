# bevy_small_menu
A type safe head-less menu plugin for (Bevy Engine)[https://bevyengine.org/]. This plugin makes your UI Nodes or Sprites into interactive menus !.

<img src="/assets/simple.gif" height="250" width="250"> |
<img src="/assets/character.gif" height="250" width="250"> 

## Getting Started
Checkout the examples for working with:
- UI Nodes [Simple selection](examples/simple.rs)
- Sprites [Character selection](examples/character.rs)

## Usage
Add bevy_small_menu as plugin and your enum type:
```rust
 app.add_plugins((
  SmallMenuPlugin::<YourType>::default(),
  SmallMenuPlugin::<YourSecondType>::default(),
))
```

the menu can be initialized in two ways:

**Declarative** with ` SmallMenuNode::bundle(payload: T, bundle: B)`
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
**Imperative** with `SmallMenuNode::fn(payload: T, fn(commands: Commands, node: Entity))`
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
Use `SmallMenuNode::fn()` if you need to manipulate the node on the first render.

The `payload` is a enum variant that is declared in the plugin initaluizeton. 


### Todo's
- Declare styling of active and no-active nodes

