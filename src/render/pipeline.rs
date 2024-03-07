use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{
            AddressMode, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, ComputePipelineDescriptor,
            FilterMode, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderDefVal,
            ShaderStages, StorageTextureAccess, TextureFormat, TextureSampleType,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};

use bevy::render::render_resource::binding_types as binding;

use super::{
    resource::{GpuAmbientLight2d, GpuPointLight2d, GpuShadowMapMeta},
    SHADOW_DISTORT_PASS_SHADER, SHADOW_MAIN_PASS_SHADER, SHADOW_MAP_FORMAT, SHADOW_PREPASS_SHADER,
    SHADOW_REDUCTION_PASS_SHADER,
};

fn get_shader_defs() -> Vec<ShaderDefVal> {
    #[cfg(feature = "compatibility")]
    return vec!["COMPATIBILITY".into()];

    #[cfg(not(feature = "compatibility"))]
    return vec![];
}

#[derive(Resource)]
pub struct Shadow2dPrepassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub prepass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dPrepassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let prepass_layout = render_device.create_bind_group_layout(
            "shadow_2d_prepass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Main texture
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    // Shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::WriteOnly,
                    ),
                    // Shadow map meta
                    binding::uniform_buffer::<GpuShadowMapMeta>(true),
                ),
            ),
        );

        let cached_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("shadow_2d_prepass_pipeline".into()),
                layout: vec![prepass_layout.clone()],
                push_constant_ranges: vec![],
                shader: SHADOW_PREPASS_SHADER,
                shader_defs: get_shader_defs(),
                entry_point: "main".into(),
            });

        Self {
            cached_id,
            prepass_layout,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dDistortPassPipeline {
    pub cached_id: CachedComputePipelineId,
    pub distort_pass_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dDistortPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let distort_layout = render_device.create_bind_group_layout(
            "shadow_2d_distort_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Source shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::ReadOnly,
                    ),
                    // Destination shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::WriteOnly,
                    ),
                    // Shadow map meta
                    binding::uniform_buffer::<GpuShadowMapMeta>(false),
                ),
            ),
        );

        let cached_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("shadow_2d_distort_pipeline".into()),
                layout: vec![distort_layout.clone()],
                push_constant_ranges: vec![],
                shader: SHADOW_DISTORT_PASS_SHADER,
                shader_defs: get_shader_defs(),
                entry_point: "main".into(),
            });

        Self {
            cached_id,
            distort_pass_layout: distort_layout,
        }
    }
}

#[derive(Resource)]
pub struct Shadow2dReductionPipeline {
    pub cached_id: CachedComputePipelineId,
    pub reduction_layout: BindGroupLayout,
}

impl FromWorld for Shadow2dReductionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let reduction_layout = render_device.create_bind_group_layout(
            "shadow_2d_reduction_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Source shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::ReadWrite,
                    ),
                    // Destination shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::ReadWrite,
                    ),
                    // Shadow map meta
                    binding::uniform_buffer::<GpuShadowMapMeta>(false),
                    // Reduction time
                    binding::uniform_buffer::<u32>(true),
                ),
            ),
        );

        let cached_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("shadow_2d_reduction_pipeline".into()),
                layout: vec![reduction_layout.clone()],
                push_constant_ranges: vec![],
                shader: SHADOW_REDUCTION_PASS_SHADER,
                shader_defs: get_shader_defs(),
                entry_point: "main".into(),
            });

        Self {
            cached_id,
            reduction_layout,
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
                    // Main texture
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                    // Shadow map
                    binding::texture_storage_2d_array(
                        SHADOW_MAP_FORMAT,
                        StorageTextureAccess::ReadOnly,
                    ),
                    // Shadow views
                    binding::uniform_buffer::<ViewUniform>(true),
                    // Shadow map meta
                    binding::uniform_buffer::<GpuShadowMapMeta>(false),
                    // Ambient light
                    binding::uniform_buffer::<GpuAmbientLight2d>(false),
                    // Point lights
                    binding::storage_buffer_read_only::<Vec<GpuPointLight2d>>(false),
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
                    label: Some("shadow_2d_main_pass_pipeline".into()),
                    layout: vec![main_pass_layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: SHADOW_MAIN_PASS_SHADER,
                        shader_defs: get_shader_defs(),
                        entry_point: "fragment".into(),
                        // entry_point: "dbg_output_shadow_map".into(),
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
