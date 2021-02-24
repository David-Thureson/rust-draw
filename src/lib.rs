extern crate euclid;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

pub mod animation_test;
pub mod barnsley_fern_animated;
pub mod barnsley_fern_raster;
pub mod color;
pub mod renderer_3;
pub mod renderer_1;
pub mod renderer_2;
pub mod try_fractal;
pub mod try_renderer;

pub use color::*;

pub type ShapeList = Vec<Shape>;

#[derive(Clone, Debug)]
pub enum Shape {
    Circle {
        //center: euclid::Point2D<f64, ScreenSpace>,
        center_x: f64,
        center_y: f64,
        radius: f64,
        color: Color1,
    }
}

impl Shape {
    pub fn circle(center_x: f64, center_y: f64, radius: f64, color: Color1) -> Shape {
        Shape::Circle {
            center_x,
            center_y,
            radius,
            color,
        }
    }
}

pub struct Frame {
    shapes: ShapeList,
    seconds_to_next: f64,
}

impl Frame {
    pub fn new(shapes: ShapeList, seconds_to_next: f64) -> Frame {
        Frame {
            shapes,
            seconds_to_next,
        }
    }
}

pub fn gradient_f64(from: f64, to: f64, step_count: usize) -> Vec<f64> {
    let step_size = (to - from) / step_count as f64;
    let mut v = vec![from];
    let mut val = from;
    for _ in 0..(step_count - 1) {
        val += step_size;
        v.push(val);
    }
    v.push(to);
    debug_assert_eq!(step_count + 1, v.len());
    v
}

pub type PointF64 = (f64, f64);

pub fn gradient_point64(from: PointF64, to: PointF64, step_count: usize) -> Vec<PointF64> {
    let x_values = gradient_f64(from.0, to.0, step_count);
    let y_values = gradient_f64(from.1, to.1, step_count);
    let v = x_values.iter()
        .zip(y_values.iter())
        .map(|(x, y)| (*x, *y))
        .collect::<Vec<_>>();
    debug_assert_eq!(step_count + 1, v.len());
    v
}
