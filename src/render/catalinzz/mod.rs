use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::{
        entity::Entity,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
        world::{FromWorld, World},
    },
    math::{UVec3, UVec4, Vec2},
    render::{
        camera::{camera_system, OrthographicProjection, PerspectiveProjection, Projection},
        color::Color,
        extract_resource::ExtractResourcePlugin,
        render_graph::RenderGraphApp,
        render_resource::{
            AddressMode, BindingResource, DynamicUniformBuffer, Extent3d, FilterMode,
            GpuArrayBuffer, SamplerDescriptor, Shader, ShaderType, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, GpuImage, TextureCache},
        view::{ColorGrading, ExtractedView, Msaa, ViewTarget, VisibilitySystems},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    transform::{components::GlobalTransform, TransformSystem},
};
use fast_poisson::Poisson2D;

use crate::{
    ecs::{
        catalinzz::{MainShadowCameraDriver, ShadowMap2dConfig},
        PointLight2d, ShadowView2d, SpotLight2d,
    },
    render::catalinzz::graph::{
        Shadow2dDistortPassNode, Shadow2dMainPassNode, Shadow2dMeshPassNode, Shadow2dNode,
        Shadow2dPrepassNode, Shadow2dReductionNode,
    },
};

use self::pipeline::{
    Shadow2dDistortPassPipeline, Shadow2dMainPassPipeline, Shadow2dPrepassPipeline,
    Shadow2dReductionPipeline,
};

use bevy::render::view::visibility as bevy_visibility;

use super::{DynamicUniformIndex, ExtractedPointLight2d};

pub mod graph;
pub mod pipeline;
pub mod visibility;

pub const SHADOW_TYPES: Handle<Shader> = Handle::weak_from_u128(1123087897454135486384145234748455);
pub const SHADOW_DISTORT_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13745315343641643643);
pub const SHADOW_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(5321368413218521485631341);
pub const SHADOW_REDUCTION_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(485648964891315351);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13643651896413518964153);
pub const SHADOW_WORKGROUP_SIZE: UVec3 = UVec3 { x: 16, y: 16, z: 1 };

#[cfg(feature = "compatibility")]
pub const SHADOW_MAP_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
#[cfg(not(feature = "compatibility"))]
pub const SHADOW_MAP_FORMAT: TextureFormat = TextureFormat::Rg32Float;

pub const ALPHA_MAP_FORMAT: TextureFormat = TextureFormat::R32Float;

pub struct CatalinzzApproachPlugin;

