use std::borrow::Cow;

use bevy::{
    pbr::RenderMaterials,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Extract, RenderApp, RenderStage,
    },
};

use crate::{rtmaterial::RTVolumeMaterial, rule::Rule, WORKGROUP_SIZE};

pub struct CAPlugin;

impl Plugin for CAPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReInit>().init_resource::<UpdateTime>();
        app.add_plugin(ExtractResourcePlugin::<CAImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ReInit>()
            .init_resource::<CAPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_reinit)
            .add_system_to_stage(RenderStage::Extract, update_timer);

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_node("cas", DispatchCA::default());
        render_graph
            .add_node_edge("cas", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

#[derive(Default, Clone, Resource)]
pub struct ReInit(pub bool);

#[derive(Resource, Clone, ExtractResource)]
pub struct CAImage(pub Handle<Image>);

fn extract_reinit(
    mut commands: Commands,
    reinit: Extract<Res<ReInit>>,
    input: Extract<Res<Input<KeyCode>>>,
) {
    commands.insert_resource(ReInit(reinit.0 || input.just_pressed(KeyCode::R)));
}

#[derive(Resource)]
pub struct UpdateTime(pub f64);

impl Default for UpdateTime {
    fn default() -> Self {
        UpdateTime(0.1)
    }
}

#[derive(Resource)]
struct DoUpdate(bool);

fn update_timer(
    mut commands: Commands,
    update_time: Extract<Res<UpdateTime>>,
    input: Extract<Res<Input<KeyCode>>>,
    time: Extract<Res<Time>>,
    mut last_update: Local<f64>,
) {
    let t = time.elapsed_seconds_f64();
    if t - *last_update > update_time.0 || input.just_pressed(KeyCode::E) {
        *last_update = t;
        commands.insert_resource(DoUpdate(true));
    } else {
        commands.insert_resource(DoUpdate(false));
    }
}

#[derive(Resource)]
pub struct CAPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for CAPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group = RTVolumeMaterial::bind_group_layout(world.resource::<RenderDevice>());

        let shader = world
            .resource::<AssetServer>()
            .load("../assets/compute.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![bind_group.clone()]),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![bind_group.clone()]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        CAPipeline {
            init_pipeline,
            update_pipeline,
        }
    }
}

enum CAState {
    Loading,
    Init,
    Update,
    UpdateRun,
}

struct DispatchCA {
    state: CAState,
}

impl Default for DispatchCA {
    fn default() -> Self {
        Self {
            state: CAState::Loading,
        }
    }
}

impl render_graph::Node for DispatchCA {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<CAPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let do_update = world.resource::<DoUpdate>();

        match self.state {
            CAState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = CAState::Init
                }
            }
            CAState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = CAState::Update
                }
            }
            CAState::Update => {
                if do_update.0 {
                    self.state = CAState::UpdateRun
                }
            }
            CAState::UpdateRun => {
                self.state = CAState::Update;
            }
        }
        if let Some(ReInit(true)) = world.get_resource() {
            self.state = CAState::Init;
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let materials = &world
            .get_resource::<RenderMaterials<RTVolumeMaterial>>()
            .unwrap()
            .0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<CAPipeline>();
        let rule = world.get_resource::<Rule>().unwrap();

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        for (_handle, prepared) in materials {
            let wg: u32 = rule.size / WORKGROUP_SIZE;
            pass.set_bind_group(0, &prepared.bind_group, &[]);
            match self.state {
                CAState::Init => {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(wg, wg, wg);
                }
                CAState::UpdateRun => {
                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(wg, wg, wg);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
