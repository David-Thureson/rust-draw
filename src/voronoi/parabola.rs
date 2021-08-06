use std::collections::BTreeMap;
use decorum::Finite;
use crate::*;
use crate::voronoi::fortune_anim::{COLOR_FOCUS_ACTIVE, COLOR_FOCUS_CONTAINED, COLOR_FOCUS_PENDING, FOCUS_RADIUS, COLOR_PARABOLA, DIRECTRIX_THICKNESS, COLOR_DIRECTRIX};
use crate::grid::Grid;
use crate::renderer_3::Renderer;
use rand::{thread_rng, Rng};
use num::traits::Pow;

pub fn main() {
    // try_draw_parabola();
    try_draw_parabolas();
}

pub struct ParabolaList {
    pub parabolas: BTreeMap<Finite<f64>, Parabola>,
}

#[derive(Clone)]
pub struct Parabola {
    pub focus_x: f64,
    pub focus_y: f64,
    pub directrix_y: f64,
    pub p: f64,
    pub h: f64,
    pub k: f64,
    pub y_max: f64,
    pub state: ParabolaState,
}

#[derive(Clone)]
pub enum ParabolaState {
    Active,
    Contained,
    Pending,
}

impl ParabolaList {
    pub fn new() -> Self {
        Self {
            parabolas: Default::default(),
        }
    }

    pub fn add_parabola(&mut self, focus_x: f64, focus_y: f64) {
        self.parabolas.insert(focus_x.into(), Parabola::new(focus_x, focus_y));
    }

    fn update_parabolas(&mut self, directrix_y: f64) {
        for parabola in self.parabolas.values_mut() {
            parabola.update(directrix_y);
        }
    }

    pub fn add_to_shapes(&mut self, shapes: &mut Vec<Shape>, width: usize, height: usize, directrix_y: f64) {
        // Update the parabolas' states.
        self.update_parabolas(directrix_y);
        let ref_parabolas = self.parabolas.clone();
        for parabola in self.parabolas.values_mut() {
            if parabola.focus_y > directrix_y {
                // The parabola is below the directrix so it's not yet active.
                parabola.state = ParabolaState::Pending;
            } else {
                parabola.state = ParabolaState::Active;
                // See if the parabola is contained in any other parabola.
                for ref_parabola in ref_parabolas.values() {
                    if ref_parabola.contains(parabola,None) {
                        parabola.state = ParabolaState::Contained;
                        break;
                    }
                }
            }
        }
        for parabola in self.parabolas.values() {
            parabola.add_to_shapes(shapes, width, height);
        }
    }

    pub fn debug_contains(&mut self, directrix_y: f64) {
        self.update_parabolas(directrix_y);
        let ref_parabolas = self.parabolas.clone();
        for (index, parabola) in self.parabolas.values().enumerate() {
            println!("\n[{}] ({}, {})", index, parabola.focus_x.round(), parabola.focus_y.round());
            for (ref_index, ref_parabola) in ref_parabolas.values().enumerate() {
                if ref_parabola.contains(parabola, None) {
                    ref_parabola.contains(parabola, Some(ref_index));
                }
            }
        }
    }

}

impl Parabola {
    pub fn new(focus_x: f64, focus_y: f64) -> Self {
        Self {
            focus_x: focus_x,
            focus_y: focus_y,
            directrix_y: 0.0,
            p: 0.0,
            h: 0.0,
            k: 0.0,
            y_max: 0.0,
            state: ParabolaState::Pending,
        }
    }

    #[inline]
    pub fn update(&mut self, directrix_y: f64) {
        self.directrix_y = directrix_y;
        self.p = self.p(directrix_y);
        let (h, k) = self.h_k(directrix_y);
        self.h = h;
        self.k = k;
        self.y_max = self.focus_y - self.p;
    }

    #[inline]
    pub fn p(&self, directrix_y: f64) -> f64 {
        (self.focus_y - directrix_y) / 2.0
    }

    #[inline]
    pub fn y_max(&self, directrix_y: f64) -> f64 {
        self.focus_y - self.p(directrix_y)
    }

    #[inline]
    pub fn h_k(&self, directrix_y: f64) -> (f64, f64) {
        (self.focus_x, self.y_max(directrix_y))
    }

    #[inline]
    pub fn y(&self, x: f64) -> f64 {
        let num = (x * x) + (-2.0 * x * self.h) + (self.h * self.h) + (4.0 * self.p * self.k);
        let denom = 4.0 * self.p;
        let y = num / denom;
        y
    }

    #[inline]
    pub fn error(&self, x: usize, y: usize) -> f64 {
        let (x, y) = (x as f64, y as f64);
        (x - self.h).pow(2) + (-4.0 * self.p * (y - self.k))
    }