impl Plugin for CatalinzzApproachPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_TYPES,
            "shaders/shadow_2d_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_DISTORT_PASS_SHADER,
            "shaders/shadow_2d_distort_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_PREPASS_SHADER,
            "shaders/shadow_2d_prepass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_REDUCTION_PASS_SHADER,
            "shaders/shadow_2d_reduction_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_MAIN_PASS_SHADER,
            "shaders/shadow_2d_main_pass.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(ExtractResourcePlugin::<ShadowMap2dConfig>::default())
            .init_resource::<ShadowMap2dConfig>()
            .register_type::<ShadowMap2dConfig>()
            .add_systems(
                PostUpdate,
                (
                    (
                        visibility::update_point_light_frusta,
                        visibility::update_spot_light_frusta,
                    )
                        .in_set(VisibilitySystems::UpdateOrthographicFrusta)
                        .after(camera_system::<OrthographicProjection>)
                        .after(TransformSystem::TransformPropagate)
                        .ambiguous_with(bevy_visibility::update_frusta::<PerspectiveProjection>)
                        .ambiguous_with(bevy_visibility::update_frusta::<Projection>),
                    visibility::check_caster_visibility
                        .in_set(VisibilitySystems::CheckVisibility)
                        .after(VisibilitySystems::CalculateBounds)
                        .after(VisibilitySystems::UpdateOrthographicFrusta)
                        .after(VisibilitySystems::UpdatePerspectiveFrusta)
                        .after(VisibilitySystems::UpdateProjectionFrusta)
                        .after(VisibilitySystems::VisibilityPropagate)
                        .after(TransformSystem::TransformPropagate)
                        .after(bevy_visibility::check_visibility),
                ),
            );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GpuMetaBuffers>()
            .add_systems(ExtractSchedule, extract_light_view)
            .add_systems(
                Render,
                (prepare_lights, prepare_poisson_disk).in_set(RenderSet::Prepare),
            )
            .add_render_graph_node::<Shadow2dMeshPassNode>(Core2d, Shadow2dNode::Shadow2dMeshPass)
            .add_render_graph_node::<Shadow2dPrepassNode>(Core2d, Shadow2dNode::Shadow2dPrepass)
            .add_render_graph_node::<Shadow2dDistortPassNode>(
                Core2d,
                Shadow2dNode::Shadow2dDistortPass,
            )
            .add_render_graph_node::<Shadow2dReductionNode>(
                Core2d,
                Shadow2dNode::Shadow2dReductionPass,
            )
            .add_render_graph_node::<Shadow2dMainPassNode>(Core2d, Shadow2dNode::Shadow2dMainPass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    Shadow2dNode::Shadow2dMeshPass,
                    Shadow2dNode::Shadow2dPrepass,
                    Shadow2dNode::Shadow2dDistortPass,
                    Shadow2dNode::Shadow2dReductionPass,
                    Shadow2dNode::Shadow2dMainPass,
                    Node2d::Bloom,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<Shadow2dPrepassPipeline>()
            .init_resource::<Shadow2dDistortPassPipeline>()
            .init_resource::<Shadow2dReductionPipeline>()
            .init_resource::<Shadow2dMainPassPipeline>()
            .init_resource::<ShadowMap2dStorage>()
            .init_resource::<PoissonDiskBuffer>();
    }
}

pub fn extract_light_view(
    mut commands: Commands,
    point_lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform)>>,
    spot_lights_query: Extract<Query<(Entity, &SpotLight2d, &GlobalTransform)>>,
    shadow_map_config: Extract<Res<ShadowMap2dConfig>>,
) {
    commands.insert_or_spawn_batch(
        point_lights_query
            .iter()
            .map(|(entity, light, transform)| {
                let transform = GlobalTransform::from_translation(transform.translation());
                (
                    entity,
                    ExtractedView {
                        projection: shadow_map_config.get_proj_mat(light.range * 2.),
                        transform,
                        view_projection: None,
                        hdr: false,
                        viewport: UVec4::ZERO,
                        color_grading: ColorGrading::default(),
                    },
                )
            })
            .collect::<Vec<_>>(),
    );

    commands.insert_or_spawn_batch(
        spot_lights_query
            .iter()
            .map(|(entity, light, transform)| {
                let transform = GlobalTransform::from_translation(transform.translation());
                (
                    entity,
                    ExtractedView {
                        projection: shadow_map_config.get_proj_mat(light.range * 2.),
                        transform,
                        view_projection: None,
                        hdr: false,
                        viewport: UVec4::ZERO,
                        color_grading: ColorGrading::default(),
                    },
                )
            })
            .collect::<Vec<_>>(),
    );
}

#[derive(ShaderType)]
pub struct GpuShadowMapMeta {
    pub index: u32,
    pub size: u32,
    pub offset: Vec2,
    pub bias: f32,
    pub alpha_threshold: f32,
    pub pcf_samples: u32,
    pub pcf_radius: f32,
}

#[derive(Resource, Default)]
pub struct GpuMetaBuffers {
    shadow_map: DynamicUniformBuffer<GpuShadowMapMeta>,
    reduction: DynamicUniformBuffer<u32>,
    reduction_offsets: Vec<u32>,
}

impl GpuMetaBuffers {
    #[inline]
    pub fn push_light_meta(
        &mut self,
        meta: GpuShadowMapMeta,
    ) -> DynamicUniformIndex<GpuShadowMapMeta> {
        DynamicUniformIndex::new(self.shadow_map.push(&meta))
    }

