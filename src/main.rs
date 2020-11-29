use bevy::{prelude::*, render::camera::Camera};
// use bevy_tiled_prototype::level;
use bevy_tiled_prototype::TiledMapCenter;

mod ferris;
mod level;
mod movement;

#[macro_use]
extern crate approx;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .add_startup_system(setup.system())
        .add_system(level::process_loaded_tile_maps2.system())
        .init_resource::<Option<level::Level>>()
        .add_system(ferris::animate_character_system.system())
        .add_system(ferris::character_input.system())
        .add_system(ferris::character_move.system())
        // .add_system(ferris::character_hit.system())
        // .add_system(ferris::character_intersect.system())
        .add_system(camera_movement.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands
        .spawn(bevy_tiled_prototype::TiledMapComponents {
            map_asset: asset_server.load("map1.tmx"),
            center: TiledMapCenter(false),
            //origin: Transform::from_scale(Vec3::new(8.0, 8.0, 1.0)),
            origin: Transform {
                scale: Vec3::new(1.0, 1.0, 1.0),
                translation: Vec3::new(0.0, 16.0 * 16.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .spawn(Camera2dComponents {
            transform: Transform::from_scale(Vec3::new(0.25, 0.25, 1.0)),
            ..Default::default()
        });

    ferris::spawn(commands, asset_server, texture_atlases);
}

fn animate_sprite_system(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if timer.finished {
            // println!("timer");
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn camera_movement(
    query: Query<(&TextureAtlasSprite, &Transform)>,
    mut cam_query: Query<(&Camera, &mut Transform)>,
) {
    let mut pos = None;

    for (_, t) in query.iter() {
        pos = Some(t.translation);
    }

    if let Some(pos) = pos {
        for (_, mut t) in cam_query.iter_mut() {
            // println!("pos: {:?}", pos);
            t.translation = pos;
        }
    }
}
