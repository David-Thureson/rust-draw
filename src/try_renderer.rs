use crate::*;
use renderer::Renderer;

pub fn main() {
    first();
}

fn first() {
    let frames = vec![
        Frame::new(vec![Shape::circle(20.0, 20.0, 5.0, Color1::white())], 2.0),
        Frame::new(vec![
            Shape::circle(40.0, 20.0, 5.0, Color1::blue()),
            Shape::circle(40.0, 40.0, 5.0, Color1::white())
        ],4.0),
        Frame::new(vec![
            Shape::circle(60.0, 20.0, 5.0, Color1::green()),
            Shape::circle(60.0, 40.0, 5.0, Color1::blue()),
            Shape::circle(60.0, 60.0, 5.0, Color1::white())
        ],8.0),
        Frame::new(vec![
            Shape::circle( 40.0,  40.0, 40.0, Color1::white()),
            Shape::circle(160.0,  40.0, 40.0, Color1::red()),
            Shape::circle( 40.0, 160.0, 40.0, Color1::green()),
            Shape::circle(160.0, 160.0, 40.0, Color1::blue())
        ],16.0),
    ];
    Renderer::display("Circle Test", 300.0, 200.0, Color1::black(), frames);
}
