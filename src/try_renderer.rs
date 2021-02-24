use crate::*;
use renderer::Renderer;

pub fn main() {
    first();
}

fn first() {
    let frames = vec![
        Frame::new(vec![Shape::circle(20.0, 20.0, 5.0, COLOR_WHITE)], 2.0),
        Frame::new(vec![
            Shape::circle(40.0, 20.0, 5.0, COLOR_BLUE),
            Shape::circle(40.0, 40.0, 5.0, COLOR_WHITE)
        ],4.0),
        Frame::new(vec![
            Shape::circle(60.0, 20.0, 5.0, COLOR_GREEN),
            Shape::circle(60.0, 40.0, 5.0, COLOR_BLUE),
            Shape::circle(60.0, 60.0, 5.0, COLOR_WHITE)
        ],8.0),
        Frame::new(vec![
            Shape::circle( 40.0,  40.0, 40.0, COLOR_WHITE),
            Shape::circle(160.0,  40.0, 40.0, COLOR_RED),
            Shape::circle( 40.0, 160.0, 40.0, COLOR_GREEN),
            Shape::circle(160.0, 160.0, 40.0, COLOR_BLUE)
        ],16.0),
    ];
    Renderer::display("Circle Test", 300.0, 200.0, COLOR_BLACK, frames);
}