    #[inline]
    pub fn init_reduction_time_buffer(&mut self, num_reductions: u32) {
        self.reduction.clear();
        self.reduction_offsets.clear();

        for i in 0..num_reductions {
            let idx = self.reduction.push(&i);
            self.reduction_offsets.push(idx);
        }
    }

    #[inline]
    pub fn get_reduction_index(&self, reduction: u32) -> u32 {
        self.reduction_offsets[reduction as usize]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.shadow_map.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.shadow_map.write_buffer(render_device, render_queue);
        self.reduction.write_buffer(render_device, render_queue);
    }

    #[inline]
    pub fn shadow_map_meta_buffer_binding(&self) -> BindingResource {
        self.shadow_map.binding().unwrap()
    }

    // This buffer keeps panic if unwrap directly, not sure why
    #[inline]
    pub fn reduction_time_buffer_binding(&self) -> Option<BindingResource> {
        self.reduction.binding()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ShadowMap2dMeta {
    pub count: u32,
    pub size: u32,
}

#[derive(Resource, Default)]
pub struct ShadowMap2dStorage {
    meta: ShadowMap2dMeta,
    primary_shadow_map: Option<GpuImage>,
    secondary_shadow_map: Option<GpuImage>,
    alpha_map: Option<GpuImage>,
    work_group_count_per_light: UVec3,
    work_group_count_total: UVec3,
    num_reductions: u32,
}

impl ShadowMap2dStorage {
    pub fn try_update(
        &mut self,
        meta: ShadowMap2dMeta,
        render_device: &RenderDevice,
        meta_buffers: &mut GpuMetaBuffers,
    ) {
        if self.meta == meta {
            return;
        }

        self.meta = meta;
        self.primary_shadow_map = Some(self.create_shadow_map(render_device, SHADOW_MAP_FORMAT));
        self.secondary_shadow_map = Some(self.create_shadow_map(render_device, SHADOW_MAP_FORMAT));
        self.alpha_map = Some(self.create_shadow_map(render_device, ALPHA_MAP_FORMAT));
        self.work_group_count_per_light = UVec3 {
            x: meta.size.div_ceil(SHADOW_WORKGROUP_SIZE.x),
            y: meta.size.div_ceil(SHADOW_WORKGROUP_SIZE.y),
            z: 1,
        };
        self.work_group_count_total = UVec3 {
            x: self.work_group_count_per_light.x,
            y: self.work_group_count_per_light.y,
            z: meta.count,
        };
        self.num_reductions = meta.size.trailing_zeros();

        assert_eq!(
            2u32.pow(self.num_reductions),
            self.meta.size,
            "Shadow map size must be a power of 2!"
        );

        meta_buffers.init_reduction_time_buffer(self.num_reductions);
    }

    #[inline]
    pub fn texture_view_primary(&self) -> &TextureView {
        &self.primary_shadow_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn texture_view_secondary(&self) -> &TextureView {
        &self.secondary_shadow_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn alpha_map_view(&self) -> &TextureView {
        &self.alpha_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn final_texture_view(&self) -> &TextureView {
        if self.num_reductions % 2 == 0 {
            self.texture_view_secondary()
        } else {
            self.texture_view_primary()
        }
    }

    #[inline]
    pub fn work_group_count_per_light(&self) -> UVec3 {
        self.work_group_count_per_light
    }

    #[inline]
    pub fn work_group_count_total(&self) -> UVec3 {
        self.work_group_count_total
    }

    #[inline]
    pub fn num_reductions(&self) -> u32 {
        self.num_reductions
    }

    fn create_shadow_map(&self, render_device: &RenderDevice, format: TextureFormat) -> GpuImage {
        let meta = self.meta;

        let shadow_map = render_device.create_texture(&TextureDescriptor {
            label: Some("shadow_map_2d"),
            size: Extent3d {
                width: meta.size,
                height: meta.size,
                depth_or_array_layers: meta.count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        GpuImage {
            texture_view: shadow_map.create_view(&TextureViewDescriptor {
                label: Some("shadow_map_2d_view"),
                format: Some(shadow_map.format()),
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: Some(meta.count),
            }),
            texture_format: shadow_map.format(),
            texture: shadow_map,
            sampler: render_device.create_sampler(&SamplerDescriptor {
                label: Some("shadow_map_2d_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            }),
            size: Vec2::splat(meta.size as f32),
            mip_level_count: 0,
        }
    }
}

#[derive(Resource)]
pub struct PoissonDiskBuffer {
    count: u32,
    buffer: GpuArrayBuffer<Vec2>,
}

impl FromWorld for PoissonDiskBuffer {
    fn from_world(world: &mut World) -> Self {
        Self {
            count: 0,
            buffer: GpuArrayBuffer::new(&world.resource::<RenderDevice>()),
        }
    }
}

impl PoissonDiskBuffer {
    pub fn regen_by_shadow_map(
        &mut self,
        shadow_map_config: &ShadowMap2dConfig,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) {
        let config = shadow_map_config.pcf;
        if self.count == config.samples {
            return;
        }

        self.buffer.clear();
        Poisson2D::new()
            .with_seed(config.seed as u64)
            .with_samples(config.samples)
            .into_iter()
            .map(|p| Vec2::new(p[0] as f32, p[1] as f32) * 2. - 1.)
            .for_each(|p| {
                self.buffer.push(p);
            });
        self.buffer.write_buffer(render_device, render_queue);
        self.count = config.samples;
    }

    #[inline]
    pub fn binding(&self) -> BindingResource {
        self.buffer.binding().unwrap()
    }
}

pub fn prepare_poisson_disk(
    mut buffer: ResMut<PoissonDiskBuffer>,
    shadow_map_config: Res<ShadowMap2dConfig>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    buffer.regen_by_shadow_map(&shadow_map_config, &render_device, &render_queue);
}

pub fn prepare_lights(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    main_views: Query<Entity, With<ViewTarget>>,
    point_lights: Query<(Entity, &ExtractedPointLight2d)>,
    shadow_map_config: Res<ShadowMap2dConfig>,
    mut shadow_map_storage: ResMut<ShadowMap2dStorage>,
    mut gpu_meta_buffers: ResMut<GpuMetaBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    msaa: Res<Msaa>,
) {
    assert_eq!(*msaa, Msaa::Off, "MSAA is not supported yet!");

    let point_light_count = point_lights.iter().count();
    if point_light_count == 0 {
        return;
    }

    gpu_meta_buffers.clear();

    let mut point_lights = point_lights.iter().collect::<Vec<_>>();
    radsort::sort_by_key(&mut point_lights, |(_, light)| light.id);

    for (light_index, (light_entity, _)) in point_lights.into_iter().enumerate() {
        // TODO support different settings for different lights
        let meta_index = gpu_meta_buffers.push_light_meta(GpuShadowMapMeta {
            index: light_index as u32,
            size: shadow_map_config.size,
            offset: shadow_map_config.offset,
            bias: shadow_map_config.bias,
            alpha_threshold: shadow_map_config.alpha_threshold,
            pcf_samples: shadow_map_config.pcf.samples,
            pcf_radius: shadow_map_config.pcf.radius,
        });

        let point_light_view_mesh_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("point_light_view_mesh_texture"),
                size: Extent3d {
                    width: shadow_map_config.size,
                    height: shadow_map_config.size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );

        let shadow_view = ShadowView2d {
            attachment: ColorAttachment::new(
                point_light_view_mesh_texture.clone(),
                None,
                Some(Color::NONE),
            ),
        };

        commands
            .entity(light_entity)
            .insert((meta_index, shadow_view));
    }

    gpu_meta_buffers.write_buffers(&render_device, &render_queue);

    shadow_map_storage.try_update(
        ShadowMap2dMeta {
            count: point_light_count as u32,
            size: shadow_map_config.size,
        },
        &render_device,
        &mut gpu_meta_buffers,
    );

    if let Some(shadow_camera) = main_views.iter().next() {
        commands
            .entity(shadow_camera)
            .insert(MainShadowCameraDriver);
    }
}
