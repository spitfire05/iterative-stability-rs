use glam::{IVec2, Vec2};
use iterative_stability::mandelbrot;
use minifb::{Key, Window, WindowOptions};
use palette::{Hsv, Hue, Srgb};
use rayon::prelude::*;
use std::time::Instant;

const WIDTH: usize = 1200;
const HEIGHT: usize = 1200;
const ZOOM_FACTOR: f32 = 0.25;

fn main() {
    let mut window = Window::new("Mandelbrot", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~30 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(33333)));

    let mut buffer_needs_update = true;
    let mut bounds_lower = Vec2::new(-2.5, -2.0);
    let mut bounds_upper = Vec2::new(1.5, 2.0);
    let mut scale = bounds_upper - bounds_lower;
    let resolution = IVec2::new(WIDTH as i32, HEIGHT as i32);
    let mut delta = scale / resolution.as_vec2();
    let mut offset = bounds_lower + bounds_upper / 2.0;
    println!("drawing {:?} {:?}", bounds_lower, bounds_upper);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if buffer_needs_update {
            let start = Instant::now();
            let buffer: Vec<u32> =
                mandelbrot::calc_screen_space::<f32>(bounds_lower, bounds_upper, resolution)
                    .map(|(iter, stable)| apply_palette(iter, stable))
                    .collect();

            println!(
                "calculations took {}",
                humantime::format_duration(start.elapsed())
            );

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            buffer_needs_update = false;
        } else {
            window.update();
            let mouse = window.get_mouse_pos(minifb::MouseMode::Discard);
            if let Some(m) = mouse {
                if window.get_mouse_down(minifb::MouseButton::Left) {
                    let m = Vec2::new(m.0, -m.1);
                    let coords =
                        ((m + ((resolution / 2) * IVec2::new(-1, 1)).as_vec2()) * delta) + offset;
                    println!("{coords}");
                    bounds_lower = coords - (scale / 2.0) + (scale * ZOOM_FACTOR);
                    bounds_upper = coords + (scale / 2.0) - (scale * ZOOM_FACTOR);
                    scale = bounds_upper - bounds_lower;
                    delta = scale / resolution.as_vec2();
                    offset = bounds_lower + bounds_upper / 2.0;
                    buffer_needs_update = true;
                    println!("drawing {} {}", bounds_lower, bounds_upper);
                }
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
