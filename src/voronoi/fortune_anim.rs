// https://www2.cs.sfu.ca/~binay/813.2011/Fortune.pdf

use rand::Rng;
use decorum::Finite;
use std::collections::BTreeMap;

pub const COLOR_BACKGROUND: usize      = 0;
pub const COLOR_DIRECTRIX: usize       = 1;
pub const COLOR_PARABOLA: usize        = 2;
pub const COLOR_FOCUS_PENDING: usize   = 3;
pub const COLOR_FOCUS_ACTIVE: usize    = 4;
pub const COLOR_FOCUS_CONTAINED: usize = 5;

pub const FOCUS_RADIUS: f64 = 3.0;

pub const DIRECTRIX_THICKNESS: f64 = 1.0;

pub fn main() {
    // try_animate();
}

struct FortuneAnim {
    pub width: usize,
    pub height: usize,
    pub points: Vec<(f64, f64)>,
}

#[allow(dead_code)]
struct FortuneEventQueue {
    events: BTreeMap<Finite<f64>, FortuneEvent>,
}

#[allow(dead_code)]
enum FortuneEvent {
    Frame {
        y: f64,
    },
    Point {
        x: f64,
        y: f64,
    },
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
            let x= rng.gen::<f64>() * width;
            let y = rng.gen::<f64>() * height;
            self.points.push((x, y));
        }
    }

    pub fn animate(&mut self, _anim_seconds: usize, frame_count: usize) {
        let mut queue = FortuneEventQueue::new();

        // Add all of the point events, when the sweep line passes over a point and thus creates a
        // new parabola.
        for (x, y) in self.points.iter() {
            queue.add_event((*y).into(), FortuneEvent::new_point(*x, *y));
        }

        // Add all of the frame events.
        let inc = self.height as f64 / frame_count as f64;
        let mut y = inc;
        for _ in 0..frame_count {
            queue.add_event(y.into(), FortuneEvent::new_frame(y));
            y += inc;
        }

        // There are no vertex events yet because the sweep line has not activated any points.

        /*
        while let Some((_, event)) = queue.events.pop_first() {
            match event {
                FortuneEvent::Frame { y } => {

                },
                FortuneEvent::Point { x, y } => {

                },
                FortuneEvent::Vertex => {

                },
            }
        }
        */

    }


}

impl FortuneEventQueue {
    pub fn new() -> Self {
        Self {
            events: Default::default(),
        }
    }

    pub fn add_event(&mut self, key: Finite<f64>, event: FortuneEvent) {
        debug_assert!(!self.events.contains_key(&key));
        self.events.insert(key, event);
    }
}

impl FortuneEvent {
    pub fn new_frame(y: f64) -> Self {
        Self::Frame {
            y,
        }
    }

    pub fn new_point(x: f64, y: f64) -> Self {
        Self::Point {
            x,
            y,
        }
    }
}

#[allow(dead_code)]
fn try_animate() {
    let (width, height, point_count, anim_seconds, frame_count) = (800, 400, 12, 30, 60);
    let mut fortune = FortuneAnim::new(width, height, point_count);
    fortune.animate(anim_seconds, frame_count);
}


