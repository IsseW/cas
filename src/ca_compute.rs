use std::{borrow::Cow, num::NonZeroU64};

use bevy::{
    core_pipeline::node::MAIN_PASS_DEPENDENCIES,
    prelude::*,
    render::{
        render_asset::*,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        RenderApp, RenderStage,
    },
};

use crate::{
    rule::{Rule, RuleBuffer},
    WORKGROUP_SIZE,
};

pub struct CAPlugin;

impl Plugin for CAPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReInit>().init_resource::<UpdateTime>();
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ReInit>()
            .init_resource::<CAPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_ca_image)
            .add_system_to_stage(RenderStage::Extract, update_timer)
            .add_system_to_stage(RenderStage::Queue, queue_bind_group.after("rule_buffer"));

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_node("cas", DispatchCA::default());
        render_graph
            .add_node_edge("cas", MAIN_PASS_DEPENDENCIES)
            .unwrap();
    }
}

#[derive(Default, Clone)]
pub struct ReInit(pub bool);

pub struct CAImage(pub Handle<Image>);
struct CABindGroup(BindGroup);

fn extract_ca_image(
    mut commands: Commands,
    image: Res<CAImage>,
    mut reinit: ResMut<ReInit>,
    input: Res<Input<KeyCode>>,
) {
    commands.insert_resource(CAImage(image.0.clone()));
    commands.insert_resource(ReInit(reinit.0 || input.just_pressed(KeyCode::R)));
    reinit.0 = false;
}

pub struct UpdateTime(pub f64);

impl Default for UpdateTime {
    fn default() -> Self {
        UpdateTime(0.1)
    }
}

struct DoUpdate(bool);

fn update_timer(
    mut commands: Commands,
    update_time: Res<UpdateTime>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut last_update: Local<f64>,
) {
    let t = time.time_since_startup().as_secs_f64();
    if t - *last_update > update_time.0 || input.just_pressed(KeyCode::E) {
        *last_update = t;
        commands.insert_resource(DoUpdate(true));
    } else {
        commands.insert_resource(DoUpdate(false));
    }
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<CAPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    ca_image: Res<CAImage>,
    rule: Res<RuleBuffer>,
    render_device: Res<RenderDevice>,
    bind_group: Option<ResMut<CABindGroup>>,
) {
    if bind_group.is_none() {
        if let Some(rule) = rule.0.as_ref() {
            let view = &gpu_images[&ca_image.0];
            let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipeline.bind_group,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&view.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: rule.as_entire_binding(),
                    },
                ],
            });
            commands.insert_resource(CABindGroup(bind_group));
        }
    }
}

pub struct CAPipeline {
    bind_group: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for CAPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::R8Uint,
                                view_dimension: TextureViewDimension::D3,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(NonZeroU64::new(64).unwrap()),
                            },
                            count: None,
                        },
                    ],
                });

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
            bind_group,
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
        let bind_group = &world.get_resource::<CABindGroup>().unwrap().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<CAPipeline>();

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        let rule = world.get_resource::<Rule>().unwrap();
        let wg: u32 = rule.size / WORKGROUP_SIZE;
        pass.set_bind_group(0, bind_group, &[]);
        match self.state {
            CAState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch(wg, wg, wg);
            }
            CAState::UpdateRun => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch(wg, wg, wg);
            }
            _ => {}
        }

        Ok(())
    }
}
