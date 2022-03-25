mod ca_compute;
mod fly_cam;
mod gui;
mod rtmaterial;
mod rule;

use bevy::{prelude::*, render::render_resource::*};
use ca_compute::{CAImage, CAPlugin};
use fly_cam::{MovementSettings, PlayerPlugin};
use gui::GuiPlugin;
use rtmaterial::{RTMatPlugin, RTVolumeMaterial};
use rule::{Rule, RulePlugin};

const WORKGROUP_SIZE: u32 = 8;

const START_SPEED: f32 = 0.1;
const START_SENSITIVITY: f32 = 0.0004;

fn main() {
    App::new()
        .add_startup_system(setup)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            present_mode: bevy::window::PresentMode::Immediate,
            title: "Cellular Automata".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(MovementSettings {
            sensitivity: START_SENSITIVITY,
            speed: START_SPEED,
        })
        .add_plugin(PlayerPlugin)
        .add_plugin(RulePlugin)
        .add_plugin(CAPlugin)
        .add_plugin(RTMatPlugin)
        .add_plugin(GuiPlugin)
        .add_system(update_size)
        .add_system(update_shape)
        .run();
}

struct CurrentSize(u32);

struct Meshes {
    meshes: Vec<(&'static str, Handle<Mesh>)>,
    current: usize,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<RTVolumeMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    rule: Res<Rule>,
    asset_server: Res<AssetServer>,
) {
    asset_server.watch_for_changes().unwrap();
    let mut image = Image::new_fill(
        Extent3d {
            width: rule.size,
            height: rule.size,
            depth_or_array_layers: rule.size,
        },
        bevy::render::render_resource::TextureDimension::D3,
        &[0],
        TextureFormat::R8Uint,
    );
    image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

    let meshes = Meshes {
        current: 0,
        meshes: vec![
            ("Box", meshes.add(Mesh::from(shape::Cube { size: 2.0 }))),
            (
                "Sphere",
                meshes.add(Mesh::from(shape::UVSphere {
                    radius: 1.0,
                    sectors: 50,
                    stacks: 50,
                })),
            ),
            (
                "Torus",
                meshes.add(Mesh::from(shape::Torus {
                    radius: 0.8,
                    ring_radius: 0.2,
                    subdivisions_segments: 75,
                    subdivisions_sides: 50,
                })),
            ),
        ],
    };
    let image = images.add(image);
    {
        commands.spawn_bundle(MaterialMeshBundle::<RTVolumeMaterial> {
            mesh: meshes.meshes[meshes.current].1.clone(),
            material: materials.add(RTVolumeMaterial {
                volume: image.clone(),
            }),
            ..Default::default()
        });
    }

    commands.insert_resource(meshes);
    commands.insert_resource(CurrentSize(rule.size));
    commands.insert_resource(CAImage(image));
}

fn update_size(
    image: Res<CAImage>,
    mut images: ResMut<Assets<Image>>,
    rule: Res<Rule>,
    mut size: ResMut<CurrentSize>,
) {
    if size.0 != rule.size {
        size.0 = rule.size;
        if let Some(image) = images.get_mut(image.0.clone()) {
            image.resize(Extent3d {
                width: rule.size,
                height: rule.size,
                depth_or_array_layers: rule.size,
            });
        }
    }
}

fn update_shape(meshes: Res<Meshes>, mut mesh: Query<&mut Handle<Mesh>>, mut last: Local<usize>) {
    if *last != meshes.current {
        for mut mesh in mesh.iter_mut() {
            *mesh = meshes.meshes[meshes.current].1.clone();
        }
        *last = meshes.current;
    }
}
