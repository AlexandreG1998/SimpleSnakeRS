use bevy::{
    diagnostic::{
        FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    input::keyboard::{Key, KeyboardInput},
    math::vec3,
    prelude::*,
};
//use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::ecs::system::SystemId;
use bevy::render::camera::Projection::Orthographic;
use bevy::render::camera::ScalingMode;
use rand::Rng;
use std::collections::HashMap;

//.add_plugins((FrameTimeDiagnosticsPlugin::default(), LogDiagnosticsPlugin::default()))
fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((FrameTimeDiagnosticsPlugin::default(), LogDiagnosticsPlugin::default()))
        .init_resource::<MySystems>()
        .add_systems(Startup, setup)
        .add_systems(Update, set_snake_direction)
        .add_systems(Update, move_snake)
        .add_systems(Update, eat.after(move_snake))
        .add_systems(Update, increase_blocks.after(eat))
        .add_systems(Update, adjust_blocks_positions.after(increase_blocks))
        .add_systems(Update, detect_collisions.after(adjust_blocks_positions))
        .run();
}

//Startup system (initiates renders)
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity_spawn = Vec3::ZERO;

    //Start snake in [0,0,0]
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(0f32, 0f32, 0f32),
            ..default()
        },
        IsSnake {},
        SnakeDirection {
            direction: Vec3::new(0f32, 0f32, 0f32),
        },
        PreviousPositions {
            previous_positions: Vec::new(),
        },
        NumBlocks { num_blocks: 0u32 },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::GREEN),
            transform: Transform::from_xyz(4f32, 8f32, -2f32),
            ..default()
        },
        IsFood {},
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(entity_spawn, Vec3::Y),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(16.0),
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    });
}

fn move_snake(
    mut snake: Query<(
        &mut IsSnake,
        &mut Transform,
        &mut SnakeDirection,
        &mut PreviousPositions,
    )>,
    time: Res<Time>,
) {
    let mut snake_ = snake.single_mut();
    let v0 = Vec3::new(0f32, 0f32, 0f32);

    let mut last_pos = snake_.3.previous_positions.last().unwrap_or(&v0);

    if (*last_pos != snake_.1.translation) {
        //println!("The previous positions wasn't in the vec so it will be pushed");
        snake_.3.previous_positions.push(snake_.1.translation);
    }

    snake_.1.translation += (snake_.2.direction * time.delta_seconds());

    if(snake_.1.translation.x < -16.0f32){
        snake_.1.translation.x = 16.0f32;
    }else if(snake_.1.translation.x > 16.0f32){
        snake_.1.translation.x = -16.0f32;
    }

    if(snake_.1.translation.y < -9.0f32){
        snake_.1.translation.y = 9.0f32;
    }else if(snake_.1.translation.y > 9.0f32){
        snake_.1.translation.y = -9.0f32;
    }
    

    //println!("Snake actual position {}, {}, {}",snake_.1.translation.x, snake_.1.translation.y, snake_.1.translation.z);
}

fn set_snake_direction(
    mut snake: Query<(&mut IsSnake, &mut Transform, &mut SnakeDirection)>,
    key: Res<ButtonInput<KeyCode>>
    
) {
    let mut snake_ = snake.single_mut();

    let fwd: Vec3 = snake_.1.forward().into();
    let fwd2: Vec3 = Vec3::new(0.0f32, 1.0f32, 0.0f32);
    let fwd3: Vec3 = Vec3::new(1.0f32, 0.0f32, 0.0f32);

    if key.just_pressed(KeyCode::KeyW) {
        //println!("Fwd");
        //let mut fwd = Transform::from_xyz(snake_.1.forward().to_array()[0], snake_.1.forward().to_array()[1], snake_.1.forward().to_array()[2]);
        //let mut target = Transform::from_xyz(x, y + 5.0f32, z);
        snake_.2.direction = (fwd2 * 5.0f32);
    } else if key.just_pressed(KeyCode::KeyS) {
        snake_.2.direction = -(fwd2 * 5.0f32);
        //println!("Back");
    } else if key.just_pressed(KeyCode::KeyA) {
        //println!("Left");
        snake_.2.direction = -(fwd3 * 5.0f32);
    } else if key.just_pressed(KeyCode::KeyD) {
        //println!("Right");
        snake_.2.direction = (fwd3 * 5.0f32);
    }
}

fn increase_blocks(
    mut snake: Query<(&mut IsSnake, &mut NumBlocks)>,
    key: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut systems: Res<MySystems>
) {
    if key.just_pressed(KeyCode::KeyZ) {
        let mut snake_ = snake.single_mut();
        snake_.1.num_blocks += 1u32;
        commands.run_system(systems.0["create_new_block"]);
        println!("Num blocks {}", snake_.1.num_blocks);
    }
}

fn create_snake_body(
    mut snake: Query<(&mut IsSnake, &mut NumBlocks, &mut PreviousPositions)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut snake_ = snake.single_mut();

    if snake_.1.num_blocks > 0 {
        for i in snake_
            .2
            .previous_positions
            .iter()
            .rev()
            .take(snake_.1.num_blocks.try_into().unwrap())
        {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::default()),
                    material: materials.add(Color::RED),
                    transform: Transform::from_xyz(i.x, i.y, i.z),
                    ..default()
                },
                SnakeBody {},
                Collider {},
            ));
        }
    }
}


