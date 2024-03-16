use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::{
        entity::{Entity, EntityHashMap},
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{UVec2, UVec3, Vec4Swizzles},
    render::{
        extract_resource::ExtractResourcePlugin,
        render_graph::RenderGraphApp,
        render_resource::{
            AddressMode, BindingResource, DynamicUniformBuffer, Extent3d, FilterMode,
            SamplerDescriptor, Shader, ShaderType, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, GpuImage, TextureCache},
        view::ExtractedView,
        Render, RenderApp, RenderSet,
    },
    utils::hashbrown::hash_map::Entry,
};

use crate::{
    ecs::{ray_marching::RayMarchingConfig, ShadowView2d},
    render::ray_marching::{
        graph::{
            Shadow2dJfaPassNode, Shadow2dJfaPrepassNode, Shadow2dMainPassNode,
            Shadow2dMeshPassNode, Shadow2dNode, Shadow2dSdfPassNode,
        },
        pipeline::Shadow2dJfaPassPipeline,
    },
};

use self::pipeline::{
    Shadow2dJfaPrepassPipeline, Shadow2dMainPassPipeline, Shadow2dSdfPassPipeline,
};

use super::DynamicUniformIndex;

pub mod graph;
pub mod pipeline;

pub const SHADOW_TYPES: Handle<Shader> = Handle::weak_from_u128(1324875856134874561658341356446384);
pub const SHADOW_JFA_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(634365103587949153484);
pub const SHADOW_JFA_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(478965431865746153863534);
pub const SHADOW_SDF_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(187843189640484036549801);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(98749653156334136411638);
pub const SHADOW_WORK_GROUP_SIZE: UVec3 = UVec3 { x: 16, y: 16, z: 1 };

pub struct RayMarchingApproachPlugin;

impl Plugin for RayMarchingApproachPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_TYPES,
            "shaders/shadow_2d_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_JFA_PREPASS_SHADER,
            "shaders/jfa_prepass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_JFA_PASS_SHADER,
            "shaders/jfa_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_SDF_PASS_SHADER,
            "shaders/sdf_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_MAIN_PASS_SHADER,
            "shaders/shadow_2d_main_pass.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(ExtractResourcePlugin::<RayMarchingConfig>::default())
            .init_resource::<RayMarchingConfig>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GpuMetaBuffers>()
            .init_resource::<SdfTextureStorage>()
            .add_systems(Render, (prepare,).in_set(RenderSet::Prepare))
            .add_render_graph_node::<Shadow2dMeshPassNode>(Core2d, Shadow2dNode::Shadow2dMeshPass)
            .add_render_graph_node::<Shadow2dJfaPrepassNode>(
                Core2d,
                Shadow2dNode::Shadow2dJfaPrepass,
            )
            .add_render_graph_node::<Shadow2dJfaPassNode>(Core2d, Shadow2dNode::Shadow2dJfaPass)
            .add_render_graph_node::<Shadow2dSdfPassNode>(Core2d, Shadow2dNode::Shadow2dSdfPass)
            .add_render_graph_node::<Shadow2dMainPassNode>(Core2d, Shadow2dNode::Shadow2dMainPass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    Shadow2dNode::Shadow2dMeshPass,
                    Shadow2dNode::Shadow2dJfaPrepass,
                    Shadow2dNode::Shadow2dJfaPass,
                    // Shadow2dNode::Shadow2dSdfPass,
                    Shadow2dNode::Shadow2dMainPass,
                    Node2d::Bloom,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<Shadow2dJfaPrepassPipeline>()
            .init_resource::<Shadow2dJfaPassPipeline>()
            .init_resource::<Shadow2dSdfPassPipeline>()
            .init_resource::<Shadow2dMainPassPipeline>();
    }
}

