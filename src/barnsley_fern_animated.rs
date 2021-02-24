// Adapted from https://rosettacode.org/wiki/Barnsley_fern#Rust

use rand::Rng;

use crate::*;
use crate::renderer_3::Renderer;

pub fn main() {
    let point_count: usize = 100_000;
    let width = 1000.0;
    let height = width;
    let back_color = Color1::white();
    let point_radius = 0.5;
    let point_color: Color1 = Color256::from_rgb(50, 205, 50).into();
    let point_color_end = Color1::red();
    let total_seconds = 10.0;
    let batch_size = 1_000;
    let frame_count = point_count / batch_size;
    let frame_seconds = total_seconds / frame_count as f64;

    let points = gen_points(width, height, point_count);

    let mut frames = vec![];
    for frame_index in 0..frame_count {
        let mut shapes = vec![];
        for point_index in 0..((frame_index + 1) * batch_size) {
            let (x, y) = points[point_index];
            // let color = point_color.clone();
            let point_frame_index = point_index / batch_size;
            let color = point_color.gradiant_one(&point_color_end, frame_count, point_frame_index);
            shapes.push(Shape::circle(x, y, point_radius, color));
        }
        frames.push(Frame::new(shapes, frame_seconds));
    }
    Renderer::display("Barnsley Fern", width, height, back_color, frames);
}

fn gen_points(height: f64, width: f64, point_count: usize) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    let max_iterations = point_count as u32;
    let height = height as i32;
    let width = width as i32;

    let mut points = vec![];

    let mut x = 0.;
    let mut y = 0.;
    for _ in 0..max_iterations {
        let r = rng.gen::<f32>();
        let cx: f64;
        let cy: f64;

        if r <= 0.01 {
            cx = 0f64;
            cy = 0.16 * y as f64;
        } else if r <= 0.08 {
            cx = 0.2 * x as f64 - 0.26 * y as f64;
            cy = 0.23 * x as f64 + 0.22 * y as f64 + 1.6;
        } else if r <= 0.15 {
            cx = -0.15 * x as f64 + 0.28 * y as f64;
            cy = 0.26 * x as f64 + 0.26 * y as f64 + 0.44;
        } else {
            cx = 0.85 * x as f64 + 0.04 * y as f64;
            cy = -0.04 * x as f64 + 0.85 * y as f64 + 1.6;
        }
        x = cx;
        y = cy;

        let circle_x = (width as f64) / 2. + x * (width as f64) / 11.;
        let circle_y = (height as f64) - y * (height as f64) / 11.;
        points.push((circle_x, circle_y));
    }

    debug_assert_eq!(point_count, points.len());
    points
}
