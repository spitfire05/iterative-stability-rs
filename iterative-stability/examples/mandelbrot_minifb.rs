use iterative_stability::mandelbrot;
use minifb::{Key, Window, WindowOptions};
use palette::{Hsv, Hue, Srgb};
use rayon::prelude::*;

const WIDTH: usize = 1200;
const HEIGHT: usize = 1200;
const ZOOM_FACTOR: f64 = 0.25;

fn main() {
    let mut window = Window::new("Mandelbrot", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~30 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(33333)));

    let mut buffer_needs_update = true;
    let mut x_bounds = (-2.5, 1.5);
    let mut y_bounds = (-2.0, 2.0);
    let mut scale = (x_bounds.1 - x_bounds.0, y_bounds.1 - y_bounds.0);
    let mut delta_x = scale.0 / WIDTH as f64;
    let mut delta_y = scale.1 / HEIGHT as f64;
    let mut offset = (
        (x_bounds.0 + x_bounds.1) / 2.0,
        (y_bounds.0 + y_bounds.1) / 2.0,
    );
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if buffer_needs_update {
            let buffer: Vec<u32> =
                mandelbrot::calc_screen_space(x_bounds, y_bounds, (WIDTH as i32, HEIGHT as i32))
                    .map(|(iter, stable)| apply_palette(iter, stable))
                    .collect();

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            buffer_needs_update = false;
        } else {
            window.update();
            let mouse = window.get_mouse_pos(minifb::MouseMode::Discard);
            match mouse {
                Some(m) => {
                    if window.get_mouse_down(minifb::MouseButton::Left) {
                        let x = ((m.0 as f64 - (WIDTH as f64 / 2.0)) * delta_x) + offset.0;
                        let y = ((-m.1 as f64 + (HEIGHT as f64 / 2.0)) * delta_y) + offset.1;
                        x_bounds = (
                            (x as f64 - (scale.0 / 2.0)) + (ZOOM_FACTOR * scale.0),
                            (x as f64 + (scale.0 / 2.0)) - (ZOOM_FACTOR * scale.0),
                        );
                        y_bounds = (
                            (y as f64 - (scale.1 / 2.0)) + (ZOOM_FACTOR * scale.1),
                            (y as f64 + (scale.1 / 2.0)) - (ZOOM_FACTOR * scale.1),
                        );
                        scale = (x_bounds.1 - x_bounds.0, y_bounds.1 - y_bounds.0);
                        delta_x = scale.0 / WIDTH as f64;
                        delta_y = scale.1 / HEIGHT as f64;
                        offset = (
                            (x_bounds.0 + x_bounds.1) / 2.0,
                            (y_bounds.0 + y_bounds.1) / 2.0,
                        );
                        println!("zooming {:?} {:?}", x_bounds, y_bounds);
                        buffer_needs_update = true;
                    }
                }
                None => {}
            }
        }
    }
}

pub fn apply_palette(iter: u64, stable: bool) -> u32 {
    if stable {
        0
    } else {
        let hsv_color = Hsv::new(0.0, 1.0, 1.0);
        let new_color: Srgb = hsv_color.shift_hue((iter as f32 * 0.7) as f32).into();
        u32::from_be_bytes([
            0xff,
            (new_color.red * 255.0) as u8,
            (new_color.green * 255.0) as u8,
            (new_color.blue * 255.0) as u8,
        ])
    }
}
