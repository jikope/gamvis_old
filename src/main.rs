use raylib::{ease::*, ffi::MaximizeWindow, prelude::*};
use std::time::Duration;
use std::collections::HashMap;

mod cqt;
mod panel;
mod pipes;

enum Scale {
    Chromatic,
    Pelog,
    Slendro,
}

fn other_scale(scale: &mut Scale) {
    use Scale::*;
    *scale = match scale {
        Chromatic => Pelog,
        Pelog => Slendro,
        Slendro => Chromatic,
    };
}

const BG_COLOR: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 100,
};

const BAR_COLOR: Color = Color {
    r: 50,
    g: 150,
    b: 180,
    a: 255,
};

fn main() {
    let m = clap::Command::new("Gamvis")
        .author("me")
        .version("0.0.0")
        .about("CQT Audio Visualizer")
        .arg(clap::arg!(-i --input <VALUE> "Input source for the visualizer."))
        .arg(clap::arg!(-b --bsize <VALUE> "Buffer size of input samples.").value_parser(clap::value_parser!(u16)))
        .get_matches();

    const NUM_CH: u16 = 1; // Looks better if 1. Also idk why it is working on stereo audio
    const WIDTH: i32 = 1200;
    const HEIGHT: i32 = 600;
    const BASE_OCTAVE: u16 = 3;
    const SAMPLE_RATE: u32 = 48000;
    const BINS_PER_OCTAVE: u16 = 12 * BASE_OCTAVE;
    const N_BINS: u16 = BINS_PER_OCTAVE * 9;

    let f_min: f32 = 27.50; // A0
    let fft_size: u16 = match m.get_one::<u16>("bsize") {
        Some(size) => *size,
        None => 4096,
    };
    // let fft_size: u16 = 3700;
    let font = "../Roboto-Mono.ttf";

    // Mutables
    let mut bar_width_f = WIDTH as f32 / N_BINS as f32;
    let mut width: i32 = WIDTH;
    let mut height: i32 = HEIGHT;
    let mut note_y_position: i32 = height - 20;

    // State
    let mut curr_scale = Scale::Chromatic;
    let mut show_panel: bool = false;
    let mut pause: bool = false;

    // Pitches
    let chromatic: [&str; 12] = ["A", "", "B", "C", "", "D", "", "E", "F", "", "G", ""];
    let chromatic_map = HashMap::from([
        (36, "A"),
        (6, "B"),
        (9, "C"),
        (15, "D"),
        (21, "E"),
        (24, "F"),
        (30, "G"),
    ]);
    let mut pelog = HashMap::from([
        (14, "1"),
        (18, "2"),
        (22, "3"),
        (29, "4"),
        (33, "5"),
        (36, "6"),
        (5, "7"),
    ]);
    let slendro = HashMap::from([(15, "2"), (22, "3"), (29, "5"), (36, "6"), (7, "1")]);

    // Audio input init
    let pcm = pipes::get_alsa_pcm("default", SAMPLE_RATE, NUM_CH).unwrap();
    let input_pipe: Box<dyn pipes::InputPipe> = match m.get_one::<String>("input") {
        Some(input_arg) => match input_arg {
            input if input == "mpd_fifo" => Box::new(pipes::MPDFifoPipe::new("/tmp/gamvis.fifo")),
            input if input == "alsa" => Box::new(pipes::AlsaPipe { pcm }),
            _input => Box::new(pipes::AlsaPipe { pcm }),
        },
        None => Box::new(pipes::AlsaPipe { pcm }),
    };

    let mut pipe: pipes::Pipe = pipes::Pipe {
        num_ch: NUM_CH,
        fft_size,
        input: vec![0.0; (fft_size * NUM_CH).into()],
        output: vec![0.0; N_BINS.into()],
        input_pipe,
    };

    // CQT init
    let time_domain_kernel =
        cqt::init_time_domain_kernel(SAMPLE_RATE, fft_size, f_min, BINS_PER_OCTAVE, N_BINS)
            .unwrap();

    println!("buff size {}", pipe.input.len());
    // Raylib init
    let (mut rl, thread) = raylib::init()
        .undecorated()
        .transparent()
        .resizable()
        .size(WIDTH, HEIGHT)
        .title("Gamelan Visualizer")
        .build();

    // Load font
    let roboto = rl.load_font(&thread, font).expect("Unable to load font");

    let mut adder_r: i32 = 1;
    let mut adder_g: i32 = 1;
    let mut adder_b: i32 = 1;
    let mut slider_value: i32 = 0;
    let mut bar_color = BAR_COLOR;
    // let mut t: f32 = 0.0;
    // let dur: f32 = 10.0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_F) {
            unsafe {
                MaximizeWindow();
            }
        }
        if d.is_key_pressed(KeyboardKey::KEY_S) {
            other_scale(&mut curr_scale);
        }
        if d.is_key_pressed(KeyboardKey::KEY_P) {
            show_panel = !show_panel;
        }
        if d.is_key_pressed(KeyboardKey::KEY_M) {
            pause = !pause;
        }

        if pause {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }

        // Process incoming audio samples
        pipe.fill_input_buffer();
        pipe.output = cqt::calc_cqt(&pipe.input, &time_domain_kernel, N_BINS).unwrap();
        d.clear_background(BG_COLOR);

        let high = pipe.get_highest_output_index();
        println!("Max {}", pipe.output[high]);

        // Window Event
        if d.is_window_resized() {
            bar_width_f = d.get_screen_width() as f32 / N_BINS as f32;
            width = d.get_screen_width();
            height = d.get_screen_height();
            note_y_position = height - 20;
        }

        // FPS
        d.draw_text_ex(
            &roboto,
            d.get_fps().to_string().as_str(),
            Vector2::new(10.0, 10.0),
            20.0,
            2.0,
            Color::WHITE,
        );

        // Draw bars
        let mut x = 0;
        let interval: i32 = (width / N_BINS as i32) + 1;
        // let interval: i32 = 1;
        for i in 0..N_BINS as i32 {
            let energy: f32 = pipe.output[i as usize];
            let y: i32 = (energy * 6000f32) as i32;
            let e_dbfs = 20.0 * (energy / 1.0).log10();
            let scale = (e_dbfs ) / (50.0) * (250.0 - 50.0) + 50.0;

            // bar_color.r = BAR_COLOR.r + scale as u8;
            // bar_color = Color::color_from_normalized(Vector4::new(energy, energy, energy, 1.0));
            // println!("Red {} Energy {}", bar_color.r, energy);
            // bar_color.b = BAR_COLOR.b + e_dbfs as u8;
            d.draw_rectangle(
                x,
                height - y - 20,
                bar_width_f.round() as i32 - 2,
                // 1,
                y.abs(),
                bar_color,
            );
            if bar_color.b < 50 || bar_color.b > 200 {
                adder_b *= -1;
            }
            if bar_color.g < 50 || bar_color.g > 100 {
                adder_g *= -1;
            }
            if bar_color.r < 50 || bar_color.r > 200 {
                adder_r *= -1;
            }
            // bar_color.r = (bar_color.r as i32 + adder_r) as u8;
            // bar_color.b = (bar_color.b as i32 + adder_b) as u8;
            // bar_color.g = (bar_color.g as i32 + adder_g) as u8;
            // d.draw_circle(x + 1, height - y - 20, 3.5, Color::VIOLET);

            // x = (bar_width_f * i as f32) as i32;
            x += interval;
        }

        // Draw notes
        match curr_scale {
            Scale::Chromatic => {
                let mut counter: i32 = 1;
                let mut x = 0;
                for n in 1..=N_BINS {
                    let note = chromatic_map.get(&counter);
                    if note.is_some() {
                        let note_str = note.unwrap();
                        d.draw_text_ex(
                            &roboto,
                            note_str,
                            Vector2::new(x as f32, note_y_position as f32),
                            22.0,
                            2.0,
                            Color::WHITE,
                        );
                    }
                    counter += 1;
                    x += interval;

                    if (n % BINS_PER_OCTAVE) == 0 {
                        counter = 1;
                    }
                }
                // let mut i = 0;
                // x = 0;
                // for n in (1..=N_BINS).step_by(BASE_OCTAVE as usize) {
                //     d.draw_text_ex(
                //         &roboto,
                //         chromatic[i],
                //         Vector2::new((x) as f32, note_y_position as f32),
                //         22.0,
                //         2.0,
                //         Color::WHITE,
                //     );
                //     i += 1;
                //     x = (bar_width_f * n as f32) as i32;

                //     if i >= 12 {
                //         i = 0;
                //     }
                // }
            }
            Scale::Pelog => {
                let mut dissonance = 0;
                let mut dissonance_adder = 3;
                let mut octave = 1;
                let mut counter = 1;
                for n in 1..=N_BINS {
                    let mut index: i32;

                    // Update slider
                    if slider_value != 0 {
                        index = counter + slider_value;
                        if index > BINS_PER_OCTAVE as i32 {
                            index -= BINS_PER_OCTAVE as i32;
                        } else if index < 1 {
                            index += BINS_PER_OCTAVE as i32;
                        }
                    } else {
                        index = counter;
                    }

                    let note = pelog.get(&index);
                    if note.is_some() {
                        let note_str = note.unwrap();
                        let pos_x = (bar_width_f * n as f32) + dissonance as f32;
                        d.draw_text_ex(
                            &roboto,
                            note_str,
                            Vector2::new(pos_x, note_y_position as f32),
                            22.0,
                            2.0,
                            Color::WHITE,
                        );

                        dissonance += dissonance_adder;
                    }

                    counter += 1;
                    if (n % BINS_PER_OCTAVE) == 0 {
                        if octave > 4 {
                            dissonance_adder += 2;
                        }
                        octave += 1;
                        counter = 1;
                    }
                }
            }
            Scale::Slendro => {
                let mut dissonance = 0;
                let mut octave = 1;
                let mut counter = 1;
                let mut x = 0;
                for n in 1..=N_BINS {
                    let mut index: i32;
                    if slider_value != 0 {
                        index = counter + slider_value;
                        if index > BINS_PER_OCTAVE as i32 {
                            index -= BINS_PER_OCTAVE as i32;
                        } else if index < 1 {
                            index += BINS_PER_OCTAVE as i32;
                        }
                    } else {
                        index = counter;
                    }

                    let note = slendro.get(&index);
                    if note.is_some() {
                        let note_str = note.unwrap();
                        // let pos_x = (bar_width_f * n as f32) + dissonance as f32;
                        d.draw_text_ex(
                            &roboto,
                            note_str,
                            Vector2::new(x as f32, note_y_position as f32),
                            22.0,
                            2.0,
                            Color::WHITE,
                        );
                        if octave >= 7 {
                            dissonance += 6;
                        }
                    }

                    x += interval;
                    counter += 1;
                    if (n % BINS_PER_OCTAVE) == 0 {
                        octave += 1;
                        counter = 1;
                    }
                }
            }
        }

        // Option Panels
        if show_panel {
            slider_value = -1
                * panel::draw_panel(
                    &mut d,
                    &roboto,
                    -1 * slider_value,
                    20.0,
                    (width as f32 * 0.1) as i32,
                    (height as f32 * 0.03) as i32,
                    300,
                    200,
                    Color::new(70, 70, 70, 180),
                );
        }
    }

    println!("Gamvis Quit!");
}
