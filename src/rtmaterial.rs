use std::num::NonZeroU64;

use bevy::{pbr::MaterialPipeline, prelude::*, reflect::TypeUuid, render::render_resource::*};

use crate::rule::{GPURule, Rule};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "1ae9c363-1234-4213-890e-192d81b00281"]
pub struct RTVolumeMaterial {
    pub volume: Option<Handle<Image>>,
    pub rule: Rule,
}

impl AsBindGroup for RTVolumeMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        images: &bevy::render::render_asset::RenderAssets<Image>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<PreparedBindGroup<Self>, AsBindGroupError> {
        let volume = self
            .volume
            .as_ref()
            .ok_or(AsBindGroupError::RetryNextUpdate)?;
        let image = images
            .get(volume)
            .ok_or(AsBindGroupError::RetryNextUpdate)?;

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("rule_buffer"),
            contents: bytemuck::bytes_of(&GPURule::from(&self.rule)),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Ok(PreparedBindGroup {
            bindings: vec![OwnedBindingResource::Buffer(buffer)],
            bind_group,
            data: (),
        })
    }

    fn bind_group_layout(render_device: &bevy::render::renderer::RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::R8Uint,
                        view_dimension: TextureViewDimension::D3,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
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
}

impl Material for RTVolumeMaterial {
    fn vertex_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct RTMatPlugin;

impl Plugin for RTMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<RTVolumeMaterial>::default());
    }
}
