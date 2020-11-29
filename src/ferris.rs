use bevy::math;
use bevy::{prelude::*, render::camera::Camera};
// use bevy_tiled_prototype::level;
use super::{level, movement};
use bevy_tiled_prototype::TiledMapCenter;

pub fn animate_character_system(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        &mut CharacterState,
    )>,
) {
    for (timer, mut sprite, texture_atlas_handle, mut state) in query.iter_mut() {
        // if timer.finished {
        //     // println!("timer");
        //     let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
        //     sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        // }
        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
        if timer.finished {
            state.frame += 1;
        }
        let offset = match state.face_dir {
            Direction::West => 0,
            Direction::East => 4,
            _ => 0,
        };

        let anim_frame = if state.walk_speed.is_some() {
            (state.frame % 4) as u32
        } else {
            0
        };
        sprite.index = offset + anim_frame;
    }
}

#[derive(Clone, Copy)]
enum Direction {
    East,
    West,
    North,
    South,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::East
    }
}

impl Direction {
    pub fn to_vec(&self) -> Vec2 {
        match self {
            Direction::East => Vec2::new(1f32, 0f32),
            Direction::West => Vec2::new(-1f32, 0f32),
            Direction::South => Vec2::new(0f32, -1f32),
            Direction::North => Vec2::new(0f32, 1f32),
        }
    }
    pub fn all() -> &'static [Direction; 4] {
        &[
            Direction::East,
            Direction::West,
            Direction::North,
            Direction::South,
        ]
    }
    pub fn index(&self) -> usize {
        match self {
            Direction::East => 0,
            Direction::West => 1,
            Direction::South => 2,
            Direction::North => 3,
        }
    }
}

#[derive(Default)]
pub struct CharacterState {
    velocity: Vec3,
    move_input: Option<Vec2>,
    face_dir: Direction,
    walk_speed: Option<f32>,
    frame: usize,
    hit: [bool; 4],
}
pub fn character_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&TextureAtlasSprite, &mut CharacterState)>,
) {
    for (_, mut state) in query.iter_mut() {
        let speed = if keyboard_input.pressed(KeyCode::LShift) {
            0.1
        } else {
            1.0
        };

        state.walk_speed = None;
        state.velocity = Vec3::zero();
        if keyboard_input.pressed(KeyCode::A) {
            state.face_dir = Direction::West;
            state.walk_speed = Some(speed);
            *state.velocity.x_mut() = -speed;
        }

        if keyboard_input.pressed(KeyCode::D) {
            state.face_dir = Direction::East;
            state.walk_speed = Some(speed);
            *state.velocity.x_mut() = speed;
        }

        if keyboard_input.pressed(KeyCode::S) {
            state.walk_speed = Some(speed);
            *state.velocity.y_mut() = -speed;
        }

        if keyboard_input.pressed(KeyCode::W) {
            state.walk_speed = Some(speed);
            *state.velocity.y_mut() = speed;
        }
    }
}

pub fn character_move(
    time: Res<Time>,
    level: Res<Option<level::Level>>,
    mut query: Query<(&mut Transform, &mut CharacterState)>,
) {
    let level = match *level {
        Some(ref level) => level,
        None => return,
    };

    for (mut transform, mut state) in query.iter_mut() {
        state.move_input = None;
        // match state.walk_speed {
        //     Some(speed) if state.hit[Direction::South.index()] => {
        //         state.move_input = Some(state.face_dir.to_vec() * speed)
        //     }
        //     _ => (),
        // }

        let pixel_coord = transform.translation.truncate();
        let mut d = (state.velocity * 128.0 * time.delta_seconds).truncate();

        let character_rect = math::Rect {
            left: pixel_coord.x() + 2.0,
            right: pixel_coord.x() + 14.0,
            top: pixel_coord.y(),
            bottom: pixel_coord.y() - 12.0,
        };

        let mut intersects = false;
        for shape in level.collision_shapes.iter() {
            match movement::try_move(shape, &character_rect, &d) {
                movement::MoveRes::Complete(_) => continue,
                movement::MoveRes::Collision(d_target, _, _, _) => {
                    println!(
                        "collision: {:?} {:?} {:?} {:?}",
                        shape, character_rect, d, d_target
                    );
                    d = d_target;
                    intersects = true;
                    break;
                }
                movement::MoveRes::Stuck => {
                    println!("stuck! {:?} {:?}", shape, character_rect);
                }
            }
        }
        transform.translation += d.extend(0.0);
    }
}

