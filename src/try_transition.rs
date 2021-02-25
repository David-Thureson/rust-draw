use crate::*;
use crate::animator::*;

pub fn main() {
    first();
}

fn first() {
    let width = 1000.0;
    let height = 1000.0;
    let back_color = Color1::white();
    // let shape_count = 10;

    /*
    // Circle with changing size.
    let path = Path::new(&Shape::circle_xy(300.0, 300., 0.0, Color1::blue()))
        .radius(300.0)
        .trans(30)
        .radius(100.0)
        .trans(50);
    let frame_seconds = 0.1;
    let mut animator = Animator::new(frame_seconds);
    animator.add_path(0, &path);
    dbg!(&animator);
    animator.show("Transitions", width, height, back_color);
    */

    /*
    // Circle with changing color and transparency.
    let trans_frames = 300;
    let frame_seconds = 0.01;
    let path = Path::new(&Shape::circle_xy(300.0, 300., 100.0, Color1::blue()))
        .color(Color1::red()).trans(trans_frames)
        .color(Color1::green()).trans(trans_frames)
        .color(Color1::black()).trans(trans_frames)
        .color(Color1::from_rgb(0.25, 0.5, 0.75)).trans(trans_frames)
        .color(Color1::from_rgba(0.25, 0.5, 0.75, 0.0)).trans(trans_frames);
    let mut animator = Animator::new(frame_seconds);
    animator.add_path(0, &path);
    //bg!(&animator);
    animator.show("Transitions", width, height, back_color);
    */

    // Moving circle with changing colors.
    let trans_frames = 60;
    let frame_seconds = 1.0 / trans_frames as f64;
    let path = Path::new(&Shape::circle_xy(25.0, 25., 25.0, Color1::blue()))
        .center_xy(975.0, 500.0).color(Color1::red()).trans(trans_frames)
        .center_xy(25.0, 850.0).color(Color1::green()).trans(trans_frames)
        .center_xy(500.0, 25.0).color(Color1::blue()).trans(trans_frames);
    let mut animator = Animator::new(frame_seconds);
    animator.add_path(0, &path);
    //bg!(&animator);
    animator.show("Transitions", width, height, back_color);
}