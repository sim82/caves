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
        let frame = &state.state.frames[state.state_step as usize];
        sprite.index = match state.face_dir {
            Direction::West => frame.0,
            Direction::East => frame.1,
            _ => 0,
        } as u32;
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

#[derive(Default, Debug)]
struct Frame(i32, i32, i32, i32, u32);

#[derive(Debug)]
enum Think {
    Walk,
    Air,
    Stand,
}

impl Default for Think {
    fn default() -> Self {
        Think::Walk
    }
}

#[derive(Debug)]
enum React {
    Walk,
    Air,
    Stand,
}

impl Default for React {
    fn default() -> Self {
        React::Walk
    }
}

#[derive(Default, Debug)]
struct StateComplex {
    frames: &'static [Frame],
    think: Think,
    react: React,
}

const FERRIS_STAND: StateComplex = StateComplex {
    frames: &[Frame(0, 4, 0, 100, 0)],
    think: Think::Stand,
    react: React::Stand,
};

const FERRIS_WALK: StateComplex = StateComplex {
    frames: &[
        Frame(0, 4, 4, 100, 1),
        Frame(1, 5, 4, 100, 2),
        Frame(2, 6, 4, 100, 3),
        Frame(3, 7, 4, 100, 0),
    ],
    think: Think::Walk,
    react: React::Walk,
};

const FERRIS_JUMP: StateComplex = StateComplex {
    frames: &[Frame(8, 9, 0, 100, 0)],
    think: Think::Air,
    react: React::Air,
};

#[derive(Clone)]
enum Movement {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Default)]
pub struct InputState {
    xaxis: Option<Movement>,
    yaxis: Option<Movement>,
    jump: bool,
}

pub struct CharacterState {
    input_state: InputState,
    face_dir: Direction,
    state: &'static StateComplex,
    state_time_left: i32,
    state_step: u32,
    pixel_coord: Vec2,
    speed: Vec2,
}

impl Default for CharacterState {
    fn default() -> Self {
        CharacterState {
            state: &FERRIS_JUMP,
            input_state: InputState::default(),
            face_dir: Direction::default(),
            state_time_left: 0,
            state_step: 0,
            pixel_coord: Vec2::new(0.0, 14.0 * 16.0),
            speed: Vec2::zero(),
        }
    }
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

        state.input_state.xaxis = None;
        if keyboard_input.pressed(KeyCode::A) {
            state.face_dir = Direction::West;
            state.input_state.xaxis = Some(Movement::Left);
        }

        if keyboard_input.pressed(KeyCode::D) {
            state.face_dir = Direction::East;
            if state.input_state.xaxis.is_none() {
                state.input_state.xaxis = Some(Movement::Right);
            } else {
                // left and right cancel out
                state.input_state.xaxis = None;
            }
        }

        state.input_state.jump = keyboard_input.pressed(KeyCode::RControl);
    }
}

