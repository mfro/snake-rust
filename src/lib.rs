use std::{collections::VecDeque, mem::swap};

use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResolution,
};
use rand::Rng;
use wasm_bindgen::prelude::*;

const GRID_SCALE: f32 = 10.0;
const WIDTH: usize = 50;
const HEIGHT: usize = 40;

#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    GRID_SCALE * WIDTH as f32 - 1.0,
                    GRID_SCALE * HEIGHT as f32 - 1.0,
                ),
                resizable: false,
                title: "snake".to_owned(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(PreStartup, setup)
        .add_systems(Startup, setup_game)
        .add_systems(Update, (input, update).chain())
        .run();
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add<Offset> for Position {
    type Output = Position;

    fn add(self, rhs: Offset) -> Self::Output {
        Self {
            x: (self.x as isize + rhs.x) as usize,
            y: (self.y as isize + rhs.y) as usize,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Offset {
    x: isize,
    y: isize,
}

impl Offset {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

impl std::ops::Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Resource)]
struct Game {
    dead: bool,

    food: Option<SnakeFood>,
    player: Snake,
    tick_timer: Timer,
    input_queue: VecDeque<Offset>,
}

struct Snake {
    nodes: Vec<SnakeNode>,
    facing: Offset,
}

struct SnakeNode {
    entity: Entity,
    position: Position,
}

struct SnakeFood {
    entity: Entity,
    position: Position,
}

fn get_transform(position: Position) -> Transform {
    Transform::from_xyz(
        (position.x as f32 - (WIDTH / 2) as f32 + 0.5) * GRID_SCALE,
        -(position.y as f32 - (HEIGHT / 2) as f32 + 0.5) * GRID_SCALE,
        0.0,
    )
}

fn is_out_of_bounds(position: Position) -> bool {
    position.x >= WIDTH || position.y >= HEIGHT
}

fn input(
    mut cmd: Commands,
    transforms: Query<&mut Transform>,
    input: Res<ButtonInput<KeyCode>>,
    spawner: Res<Spawner>,
    mut game: ResMut<Game>,
    mut exit: EventWriter<AppExit>,
) {
    if !game.dead {
        if input.just_pressed(KeyCode::ArrowUp) {
            game.input_queue.push_back(Offset::new(0, -1));
        }
        if input.just_pressed(KeyCode::ArrowDown) {
            game.input_queue.push_back(Offset::new(0, 1));
        }
        if input.just_pressed(KeyCode::ArrowRight) {
            game.input_queue.push_back(Offset::new(1, 0));
        }
        if input.just_pressed(KeyCode::ArrowLeft) {
            game.input_queue.push_back(Offset::new(-1, 0));
        }
    }

    if input.just_released(KeyCode::KeyR) {
        cleanup_game(&mut cmd, &*game);
        setup_game(cmd, transforms, spawner);
    }

    if input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn update(
    mut cmd: Commands,
    mut transforms: Query<&mut Transform>,
    spawner: Res<Spawner>,
    mut game: ResMut<Game>,
    time: Res<Time>,
) {
    if !game.dead && game.tick_timer.tick(time.delta()).just_finished() {
        while let Some(next) = game.input_queue.pop_front() {
            if next != game.player.facing && next != -game.player.facing {
                game.player.facing = next;
                break;
            }
        }

        let head_position = game.player.nodes.last().unwrap().position;
        let next_position = head_position + game.player.facing;

        if Some(next_position) == game.food.as_ref().map(|f| f.position) {
            let node = spawner.new_node(&mut cmd, next_position);

            game.player.nodes.push(node);

            new_food(&mut cmd, &mut transforms, &spawner, &mut *game);
        } else {
            let mut position = next_position;

            for node in game.player.nodes.iter_mut().rev() {
                swap(&mut position, &mut node.position);
                *transforms.get_mut(node.entity).unwrap() = get_transform(node.position);
            }
        }

        let overlapping = game
            .player
            .nodes
            .iter()
            .filter(|n| n.position == next_position)
            .count();

        if overlapping > 1 || is_out_of_bounds(next_position) {
            game.dead = true;
        }
    }
}

#[derive(Resource)]
struct Spawner {
    mesh: Mesh2dHandle,
    material: Handle<ColorMaterial>,
}

impl Spawner {
    fn setup(meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) -> Self {
        let mesh = Mesh2dHandle(meshes.add(Rectangle::new(GRID_SCALE - 1.0, GRID_SCALE - 1.0)));

        let color = Color::rgb(0.0, 0.0, 0.0);
        let material = materials.add(color);

        Self { mesh, material }
    }

    pub fn new_node(&self, cmd: &mut Commands, position: Position) -> SnakeNode {
        let entity = cmd
            .spawn(MaterialMesh2dBundle {
                mesh: self.mesh.clone(),
                material: self.material.clone(),
                transform: get_transform(position),
                ..Default::default()
            })
            .id();

        SnakeNode { entity, position }
    }

    pub fn new_food(&self, cmd: &mut Commands, position: Position) -> SnakeFood {
        let entity = cmd
            .spawn(MaterialMesh2dBundle {
                mesh: self.mesh.clone(),
                material: self.material.clone(),
                transform: get_transform(position),
                ..Default::default()
            })
            .id();

        SnakeFood { entity, position }
    }
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle::default());

    let spawner = Spawner::setup(&mut *meshes, &mut *materials);
    cmd.insert_resource(spawner);
}

fn cleanup_game(cmd: &mut Commands, game: &Game) {
    for node in game.player.nodes.iter() {
        cmd.entity(node.entity).despawn();
    }

    if let Some(food) = game.food.as_ref() {
        cmd.entity(food.entity).despawn();
    }
}

fn setup_game(mut cmd: Commands, mut transforms: Query<&mut Transform>, spawner: Res<Spawner>) {
    let mut game = Game {
        dead: false,
        food: None,
        player: Snake {
            nodes: vec![],
            facing: Offset::new(1, 0),
        },
        tick_timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
        input_queue: VecDeque::new(),
    };

    for i in 0..5 {
        game.player
            .nodes
            .push(spawner.new_node(&mut cmd, Position::new(5 + i, 5)));
    }

    new_food(&mut cmd, &mut transforms, &*spawner, &mut game);

    cmd.insert_resource(game);
}

fn new_food(
    cmd: &mut Commands,
    transforms: &mut Query<&mut Transform>,
    spawner: &Spawner,
    game: &mut Game,
) {
    let mut rng = rand::thread_rng();

    let position = loop {
        let x = rng.gen_range(0..WIDTH);
        let y = rng.gen_range(0..HEIGHT);
        let position = Position::new(x, y);

        if !game.player.nodes.iter().any(|n| n.position == position) {
            break position;
        }
    };

    if let Some(food) = game.food.as_mut() {
        *transforms.get_mut(food.entity).unwrap() = get_transform(position);
        food.position = position;
    } else {
        game.food = Some(spawner.new_food(cmd, position));
    }
}
