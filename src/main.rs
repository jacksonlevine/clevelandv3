
mod jclient;
mod jserver;
mod camera;
mod chunk;
mod cube; 
mod blockinfo;
mod revindices;

use std::{env, f32::consts::PI, time::Duration};

use bevy::{animation::animate_targets, input::mouse::MouseMotion, pbr::CascadeShadowConfigBuilder, prelude::*, render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages, Render}, utils::HashMap, window::{CursorGrabMode, PrimaryWindow}};
use camera::JCamera;
use chunk::{ChunkPlugin, JPerlin, RebuildThisChunk, CW};
use dashmap::DashMap;
use jserver::start_listening;
use bevy_rapier3d::prelude::*;
use once_cell::sync::Lazy;
use uuid::Uuid;

const GROUND_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 4.0;
const JUMP_SPEED: f32 = 14.0;
const GRAVITY: f32 = -9.81;

#[derive(Resource)]
pub struct UserDataMap {
    pub map: HashMap<IVec3, u32>
}

#[derive(Resource, Reflect)] 
#[reflect(Resource)]
pub struct MyPlayerInitialized {
    pub value: bool
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChildJId {
    uuid: Uuid
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct JId {
    uuid: Uuid
}

#[derive(Resource)]
pub struct ChunkSurveyTimer(Timer);

impl Default for MyPlayerInitialized {
    fn default() -> Self {
        Self { value: false }
    }
}

#[derive(Resource)] 
pub struct JMyId {
    pub uuid: Uuid
}

#[derive(Component)]
pub struct MyHead {}

#[derive(Component)]
pub struct MyCollider {}

#[derive(Component)]
struct JMyPlayer;

#[derive(Resource, Component, Reflect)]
#[reflect(Resource, Component)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    pub graph: Handle<AnimationGraph>,
}

#[derive(Resource, Default)] 
pub struct JControls {
    f: bool,
    b: bool,
    r: bool,
    l: bool,
    sprinting: bool,
    jump: bool,
    moving: bool
}

#[derive(Resource, Reflect, Clone)] 
#[reflect(Resource)]

pub struct JVars {
    first_mouse: bool,
    mouse_focused: bool,
    sensitivity: f32
}

impl Default for JVars {
    fn default() -> Self {
        Self {
            first_mouse: true,
            mouse_focused: false,
            sensitivity: 0.2
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct JMoveState {
    pub moving: bool,
    pub did_update_anim_state: bool
}

impl Default for JMoveState {
    fn default() -> Self {
        Self { moving: false, did_update_anim_state: false }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut isserver = false;

    for arg in args {
        if arg == "s" { //ITS A SERVER IF ARG S IS THERE
            isserver = true;
            break;
        } else {
            
        }
    }

    let mut a = App::new();

    if isserver {
        a.add_plugins(MinimalPlugins)
        .add_systems(Startup, start_listening)
        ;


    } else {



        a.add_plugins((DefaultPlugins.set (
            ImagePlugin::default_nearest()
        ),
            RapierPhysicsPlugin::<NoUserData>::default()))
        .init_resource::<JPerlin>()
        .add_plugins(ChunkPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .init_resource::<JControls>()
        .init_resource::<JVars>()
        .init_resource::<JCamera>()
        .insert_resource(ChunkSurveyTimer(Timer::new(Duration::from_secs(2), TimerMode::Repeating)))
        
        .init_resource::<MyPlayerInitialized>()
        .insert_resource(
            {
                let id = Uuid::new_v4();
                println!("My id is {}", id);
                JMyId { uuid: id }
            }
           
        )
        .add_systems(Startup, start_physical_world)
        .add_systems(Update, player_movement)
            //a.add_systems(Update, add_colliders_to_meshes);
        .add_systems(Startup, initial_grab_cursor)
        .add_systems(Update, handle_input)
        .add_systems(Update, set_anim_states)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Startup, animations_setup)
        .add_systems(Update, move_from_controls)
        ;
    }

    a.run();

    
}

pub fn set_anim_states(
    mut query: Query<(&mut AnimationTransitions, &mut AnimationPlayer, &JMoveState)>,
    animations: Res<Animations>,
) {
    for (mut animtrans, mut player, movestate) in query.iter_mut() {
        //println!("Match amtcde");
        if movestate.moving {
            //println!("Moving");

            if player.playing_animations().next().is_none() {
                //println!("No animation playing, play the animation");
                animtrans
                    .play(&mut player, animations.animations[0], Duration::from_secs_f32(0.1))
                    .repeat();
            } else {
                let playing_index = player.playing_animations().next().unwrap().0.clone();
                let mut animation = player.animation_mut(playing_index).unwrap();
                if animation.is_finished() {
                   // println!("Animation finished, playing again");
                    animtrans
                        .play(&mut player, animations.animations[0], Duration::from_secs_f32(0.1))
                        .repeat();
                }
            }
        } else {
            player.stop_all();
        }
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}



pub static mut GOTTEN_SPOTS: Lazy<DashMap<IVec2, bool>> = Lazy::new(|| DashMap::new());


pub fn start_physical_world(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,

) {
    // Import the custom texture.
    let custom_texture_handle: Handle<Image> = asset_server.load("world.png");
    // Create and save a handle to the mesh.


    // // Render the mesh with the custom texture using a PbrBundle, add the marker.
    // for i in -6..6 {
    //     for j in -6..6 {
    //         let offset = Vec3::new(i as f32 * CW as f32, 0.0, j as f32  * CW as f32);

    //         let cube_mesh_handle: Handle<Mesh> = meshes.add(Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD));


    //         commands.spawn((
    //             PbrBundle {
    //                 mesh: cube_mesh_handle,
    //                 material: materials.add(StandardMaterial {
    //                     base_color_texture: Some(custom_texture_handle.clone()),
    //                     ..default()
    //                 }),
    //                 transform: Transform::from_xyz(offset.x, offset.y, offset.z),
    //                 ..default()
    //             },
    //             RebuildThisChunk,
    //             Collider::halfspace(Vec3::Y).unwrap()
    //         )); 
    //     }
    // }

    //spawn a test floor
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cityblank.gltf")),
            ..default()
        },

        Collider::halfspace(Vec3::Y).unwrap())
    );

    // // Create an empty mesh and get its handle
    // let mesh_handle = meshes.add(Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD));

    // // Spawn an entity with the mesh handle and a default transform bundle
    // commands.spawn((
    //     mesh_handle,
    //     RebuildThisChunk{},
    //     TransformBundle::default(),
    // ));




    let trans = Transform::from_xyz(0.0, 1000.0, 0.0);
    commands
        .spawn((
            SpatialBundle {
                transform: trans,
                ..default()
            },
            Collider::round_cylinder(0.99, 0.3, 0.2),
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
            MyCollider{},
            
        ))
        .with_children(|b| {

            b.spawn(
                        (
                            TransformBundle::from_transform(
                                Transform::from_xyz(0.0, 0.8, 0.3).looking_at(
                                    Vec3::new(0.0, 1.0, 5.0), Dir3::Y
                                ),
                            ),
                            MyHead{}
                        )
                        
                    ).with_children(|b| {


                                b.spawn((
                                    Camera3dBundle {
                                        projection: Projection::Perspective(PerspectiveProjection {
                                            fov: PI/1.7, // Set FOV to 90 degrees (π/2 radians)
                                            aspect_ratio: 16.0 / 9.0,         // Aspect ratio (width/height)
                                            near: 0.01,                        // Near clipping plane
                                            far: 1000.0,                      // Far clipping plane
                                            ..Default::default()
                                        }),
                                        
                                        ..Default::default()
                                    },
                                    FogSettings {
                                        color: Color::srgb(0.25, 0.25, 0.25),
                                        falloff: FogFalloff::Exponential { density: 0.02 },
                                        ..default()
                                    },
                                    
                                ));

                    }); 
            
            b.spawn((SceneBundle {
                scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/player.glb")),
                transform: Transform::from_scale(Vec3::ONE * 0.3),
                ..default()
            }, 

            ));
        } 
    );
}











fn move_from_controls(
    mut controls: ResMut<JControls>,
    mut camera: ResMut<JCamera>,
    mut playmodel: Query<&mut JMoveState, With<JMyPlayer>>,
    time: Res<Time>
) {

    let mul = if controls.sprinting { 20.0 } else { 10.0 };

    let mut going = false;

    if controls.l {
        going = true;
        camera.velocity += Vec3::new(1.0, 0.0, 0.0);//dir * mul * time.delta_seconds();
    }
    if controls.r {
        going = true;
        camera.velocity += Vec3::new(-1.0, 0.0, 0.0); //dir * mul * time.delta_seconds();
    }
    if controls.f {
        going = true;
        camera.velocity += Vec3::new(0.0, 0.0, 1.0); //dir * mul * time.delta_seconds();
    }
    if controls.b {
        going = true;
        camera.velocity += Vec3::new(0.0, 0.0, -1.0);  //dir * mul * time.delta_seconds();
    }
    if controls.jump {
        camera.velocity.y = 1.0;
        controls.jump = false;
    }

    controls.moving = going;
    
    for mut t in playmodel.iter_mut() {
        (*t).moving = going;
    }


}



fn player_movement(
    time: Res<Time>,
    controls: Res<JControls>,
    mut jcamera: ResMut<JCamera>,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {

    let input = jcamera.velocity;

    //println!("Input: {}", input);

    let Ok((transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };
    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED * if controls.sprinting { 1.8 } else { 1.0 };
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    jcamera.velocity = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
    
}



fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controls: ResMut<JControls>,
    vars: Res<JVars>,
    mut mouse_events: EventReader<MouseMotion>,
    mut camera: ResMut<JCamera>,
    mut realcamera: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    ), Without<MyHead>>,
    mut head: Query<(
        &mut Transform,
        
    ), With<MyHead>>
) {
    controls.f = keyboard.pressed(KeyCode::KeyW);
    controls.b = keyboard.pressed(KeyCode::KeyS);
    controls.r = keyboard.pressed(KeyCode::KeyD);
    controls.l = keyboard.pressed(KeyCode::KeyA);
    controls.jump = keyboard.pressed(KeyCode::Space);
    controls.sprinting = keyboard.pressed(KeyCode::ShiftLeft);

    for event in mouse_events.read() {




        camera.yaw -= event.delta.x * vars.sensitivity;
        camera.pitch  -= event.delta.y * vars.sensitivity;
        
        camera.pitch = camera.pitch.clamp(-89.9, 89.9); // Limit pitch
        camera.recalculate();

        for (mut transform, kcc, _) in realcamera.iter_mut() {

            // Simple 3D rotation by Euler-angles (X, Y, Z)
            
            (*transform.rotation) = *Quat::from_euler(
                // YXZ order corresponds to the common
                // "yaw"/"pitch"/"roll" convention
                EulerRot::YXZ,
                camera.yaw.to_radians(),
                0.0,
                0.0
            );

        }

        for mut transform in head.iter_mut() {

            // Simple 3D rotation by Euler-angles (X, Y, Z)
            
            (*transform.0.rotation) = *Quat::from_euler(
                // YXZ order corresponds to the common
                // "yaw"/"pitch"/"roll" convention
                EulerRot::YXZ,
                PI,
                camera.pitch.to_radians(),
                0.0
            );

        }
    }
}


fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut mpi: ResMut<MyPlayerInitialized>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    id_query: Query<&JId>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions)
            .insert(JMoveState::default())
            //.insert(JGameCam{})
            ;
        let mut current_jid = None;
             // Traverse up the parent hierarchy to find an entity with JId
        let mut current_entity = entity;
        while let Ok(parent) = parents_query.get(current_entity) {
            current_entity = parent.get();
            
            let jid = id_query.get(current_entity);

            if jid.is_ok() {
                println!("Found JId component in parent entity: {:?}", current_entity);
                current_jid = Some(jid.unwrap());
                // Perform any additional logic if needed
                break;
            }
        }

        match current_jid {
            Some(jid) => {
                commands.entity(entity).insert(ChildJId {
                    uuid: jid.uuid
                });
            }
            None => {

            }
        }

        


        if !(*mpi).value {

            commands.entity(entity).insert(JMyPlayer{});
            
            (*mpi).value = true;
        }
        

    }
}


fn animations_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(0).from_asset("models/player.glb"),
            ]
            .into_iter()
            .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });





    // Camera
    // commands.spawn((Camera3dBundle {
    //     transform: Transform::from_xyz(0.0, 2.0, 3.0)
    //         .looking_at(Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
    //     ..default()
    // }, JGameCam{}));

    // // Plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0)),
    //     material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
    //     ..default()
    // });




    // // Light
    // commands.spawn(DirectionalLightBundle {
    //     transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
    //     directional_light: DirectionalLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     cascade_shadow_config: CascadeShadowConfigBuilder {
    //         first_cascade_far_bound: 200.0,
    //         maximum_distance: 400.0,
    //         ..default()
    //     }
    //     .into(),
    //     ..default()
    // });



}