fn adjust_blocks_positions(
    mut blocks: Query<(Entity, &mut SnakeBody, &mut Transform)>,
    mut snake: Query<(&mut PreviousPositions, &mut IsSnake)>,
){
    let mut snake_ = snake.single_mut();

    let positions_needed = blocks.iter().len();
    let mut positions_len =snake_.0.previous_positions.len();

    let mut new_positions: Vec<Vec3> = Vec::new();

    for mut v in snake_.0.previous_positions.iter().rev().take(positions_needed){
        new_positions.push(*v);
    }



    let mut index: usize = 0;
    for (e, sb, mut t) in blocks.iter_mut(){
        t.translation = Vec3::new(new_positions[index].x, new_positions[index].y, new_positions[index].z);
        index+=1;
    }

}
fn destroy_previous_blocks(
    blocks: Query<(Entity, &mut SnakeBody)>,
    mut snake: Query<(&mut IsSnake, &mut NumBlocks)>,
    mut commands: Commands,
) {
    let mut snake_ = snake.single_mut();

    for (e, sb) in blocks.iter() {
        commands.entity(e).despawn();
    }
    // if (snake_.1.num_blocks > 0) {
        
    // }
}

fn detect_collisions(
    obstacles: Query<(&mut Collider, &mut Transform)>,
    mut snake: Query<
        (
            &mut IsSnake,
            &mut Transform,
            &mut PreviousPositions,
            &mut NumBlocks,
        ),
        Without<Collider>,
    >,
    mut commands: Commands,
    mut systems: Res<MySystems>
    ) {
    let mut snake_ = snake.single_mut();
    // if(snake_.3.num_blocks > 0){
    //     let previous = &snake_.2.previous_positions;
    //     for p in previous.iter().rev().take(snake_.3.num_blocks.try_into().unwrap()){
    //         let distance = (*p - snake_.1.translation).length();

    //         if(distance <= 0.04){
    //             println!("Snake head pos {};{};{} and body block pos {};{};{} with distance: {}",  snake_.1.translation.x,  snake_.1.translation.y,  snake_.1.translation.z, p.x,  p.y,  p.z,distance);
    //         }
    //     }
    // }
    let mut num_obstacles: i32 = 0i32;

    for (c, t) in obstacles.iter() {
        num_obstacles += 1;
        let distance = (t.translation - snake_.1.translation).length();
        if (distance <= 0.02) {
            println!("Collison detected!");
            println!(
                "Snake head pos {};{};{} and body block pos {};{};{} with distance: {}",
                snake_.1.translation.x,
                snake_.1.translation.y,
                snake_.1.translation.z,
                t.translation.x,
                t.translation.y,
                t.translation.z,
                distance
            );
            commands.run_system(systems.0["destroy_previous_blocks"]);
            snake_.3.num_blocks = 0u32;
        } else {
        }
    }

    //println!("Num Obstacles: {}", num_obstacles);
}


fn create_new_block(
    mut snake: Query<(&mut IsSnake, &mut NumBlocks, &mut PreviousPositions)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){

    let mut snake_ = snake.single_mut();
    let mut position = snake_.2.previous_positions.last().unwrap();


    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        },
        SnakeBody {},
        Collider {},
    ));

}
fn eat(mut snake: Query<
    (
        &mut IsSnake,
        &mut Transform,
        &mut PreviousPositions,
        &mut NumBlocks,
    ), Without<IsFood>>,
    mut food : Query<(Entity, &mut IsFood, &mut Transform)>,
    mut commands: Commands,
    mut systems: Res<MySystems>
){
    let mut snake_ = snake.single_mut();
    let mut food_ = food.single_mut();
    
    let mut distance_x = (snake_.1.translation.x - food_.2.translation.x).abs();
    let mut distance_y = (snake_.1.translation.y - food_.2.translation.y).abs();
    let mut distance_total = distance_x + distance_y;

    let mut distance_params = 0.4f32;
    if(snake_.1.translation.y > food_.2.translation.y){
        distance_params = 1.8f32;
    }
    if(distance_total <= distance_params){
        println!("Eat food");
        snake_.3.num_blocks +=1;
        commands.run_system(systems.0["create_new_block"]);
        commands.entity(food_.0).despawn();
        commands.run_system(systems.0["gen_food"]);

    }

}

fn gen_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
){


    let mut rng = rand::thread_rng();

    let mut x: f32 = rng.gen_range(-15.0f32..15.0f32);
    let mut y = rng.gen_range(-4.0f32..4.0f32);
    println!("Food actual position {}, {}, {}",x, y, -2f32);

    commands.spawn(
        (
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::GREEN),
            transform: Transform::from_xyz(x, y, -2f32),
            ..default()
        },
        IsFood {},
    )
    );
}
//Components

#[derive(Component)]
struct IsSnake {}

#[derive(Component)]
struct SnakeDirection {
    direction: Vec3,
}

#[derive(Component)]
struct PreviousPositions {
    previous_positions: Vec<Vec3>,
}

#[derive(Component)]
struct NumBlocks {
    num_blocks: u32,
}

#[derive(Component)]
struct SnakeBody {}

#[derive(Component)]
struct Collider {}

#[derive(Component)]
struct IsFood {}




//Resources

#[derive(Resource)]
struct MySystems(HashMap<String, SystemId>);

impl FromWorld for MySystems{
    fn from_world(world: &mut World) -> Self {
        let mut my_systems_hash = MySystems(HashMap::new());
        my_systems_hash.0.insert(
            "gen_food".into(),
            world.register_system(gen_food)
        );

        my_systems_hash.0.insert(
            "destroy_previous_blocks".into(),
            world.register_system(destroy_previous_blocks)
        );

        my_systems_hash.0.insert(
            "create_new_block".into(),
            world.register_system(create_new_block)
        );

        my_systems_hash
    }
}
