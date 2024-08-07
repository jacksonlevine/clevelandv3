use bevy::{prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}, tasks::{futures_lite::future, AsyncComputeTaskPool, Task}};
use bevy_rapier3d::prelude::Collider;
use noise::{NoiseFn, Perlin};
use num_enum::FromPrimitive;

use crate::{blockinfo::Blocks, cube::{get_normal, Cube, CubeSide}};


pub static CW: i32 = 16;
pub static CH: i32 = 256;



pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, remesh_chunks)
        .add_systems(Update, handle_completed_chunks)
        ;
    }
}


#[derive(Component)]
pub struct RebuildThisChunk;

#[derive(Component)]
struct MeshRebuildTask(Task<(Mesh, Collider)>);

#[derive(Resource, Default)]
pub struct JPerlin {
    perlin: Perlin
}

pub fn spot_to_chunk_pos(spot: &IVec3) -> IVec2 {
    return IVec2 {
        x: (spot.x as f32 / CW as f32).floor() as i32,
        y: (spot.z as f32 / CW as f32).floor() as i32,
    };
}



pub fn remesh_chunks(mut commands: Commands, mut chunks: Query<(Entity, &mut Handle<Mesh>, &Transform, &mut Collider), (With<RebuildThisChunk>, Without<MeshRebuildTask>)>, mut meshes: ResMut<Assets<Mesh>>, perlin: Res<JPerlin>) {
    let task_pool = AsyncComputeTaskPool::get();
    
    let perlin = perlin.perlin;


    
    for (entity, meshhandle, transform, mut collider) in chunks.iter_mut() {

        println!("HIT ONE");

        let mut vertindex = 0;

        let chunkpos = spot_to_chunk_pos(&transform.translation.as_ivec3());

        let task = task_pool.spawn(async move {
            
        
            let mut positions = Vec::new();
            let mut uvs = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();

            for i in 0..CW {
                for k in 0..CW {
    
                    for j in (0..CH).rev() {
                        let spot = IVec3 {
                            x: (chunkpos.x * CW) + i,
                            y: j,
                            z: (chunkpos.y * CW) + k,
                        };
                        let combined = blockat(&perlin, spot);
                        let block = combined & Blocks::block_id_bits();
                        let flags = combined & Blocks::block_flag_bits();
                        
    
                        if block != 0 {
                            // if !weatherstoptops.contains_key(&vec::IVec2 {
                            //     x: i,
                            //     y: k,
                            // }) {
                            //     weatherstoptops.insert(
                            //         vec::IVec2 {
                            //             x: i,
                            //             y: k,
                            //         },
                            //         spot.y,
                            //     );
                            // }
                            
    
                            
    
    
                                if Blocks::is_transparent(block) || Blocks::is_semi_transparent(block) || true {
                                    for (indie, neigh) in Cube::get_neighbors().iter().enumerate() {
                                        let neighspot = spot + *neigh;
                                        let neigh_block = blockat( &perlin, neighspot)
                                            & Blocks::block_id_bits();
                                        let cubeside = CubeSide::from_primitive(indie);
                                        let neigh_semi_trans = Blocks::is_semi_transparent(neigh_block);
                                        let water_bordering_transparent = block == 2
                                            && neigh_block != 2
                                            && Blocks::is_transparent(neigh_block);
    
                                        // let lmlock = self.lightmap.lock().unwrap();
    
                                        // let blocklighthere = match lmlock.get(&neighspot) {
                                        //     Some(k) => k.sum(),
                                        //     None => LightColor::ZERO,
                                        // };
    
                                        // if blocklighthere != 0 {
                                        //     info!("Block light here: {}", blocklighthere);
                                        // }
                                        //drop(lmlock);
    
    
    
                                        if neigh_block == 0
                                            || neigh_semi_trans
                                            || water_bordering_transparent
                                        {
                                            let side = Cube::get_side(cubeside);
    
    
                                            let texcoord = Blocks::get_tex_coords(block, cubeside);
                                            let uvcoord = Blocks::get_uv_coords(*texcoord);
    
                                            let normal = get_normal(cubeside);
    
    
                                            for (ind, v) in side.chunks(3).enumerate() {
                                      
    
                                                // let pack = PackedVertex::pack(
                                                //     i as u8 + v[0],
                                                //     j as u8 + v[1],
                                                //     k as u8 + v[2],
                                                //     ind as u8,
                                                //     clamped_light,
                                                //     0u8, //TEMPORARY UNUSED
                                                //     texcoord.0,
                                                //     texcoord.1,
                                                // );
    
                                                positions.extend_from_slice(&[
                                                    [(i + v[0] as i32) as f32, (j + v[1] as i32) as f32, (k + v[2] as i32) as f32 ]
                                                ]);
                                                uvs.extend_from_slice(&[
                                                    [uvcoord[ind].0, uvcoord[ind].1]
                                                ]);
                                                
                                                normals.extend_from_slice(&[
                                                    [normal.x as f32, normal.y as f32, normal.z as f32]
                                                ]);
                                                indices.extend_from_slice(&[
                                                    vertindex
                                                ]);
                                                vertindex += 1;
    
                                            }
    
                                        } else {
                                            // tops.insert(
                                            //     vec::IVec2 {
                                            //         x: i + neigh.x,
                                            //         y: k + neigh.z,
                                            //     },
                                            //     j + neigh.y,
                                            // );
                                        }
                                    }
                                }
    
    
    
    
                            
                        }
                    }
    
                    
                }
            }

            let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U32(indices.iter().rev().map(|s| *s).collect::<Vec<_>>()));
            
            let collider = Collider::from_bevy_mesh(&mesh, &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh).unwrap();
            
            (mesh, collider)
        
        });

        
        commands.entity(entity).insert(MeshRebuildTask(task));

        

        

        // let mesh = meshes.get_mut(meshhandle.id()).unwrap();

        // (*mesh) = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        // .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        // .with_inserted_indices(Indices::U32(indices));
        // //Remove the rebuild indicator
        // commands.entity(entity).remove::<RebuildThisChunk>();

        // (*collider) = Collider::from_bevy_mesh(&mesh, &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh).unwrap();



    }
}

