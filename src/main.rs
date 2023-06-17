use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::transform::commands;
use bevy_flycam::{MovementSettings, PlayerPlugin};
use bevy_framepace::{FramepaceSettings, Limiter};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use std::f32::consts::PI;

const SCL: u32 = 2;
const DIMENTIONS: u32 = 600;
const SIZE: u32 = DIMENTIONS / SCL;
const PERLIN_HEIGHT_SCALE: f64 = 2.0;
const PERLIN_FREQ: f64 = 0.0005765625;
const FRAMES_NUMBER: u32 = 60;

const S: u32 = 6;
const CHUNK_RES: (u32, u32) = (2 << S, 2 << S);

#[derive(Component)]
pub struct Cube {}

#[derive(Component)]
pub struct Mash {}

#[derive(Resource)]
pub struct SpawnTimer(Timer);

#[derive(Resource)]
pub struct SpawnMesh {}

#[derive(Resource)]
pub struct MeshData {
    verticies: Vec<[f32; 3]>,
    is_spawn: bool,
    indices: Vec<u32>,
    mash_frames: Vec<Vec<f32>>,
    current_frame: i32,
    frame_direction: i8,
}

impl Plugin for SpawnMesh {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(
            0.008333333,
            TimerMode::Repeating,
        )))
        .insert_resource(MeshData {
            verticies: vec![],
            is_spawn: true,
            indices: vec![],
            mash_frames: vec![],
            current_frame: 0,
            frame_direction: 1,
        })
        .add_system(wave_effect);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            },
        }))
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_plugin(SpawnMesh {})
        .add_plugin(PlayerPlugin)
        .add_plugin(bevy_framepace::FramepacePlugin)
        // .add_system(update_planets)
        .insert_resource(MovementSettings {
            sensitivity: 0.0002,
            speed: 20.0,
        })
        .run();
}

fn setup(
    mut commands: Commands,
    mut mesh_data: ResMut<MeshData>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut frameSettings: ResMut<FramepaceSettings>,
) {
    wireframe_config.global = false;
    frameSettings.limiter = Limiter::from_framerate(60.0);
    let mut offset = (0. * CHUNK_RES.0 as f32, 0.1 * CHUNK_RES.1 as f32);
    for z in 0..SIZE + 1 {
        for x in 0..SIZE + 1 {
            let perlin = Perlin::new(1);
            let y = perlin.get([
                ((x as f64 + offset.0 as f64) * PERLIN_FREQ),
                ((z as f64 + offset.1 as f64) * PERLIN_FREQ),
            ]) * PERLIN_HEIGHT_SCALE;
            mesh_data.verticies.push([x as f32, y as f32, z as f32]);
            offset = (x as f32 * CHUNK_RES.0 as f32, z as f32 * CHUNK_RES.1 as f32);
        }
    }

    for i in 0..SIZE + 3 {
        mesh_data
            .verticies
            .extend([[i as f32, 0., 0.], [i as f32, 0., 1.]]);
    }

    for i in 0..FRAMES_NUMBER {
        let mut a = vec![];
        for vertex in mesh_data.verticies.clone() {
            let coefficient = vertex[1] * 2. / (FRAMES_NUMBER - 1) as f32;
            a.push(vertex[1] - coefficient * i as f32);
        }
        mesh_data.mash_frames.push(a);
    }

    let mut n = 0;
    for _ in 0..SIZE {
        mesh_data.indices.push(n);
        n += SIZE + 1;
        mesh_data.indices.push(n);
        n -= SIZE;
        mesh_data.indices.extend([n, n]);
        n += SIZE;
        mesh_data.indices.extend([n, n + 1]);
        n -= SIZE;
    }

    for _ in 0..SIZE - 1 {
        let elems_to_map = mesh_data
            .indices
            .iter()
            .rev()
            .take((6 * SIZE) as usize)
            .rev()
            .collect::<Vec<_>>();
        let a = elems_to_map
            .iter()
            .map(|&x| x + SIZE + 1)
            .collect::<Vec<_>>();
        mesh_data.indices.extend(a);
    }

    //light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 4.0, 4.0),
        ..default()
    });
}

pub fn wave_effect(
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut mesh_data: ResMut<MeshData>,
    // mut wireframe_config: ResMut<WireframeConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain: Query<Entity, With<Mash>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies.clone());
    mesh.set_indices(Some(Indices::U32(mesh_data.indices.clone())));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            ..default()
        },
        Wireframe,
        Mash {},
    ));

    if timer.0.tick(time.delta()).just_finished() {
        if mesh_data.is_spawn {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies.clone());
            mesh.set_indices(Some(Indices::U32(mesh_data.indices.clone())));

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    ..default()
                },
                Wireframe,
                Mash {},
            ));

            mesh_data.is_spawn = false;
        } else {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1., 1., 1.]; 91207]);

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.verticies.clone());
            mesh.set_indices(Some(Indices::U32(mesh_data.indices.clone())));

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    ..default()
                },
                Wireframe,
                Mash {},
            ));

            for (i, entity) in terrain.iter().enumerate() {
                if i != 1 {
                    commands.entity(entity).despawn_recursive();
                }
            }

            let mut new_verticies = vec![];
            for (i, vertex) in mesh_data.verticies.clone().into_iter().enumerate() {
                let index: usize = mesh_data.current_frame as usize;
                let y = &mesh_data.mash_frames[index][i];
                new_verticies.push([vertex[0], *y, vertex[2]]);
            }
            mesh_data.current_frame += mesh_data.frame_direction as i32;
            if mesh_data.current_frame == (FRAMES_NUMBER - 1) as i32 || mesh_data.current_frame == 0
            {
                mesh_data.frame_direction = -mesh_data.frame_direction;
            }
            mesh_data.verticies = new_verticies;

            mesh_data.is_spawn = true;
        }
    }
}

fn update_planets(
    mut query: Query<(&Transform, &Handle<Mesh>, Entity)>,
    assets: ResMut<Assets<Mesh>>,
    mut mesh_data: ResMut<MeshData>,
) {
    let (transform, handle, entity) = query.get_single_mut().expect("");
    // let mesh = query.get_mut(entity);
    // println!("{:?}", entity);
    println!("{:?}", handle);

    let mut new_verticies = vec![];
    for (i, vertex) in mesh_data.verticies.clone().into_iter().enumerate() {
        let index: usize = mesh_data.current_frame as usize;
        let y = &mesh_data.mash_frames[index][i];
        new_verticies.push([vertex[0], *y, vertex[2]]);
    }
    mesh_data.current_frame += mesh_data.frame_direction as i32;
    if mesh_data.current_frame == (FRAMES_NUMBER - 1) as i32 || mesh_data.current_frame == 0 {
        mesh_data.frame_direction = -mesh_data.frame_direction;
    }
    // mesh_data.verticies = new_verticies;

    // mesh_data.is_spawn = true;

    // mesh.unwrap()
    //     .insert_attribute(Mesh::ATTRIBUTE_POSITION, new_verticies);

    // if mesh.is_some() {
    //     let positions = temp.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    //     if let VertexAttributeValues::Float32x3(thing) = positions {
    //         let mut temporary = Vec::new();
    //         for i in thingy {
    //             let temp = Vec3::new(i[0], i[1], i[2]);
    //             ... // Modify temp here
    //             temporary.push(temp);
    //         }

    //         mesh.unwrap().insert_attribute(Mesh::ATTRIBUTE_POSITION, temporary);
    //     }
    // }
}