    pub fn x_coords(&self, y: f64, debug: bool) -> Vec<f64> {
        if debug { println!("\t\tx_coords(): directrix_y = {}; y = {}; y_max = {}", self.directrix_y.round(), y.round(), self.y_max.round()); }
        let mut v = vec![];
        if y == self.y_max {
            v.push(self.focus_x);
        } else {
            if y < self.y_max {
                // (x-h)^2 = 4p(y-k)
                // x^2 - 2xh + h^2 - 4p(y-k) = 0
                // So for the quadratic formula:
                let a: f64 = 1.0;
                let b: f64 = -2.0 * self.h;
                // let c: f64 = 4.0 * p * (y - k);
                let c: f64 = (self.h * self.h) + (-4.0 * self.p * (y - self.k));
                if debug { println!("\t\t\tp = {}, h = {}, k = {}, a = {}, b = {}, c = {}", self.p.round(), self.h.round(), self.k.round(), a.round(), b.round(), c.round()); }
                let discriminant = discriminant(a, b, c);
                assert!(discriminant > 0.0);
                let roots = quadratic_roots(a, b, c);
                if roots.0 < roots.1 {
                    v.push(roots.0);
                    v.push(roots.1);
                } else {
                    v.push(roots.1);
                    v.push(roots.0);
                }
            }
        }
        v
    }

    pub fn contains(&self, other: &Self, debug_self_index: Option<usize>) -> bool {
        let debug = debug_self_index.is_some();
        if debug { println!("\n\t[{}] ({}, {}) contains ({}, {}):", debug_self_index.unwrap(), self.focus_x.round(), self.focus_y.round(), other.focus_x.round(), other.focus_y.round()); }
        if self.focus_y > self.directrix_y || other.focus_y > self.directrix_y {
            // One or both parabolas is beyond the directrix and thus doesn't count yet.
            if debug { println!("\t\tOne or more below the directrix."); }
            return false;
        }
        if debug { println!("\t\tself.y_max = {}; other.y_max = {}", self.y_max.round(), other.y_max.round()); }
        if self.y_max <= other.y_max {
            // The current parabola must be below (higher y) the other parabola to contain it.
            if debug { println!("\t\tCurrent parabola is not below other parabola."); }
            return false;
        }
        // Find the left and right points on the current parabola at the same y coordinate as the
        // maximum point of the other parabola.
        let x_coords = self.x_coords(other.y_max, debug);
        assert_eq!(2, x_coords.len());
        if debug { println!("\t\tx_coords = [{}, {}]", x_coords[0].round(), x_coords[1].round()); }
        let contains = other.focus_x > x_coords[0] && other.focus_x < x_coords[1];
        if debug { println!("\t\tcontains = {}", contains); }
        contains
    }

    pub fn add_to_shapes(&self, shapes: &mut Vec<Shape>, width: usize, height: usize) {
        // Add the focus circle.
        let color_index = match self.state {
            ParabolaState::Active => COLOR_FOCUS_ACTIVE,
            ParabolaState::Contained => COLOR_FOCUS_CONTAINED,
            ParabolaState::Pending => COLOR_FOCUS_PENDING,
        };
        shapes.push(Shape::circle_fast(self.focus_x, self.focus_y, FOCUS_RADIUS, color_index));

        match self.state {
            ParabolaState::Active => {
                // Draw the right half in the old way.
                for x in self.focus_x.round() as usize..width {
                    let x = x as f64;
                    // (x-h)^2 = 4p(y-k)
                    // y = (x^2 - 2xh + h^2 + 4pk) / 4p
                    let num = (x * x) + (-2.0 * x * self.h) + (self.h * self.h) + (4.0 * self.p * self.k);
                    let denom = 4.0 * self.p;
                    let y = (num / denom).round();
                    if y >= 0.0 && y < height as f64 {
                        shapes.push(Shape::rectangle_fast(x, y, 1.0, 1.0, COLOR_PARABOLA));
                    }
                }
                // Draw the left half using http://members.chello.at/easyfilter/bresenham.pdf, page 10.
                // Note that the y axis is reversed so this is really an upward slope.
                let (mut x, mut y) = {
                    // Try x = 0 and see if y is on the screen.
                    let x = 0.0;
                    let y = self.y(x);
                    if y > 0.0 {
                        (x as usize, y.round() as usize)
                    } else {
                        // Instead start with y = 0 and find x.
                        let y = 0.0;
                        let x = self.x_coords(y, false)[0];
                        (x.round() as usize, y as usize)
                    }
                };
                let x_stop = self.focus_x.round() as usize;
                while x < x_stop {
                    //rintln!("x = {}, y = {}", x, y);
                    shapes.push(Shape::rectangle_fast(x as f64, y as f64, 1.0, 1.0, COLOR_PARABOLA));
                    let err_xy = self.error(x + 1, y + 1).abs();
                    let err_x = self.error(x, y + 1).abs();
                    let err_y = self.error(x + 1, y).abs();
                    //rintln!("\terr_xy = {}, err_x = {}, err_y = {}, err_x + err_xy = {}, err_y + err_xy = {}", err_xy, err_x, err_y, err_x + err_xy, err_y + err_xy);
                    /*
                    let mut inc = false;
                    if err_x + err_xy > 0.0 {
                        x += 1;
                        inc = true;
                    }
                    if err_y + err_xy > 0.0 {
                        y += 1;
                        inc = true;
                    }
                    if !inc {
                        // panic!();
                        break;
                    }*/
                    if err_xy < err_x {
                        if err_xy < err_y {
                            x += 1;
                            y += 1;
                        } else {
                            x += 1;
                        }
                    } else {
                        if err_x < err_y {
                            y += 1;
                        } else {
                            x += 1;
                        }
                    }
                }

                // For debugging, draw a horizontal line through the focus and another several
                // pixels higher.
                /*
                let y = self.focus_y;
                let x_coords = self.x_coords(directrix_y, y, false);
                shapes.push(Shape::line_fast(x_coords[0], y, x_coords[1], y, 1.0, COLOR_FOCUS_ACTIVE));
                let y = self.focus_y - 30.0;
                let x_coords = self.x_coords(directrix_y, y, false);
                shapes.push(Shape::line_fast(x_coords[0], y, x_coords[1], y, 1.0, COLOR_FOCUS_ACTIVE));

                 */
            },
            _ => {},
        }
    }
}

