use std::ffi::CString;

use raylib::prelude::*;

pub fn draw_panel(
    d: &mut RaylibDrawHandle,
    font: &Font,
    slider_value: i32,
    max_value: f32,
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
    bg_color: Color,
) -> i32 {
    let line_thickness = 2;
    let mut current_y = pos_y as f32;

    d.draw_rectangle(pos_x, pos_y, width, height, Color::new(70, 70, 70, 180));
    d.draw_rectangle_lines_ex(
        Rectangle::new(
            pos_x as f32 - line_thickness as f32,
            pos_y as f32 - line_thickness as f32,
            width as f32 + line_thickness as f32 * 2.0,
            height as f32 + line_thickness as f32 * 2.0,
        ),
        5,
        Color::new(230, 230, 230, 255),
    );

    current_y += 10.0;
    d.draw_text_ex(
        font,
        "Note Offset",
        Vector2::new(pos_x as f32 + (width as f32 / 3.0), current_y),
        20.0,
        2.0,
        Color::WHITE,
    );

    current_y += 40.0;
    let value = d.gui_slider(
        Rectangle::new(pos_x as f32 + 20.0, current_y, width as f32 - 40.0, 20.0),
        Some(CString::new(max_value.to_string()).unwrap().as_c_str()),
        Some(CString::new(max_value.to_string()).unwrap().as_c_str()),
        slider_value as f32,
        -max_value,
        max_value,
    );

    current_y += 25.0;
    d.draw_text_ex(
        font,
        format!("{:?}", value as i32).as_str(),
        Vector2::new(pos_x as f32 + (width as f32 / 2.0), current_y),
        20.0,
        2.0,
        Color::WHITE,
    );

    value as i32
}
