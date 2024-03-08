use bevy::{asset::Handle, ecs::component::Component, render::texture::Image};

macro_rules! special_texture_handle {
    ($ty:ident) => {
        #[derive(Component)]
        pub struct $ty(Handle<Image>);

        impl From<Handle<Image>> for $ty {
            fn from(handle: Handle<Image>) -> Self {
                Self(handle)
            }
        }
    };
}

special_texture_handle!(HeightTexture);
special_texture_handle!(NormalTexture);
