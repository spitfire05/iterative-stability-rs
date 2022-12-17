mod wgpu;

use glam::{IVec2, Vec2};
use num_complex::Complex;
use num_traits::{Float, NumCast};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub fn is_stable<F, G, U>(
    function: F,
    initial: U,
    stability_check: G,
    max_iterations: u64,
) -> (u64, bool)
where
    F: Fn(U) -> U,
    G: Fn(&U) -> bool,
    U: PartialEq + Copy,
{
    let mut n = initial;
    let mut i: u64 = 0;
    let mut last = None;
    loop {
        if !stability_check(&n) {
            return (i, false);
        }
        if i == max_iterations {
            return (i, true);
        }
        n = function(n);
        if last.is_some() && last.unwrap() == n {
            return (i, true);
        }
        last = Some(n);
        i += 1;
    }
}

#[cfg(not(feature = "parallel"))]
pub mod mandelbrot {
    use crate::{from_screen_pixel_mandelbrot, SpaceParams};
    use num_traits::Float;

    pub fn calc_screen_space<F>(
        x_bounds: (F, F),
        y_bounds: (F, F),
        resolution: (i32, i32),
    ) -> impl Iterator<Item = (u64, bool)>
    where
        F: Float,
    {
        let sp = SpaceParams::<F>::calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .map(move |index| from_screen_pixel_mandelbrot(index, resolution, sp))
    }
}

#[cfg(feature = "parallel")]
pub mod mandelbrot {
    use crate::{from_screen_pixel_mandelbrot, SpaceParams};
    use glam::{IVec2, Vec2};
    use num_traits::Float;
    use rayon::prelude::*;

    pub fn calc_screen_space<F>(lower: Vec2, upper: Vec2, resolution: IVec2) -> Vec<(u64, bool)>
    where
        F: Float + Send + Sync,
    {
        let sp = SpaceParams::new(lower, upper, resolution);

        // (0i32..(resolution.x * resolution.y))
        //     .into_par_iter()
        //     .map(move |index| from_screen_pixel_mandelbrot::<F>(index, resolution, sp))
        //     .collect()

        crate::wgpu_from_screen_pixels_mandelbrot(
            (0i32..(resolution.x * resolution.y)).into_par_iter(),
            resolution,
            sp,
        )
    }
}

#[cfg(feature = "parallel")]
pub mod julia {
    use crate::{from_screen_pixel_julia, SpaceParams};
    use glam::{IVec2, Vec2};
    use num_traits::Float;
    use rayon::prelude::*;

    pub fn calc_screen_space<F>(
        x_bounds: Vec2,
        y_bounds: Vec2,
        resolution: IVec2,
        c: (F, F),
    ) -> impl ParallelIterator<Item = (u64, bool)>
    where
        F: Float + Send + Sync,
    {
        let sp = SpaceParams::new(x_bounds, y_bounds, resolution);

        (0i32..(resolution.x * resolution.y))
            .into_par_iter()
            .map(move |index| from_screen_pixel_julia(index, resolution, sp, c))
    }
}

#[cfg(not(feature = "parallel"))]
pub mod julia {
    use crate::{from_screen_pixel_julia, SpaceParams};
    use num_traits::Float;

    pub fn calc_screen_space<F>(
        x_bounds: (F, F),
        y_bounds: (F, F),
        resolution: (i32, i32),
        c: (F, F),
    ) -> impl Iterator<Item = (u64, bool)>
    where
        F: Float,
    {
        let sp = SpaceParams::<F>::calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .map(move |index| from_screen_pixel_julia(index, resolution, sp, c))
    }
}

#[derive(Copy, Clone, Debug)]
struct SpaceParams {
    scale: Vec2,
    offset: Vec2,
    delta: Vec2,
}

impl SpaceParams {
    fn new(lower: Vec2, upper: Vec2, resolution: IVec2) -> SpaceParams {
        let scale = Vec2::new((upper.x - lower.x).abs(), (upper.y - lower.y).abs());
        let offset = (lower + upper) / 2.0;

        let delta = scale / resolution.as_vec2();

        SpaceParams {
            scale,
            offset,
            delta,
        }
    }
}

fn wgpu_from_screen_pixels_mandelbrot(
    index: impl ParallelIterator<Item = i32>,
    resolution: IVec2,
    sp: SpaceParams,
) -> Vec<(u64, bool)> {
    let cart: Vec<_> = index
        .map(|i| from_screen_point_to_cartesian(i, resolution, sp))
        .collect();

    let results = pollster::block_on(wgpu::execute_gpu(&cart)).unwrap();

    results
        .iter()
        .map(|i| {
            if *i == 500 {
                return (*i as u64, true);
            }

            (*i as u64, false)
        })
        .collect()
}

fn from_screen_pixel_mandelbrot<F>(index: i32, resolution: IVec2, sp: SpaceParams) -> (u64, bool)
where
    F: Float,
{
    let cart = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c: Complex<F>| {
            c.powu(2)
                + Complex::<F>::new(
                    NumCast::from(cart.x).unwrap(),
                    NumCast::from(cart.y).unwrap(),
                )
        },
        Complex::<F>::new(F::zero(), F::zero()),
        |f| f.re < F::infinity() && f.im < F::infinity(),
        1000,
    )
}

fn from_screen_pixel_julia<F>(
    index: i32,
    resolution: IVec2,
    sp: SpaceParams,
    c_: (F, F),
) -> (u64, bool)
where
    F: Float,
{
    let cart = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c: Complex<F>| c.powu(2) + Complex::<F>::new(c_.0, c_.1),
        Complex::<F>::new(
            NumCast::from(cart.x).unwrap(),
            NumCast::from(cart.y).unwrap(),
        ),
        |f| f.re < F::infinity() && f.im < F::infinity(),
        1000,
    )
}

fn from_screen_point_to_cartesian(index: i32, resolution: IVec2, sp: SpaceParams) -> Vec2 {
    let screen_x = index % resolution.x;
    let screen_y = index / resolution.x;

    // convert to cartesian
    let cart = IVec2::new(
        screen_x - (resolution.x) / 2,
        -screen_y + (resolution.y) / 2,
    );

    // convert to destination space
    let cart = (cart.as_vec2() * sp.delta) + sp.offset;

    cart
}

#[cfg(test)]
mod tests {
    use num_complex::Complex64;

    use crate::is_stable;

    #[test]
    fn unstable_positive_integer() {
        let (_, stable) = is_stable(
            |q| q.powu(2),
            Complex64::new(2.0, 0.0),
            |s| s.re < 999999.0,
            50000,
        );
        assert_eq!(stable, false);
    }

    #[test]
    fn stable_positive_float() {
        let (_, stable) = is_stable(
            |q| q.powu(2),
            Complex64::new(0.5, 0.0),
            |s| s.re < 999999.0,
            50000,
        );
        assert_eq!(stable, true);
    }
}
