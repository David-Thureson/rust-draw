use crate::*;

#[derive(Debug)]
pub struct Animator {
    frames: Vec<Frame>,
    frame_seconds: f64,
}

pub struct Path {
    from_shape: Shape,
    transitions: Vec<Transition>,
    working_shape: Shape,
}

pub struct Transition {
    to_shape: Shape,
    frame_count: usize,
}

impl Animator {
    pub fn new(frame_seconds: f64) -> Self {
        Animator {
            frames: vec![],
            frame_seconds,
        }
    }

    pub fn add_path(&mut self, start_frame_index: usize, path: &Path) {
        let mut from_shape = path.from_shape.clone();
        let mut frame_index = start_frame_index;
        for (transition_index, transition) in path.transitions.iter().enumerate() {
            dbg!(&from_shape);
            self.confirm_frame_capacity(frame_index + transition.frame_count + 1);
            if transition_index == 0 {
                self.frames[frame_index].shapes.push(from_shape.clone());
            }
            for local_frame_index in 1..(transition.frame_count) {
                let one_shape = from_shape.gradiant_one(&transition.to_shape, transition.frame_count, local_frame_index);
                //bg!(&one_shape);
                self.frames[frame_index + local_frame_index].shapes.push(one_shape);
            }
            frame_index = frame_index + transition.frame_count;
            self.frames[frame_index].shapes.push(transition.to_shape.clone());
            from_shape = transition.to_shape.clone();
        }
    }

    pub fn show(&mut self, title: &str, width: f64, height: f64, back_color: crate::Color1) {
        let frames = std::mem::replace(&mut self.frames, vec![]);
        renderer_3::Renderer::display(title, width, height, back_color, frames)
    }

    fn confirm_frame_capacity(&mut self, frame_count: usize) {
        if frame_count > self.frames.len() {
            for _ in self.frames.len()..frame_count {
                self.frames.push(Frame::new(vec![], self.frame_seconds));
            }
        }
    }
}

impl Path {
    pub fn new(from_shape: &Shape) -> Self {
        Path {
            from_shape: from_shape.clone(),
            transitions: vec![],
            working_shape: from_shape.clone(),
        }
    }

    pub fn center(mut self, center: PointF64) -> Self {
        self.working_shape = self.working_shape.with_center(center);
        self
    }

    pub fn center_xy(mut self, center_x: f64, center_y: f64) -> Self {
        self.working_shape = self.working_shape.with_center_xy(center_x, center_y);
        self
    }

    pub fn radius(mut self, radius: f64) -> Self {
        self.working_shape = self.working_shape.with_radius(radius);
        self
    }

    pub fn color(mut self, color: Color1) -> Self {
        self.working_shape = self.working_shape.with_color(color);
        self
    }

    pub fn trans(mut self, frame_count: usize) -> Self {
        self.transitions.push(Transition::new(&self.working_shape, frame_count));
        self
    }
}

impl Transition {
    pub fn new(to_shape: &Shape, frame_count: usize) -> Self {
        Self {
            to_shape: to_shape.clone(),
            frame_count,
        }
    }
}