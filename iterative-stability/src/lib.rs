use num_complex::Complex64;

pub fn is_stable<F, G>(
    function: F,
    initial: Complex64,
    stability_check: G,
    max_iterations: u64,
) -> (u64, bool)
where
    F: Fn(Complex64) -> Complex64,
    G: Fn(&Complex64) -> bool,
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
        //println!("{:?} prec: {:?}", n, n.prec());
        if last.is_some() && last.unwrap() == n {
            return (i, true);
        }
        last = Some(n);
        i += 1;
    }
}

#[cfg(not(feature = "parallel"))]
pub mod mandelbrot {
    use crate::calc_space_params;
    use crate::from_screen_pixel_mandelbrot;

    pub fn calc_screen_space(
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        resolution: (i32, i32),
    ) -> impl Iterator<Item = (u64, bool)> {
        let sp = calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .map(move |index| from_screen_pixel_mandelbrot(index, resolution, sp))
    }
}

///
#[cfg(feature = "parallel")]
pub mod mandelbrot {
    use crate::calc_space_params;
    use crate::from_screen_pixel_mandelbrot;
    use rayon::prelude::*;

    pub fn calc_screen_space(
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        resolution: (i32, i32),
    ) -> impl ParallelIterator<Item = (u64, bool)> {
        let sp = calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .into_par_iter()
            .map(move |index| from_screen_pixel_mandelbrot(index, resolution, sp))
    }
}

#[cfg(feature = "parallel")]
pub mod julia {
    use crate::{calc_space_params, from_screen_pixel_julia};
    use rayon::prelude::*;

    pub fn calc_screen_space(
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        resolution: (i32, i32),
        c: (f64, f64),
    ) -> impl ParallelIterator<Item = (u64, bool)> {
        let sp = calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .into_par_iter()
            .map(move |index| from_screen_pixel_julia(index, resolution, sp, c))
    }
}

#[cfg(not(feature = "parallel"))]
pub mod julia {
    use crate::{calc_space_params, from_screen_pixel_julia};

    pub fn calc_screen_space(
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        resolution: (i32, i32),
        c: (f64, f64),
    ) -> impl Iterator<Item = (u64, bool)> {
        let sp = calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .map(move |index| from_screen_pixel_julia(index, resolution, sp, c))
    }
}

#[derive(Copy, Clone, Debug)]
struct SpaceParams {
    scale: (f64, f64),
    offset: (f64, f64),
    delta_x: f64,
    delta_y: f64,
}

fn calc_space_params(
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    resolution: (i32, i32),
) -> SpaceParams {
    let scale = (x_bounds.1 - x_bounds.0, y_bounds.1 - y_bounds.0);

    let offset = (
        (x_bounds.0 + x_bounds.1) / 2.0,
        (y_bounds.0 + y_bounds.1) / 2.0,
    );

    let delta_x = scale.0 / resolution.0 as f64;
    let delta_y = scale.1 / resolution.1 as f64;

    SpaceParams {
        scale,
        offset,
        delta_x,
        delta_y,
    }
}

fn from_screen_pixel_mandelbrot(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams,
) -> (u64, bool) {
    let (x, y) = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c| c.powu(2) + Complex64::new(x, y),
        Complex64::new(0.0, 0.0),
        |f| f.re < f64::INFINITY && f.im < f64::INFINITY,
        1000,
    )
}

fn from_screen_pixel_julia(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams,
    c_: (f64, f64),
) -> (u64, bool) {
    let (x, y) = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c| c.powu(2) + Complex64::new(c_.0, c_.1),
        Complex64::new(x, y),
        |f| f.re < f64::INFINITY && f.im < f64::INFINITY,
        1000,
    )
}

fn from_screen_point_to_cartesian(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams,
) -> (f64, f64) {
    let screen_x = index % resolution.0;
    let screen_y = index / resolution.0;

    // convert to cartesian
    let x = screen_x as f64 - (resolution.0 as f64 / 2.0);
    let y = -screen_y as f64 + (resolution.1 as f64 / 2.0);

    // convert to destination space
    let x = (x * sp.delta_x) + sp.offset.0;
    let y = (y * sp.delta_y) + sp.offset.1;

    (x, y)
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