pub fn character_move_state(
    time: Res<Time>,
    level: Res<Option<level::Level>>,
    mut query: Query<(&mut Transform, &mut CharacterState)>,
) {
    let level = match *level {
        Some(ref level) => level,
        None => return,
    };

    for (mut transform, mut state) in query.iter_mut() {
        let d_ms = (time.delta_seconds * 1000.0) as i32;
        let mut movex = 0f32;
        let mut movey = 0f32;
        state.state_time_left -= d_ms;
        println!("time: {} {}", state.state_time_left, d_ms);
        let mut intra_frame = Vec2::zero();

        while state.state_time_left <= 0 {
            state.state_step = state.state.frames[state.state_step as usize].4;
            let Frame(sl, sr, x, time, next) = &state.state.frames[state.state_step as usize];
            state.state_time_left += time;

            movex += match state.input_state.xaxis {
                Some(Movement::Left) => -x,
                Some(Movement::Right) => *x,
                _ => 0,
            } as f32;
            // movex += state.state_time_left += statec.frames[state.state_step as usize].3;
        }

        {
            let Frame(_, _, x, time, _) = &state.state.frames[state.state_step as usize];
            let prog = 1.0 - state.state_time_left as f32 / *time as f32;
            let x = match state.input_state.xaxis {
                Some(Movement::Left) => -x,
                Some(Movement::Right) => *x,
                _ => 0,
            };
            intra_frame = Vec2::new(x as f32, 0f32) * prog;
        }
        match state.state.think {
            Think::Walk => {
                // println!("walk")
                if state.input_state.jump {
                    state.speed.set_y(100.0);
                    state.state = &FERRIS_JUMP;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_JUMP.frames[0].3;
                    movey += state.speed.y() * time.delta_seconds;
                    let runjump_speed = 32f32;
                    let speed = match state.input_state.xaxis {
                        Some(Movement::Left) => -runjump_speed,
                        Some(Movement::Right) => runjump_speed,
                        _ => 0f32,
                    };
                    state.speed.set_x(speed);
                } else if state.input_state.xaxis.is_none() {
                    state.state = &FERRIS_STAND;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_STAND.frames[0].3;
                }
            }
            Think::Air => {
                movey += state.speed.y() * time.delta_seconds;

                if state.speed.y() > -50.0 {
                    *state.speed.y_mut() -= 5.0;
                }

                match state.input_state.xaxis.clone() {
                    Some(movement) => do_accel_x(&mut state.speed, &movement),
                    None => do_friction_x(&mut state.speed),
                }

                // match state.input_state.xaxis {
                //     Some(Movement::Left) => *state.speed.x_mut() -= 4f32,
                //     Some(Movement::Right) => *state.speed.x_mut() += 4f32,
                //     _ => *state.speed.x_mut() *= 0.5,
                // }
                movex += state.speed.x() * time.delta_seconds;
                movey += state.speed.y() * time.delta_seconds;
            }
            Think::Stand => {
                if state.input_state.jump {
                    state.speed.set_y(100.0);
                    state.state = &FERRIS_JUMP;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_JUMP.frames[0].3;
                    movey += state.speed.y() * time.delta_seconds;
                } else if state.input_state.xaxis.is_some() {
                    state.state = &FERRIS_WALK;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_WALK.frames[0].3;
                }
            }
        }

        println!(
            "move: {} {} speed {:?} {:?} {} think: {:?}",
            movex, movey, state.speed, state.state, state.state_step, state.state.think
        );
        // let pixel_coord = transform.translation.truncate();
        // let mut d = (state.velocity * 128.0 * time.delta_seconds).truncate();
        let new_pixel_coord = state.pixel_coord + Vec2::new(movex as f32, movey);

        println!("coord: {:?} {:?}", state.pixel_coord, new_pixel_coord);
        let probe_pos = new_pixel_coord + Vec2::new(8.0, -14.0);

        let mut on_ground = false;

        for shape in level.collision_shapes.iter() {
            let level::CollisionShape::Rect(r1) = shape;
            if probe_pos.x() >= r1.left
                && probe_pos.x() <= r1.right
                && probe_pos.y() >= r1.bottom
                && probe_pos.y() <= r1.top
            {
                on_ground = true;
                break;
            }
        }

        println!("intra: {:?}", intra_frame);
        transform.translation = (new_pixel_coord + intra_frame).extend(0.0);
        state.pixel_coord = new_pixel_coord;
        match state.state.react {
            React::Walk => {
                // println!("react walk");
                if !on_ground {
                    state.state = &FERRIS_JUMP;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_WALK.frames[0].3;
                }
            }
            React::Air => {
                // println!("react air: {}", on_ground);
                if on_ground {
                    state.state = &FERRIS_WALK;
                    state.state_step = 0;
                    state.state_time_left = FERRIS_WALK.frames[0].3;
                    state.speed.set_y(0.0);
                }
            }
            React::Stand => {}
        }
    }
}

fn do_friction_x(speed: &mut Vec2) -> () {
    let decel = 1f32;
    if speed.x().abs() <= decel {
        speed.set_x(0f32)
    } else if speed.x() > 0.0 {
        *speed.x_mut() -= decel
    } else if speed.x() < 0.0 {
        *speed.x_mut() += decel
    }
}

fn do_accel_x(speed: &mut Vec2, movement: &Movement) -> () {
    let accel = 4.0;
    let maxspeed = 32.0;
    let x = speed.x_mut();
    match movement {
        Movement::Right => {
            *x += accel;
            if *x > maxspeed {
                *x = maxspeed;
            }
        }
        Movement::Left => {
            *x -= accel;
            if *x < -maxspeed {
                *x = -maxspeed;
            }
        }
        _ => (),
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
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 10, 1);
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
