extern crate euclid;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

pub mod animation_test;
pub mod barnsley_fern_animated;
pub mod barnsley_fern_raster;
pub mod color;
pub mod renderer;
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
            color
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
