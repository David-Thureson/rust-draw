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
    additive: bool,
    back_color: crate::Color1,
    colors: Vec<Color1>,
    frames: Vec<Frame>,
    // glyph_cache: GlyphCache<'static>,
    start_time: Instant,
    next_frame_time: Instant,
    frame_index: usize,
    current_frame: Frame,
    _shapes: Vec<Shape>,
    remove_count: usize,
    remove_seconds: f32,
    swap_count: usize,
    swap_seconds: f32,
    draw_count: usize,
    draw_seconds: f32,
    draw_shape_count: usize,
}

impl Renderer {
    pub fn display(title: &str, width: f64, height: f64, back_color: Color1, frames: Vec<Frame>) {
        Self::display_additive(title, width, height, back_color, frames, false);
    }

    pub fn display_additive(title: &str, width: f64, height: f64, back_color: crate::Color1, frames: Vec<Frame>, additive: bool) {
        Self::display_additive_with_colors(title, width, height, back_color, frames, additive, vec![]);
    }

    pub fn display_additive_with_colors(title: &str, width: f64, height: f64, back_color: crate::Color1, frames: Vec<Frame>, additive: bool, colors: Vec<Color1>) {
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

        /*
        let glyph_cache = GlyphCache::new(
            "./assets/fonts/Lato-Regular.ttf",
            (),
            TextureSettings::new().min(Filter::Nearest),
        ).unwrap();

         */

        let mut rend = Renderer {
            gl: GlGraphics::new(opengl),
            additive,
            back_color,
            colors,
            frames,
            // glyph_cache,
            start_time: Instant::now(),
            next_frame_time,
            frame_index: usize::max_value(),
            current_frame,
            _shapes: vec![],
            remove_count: 0,
            remove_seconds: 0.0,
            swap_count: 0,
            swap_seconds: 0.0,
            draw_count: 0,
            draw_seconds: 0.0,
            draw_shape_count: 0
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
                    println!("remove_count = {}; remove_seconds = {}; swap_count = {}; swap_seconds = {}; draw_count = {}; draw_seconds = {}", self.remove_count, self.remove_seconds, self.swap_count, self.swap_seconds, self.draw_count, self.draw_seconds);
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

        let clear_background= !self.additive || self.frame_index == 0;

        self.draw_count += 1;
        self.draw_shape_count += shapes.len();
        let start_time = Instant::now();
        let colors = self.colors.clone();
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            if clear_background {
                clear(back_color.into(), gl);
            }

            for shape in shapes.iter() {
                //bg!(&shape);
                match shape {
                    Shape::Circle { center, radius, color} => {
                        let rect = [center.x - radius, center.y - radius, radius * 2.0, radius * 2.0];
                        let transform = c.transform;
                        ellipse((*color).into(), rect, transform, gl);
                        //ellipse([1.0, 1.0, 1.0, 1.0], *rect, transform, gl);
                    },
                    Shape::Rectangle { top_left, bottom_right, color} => {
                        // if *color != back_color {
                        let rect = [top_left.x, top_left.y, bottom_right.x - top_left.x, bottom_right.y - top_left.y];
                        let transform = c.transform;
                        rectangle((*color).into(), rect, transform, gl);
                        // }
                    }
                    Shape::RectangleFast { x, y, width, height, color_index} => {
                        // if self.additive || *color_index != 0 {
                        if *color_index != 0 {
                            let rect = [*x, *y, *width, *height];
                            let transform = c.transform;
                            rectangle(colors[*color_index].for_render, rect, transform, gl);
                        }
                    }
                    // _ => unimplemented!(),
                }
            }

        });
        self.draw_seconds += (Instant::now() - start_time).as_secs_f32();

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


// use piston_window::*;
/*
pub fn try_draw_text_in_window() {
    // https://medium.com/@arpith/rendering-text-in-rust-with-piston-window-5811b63b1324
    let text = "abc 123";
    // let mut window: piston_window::WindowSettings = piston_window::WindowSettings::new(
    let mut window: piston_window::PistonWindow = piston_window::WindowSettings::new(
    // let mut window: WindowSettings = WindowSettings::new(
        text,
        [200, 200]
    )
    .fullscreen(true)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let ref font = assets.join("FiraSans-Regular.ttf");
    let factory = window.factory.clone();
    let texture = TextureSettings::new();
    // let mut glyphs = Glyphs::new(font, factory, texture).unwrap();

    window.draw
}
 */