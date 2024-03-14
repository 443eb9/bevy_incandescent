use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{
            AddressMode, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            CachedPipeline, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            ComputePipelineDescriptor, FilterMode, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, StorageTextureAccess, TextureFormat,
            TextureSampleType,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use bevy::render::render_resource::binding_types as binding;

use super::{SHADOW_JFA_PREPASS_SHADER, SHADOW_MAIN_PASS_SHADER};

#[derive(Resource)]
pub struct Shadow2dJfaPrepassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub jfa_pass_layout: BindGroupLayout,
    pub main_texture_sampler: Sampler,
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
                    binding::sampler(SamplerBindingType::Filtering),
                    // Jfa texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::WriteOnly,
                    ),
                ),
            ),
        );

        let main_texture_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("main_texture_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

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
            main_texture_sampler,
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
                    // Jfa texture
                    binding::texture_storage_2d(
                        TextureFormat::Rgba32Float,
                        StorageTextureAccess::ReadOnly,
                    ),
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
