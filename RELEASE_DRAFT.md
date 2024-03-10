# What's New:

- Removed redundant components in `ShadowCaster2dBundle`.
- Moved redundant Bevy features into dev-dependencies.
- Added `alpha_threshold` to `ShadowMap2dConfig`.
- Samples of PCF is no longer limited to 32.

# What's Fixed:

- Projection matrices are doubled because of some weird reason. But actually caused by some visibility calculation stuff.
- Sprites without `ShadowCaster2d` still cast shadows.
- Program panics when there's no 2d light in the scene.
