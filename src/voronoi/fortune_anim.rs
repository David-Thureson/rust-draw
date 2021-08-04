// https://www2.cs.sfu.ca/~binay/813.2011/Fortune.pdf

use rand::Rng;
use decorum::Finite;
use std::collections::BTreeMap;

pub fn main() {
    try_animate();
}

struct FortuneAnim {
    pub width: usize,
    pub height: usize,
    pub points: Vec<(Finite<f64>, Finite<f64>)>,
}

struct FortuneEventQueue {
    events: BTreeMap<Finite<f64>, FortuneEventType>,
}

enum FortuneEventType {
    Point,
    Vertex,
}

impl FortuneAnim {
    pub fn new(width: usize, height: usize, point_count: usize) -> Self {
        let mut fortune = Self {
            width,
            height,
            points: vec![],
        };
        fortune.add_points(point_count);
        fortune
    }

    fn add_points(&mut self, point_count: usize) {
        let mut rng = rand::thread_rng();
        let (width, height) = (self.width as f64, self.height as f64);
        for _ in 0..point_count {
            let x= (rng.gen::<f64>() * width).into();
            let y = (rng.gen::<f64>() * height).into();
            self.points.push((x, y));
        }
    }

    pub fn animate(&mut self, anim_seconds: usize, frame_count: usize) {

    }

    fn x_to_key(&self, x: f64) {

    }
}

fn try_animate() {
    let (width, height, point_count, anim_seconds, frame_count) = (800, 400, 12, 30, 60);
    let mut fortune = FortuneAnim::new(width, height, point_count);
    fortune.animate(anim_seconds, frame_count);
}