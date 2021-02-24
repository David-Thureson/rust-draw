extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate euclid;

pub mod animation_test;
pub mod renderer;
pub mod try_renderer;

pub type Color = [f32; 4];

pub const COLOR_BLACK: Color = [0.0, 0.0, 0.0, 1.0];
pub const COLOR_WHITE: Color = [1.0, 1.0, 1.0, 1.0];
pub const COLOR_RED: [f32; 4]   = [1.0, 0.0, 0.0, 1.0];
pub const COLOR_GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const COLOR_BLUE: [f32; 4]  = [0.0, 0.0, 1.0, 1.0];

pub type ShapeList = Vec<Shape>;

#[derive(Clone, Debug)]
pub enum Shape {
    Circle {
        //center: euclid::Point2D<f64, ScreenSpace>,
        center_x: f64,
        center_y: f64,
        radius: f64,
        color: Color,
    }
}

impl Shape {
    pub fn circle(center_x: f64, center_y: f64, radius: f64, color: Color) -> Shape {
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
