use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    ecs::{
        entity::{Entity, EntityHashMap},
        system::Resource,
    },
    math::{UVec2, UVec3},
    render::{
        render_resource::{
            AddressMode, Extent3d, FilterMode, SamplerDescriptor, Shader, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::GpuImage,
    },
    utils::hashbrown::hash_map::Entry,
};

pub mod graph;
pub mod pipeline;

pub const SHADOW_JFA_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(634365103587949153484);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(98749653156334136411638);
pub const SHADOW_JFA_PREPASS_WORK_GROUP_SIZE: UVec3 = UVec3 { x: 16, y: 16, z: 1 };

pub struct SdfApproachPlugin;

impl Plugin for SdfApproachPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_JFA_PREPASS_SHADER,
            "shaders/jfa.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Resource, Default)]
pub struct JfaTextureStorage(EntityHashMap<(GpuImage, UVec2)>);

impl JfaTextureStorage {
    pub fn try_add_main_view(
        &mut self,
        main_view: Entity,
        size: UVec2,
        render_device: &RenderDevice,
    ) {
        let entry = self.0.entry(main_view);
        match &entry {
            Entry::Occupied(occ_e) => {
                if occ_e.get().1 == size {
                    return;
                }
            }
            Entry::Vacant(_) => {}
        }

        let texture = render_device.create_texture(&TextureDescriptor {
            label: Some("jfa_texture"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 0,
            sample_count: 0,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &vec![],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("jfa_texture_view"),
            format: Some(texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("jfa_texture_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 0,
            border_color: None,
        });

        entry.insert((
            GpuImage {
                texture_format: texture.format(),
                texture,
                texture_view,
                sampler,
                size: size.as_vec2(),
                mip_level_count: 0,
            },
            size,
        ));
    }

    #[inline]
    pub fn get_jfa_texture(&self, main_view: Entity) -> (&TextureView, UVec2) {
        self.0
            .get(&main_view)
            .map(|(tex, size)| (&tex.texture_view, *size))
            .unwrap()
    }
}
