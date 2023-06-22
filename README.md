# GamVis (Gamelan Visualizer)

Audio / Music visualizer written in rust. Shows musical spectrum in logarithmic way using Constant Q Transform. Currently support 3 scales (Chromatic, Pelog, Slendro).

## How to run
Alsa will grab audio input from default input device.
```
cargo run --release
```
## Keybindings

|-----|-----------------------|
| `s` | toggle between scales |
| `f` | maximize window       |
| `p` | show/hide panel       |
|-----|-----------------------|

## Dependencies

- [Alsa](https://docs.rs/alsa/latest/alsa/)
- [Raylib](https://docs.rs/raylib/latest/raylib/)
