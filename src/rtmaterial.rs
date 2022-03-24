use std::num::NonZeroU64;

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{render_asset::*, render_resource::*, renderer::RenderDevice},
};

use crate::rule::RuleBuffer;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "1ae9c363-1234-4213-890e-192d81b00281"]
pub struct RTVolumeMaterial {
    pub volume: Handle<Image>,
}

pub struct GpuRTVolumeMaterial {
    bind_group: BindGroup,
}

struct CellInclude(Handle<Shader>);

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CellInclude(asset_server.load("cells.wgsl")));
}

impl RenderAsset for RTVolumeMaterial {
    type ExtractedAsset = RTVolumeMaterial;
    type PreparedAsset = GpuRTVolumeMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<MaterialPipeline<Self>>,
        SRes<RenderAssets<Image>>,
        SRes<RuleBuffer>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, material_pipeline, gpu_images, rule): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let (view, _) = if let Some(result) = material_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &Some(material.volume.clone()))
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let buf = if let Some(rule) = rule.0.as_ref() {
            rule.as_entire_binding()
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buf,
                },
            ],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuRTVolumeMaterial { bind_group })
    }
}

impl Material for RTVolumeMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shader.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shader.wgsl"))
    }

    fn bind_group(material: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(64).unwrap()),
                    },
                    count: None,
                },
            ],
        })
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct RTMatPlugin;

impl Plugin for RTMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<RTVolumeMaterial>::default())
            .add_system(load);
    }
}
