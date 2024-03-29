#![feature(assoc_char_funcs)]
#![feature(duration_zero)]
#![feature(div_duration)]
#![feature(step_trait)]
#![feature(map_first_last)]

extern crate euclid;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

pub use util::format::{fc, ff, list_of_counts, list_of_counts_indexed};

pub mod algorithms;
pub mod animation_test;
pub mod animator;
pub mod barnsley_fern_animated;
pub mod barnsley_fern_raster;
pub mod carpet;
pub mod cave_cell;
pub mod cell_auto;
pub mod color;
pub mod geometry;
pub mod grid;
pub mod image_util;
// pub mod renderer_1;
// pub mod renderer_2;
pub mod renderer_3;
pub mod shape;
pub mod try_fractal;
pub mod try_transition;
// pub mod try_renderer;
pub mod voronoi;

pub use color::*;
// pub use grid::*;
pub use geometry::*;
pub use shape::*;
use std::time::Instant;

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

pub fn print_elapsed_time<F>(label: &str, operation: F)
    where F: FnOnce()
{
    let start_time = Instant::now();
    operation();
    println!("{}: {:?}", label, Instant::now() - start_time);
}

#[inline]
pub fn cell_index_to_x_y_usize(width: usize, cell_index: usize) -> (usize, usize) {
    (cell_index % width, cell_index / width)
}

#[inline]
pub fn cell_index_to_x_y_isize(width: isize, cell_index: usize) -> (isize, isize) {
    let cell_index = cell_index as isize;
    (cell_index % width, cell_index / width)
}

#[inline]
pub fn x_y_to_cell_index_usize(width: usize, x: usize, y: usize) -> usize {
    (y * width) + x
}

#[inline]
pub fn x_y_to_cell_index_isize(width: isize, x: isize, y: isize) -> usize {
    ((y * width) + x) as usize
}

#[inline]
pub fn discriminant(a: f64, b: f64, c: f64) -> f64 {
    (b * b) - (4.0 * a * c)
}


#[inline]
pub fn quadratic_roots(a: f64, b: f64, c: f64) -> (f64, f64) {
    (quadratic_root(a, b, c, -1.0), quadratic_root(a, b, c, 1.0))
}

#[inline]
pub fn quadratic_root(a: f64, b: f64, c: f64, mult: f64) -> f64 {
    let num = (-1.0 * b) + (mult * discriminant(a, b, c).sqrt());
    let denom = 2.0 * a;
    num / denom
}