pub fn prepare(
    mut commands: Commands,
    main_view_query: Query<(Entity, &ExtractedView)>,
    mut sdf_texture_storage: ResMut<SdfTextureStorage>,
    mut texture_cache: ResMut<TextureCache>,
    ray_marching_config: Res<RayMarchingConfig>,
    mut gpu_meta_buffers: ResMut<GpuMetaBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    gpu_meta_buffers.clear();

    for (main_view_entity, extracted_view) in &main_view_query {
        let sdf_tex_size =
            (extracted_view.viewport.zw().as_vec2() * ray_marching_config.scale).as_uvec2();
        sdf_texture_storage.try_add_main_view(main_view_entity, sdf_tex_size, &render_device);
        gpu_meta_buffers.init_jfa_iteration_buffer(
            main_view_entity,
            sdf_tex_size.x.ilog2().max(sdf_tex_size.y.ilog2()) as u32,
        );

        let offset = gpu_meta_buffers.add_sdf_meta(SdfMeta {
            size: sdf_tex_size,
            alpha_threshold: 0.1,
        });

        let main_view_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("main_view_texture"),
                size: Extent3d {
                    width: sdf_tex_size.x,
                    height: sdf_tex_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        commands.entity(main_view_entity).insert((
            offset,
            ShadowView2d {
                attachment: ColorAttachment::new(main_view_texture, None, None),
            },
        ));
    }

    gpu_meta_buffers.write_buffers(&render_device, &render_queue);
}

pub struct SdfTexture {
    primary: GpuImage,
    secondary: GpuImage,
    size: UVec2,
    jfa_iterations: u32,
}

impl SdfTexture {
    pub fn new(size: UVec2, render_device: &RenderDevice) -> Self {
        let primary = Self::create_sdf_texture(size, render_device);
        let secondary = Self::create_sdf_texture(size, render_device);

        Self {
            primary,
            secondary,
            size,
            jfa_iterations: size.x.max(size.y).ilog2()
        }
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn get_texture(&self) -> &GpuImage {
        if self.jfa_iterations % 2 == 0 {
            self.get_primary_texture()
        } else {
            self.get_secondary_texture()
        }
    }

    #[inline]
    pub fn get_primary_texture(&self) -> &GpuImage {
        &self.primary
    }

    #[inline]
    pub fn get_secondary_texture(&self) -> &GpuImage {
        &self.secondary
    }

    fn create_sdf_texture(size: UVec2, render_device: &RenderDevice) -> GpuImage {
        let texture = render_device.create_texture(&TextureDescriptor {
            label: Some("sdf_texture"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &vec![],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("sdf_texture_view"),
            format: Some(texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("sdf_texture_sampler"),
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
        });

        GpuImage {
            texture_format: texture.format(),
            texture,
            texture_view,
            sampler,
            size: size.as_vec2(),
            mip_level_count: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct SdfTextureStorage(EntityHashMap<SdfTexture>);

impl SdfTextureStorage {
    pub fn try_add_main_view(
        &mut self,
        main_view: Entity,
        size: UVec2,
        render_device: &RenderDevice,
    ) {
        let entry = self.0.entry(main_view);
        match &entry {
            Entry::Occupied(occ_e) => {
                if occ_e.get().size == size {
                    return;
                }
            }
            Entry::Vacant(_) => {}
        }

        entry.insert(SdfTexture::new(size, render_device));
    }

    #[inline]
    pub fn get_sdf_texture(&self, main_view: Entity) -> &SdfTexture {
        self.0.get(&main_view).unwrap()
    }
}

#[derive(ShaderType)]
pub struct SdfMeta {
    pub size: UVec2,
    pub alpha_threshold: f32,
}

#[derive(Resource, Default)]
pub struct GpuMetaBuffers {
    sdf_meta: DynamicUniformBuffer<SdfMeta>,
    jfa: EntityHashMap<(DynamicUniformBuffer<u32>, Vec<u32>)>,
}

impl GpuMetaBuffers {
    #[inline]
    pub fn add_sdf_meta(&mut self, meta: SdfMeta) -> DynamicUniformIndex<SdfMeta> {
        DynamicUniformIndex::new(self.sdf_meta.push(&meta))
    }

    #[inline]
    pub fn init_jfa_iteration_buffer(&mut self, main_view_entity: Entity, jfa_iterations: u32) {
        let (jfa_iteration, jfa_iteration_offsets) = self.jfa.entry(main_view_entity).or_default();

        jfa_iteration.clear();
        jfa_iteration_offsets.clear();

        for i in 0..jfa_iterations {
            let idx = jfa_iteration.push(&i);
            jfa_iteration_offsets.push(idx);
        }
    }

    #[inline]
    pub fn sdf_meta_binding(&self) -> BindingResource {
        self.sdf_meta.binding().unwrap()
    }

    #[inline]
    pub fn jfa_iteration_binding(&self, main_view_entity: Entity) -> BindingResource {
        self.jfa[&main_view_entity].0.binding().unwrap()
    }

    #[inline]
    pub fn get_jfa_iteration_index(&self, main_view_entity: Entity, iteration: u32) -> u32 {
        self.jfa[&main_view_entity].1[iteration as usize]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.sdf_meta.clear();
        self.jfa.iter_mut().for_each(|(_, (it, idx))| {
            it.clear();
            idx.clear();
        });
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.sdf_meta.write_buffer(render_device, render_queue);
        self.jfa.iter_mut().for_each(|(_, (it, _))| {
            it.write_buffer(render_device, render_queue);
        });
    }
}
