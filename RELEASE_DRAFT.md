# What's New:

- Removed redundant components in `ShadowCaster2dBundle`.
- Moved redundant Bevy features into dev-dependencies.
- Added `alpha_threshold` to `ShadowMap2dConfig`.

# What's Fixed:

- Projection matrices are doubled because of some weird reason. But actually caused by some visibility calculation stuff.
- Sprites without `ShadowCaster2d` still cast shadows.
