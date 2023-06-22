use raylib::{ffi::MaximizeWindow, prelude::*};
use std::collections::HashMap;
use std::time::Duration;

mod cqt;
mod panel;
mod pipes;

type SampleFormat = f32;

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
    r: 10,
    g: 100,
    b: 150,
    a: 255,
};

fn main() {
    const NUM_CH: u16 = 1; // Looks better if 1. Also idk why it is working on stereo audio
    const WIDTH: i32 = 1200;
    const HEIGHT: i32 = 600;
    const BASE_OCTAVE: u16 = 3;
    const SAMPLE_RATE: u32 = 44100;
    const BINS_PER_OCTAVE: u16 = 12 * BASE_OCTAVE;
    const N_BINS: u16 = BINS_PER_OCTAVE * 9;

    let f_min: f32 = 27.50;
    let fft_size: u16 = 3500;
    let font = "../Roboto-Mono.ttf";

    // Mutables
    let mut bar_width_f = WIDTH as f32 / N_BINS as f32;
    let mut width: i32 = WIDTH;
    let mut height: i32 = HEIGHT;
    let mut note_y_position: i32 = height - 20;

    // State
    let mut curr_scale = Scale::Pelog;
    let mut show_panel: bool = false;

    // Pitches
    // let chromatic: [&str; 12] = ["A", "A'", "B", "C", "C'", "D", "D'", "E", "F", "F'", "G", "G'",];
    let chromatic: [&str; 12] = ["A", "", "B", "C", "", "D", "", "E", "F", "", "G", ""];
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
    let mut pipe: pipes::Pipe = pipes::Pipe {
        num_ch: NUM_CH,
        fft_size,
        input: vec![0.0; (fft_size * NUM_CH).into()],
        output: vec![0.0; N_BINS.into()],
        pcm,
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

    let mut adder: i32 = 1;
    let mut slider_value: i32 = 0;

    while !rl.window_should_close() {
        pipe.fill_input_buffer().unwrap();

        pipe.output = cqt::calc_cqt(&pipe.input, &time_domain_kernel, N_BINS).unwrap();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BG_COLOR);

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
        let mut bar_color = BAR_COLOR;
        for i in 0..N_BINS as i32 {
            let y: i32 = (pipe.output[i as usize] * 6000f32) as i32;
            d.draw_rectangle(
                x,
                height - y - 20,
                bar_width_f.round() as i32 - 2,
                y.abs(),
                bar_color,
            );
            if bar_color.b == 250 {
                adder = -1;
            } else if bar_color.b == 180 {
                adder = 1;
            }
            bar_color.b += adder as u8;

            x = (bar_width_f * i as f32) as i32;
        }

        // Draw notes
        match curr_scale {
            Scale::Chromatic => {
                let mut i = 0;
                x = 0;
                for n in (1..=N_BINS).step_by(BASE_OCTAVE as usize) {
                    d.draw_text_ex(
                        &roboto,
                        chromatic[i],
                        Vector2::new((x) as f32, note_y_position as f32),
                        22.0,
                        2.0,
                        Color::WHITE,
                    );
                    i += 1;
                    x = (bar_width_f * n as f32) as i32;

                    if i >= 12 {
                        i = 0;
                    }
                }
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
                for n in 1..=N_BINS {
                    let note = slendro.get(&counter);
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
                        if octave >= 5 {
                            dissonance += 2;
                        }
                    }

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
            slider_value = -1 * panel::draw_panel(
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
