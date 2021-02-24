use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use euclid;
use std::time::{Duration, Instant};
use std::cmp::min;
use std::ops::Add;

type Color = [f32; 4];

const COLOR_BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const COLOR_WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const COLOR_RED: [f32; 4]   = [1.0, 0.0, 0.0, 1.0];
const COLOR_GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const COLOR_BLUE: [f32; 4]  = [0.0, 0.0, 1.0, 1.0];

pub struct Animator {
    gl: GlGraphics, // OpenGL drawing backend.
    back_color: Color,
    //frame_function: &dyn Fn() -> (ShapeList, u16),
    frame_seconds: f64,
    frames: Vec<ShapeList>,
    next_frame_time: Instant,
    current_frame: usize,
}

impl Animator {
    fn render(&mut self, args: &RenderArgs) {

        //rintln!("Animator::render()");

        use graphics::*;

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0.0;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let back_color = self.back_color.clone();

        // let mut frame_index = (args.ext_dt / self.frame_seconds).floor() as usize;
        if Instant::now() >= self.next_frame_time {
            self.next_frame_time = self.next_frame_time.add(Duration::from_secs_f64(self.frame_seconds));
            if self.current_frame < self.frames.len() - 1 {
                self.current_frame += 1;
            }
            dbg!(self.current_frame, args.ext_dt);
        }
        // frame_index = min(frame_index, self.frames.len() - 1);
        let shapes = self.frames[self.current_frame].clone();
        //bg!(&shapes);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(back_color, gl);

            /*
            let rect = [20.0, 40.0, 10.0, 10.0];
            let transform = c.transform;
            ellipse(COLOR_WHITE, rect, transform, gl);

            let transform = c.transform.trans(50.0, 50.0);
            ellipse(COLOR_BLUE, rect, transform, gl);
            */

            for shape in shapes {
                match shape {
                    Shape::Circle { center_x, center_y, radius, color } => {
                        let rect = [center_x - radius, center_y - radius, radius * 2.0, radius * 2.0];
                        let transform = c.transform;
                        ellipse(color, rect, transform, gl);
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
            rectangle(COLOR_WHITE, square, transform, gl);
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
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut anim = Animator {
        gl: GlGraphics::new(opengl),
        back_color: COLOR_BLACK,
        frame_seconds: 1.0,
        frames: vec![
            vec![Shape::circle(20.0, 20.0, 5.0, COLOR_WHITE)],
            vec![Shape::circle(40.0, 20.0, 5.0, COLOR_BLUE), Shape::circle(40.0, 40.0, 5.0, COLOR_WHITE)],
            vec![Shape::circle(60.0, 20.0, 5.0, COLOR_GREEN), Shape::circle(60.0, 40.0, 5.0, COLOR_BLUE), Shape::circle(60.0, 60.0, 5.0, COLOR_WHITE)],
        ],
        next_frame_time: Instant::now().add(Duration::from_secs_f64(1.0)),
        current_frame: 0,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            anim.render(&args);
        }

        if let Some(args) = e.update_args() {
            anim.update(&args);
        }
    }
}

// pub struct ScreenSpace;

pub type ShapeList = Vec<Shape>;

#[derive(Clone, Debug)]
pub enum Shape {
    Circle {
        //center: euclid::Point2D<f64, ScreenSpace>,
        center_x: f64,
        center_y: f64,
        radius: f64,
        color: Color,
    }
}

impl Shape {
    pub fn circle(center_x: f64, center_y: f64, radius: f64, color: Color) -> Shape {
        Shape::Circle {
            center_x,
            center_y,
            radius,
            color
        }
    }
}
