use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use std::time::{Duration, Instant};
use std::ops::Add;

use crate::*;

pub struct Renderer {
    gl: GlGraphics, // OpenGL drawing backend.
    back_color: crate::Color1,
    frames: Vec<Frame>,
    start_time: Instant,
    next_frame_time: Instant,
    frame_index: usize,
    current_frame: Frame,
    _shapes: Vec<Shape>,
    remove_count: usize,
    remove_seconds: f32,
    swap_count: usize,
    swap_seconds: f32,
}

impl Renderer {
    pub fn display(title: &str, width: f64, height: f64, back_color: crate::Color1, frames: Vec<Frame>) {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        let mut window: Window = WindowSettings::new(title, [width, height])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // Start with a dummy frame to make the loop in render() simpler.
        let current_frame = Frame::new(vec![], 0.001);
        let next_frame_time = Instant::now();

        let mut rend = Renderer {
            gl: GlGraphics::new(opengl),
            back_color,
            frames,
            start_time: Instant::now(),
            next_frame_time,
            frame_index: usize::max_value(),
            current_frame,
            _shapes: vec![],
            remove_count: 0,
            remove_seconds: 0.0,
            swap_count: 0,
            swap_seconds: 0.0,
        };

        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut window) {
            if let Some(args) = e.render_args() {
                rend.render(&args);
            }

            if let Some(args) = e.update_args() {
                rend.update(&args);
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {

        //rintln!("Animator::render()");

        use graphics::*;

        let back_color = self.back_color.clone();

        // let mut frame_index = (args.ext_dt / self.frame_seconds).floor() as usize;
        let now = Instant::now();
        if now >= self.next_frame_time {
            if !self.frames.is_empty() {
                self.frame_index = if self.frame_index == usize::max_value() { 0 } else { self.frame_index + 1 };
                //rintln!("\nnow = {}", now.duration_since(self.start_time).as_secs_f32());
                //rintln!("next_frame_time = {}", self.next_frame_time.duration_since(self.start_time).as_secs_f32());
                //rintln!("frame_index = {}", self.frame_index);
                let start_time = Instant::now();
                self.current_frame = self.frames.remove(0);
                self.remove_seconds += (Instant::now() - start_time).as_secs_f32();
                self.remove_count += 1;
                //rintln!("shapes in this frame = {}", self.current_frame.shapes.len());
                self.next_frame_time = self.next_frame_time.add(Duration::from_secs_f64(self.current_frame.seconds_to_next));
                if self.frames.is_empty() {
                    println!("Last frame reached at elapsed seconds = {}", (Instant::now() - self.start_time).as_secs_f32());
                    println!("remove_count = {}; remove_seconds = {}; swap_count = {}; swap_seconds = {}", self.remove_count, self.remove_seconds, self.swap_count, self.swap_seconds);
                }
                //rintln!("seconds_to_next = {}", self.frames[self.current_frame].seconds_to_next);
                //rintln!("next_frame_time = {}", self.next_frame_time.duration_since(self.start_time).as_secs_f32());
                //bg!(self.current_frame, self.next_frame_time, self.frames[self.current_frame].seconds_to_next);
            }
        }
        // frame_index = min(frame_index, self.frames.len() - 1);
        //let shapes = self.frames[self.current_frame].shapes.clone();
        //bg!(&shapes);

        let mut shapes = vec![];
        let start_time = Instant::now();
        std::mem::swap(&mut shapes, &mut self.current_frame.shapes);
        self.swap_seconds += (Instant::now() - start_time).as_secs_f32();
        self.swap_count += 1;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(back_color.into(), gl);

            for shape in shapes.iter() {
                match shape {
                    Shape::Circle { center_x, center_y, radius, color, .. } => {
                        let rect = [center_x - radius, center_y - radius, radius * 2.0, radius * 2.0];
                        let transform = c.transform;
                        ellipse((*color).into(), rect, transform, gl);
                    },
                    // _ => unimplemented!(),
                }
            }

        });

        let start_time = Instant::now();
        std::mem::swap(&mut self.current_frame.shapes, &mut shapes);
        self.swap_seconds += (Instant::now() - start_time).as_secs_f32();
        self.swap_count += 1;
    }

    fn update(&mut self, _args: &UpdateArgs) {
        //rintln!("Animator::update()");
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }
}