pub fn handle_completed_chunks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut MeshRebuildTask, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task, mesholdhandle) in tasks.iter_mut() {
        if let Some((mesh, collider)) = future::block_on(future::poll_once(&mut task.0)) {

            if let Some(meshold) = meshes.get_mut(mesholdhandle.id()) {
                *meshold = mesh;

            }


            commands.entity(entity).remove::<Collider>().insert(collider).remove::<MeshRebuildTask>().remove::<RebuildThisChunk>();
        }
    }
}


pub fn blockat(perlin: &Perlin, spot: IVec3) -> u32 {
    // if self.headless {
    //     if self.generated_chunks.contains_key(&ChunkSystem::spot_to_chunk_pos(&spot)) {

    //     } else {
    //         self.generate_chunk(&ChunkSystem::spot_to_chunk_pos(&spot))
    //     }
    // }

    // match self.userdatamap.get(&spot) {
    //     Some(id) => {
    //         return *id;
    //     }
    //     None => {}
    // }

    // match self.nonuserdatamap.get(&spot) {
    //     Some(id) => {
    //         return *id;
    //     }
    //     None => return natural_blockat(perlin, spot),
    // }

    return natural_blockat(perlin, spot);
}

pub fn biome_noise(perlin: &Perlin, spot: IVec2) -> f64 {
    const XZDIVISOR1: f64 = 100.35 * 4.0;

    let y = 20;

    let noise1 = f64::max(
        0.0,
        perlin.get([
            spot.x as f64 / XZDIVISOR1,
            y as f64,
            spot.y as f64 / XZDIVISOR1,
        ]),
    );

    noise1
}

pub fn ore_noise(perlin: &Perlin, spot: IVec3) -> f64 {
    const XYZDIVISOR: f64 = 15.53;

    let noise1 = f64::max(
        0.0,
        perlin.get([
            spot.x as f64 / XYZDIVISOR,
            spot.y as f64 / XYZDIVISOR,
            spot.z as f64 / XYZDIVISOR,
        ]),
    );

    noise1 * ((60.0 - spot.y as f64).max(0.0) / 7.0)
}

pub fn feature_noise(perlin: &Perlin, spot: IVec3) -> f64 {
    const XZDIVISOR1: f64 = 45.35 * 4.0;

    let y = 20;

    let noise1 = f64::max(
        0.0,
        perlin.get([
            (spot.x as f64 + 200.0) / XZDIVISOR1,
            y as f64,
            spot.y as f64 / XZDIVISOR1,
        ]),
    );

    noise1
}

pub fn cave_noise(perlin: &Perlin, spot: IVec3) -> f64 {
    const XZDIVISOR1: f64 = 25.35;

    let noise1 = f64::max(
        0.0,
        perlin.get([
            (spot.x as f64) / XZDIVISOR1,
            (spot.y as f64) / XZDIVISOR1,
            spot.z as f64 / XZDIVISOR1,
        ]),
    );

    noise1
}

