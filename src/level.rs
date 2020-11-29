use bevy::{math::Rect, prelude::*};

use tiled::Tileset;

use bevy_tiled_prototype::Map;

#[derive(Debug)]
pub enum CollisionShape {
    Rect(Rect<f32>),
}

pub struct Level {
    pub collision_shapes: Vec<CollisionShape>,
}

impl Level {
    pub fn new(map: &tiled::Map) -> Self {
        let mut collision_shapes = Vec::new();
        if map.tilesets.len() != 1 {
            panic!("only support one tileset");
        }
        let tileset: &Tileset = &map.tilesets[0];

        println!("first gid: {}", tileset.first_gid);
        let mut tile_map = std::collections::HashMap::new();
        for tile in tileset.tiles.iter() {
            println!("tilemap: {}", tile.id);
            tile_map.insert(tile.id, tile);
        }
        // tileset.tiles
        for layer in map.layers.iter() {
            if !layer.visible {
                continue;
            }
            for y in 0..map.height {
                let mut line = String::new();
                let y2 = map.height - y - 1;
                for x in 0..map.width {
                    let map_tile = match &layer.tiles {
                        tiled::LayerData::Finite(tiles) => &tiles[y as usize][x as usize],
                        _ => panic!("Infinte maps not supported"),
                    };
                    let rect = Rect {
                        left: (x * 16) as f32,
                        right: (x * 16 + 16) as f32,
                        top: (y2 * 16 + 16) as f32,
                        bottom: (y2 * 16) as f32,
                    };
                    // println!( "rect: {:?}", rect);
                    if map_tile.gid != 0 {
                        let mut shape = None;
                        line.push('#');
                        if map_tile.gid < tileset.first_gid
                            || map_tile.gid >= tileset.tilecount.unwrap()
                        {
                            panic!("tile gid out of range: {}", map_tile.gid);
                        }
                        let idx = map_tile.gid - tileset.first_gid;
                        println!("lookup: {}", idx);
                        let tile = tile_map.get(&idx).unwrap();
                        match &tile.objectgroup {
                            Some(objectgroup) => {
                                for obj in objectgroup.objects.iter() {
                                    match &obj.shape {
                                        tiled::ObjectShape::Rect { width, height } => {
                                            println!(
                                                "rect: {} {} {} {}",
                                                obj.x, obj.y, width, height
                                            );
                                            let bottom =
                                                (y2 * 16) as f32 + (16f32 - (obj.y + height));
                                            let top = bottom + height;
                                            let left = (x * 16) as f32 + obj.x;
                                            let right = left + width;
                                            // TODO: transform tiled Object shape (top,left -> bottom,left...)
                                            shape = Some(CollisionShape::Rect(Rect {
                                                left,
                                                right,
                                                top,
                                                bottom,
                                            }));
                                        }
                                        tiled::ObjectShape::Polygon { points } => {
                                            println!("polygon: {}", points.len());
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            _ => (),
                        }
                        if let Some(shape) = shape {
                            collision_shapes.push(shape);
                        } else {
                            collision_shapes.push(CollisionShape::Rect(rect));
                        }
                    } else {
                        line.push(' ')
                    }

                    // println!("map tile: {:?}", map_tile);
                }
                println!("{}", line)
            }

            // match &layer.tiles {
            //     tiled::LayerData::Finite(tiles) => {
            //         println!("size: {}", tiles.len());

            //         for span in tiles {
            //             println!("size x: {}", span.len());
            //         }
            //     }
            //     _ => panic!("Infinte maps not supported"),
            // }
        }

        Level { collision_shapes }
    }
}

#[derive(Default)]
pub struct MapResourceProviderState2 {
    map_event_reader: EventReader<AssetEvent<Map>>,
}
pub fn process_loaded_tile_maps2(
    // asset_server: Res<AssetServer>,
    mut state: Local<MapResourceProviderState2>,
    map_events: Res<Events<AssetEvent<Map>>>,
    mut maps: ResMut<Assets<Map>>,
    mut level: ResMut<Option<Level>>,
) {
    for event in state.map_event_reader.iter(&map_events) {
        match event {
            AssetEvent::Created { handle } => {
                let map = maps.get_mut(handle).unwrap();

                *level = Some(Level::new(&map.map));
                // match &layer.tiles {
                //     tiled::LayerData::Finite(tiles) => {
                //         println!("size: {}", tiles.len());

                //         for span in tiles {
                //             println!("size x: {}", span.len());
                //         }
                //     }
                //     _ => panic!("Infinte maps not supported"),
                // }

                println!("created: {:?}", handle);
            }
            AssetEvent::Modified { handle } => {
                // println!("modified: {:?}", handle);
            }
            AssetEvent::Removed { handle } => {
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                // println!("removed: {:?}", handle);
            }
        }
    }
}
