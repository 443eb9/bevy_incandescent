use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res},
    },
    math::{Vec2, Vec3, Vec4Swizzles},
    render::{
        color::Color,
        extract_resource::ExtractResourcePlugin,
        render_phase::RenderPhase,
        render_resource::{Shader, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, Msaa, ViewTarget, VisibilitySystems, VisibleEntities},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    transform::components::GlobalTransform,
};

use crate::{
    ecs::{AmbientLight2d, PointLight2d, SpotLight2d},
    render::light::{GpuAmbientLight2d, GpuAmbientLight2dBuffer},
};

use self::light::{GpuLights2d, GpuPointLight2d};

#[cfg(feature = "catalinzz")]
pub mod catalinzz;
pub mod light;
pub mod visibility;

pub const HASH_SHADER: Handle<Shader> = Handle::weak_from_u128(94897465132296841564891368745312587);
pub const MATH_SHADER: Handle<Shader> = Handle::weak_from_u128(45341649741532875412078496512304512);
pub const LIGHTING_SHADER: Handle<Shader> = Handle::weak_from_u128(1351654315646451321546531153891);

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, HASH_SHADER, "shaders/hash.wgsl", Shader::from_wgsl);

        load_internal_asset!(app, MATH_SHADER, "shaders/math.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            LIGHTING_SHADER,
            "shaders/lighting.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(ExtractResourcePlugin::<AmbientLight2d>::default())
            .init_resource::<AmbientLight2d>()
            .register_type::<AmbientLight2d>()
            .add_systems(
                PostUpdate,
                visibility::calc_light_bounds.in_set(VisibilitySystems::CalculateBounds),
            );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GpuAmbientLight2dBuffer>()
            .add_systems(ExtractSchedule, extract_lights)
            .add_systems(Render, prepare_lights.in_set(RenderSet::Prepare));
    }
}

#[derive(Component)]
pub struct DynamicUniformIndex<S: ShaderType> {
    index: u32,
    _marker: PhantomData<S>,
}

impl<S: ShaderType> DynamicUniformIndex<S> {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Component, Clone, Copy)]
pub struct ExtractedPointLight2d {
    pub id: u32,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub spot_light_angles: [f32; 2],
}

pub fn extract_lights(
    mut commands: Commands,
    point_lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform, &VisibleEntities)>>,
    spot_lights_query: Extract<Query<(Entity, &SpotLight2d, &GlobalTransform, &VisibleEntities)>>,
) {
    let mut id = 0;

    commands.insert_or_spawn_batch(
        point_lights_query
            .iter()
            .map(|(entity, light, transform, visible_entities)| {
                let transform = GlobalTransform::from_translation(transform.translation());
                id += 1;
                (
                    entity,
                    (
                        ExtractedPointLight2d {
                            id: id - 1,
                            color: light.color,
                            intensity: light.intensity,
                            range: light.range,
                            radius: light.radius,
                            spot_light_angles: [0., std::f32::consts::TAU],
                        },
                        transform,
                        visible_entities.clone(),
                        RenderPhase::<Transparent2d>::default(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    );

    commands.insert_or_spawn_batch(
        spot_lights_query
            .iter()
            .map(|(entity, light, transform, visible_entities)| {
                let transform = GlobalTransform::from_translation(transform.translation());
                id += 1;
                (
                    entity,
                    (
                        ExtractedPointLight2d {
                            id: id - 1,
                            color: light.color,
                            intensity: light.intensity,
                            range: light.range,
                            radius: light.radius,
                            spot_light_angles: light.sector.into_extent(),
                        },
                        transform,
                        visible_entities.clone(),
                        RenderPhase::<Transparent2d>::default(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    );
}

pub fn prepare_lights(
    mut commands: Commands,
    main_views: Query<(Entity, &ExtractedView, &VisibleEntities), With<ViewTarget>>,
    lights_query: Query<(&ExtractedPointLight2d, &GlobalTransform)>,
    ambient_light: Res<AmbientLight2d>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    msaa: Res<Msaa>,
) {
    assert_eq!(*msaa, Msaa::Off, "MSAA is not supported yet!");

    commands.insert_resource(GpuAmbientLight2dBuffer::new(
        GpuAmbientLight2d {
            color: ambient_light.color.rgba_linear_to_vec4(),
            intensity: ambient_light.intensity,
        },
        &render_device,
        &render_queue,
    ));

    for (main_view_entity, main_view, visible_entities) in &main_views {
        let mut buffer = GpuLights2d::new(&render_device);

        let main_view_pos_ws = main_view.transform.translation();
        let view_proj = main_view.view_projection.unwrap_or_else(|| {
            main_view.projection * main_view.transform.compute_matrix().inverse()
        });

        let mut visible_lights = visible_entities
            .entities
            .iter()
            .filter_map(|e| lights_query.get(*e).ok())
            .collect::<Vec<_>>();
        radsort::sort_by_key(&mut visible_lights, |(light, _)| light.id);

        for (light, light_transform) in &visible_lights {
            let position_ws = light_transform.translation().extend(1.);
            let screen_size = 2.
                / Vec2::new(
                    main_view.projection.x_axis[0],
                    main_view.projection.y_axis[1],
                );

            let mut position_ndc = (view_proj * position_ws).xy();
            position_ndc.y = -position_ndc.y;
            let range_ndc =
                view_proj * (Vec3::new(light.range, 0., 0.) + main_view_pos_ws).extend(1.);

            let range_ndc = range_ndc.x / range_ndc.w / 2.;
            let radius_ndc = light.radius / light.range * range_ndc;

            buffer.add_point_light(GpuPointLight2d {
                intensity: light.intensity,
                position_ss: (position_ndc + 1.) / 2. * screen_size,
                radius_ss: radius_ndc * screen_size.x,
                range_ss: range_ndc * screen_size.x,
                color: light.color.rgba_linear_to_vec4(),
                angles: light.spot_light_angles,
            });
        }

        buffer.write_buffers(&render_device, &render_queue);
        commands.entity(main_view_entity).insert(buffer);
    }
}
