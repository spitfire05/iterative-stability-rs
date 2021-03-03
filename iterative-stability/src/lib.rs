use num_complex::Complex;
use num_traits::{Float, NumCast};

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
    use num_traits::Float;
    use rayon::prelude::*;

    pub fn calc_screen_space<F>(
        x_bounds: (F, F),
        y_bounds: (F, F),
        resolution: (i32, i32),
    ) -> impl ParallelIterator<Item = (u64, bool)>
    where
        F: Float + Send + Sync,
    {
        let sp = SpaceParams::<F>::calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
            .into_par_iter()
            .map(move |index| from_screen_pixel_mandelbrot(index, resolution, sp))
    }
}

#[cfg(feature = "parallel")]
pub mod julia {
    use crate::{from_screen_pixel_julia, SpaceParams};
    use num_traits::Float;
    use rayon::prelude::*;

    pub fn calc_screen_space<F>(
        x_bounds: (F, F),
        y_bounds: (F, F),
        resolution: (i32, i32),
        c: (F, F),
    ) -> impl ParallelIterator<Item = (u64, bool)>
    where
        F: Float + Send + Sync,
    {
        let sp = SpaceParams::<F>::calc_space_params(x_bounds, y_bounds, resolution);

        (0i32..(resolution.0 * resolution.1))
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
struct SpaceParams<F>
where
    F: Float,
{
    scale: (F, F),
    offset: (F, F),
    delta_x: F,
    delta_y: F,
}

impl<F> SpaceParams<F>
where
    F: Float,
{
    fn calc_space_params(
        x_bounds: (F, F),
        y_bounds: (F, F),
        resolution: (i32, i32),
    ) -> SpaceParams<F> {
        let scale = (x_bounds.1 - x_bounds.0, y_bounds.1 - y_bounds.0);

        let offset = (
            (x_bounds.0 + x_bounds.1) / F::from(2).unwrap(),
            (y_bounds.0 + y_bounds.1) / F::from(2).unwrap(),
        );

        let delta_x = scale.0 / F::from(resolution.0).unwrap();
        let delta_y = scale.1 / F::from(resolution.1).unwrap();

        SpaceParams {
            scale,
            offset,
            delta_x,
            delta_y,
        }
    }
}

fn from_screen_pixel_mandelbrot<F>(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams<F>,
) -> (u64, bool)
where
    F: Float,
{
    let (x, y) = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c: Complex<F>| {
            c.powu(2) + Complex::<F>::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap())
        },
        Complex::<F>::new(F::zero(), F::zero()),
        |f| f.re < F::infinity() && f.im < F::infinity(),
        1000,
    )
}

fn from_screen_pixel_julia<F>(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams<F>,
    c_: (F, F),
) -> (u64, bool)
where
    F: Float,
{
    let (x, y) = from_screen_point_to_cartesian(index, resolution, sp);

    is_stable(
        |c: Complex<F>| c.powu(2) + Complex::<F>::new(c_.0, c_.1),
        Complex::<F>::new(NumCast::from(x).unwrap(), NumCast::from(y).unwrap()),
        |f| f.re < F::infinity() && f.im < F::infinity(),
        1000,
    )
}

fn from_screen_point_to_cartesian<F>(
    index: i32,
    resolution: (i32, i32),
    sp: SpaceParams<F>,
) -> (F, F)
where
    F: Float,
{
    let screen_x = index % resolution.0;
    let screen_y = index / resolution.0;

    // convert to cartesian
    let x = F::from(screen_x).unwrap() - (F::from(resolution.0).unwrap() / F::from(2).unwrap());
    let y = F::from(-screen_y).unwrap() + (F::from(resolution.1).unwrap() / F::from(2).unwrap());

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
