# What's New:

- Removed redundant components in `ShadowCaster2dBundle`.
- Moved redundant Bevy features into dev-dependencies.
- Added `alpha_threshold` to `ShadowMap2dConfig`.
- Samples of PCF is no longer limited to 32.
- Added `SpotLight2d`.
- Added alpha map to clip shadows, which improved shadow accuracy.
- Added `catalinzz` feature. It will be enabled as default. In the future, in order to support more fancy features, there will be more shading approaches like SDF+RayMarching and Ray Tracing, and you can choose according to your needs.

# What's Fixed:

- Sprites without `ShadowCaster2d` still cast shadows.
- Program panics when there's no 2d light in the scene.