pub fn natural_blockat(perlin: &Perlin, spot: IVec3) -> u32 {
    if spot.y == 0 {
        return 15;
    }


            static WL: f32 = 30.0;

            let biomenum = biome_noise(perlin, IVec2 {
                x: spot.x,
                y: spot.z,
            });
            let biomenum2 = biome_noise(perlin, IVec2 {
                x: spot.x * 20 + 5000,
                y: spot.z * 20 + 5000,
            });

            let mut underdirt = 5;
            let mut surface = 3;
            let mut undersurface = 4;
            let mut liquid = 2;
            let mut beach = 1;

            if biomenum > 0.0 {
                underdirt = 1;
                surface = 1;
                undersurface = 1;
                liquid = 2;
                beach = 1;
            } else {
                if biomenum2 > 0.0 {
                    surface = 34;
                }
            }

            let ret = if noise_func(perlin,spot) > 10.0 {
                if noise_func(perlin,spot + IVec3 { x: 0, y: 10, z: 0 }) > 10.0 {
                    if ore_noise(perlin,spot) > 1.0 {
                        35
                    } else {
                        underdirt
                    }
                } else {

                    let beachnoise = perlin.get([spot.y as f64/7.5, spot.z as f64/7.5, spot.x as f64/7.5]);
                    if spot.y > (WL + beachnoise as f32) as i32
                    || noise_func(perlin,spot + IVec3 { x: 0, y: 5, z: 0 }) > 10.0
                    {
                        if noise_func(perlin,spot + IVec3 { x: 0, y: 1, z: 0 }) < 10.0 {
                            surface
                        } else {
                            undersurface
                        }
                        
                    } else {
                        beach
                    }
                }


                
            } else {
                if spot.y < WL as i32 {
                    liquid
                } else {
                    0
                }
            };
        

    if ret != 2 {
        if cave_noise(perlin, spot) > 0.5 {
            return 0;
        }
    }
    ret
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

pub fn noise_func(perlin: &Perlin, spot: IVec3) -> f64 {

    let spot = spot;
    let spot = (Vec3::new(spot.x as f32, spot.y as f32, spot.z as f32) / 3.0) + Vec3::new(0.0, 10.0, 0.0);
    //let xzdivisor1 = 600.35 * 4.0;
    let xzdivisor2 = 1000.35 * 4.0;

    let mut y = spot.y - 20.0;

    let noise1 = f64::max(
        0.0,
        20.0 + perlin.get([
            spot.x as f64 / xzdivisor2,
            y as f64 / xzdivisor2,
            spot.z as f64 / xzdivisor2,
        ]) * 5.0
            - f64::max(
                y as f64 / 1.7
                    + perlin
                        .get([spot.x as f64 / 65.0, spot.z as f64 / 65.0])
                        * 10.0,
                0.0,
            ),
    ) * 2.0;

    y += 100.0;

    let noise2 = f64::max(
        0.0,
        50.0 + perlin.get([
            spot.x as f64 / 100.35,
            y as f64 / 50.35,
            spot.z as f64 / 100.35,
        ]) * 10.0
            + perlin.get([
                spot.x as f64 / 300.35,
                y as f64 / 100.35,
                spot.z as f64 / 300.35,
            ]) * 10.0
            - f64::max(y as f64 / 3.0, 0.0),
    );

    let mut p = 
        perlin
        .get([spot.x as f64 / 500.0, spot.z as f64 / 500.0])
        * 2.0;

    p = f64::max(p, 0.0);
    p = f64::min(p, 1.0);

    // Mixing noise1 and noise2 based on p, assuming `mix` is a function that blends the two values
    // Rust doesn't have a direct `mix` function, but you can create one or use a linear interpolation
    let noisemix = mix(noise1, noise2, p);



    let texture = perlin.get([
        spot.x as f64 / 12.35,
        y as f64 / 12.35,
        spot.z as f64 / 12.35,
    ]) * 1.0;

    let noise3 = f64::max(
        0.0,
        50.0 + perlin.get([
            spot.x as f64 / 25.35,
            y as f64 / 25.35,
            spot.z as f64 / 25.35,
        ]) * 10.0
            + perlin.get([
                spot.x as f64 / 60.35,
                y as f64 / 50.35,
                spot.z as f64 / 60.35,
            ]) * 10.0
            - f64::max(y as f64 / 3.0, 0.0),
    );

    let mut p2 = 0.5 + perlin.get([
        (spot.x as f64 + 4500.0) / 150.0,
        (spot.y as f64 + 5000.0) / 150.0,
        (spot.z as f64 - 5000.0) / 150.0,
    ]) * 1.0;

    let p3 = (perlin.get([
        (spot.x as f64 - 1500.0) / 3500.0,
        (spot.z as f64 + 1000.0) / 3500.0,
    ]) * 10.0).min(9.0);



    p2 = f64::max(p2, 0.0);
    p2 = f64::min(p2, 1.0);

    mix(noisemix + texture, noise3, p2.clamp(0.0, 1.0)).min(20.0) + p3
}

