use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{
            AddressMode, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FilterMode, FragmentState, MultisampleState,
            PipelineCache, PrimitiveState, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use bevy::render::render_resource::binding_types as binding;

use super::{
    resource::{GpuPointLight2d, GpuShadowView2d},
    SHADOW_MAIN_PASS_SHADER, SHADOW_PREPASS_SHADER,
};

#[derive(Resource)]
pub struct Shadow2dPrepassPipeline {
    pub cached_id: CachedRenderPipelineId,
    pub shadow_sampler: Sampler,
    pub prepass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dPrepassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let shadow_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("shadow2d_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let prepass_layout = render_device.create_bind_group_layout(
            "shadow2d_prepass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // Main texture
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let cached_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("shadow2d_prepass_pipeline".into()),
                    layout: vec![prepass_layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: SHADOW_PREPASS_SHADER,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::Rg32Float,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                });

        Self {
            cached_id,
            prepass_layout,
            shadow_sampler,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dMainPassPipeline {
    pub cached_id: CachedRenderPipelineId,
    pub main_pass_layout: BindGroupLayout,
    pub main_texture_sampler: Sampler,
}

impl FromWorld for Shadow2dMainPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let main_pass_layout = render_device.create_bind_group_layout(
            "main_pass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // Shadow map
                    binding::texture_2d(TextureSampleType::Float { filterable: false }),
                    binding::sampler(SamplerBindingType::NonFiltering),
                    // Main texture
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                    // Point lights
                    binding::storage_buffer::<Vec<GpuPointLight2d>>(false),
                    // Shadow views
                    binding::storage_buffer::<Vec<GpuShadowView2d>>(false),
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
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let cached_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("shadow2d_main_pass_pipeline".into()),
                    layout: vec![main_pass_layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
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
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                });

        Self {
            cached_id,
            main_pass_layout,
            main_texture_sampler,
        }
    }
}
