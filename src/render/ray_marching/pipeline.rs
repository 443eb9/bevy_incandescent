use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, ComputePipelineDescriptor,
            FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, ShaderStages, StorageTextureAccess, TextureFormat,
            TextureSampleType,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use bevy::render::render_resource::binding_types as binding;

use super::{
    SdfMeta, SHADOW_JFA_PASS_SHADER, SHADOW_JFA_PREPASS_SHADER, SHADOW_MAIN_PASS_SHADER,
    SHADOW_SDF_PASS_SHADER,
};

#[derive(Resource)]
pub struct Shadow2dJfaPrepassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub jfa_pass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dJfaPrepassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let jfa_pass_layout = render_device.create_bind_group_layout(
            "shadow_2d_jfa_prepass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Main texture
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    // Sdf texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::WriteOnly,
                    ),
                    // Sdf meta
                    binding::uniform_buffer::<SdfMeta>(true),
                    // Is inverted
                    binding::uniform_buffer::<u32>(true),
                ),
            ),
        );

        let cached_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("shadow_2d_jfa_prepass_pipeline".into()),
                layout: vec![jfa_pass_layout.clone()],
                push_constant_ranges: vec![],
                shader: SHADOW_JFA_PREPASS_SHADER,
                shader_defs: vec![],
                entry_point: "main".into(),
            });

        Self {
            cached_id,
            jfa_pass_layout,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dJfaPassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub jfa_pass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dJfaPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let jfa_pass_layout = render_device.create_bind_group_layout(
            "shadow_2d_jfa_pass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Source sdf texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::ReadWrite,
                    ),
                    // Destination sdf texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::ReadWrite,
                    ),
                    // Sdf meta
                    binding::uniform_buffer::<SdfMeta>(true),
                    // Jfa iteration
                    binding::uniform_buffer::<u32>(true),
                ),
            ),
        );

        let cached_id =
            world
                .resource::<PipelineCache>()
                .queue_compute_pipeline(ComputePipelineDescriptor {
                    label: Some("shadow_2d_jfa_pass_pipeline".into()),
                    layout: vec![jfa_pass_layout.clone()],
                    push_constant_ranges: vec![],
                    shader: SHADOW_JFA_PASS_SHADER,
                    shader_defs: vec![],
                    entry_point: "main".into(),
                });

        Self {
            cached_id,
            jfa_pass_layout,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dSdfPassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub sdf_pass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dSdfPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let sdf_pass_layout = render_device.create_bind_group_layout(
            "shadow_2d_sdf_pass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Sdf texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::ReadWrite,
                    ),
                    // Sdf meta
                    binding::uniform_buffer::<SdfMeta>(true),
                ),
            ),
        );

        let cached_id =
            world
                .resource::<PipelineCache>()
                .queue_compute_pipeline(ComputePipelineDescriptor {
                    label: Some("shadow_2d_jfa_pass_pipeline".into()),
                    layout: vec![sdf_pass_layout.clone()],
                    push_constant_ranges: vec![],
                    shader: SHADOW_SDF_PASS_SHADER,
                    shader_defs: vec![],
                    entry_point: "main".into(),
                });

        Self {
            cached_id,
            sdf_pass_layout,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dMainPassPipeline {
    pub cached_id: CachedRenderPipelineId,
    pub main_pass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dMainPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let main_pass_layout = render_device.create_bind_group_layout(
            "shadow_2d_main_pass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // Sdf texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::ReadOnly,
                    ),
                    // Sdf meta
                    binding::uniform_buffer::<SdfMeta>(true),
                ),
            ),
        );

        let cached_id =
            world
                .resource::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("shadow_2d_main_pass_pipeline".into()),
                    layout: vec![main_pass_layout.clone()],
                    push_constant_ranges: vec![],
                    vertex: fullscreen_shader_vertex_state(),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        shader: SHADOW_MAIN_PASS_SHADER,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                });

        Self {
            cached_id,
            main_pass_layout,
        }
    }
}
