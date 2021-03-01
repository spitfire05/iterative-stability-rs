mod utils;

use iterative_stability::julia;
use palette::{Hsv, Hue, Srgb};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn gen(palette_length: u32, palette_hue: f32, cx: f64, cy: f64) -> Vec<u32> {
    julia::calc_screen_space((-2.0, 2.0), (-2.0, 2.0), (1000, 1000), (cx, cy))
        .map(|(iter, stable)| apply_palette(iter, stable, palette_length, palette_hue))
        .collect()
}

fn apply_palette(iter: u64, stable: bool, length: u32, hue: f32) -> u32 {
    if stable {
        0xff000000
    } else {
        let hsv_color = Hsv::new(hue, 1.0, 1.0);
        let new_color: Srgb = hsv_color
            .shift_hue((iter as f32 * (360.0 / length as f32)) as f32)
            .into();
        u32::from_be_bytes([
            0xff,
            (new_color.blue * 255.0) as u8,
            (new_color.green * 255.0) as u8,
            (new_color.red * 255.0) as u8,
        ])
    }
}
