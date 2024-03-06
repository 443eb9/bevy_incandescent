# Bevy Incandescent 💡

A 2d lighting crate for bevy. Currently wip.

![](https://raw.githubusercontent.com/443eb9/bevy_incandescent/master/doc/imgs/readme_showcase.png)

## Future Goals

- MSM Approach (PCSS -> VSM -> MSM step by step)
- PBS Lighting
- Rim Lights
- Volumetric Fog
- Volumetric Clouds
- Screen Space SSAO
- Support particle system [`bevy_hanabi`](https://github.com/djeedai/bevy_hanabi)
- Support tilemap system [`bevy_entitiles`](https://github.com/443eb9/bevy_entitiles)

## Feature Flags

| Flag            | Functionality                                                                                                                                      |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `debug`         | Show some debug info like light ranges.                                                                                                            |
| `compatibility` | Prefer compatibility to performance as this crate uses things that are not supported by every platform including textures with `Rg32Float` format. |

## Render Graph

![](https://raw.githubusercontent.com/443eb9/bevy_incandescent/master/doc/imgs/render_graph.png)

## Special Thanks

- Lighting technique from [Catalin ZZ's blog](https://web.archive.org/web/20200305042232/https://www.catalinzima.com/2010/07/my-technique-for-the-shader-based-dynamic-2d-shadows/).
- SSAO, Volumetric Clouds and Fogs and Rim Lights are inspired by [@bit琪露诺](https://space.bilibili.com/84362619).

## Versions

| Bevy ver | Incandescent ver |
| -------- | ---------------- |
| 0.13.x   | 0.1.0            |
