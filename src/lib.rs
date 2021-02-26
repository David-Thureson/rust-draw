#![feature(assoc_char_funcs)]
extern crate euclid;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

pub mod animation_test;
pub mod animator;
pub mod barnsley_fern_animated;
pub mod barnsley_fern_raster;
pub mod carpet;
pub mod color;
pub mod geometry;
// pub mod renderer_1;
// pub mod renderer_2;
pub mod renderer_3;
pub mod shape;
pub mod try_fractal;
pub mod try_transition;
// pub mod try_renderer;

pub use color::*;
pub use geometry::*;
pub use shape::*;

#[derive(Debug)]
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

pub fn gradiant_f64_one(from: f64, to: f64, step_count: usize, step_index: usize) -> f64 {
    debug_assert!(step_index <= step_count);
    let step_count = step_count as f64;
    let step_index = step_index as f64;
    let step_size = (to - from) / step_count;
    from + (step_size * step_index)
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

