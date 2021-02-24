use crate::*;
use rand::prelude::*;
use renderer::Renderer;

pub fn main() {
    first();
}

fn first() {
    let width = 1000.0;
    let height = 1000.0;
    let back_color = COLOR_BLACK;
    let anchor_radius = 5.0;
    let anchor_color = COLOR_BLUE;
    let point_radius = 0.5;
    let point_color = COLOR_WHITE;
    let anchor_count = 3;
    // let point_count: usize = 1000;
    // let total_seconds = 10.0;
    let point_count: usize = 500000;
    let total_seconds = 10.0;
    let batch_size = 5000;
    let frame_count = point_count / batch_size;
    let frame_seconds = total_seconds / frame_count as f64;

    let mut rng = thread_rng();

    let mut anchors: Vec<(f64, f64)> = vec![];
    // for _ in 0..anchor_count {
    //    anchors.push((rng.gen_range(0.0..width), rng.gen_range(0.0..height)));
    //}
    anchors.push((20.0, 980.0));
    anchors.push((500.0, 20.0));
    anchors.push((980.0, 980.0));
    //bg!(&anchors);

    let points = gen_points(&mut rng, &anchors, point_count);

    let mut frames = vec![];
    for frame_index in 0..frame_count {
        let mut shapes = vec![];
        for anchor in anchors.iter() {
            let (x, y) = anchor;
            shapes.push(Shape::circle(*x, *y, anchor_radius, anchor_color));
        }
        for point_index in 0..(frame_index * batch_size) {
            let (x, y) = points[point_index];
            shapes.push(Shape::circle(x, y, point_radius, point_color));
        }
        frames.push(Frame::new(shapes, frame_seconds));
    }
    Renderer::display("Fractal", width, height, back_color, frames);
}

fn gen_points(rng: &mut ThreadRng, anchors: &Vec<(f64, f64)>, point_count: usize) -> Vec<(f64, f64)> {
    let distance_to_anchor = 0.5;
    let throwaway_count = 10;
    let anchor_count = anchors.len();
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    for anchor in anchors.iter() {
        let (x, y) = anchor;
        x_min = x_min.min(*x);
        x_max = x_max.max(*x);
        y_min = y_min.min(*y);
        y_max = y_max.max(*y);
    }
    let mut points = vec![];
    let mut x = (x_min + x_max) / 2.0;
    let mut y = (y_min + y_max) / 2.0;
    //bg!(anchors);
    for point_index in 0..(point_count + throwaway_count) {
        //rintln!("\npoint_index = {}; x = {}; y = {}", point_index, x, y);
        let anchor_index = rng.gen_range(0..anchor_count);
        let (anchor_x, anchor_y) = anchors[anchor_index];
        x = x + ((anchor_x - x) * distance_to_anchor);
        y = y + ((anchor_y - y) * distance_to_anchor);
        //rintln!("anchor_index = {}; anchor_x = {}; anchor_y = {}; x = {}, y = {}", anchor_index, anchor_x, anchor_y, x, y);
        if point_index >= throwaway_count {
            points.push((x, y));
        }
    }
    assert_eq!(point_count, points.len());
    points
}