#[allow(dead_code)]
fn try_draw_parabola() {
    // let (width, height, focus_x, focus_y, directrix_y) = (800, 400, 300.0, 250.0, 350.0);
    let (width, height, focus_x, focus_y, directrix_y) = (100, 80, 60.0, 45.0, 65.0);
    let (origin_x, origin_y) = (focus_x, (focus_y + directrix_y) / 2.0);
    let a = (focus_y - directrix_y) / 2.0;
    let (display_width, display_height) = (width as f64, height as f64);
    let mut grid = Grid::new(width, height, 0);
    println!("focus = ({}, {}), directrix_y = {}, origin = ({}, {}), a = {}", focus_x, focus_y, directrix_y, origin_x, origin_y, a);

    // x^2 = 4ay
    for x in 0..width {
        let x_from_origin = x as f64 - origin_x;
        let y_from_origin = (x_from_origin * x_from_origin) / (4.0 * a);
        println!("{}, {}", x_from_origin, y_from_origin);
        let y = (y_from_origin + origin_y).round();
        if y >= 0.0 && y < height as f64 {
            grid.set_xy(x, y as usize, 1);
        }
    }

    let mut frames = vec![];
    let frame = grid.as_frame_color_index(display_width, display_height, 1.0);
    frames.push(frame);

    let additive = false;
    let back_color = Color1::black();
    Renderer::display_additive_with_colors("Parabola", display_width, display_height, back_color, frames, additive, vec![Color1::black(), Color1::white()]);
}

#[allow(dead_code)]
fn try_draw_parabolas() {
    let mut rng = thread_rng();
    // let (width, height, parabola_count, anim_seconds, frame_count) = (800, 800, 10, 1, 1);
    // let (width, height, parabola_count, anim_seconds, frame_count) = (1_000, 1_000, 10, 20, 20);
    let (width, height, parabola_count, anim_seconds, frame_count) = (1_600, 800, 100, 30, 400);
    let (display_width, display_height) = (width as f64, height as f64);
    let frame_seconds = anim_seconds as f64 / frame_count as f64;

    let mut parabolas = ParabolaList::new();
    for _ in 0..parabola_count {
        parabolas.add_parabola(rng.gen::<f64>() * width as f64, rng.gen::<f64>() * height as f64);
    }

    let mut frames = vec![];

    // let directrix_y = height as f64 * 0.75;

    // parabolas.debug_contains(directrix_y);

    for frame_index in 0..frame_count {
        let directrix_y = (height as f64 / frame_count as f64) * (frame_index + 1) as f64;
        let mut shapes = vec![];
        // Add the directrix as a line shape.
        shapes.push(Shape::line_fast(0.0, directrix_y, display_width, directrix_y, DIRECTRIX_THICKNESS, COLOR_DIRECTRIX));
        parabolas.add_to_shapes(&mut shapes, width, height, directrix_y);
        let frame = Frame::new(shapes, frame_seconds);
        frames.push(frame);
    }

    let additive = false;
    let back_color = Color1::white();
    let colors = vec![Color1::white(), Color1::black(), Color1::black(), Color1::light_gray(), Color1::blue(), Color1::red()];
    Renderer::display_additive_with_colors("Parabolas", display_width, display_height, back_color, frames, additive, colors);
}
