# NOT READY FOR PRODUCTION USE YET!

# Bevy Incandescent 💡

A 2d lighting crate for bevy. Currently wip.

![](https://raw.githubusercontent.com/443eb9/bevy_incandescent/master/doc/imgs/readme_showcase.png)

**Disclaimer**

The crate was originally created because I(<u>**@443eb9**</u>) wanted to learn about 2D lighting techniques and some graphics knowledge. Therefore, its functionality may be somewhat complicated, as I wanted to learn a lot of related knowledge.

## Future Goals

- SDF+RayMarching Approach Implementation
- Compatibility with camera rotation
- PBR Lighting (Normal Mapping, Specular Mapping, and virtual height for lights)
- Support particle system [`bevy_hanabi`](https://github.com/djeedai/bevy_hanabi)
- MSM Approach (PCSS -> VSSM -> MSM step by step)
- Edge Lighting
- Compatibility with camera rotation
- Rim Lights
- Volumetric Fog
- Volumetric Clouds
- Tyndall Effect
- Screen Space SSAO
- Support tilemap system [`bevy_entitiles`](https://github.com/443eb9/bevy_entitiles)

## Feature Flags

| Flag            | Functionality                                                                                                                                      |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `debug`         | Show some debug info like light ranges.                                                                                                            |
| `catalinzz`     | Render shadow using the approach from Catalin ZZ.                                                                                                  |
| `compatibility` | Prefer compatibility to performance as this crate uses things that are not supported by every platform including textures with `Rg32Float` format. |
| `ray_marching`  | Render shadow using SDF+Raymarching.                                                                                                               |

***Which feature should I choose for my game?***

| Feature        | Time             | Memory Usage | Extra Features |
| -------------- | ---------------- | ------------ | -------------- |
| `catalinzz`    | :) -> :( -> :((  | High         | None           |
| `ray_marching` | :\| -> :) -> :)) | Medium       | None           |

*The data in the Time column represents the recommendation index of the feature under different orders of magnitude of the number of lights (1->10->100):*
- `:((` Don't use this!!
- `:(` Slow.
- `:|` Not bad.
- `:)` Nice.
- `:))` Very very very fast!!

## References

- Technique `catalinzz` from [Catalin ZZ's blog](https://web.archive.org/web/20200305042232/https://www.catalinzima.com/2010/07/my-technique-for-the-shader-based-dynamic-2d-shadows/).
- Technique `ray_marching` from [2D SDF Shadows - Ronja's tutorials](https://www.ronja-tutorials.com/post/037-2d-shadows/)
- SDF construction method JFA from [JUMP FLOODING ALGORITHM ON GRAPHICS HARDWARE AND ITS APPLICATIONS - Rong Guodong](https://www.comp.nus.edu.sg/~tants/jfa/rong-guodong-phd-thesis.pdf)
- SSAO, Volumetric Clouds and Fogs and Rim Lights are inspired by [@bit琪露诺](https://space.bilibili.com/84362619).

## Versions

| Bevy ver | Incandescent ver |
| -------- | ---------------- |
| 0.13.x   | 0.1.0-0.2.0      |
