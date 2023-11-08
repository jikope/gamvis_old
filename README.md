# GamVis (Gamelan Visualizer)

Audio / Music visualizer written in rust. Shows musical spectrum in logarithmic way using Constant Q Transform. Currently support 3 scales (Chromatic, Pelog, Slendro).

https://github.com/jikope/gamvis/assets/42602197/4728bbe6-4f5f-4372-a4f8-bc0b8beab080


## How to run
Alsa will grab audio input from default input device.
```
cargo run --release
```


## Keybindings
- `s` : toggle between scales 
- `f` : maximize window       
- `p` : show/hide panel       


## Dependencies
- [Alsa](https://docs.rs/alsa/latest/alsa/)
- [Raylib](https://docs.rs/raylib/latest/raylib/)


## References
- [Javanese Gamelan Tuning and Dissonance](https://www.jstor.org/stable/1513182)
- [Time-domain CQT Implementation](https://github.com/KinWaiCheuk/nnAudio)