fn intersect(shape: &level::CollisionShape, rect: &math::Rect<f32>) -> bool {
    match shape {
        level::CollisionShape::Rect(shape) => {
            rect.left <= shape.right
                && rect.right >= shape.left
                && rect.top >= shape.bottom
                && rect.bottom <= shape.top
        }
    }
}

pub fn character_hit(
    time: Res<Time>,
    level: Res<Option<level::Level>>,
    mut query: Query<(&Transform, &mut CharacterState)>,
) {
    if level.is_none() {
        return;
    }

    let level = level.as_ref().unwrap();

    for (transform, mut state) in query.iter_mut() {
        for dir in Direction::all() {
            println!("{}", 128f32 * time.delta_seconds);
            let v = dir.to_vec() * 128f32 * time.delta_seconds;
            let new_translation = transform.translation;
            //println!("transform: {:?}", transform);
            let mut pixel_coord = new_translation.truncate() + v;
            // *pixel_coord.y_mut() *= -1f32;

            let character_rect = math::Rect {
                left: pixel_coord.x() + 2.0,
                right: pixel_coord.x() + 14.0,
                top: pixel_coord.y(),
                bottom: pixel_coord.y() - 12.0,
            };

            let mut intersects = false;
            for shape in level.collision_shapes.iter() {
                if intersect(shape, &character_rect) {
                    // println!("intersect {:?} {:?}", character_rect, shape);
                    intersects = true;
                    break;
                }
            }
            state.hit[dir.index()] = intersects;
        }
        println!("hit: {:?}", state.hit);
    }
}

pub fn character_intersect(
    time: Res<Time>,
    level: Res<Option<level::Level>>,
    mut query: Query<(&TextureAtlasSprite, &mut Transform, &mut CharacterState)>,
) {
    if level.is_none() {
        return;
    }

    let level = level.as_ref().unwrap();

    for (_, mut transform, mut state) in query.iter_mut() {
        state.velocity = Vec3::zero();

        if let Some(move_input) = state.move_input {
            println!("move: {:?}", state.move_input);
            state.velocity += move_input.extend(0f32) * 128.0;
        }

        // if !state.hit[Direction::South.index()] {
        //     // do gravity
        //     state.velocity += Direction::South.to_vec().extend(0f32) * 128f32;
        //     let speed = 128f32;
        //     transform.translation +=
        //         Direction::South.to_vec().extend(0f32) * 128f32 * time.delta_seconds;
        // }

        let new_translation = transform.translation + state.velocity * time.delta_seconds;
        let mut pixel_coord = new_translation.truncate();
        let character_rect = math::Rect {
            left: pixel_coord.x() + 2.0,
            right: pixel_coord.x() + 14.0,
            top: pixel_coord.y(),
            bottom: pixel_coord.y() - 12.0,
        };

        let mut intersects = false;
        for shape in level.collision_shapes.iter() {
            let iv = movement::intersect_dist(shape, &character_rect);
            println!("intersect {:?}", iv);
            if intersect(shape, &character_rect) {
                intersects = true;
                break;
            }
        }

        if !intersects {
            transform.translation = new_translation;
        }

        state.hit[Direction::South.index()] = true;
        // else if state.walk_speed.is_some() && !state.hit[state.face_dir.index()] {
        //     let speed = 128f32;

        //     transform.translation +=
        //         state.face_dir.to_vec().extend(0f32) * speed * time.delta_seconds;
        // }
    }
}

pub(crate) fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) -> () {
    // let texture_handle = asset_server.load("gabe-idle-run.png");
    // let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_handle = asset_server.load("ferris2.0.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 8, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(SpriteSheetComponents {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                scale: Vec3::splat(8.0 / 8.0),
                translation: Vec3::new(0.0 * 8.0, 14.0 * 16.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Timer::from_seconds(0.1, true))
        .with(CharacterState::default());
}