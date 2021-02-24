use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use euclid;
use std::time::{Duration, Instant};
use std::cmp::min;
use std::ops::Add;

use crate::{Frame, Shape, ShapeList};

pub struct Renderer1 {
    gl: GlGraphics, // OpenGL drawing backend.
    back_color: crate::Color1,
    frames: Vec<Frame>,
    start_time: Instant,
    next_frame_time: Instant,
    current_frame: usize,
}

impl Renderer1 {
    pub fn display(title: &str, width: f64, height: f64, back_color: crate::Color1, frames: Vec<Frame>) {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        let mut window: Window = WindowSettings::new(title, [width, height])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let next_frame_time = Instant::now().add(Duration::from_secs_f64(frames[0].seconds_to_next));

        let mut rend = Self {
            gl: GlGraphics::new(opengl),
            back_color,
            frames,
            start_time: Instant::now(),
            next_frame_time,
            current_frame: 0,
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

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0.0;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let back_color = self.back_color.clone();

        // let mut frame_index = (args.ext_dt / self.frame_seconds).floor() as usize;
        let now = Instant::now();
        if now >= self.next_frame_time {
            //rintln!("\nnow = {}", now.duration_since(self.start_time).as_secs_f32());
            //rintln!("next_frame_time = {}", self.next_frame_time.duration_since(self.start_time).as_secs_f32());
            if self.current_frame < self.frames.len() - 1 {
                self.current_frame += 1;
                if self.current_frame == self.frames.len() - 1 {
                    println!("Last frame reached at elapsed seconds = {}", (Instant::now() - self.start_time).as_secs_f32());
                }
            }
            self.next_frame_time = self.next_frame_time.add(Duration::from_secs_f64(self.frames[self.current_frame].seconds_to_next));
            //rintln!("seconds_to_next = {}", self.frames[self.current_frame].seconds_to_next);
            //rintln!("next_frame_time = {}", self.next_frame_time.duration_since(self.start_time).as_secs_f32());
            //bg!(self.current_frame, self.next_frame_time, self.frames[self.current_frame].seconds_to_next);
        }
        // frame_index = min(frame_index, self.frames.len() - 1);
        let shapes = self.frames[self.current_frame].shapes.clone();
        //bg!(&shapes);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(back_color.into(), gl);

            /*
            let rect = [20.0, 40.0, 10.0, 10.0];
            let transform = c.transform;
            ellipse(Color1::white(), rect, transform, gl);

            let transform = c.transform.trans(50.0, 50.0);
            ellipse(Color1::blue(), rect, transform, gl);
            */

            for shape in shapes {
                match shape {
                    Shape::Circle { center_x, center_y, radius, color } => {
                        let rect = [center_x - radius, center_y - radius, radius * 2.0, radius * 2.0];
                        let transform = c.transform;
                        ellipse(color.into(), rect, transform, gl);
                    },
                    _ => unimplemented!(),
                }
            }

            /*
            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(Color1::white(), square, transform, gl);
            */

        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        //rintln!("Animator::update()");
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }
}

pub fn main() {
}
