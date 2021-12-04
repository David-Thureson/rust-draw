use std::time::{Instant, Duration};

use crate::*;
use renderer_3::*;
use crate::grid::*;
use std::sync::mpsc;
use std::thread;
use std::collections::BTreeMap;
use std::path::Path;
use bit_vec::BitVec;

const PATH_IMAGE_FILES: &str = r"C:\Graphics\Carpet";

pub fn main() {
    // first();
    // draw_one(200, 2.0, 7, 0.68, &CarpetAlgorithm::Simple);
    // test_point_in_wedge();
    // try_draw_wedge();
    // try_animation();
    // try_write_and_read_grid();
    // optimize_build_grid();
    // try_algorithms();
    // try_combine_carpets();
    // make_gallery();
    // make_gallery_other_mod();
    // make_gallery_mult_mod_xor();
    // make_gallery_mod_large();
    // make_gallery_mod_medium();
    // make_gallery_mod_4_combo_1();
    // make_gallery_mod_4_combo();
    // debug_edge_issue();
    generate_grids_parallel();
}

#[derive(Clone)]
pub enum CarpetAlgorithm {
    Simple,
    Wedge,
    FlatSquare,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub struct Carpet {
    size: usize,
    min_length: usize,
    mult: f32,
    algorithm: CarpetAlgorithm,
    record_events: bool,
    grid: Grid<usize>,
    cells: Vec<usize>,
    count_square: usize,
    count_side: usize,
    count_touch_rect: usize,
    count_check_square_in_wedge: usize,
    time_check_square_in_wedge: Duration,
}

impl CarpetAlgorithm {
    pub fn to_name(&self) -> &str {
        match self {
            CarpetAlgorithm::Simple => "Simple",
            CarpetAlgorithm::Wedge => "Wedge",
            CarpetAlgorithm::FlatSquare => "FlatSquare",
        }
    }
}

impl PartialEq for CarpetAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.to_name() == other.to_name()
    }
}

impl Eq for CarpetAlgorithm {}

impl Direction {
    pub fn ccw(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }

    }
}

impl Carpet {
    pub fn new(size: usize, min_length: usize, mult: f32, algorithm: &CarpetAlgorithm, record_events: bool) -> Self {
        let mut grid = Grid::new(size, size, 0);
        grid.record_events = record_events;
        Self {
            size,
            min_length,
            mult,
            algorithm: algorithm.clone(),
            record_events,
            grid,
            cells: vec![],
            count_square: 0,
            count_side: 0,
            count_touch_rect: 0,
            count_check_square_in_wedge: 0,
            time_check_square_in_wedge: Duration::zero(),
        }
    }

    pub fn copy_set_from_bits(&self, bits: &BitVec) -> Self {
        let mut grid = Grid::new(self.size, self.size, 0);
        for y in 0..self.size {
            for x in 0..self.size {
                let index = x_y_to_cell_index_usize(self.size, x, y);
                grid.set_xy(x, y, true_for_white_to_count(bits.get(index).unwrap()));
            }
        }
        Self {
            size: self.size,
            min_length: self.min_length,
            mult: self.mult,
            algorithm: self.algorithm.clone(),
            record_events: false,
            grid,
            cells: vec![],
            count_square: 0,
            count_side: 0,
            count_touch_rect: 0,
            count_check_square_in_wedge: 0,
            time_check_square_in_wedge: Duration::zero(),
        }
    }

    pub fn negative(&self) -> Self {
        let mut bits = self.to_bit_vec();
        bits.negate();
        self.copy_set_from_bits(&bits)
    }

    pub fn xor(&self, other: &Carpet) -> Self {
        let bits_a = self.to_bit_vec();
        let bits_b = other.to_bit_vec();
        let mut bits = BitVec::from_elem(bits_a.len(), false);
        for i in 0..bits.len() {
            if bits_a.get(i).unwrap() != bits_b.get(i).unwrap() {
                bits.set(i, true);
            }
        }
        self.copy_set_from_bits(&bits)
    }

    pub fn go(&mut self) {
        match self.algorithm {
            CarpetAlgorithm::Simple | CarpetAlgorithm::Wedge => self.go_ccw(),
            CarpetAlgorithm::FlatSquare => self.go_flat_square(),
        }
    }

    fn go_ccw(&mut self) {
        // Algorithm: Draw a square around the edges of the carpet. Drawing a square means drawing
        // each side going counter-clockwise. Drawing a side means doing the side itself and then
        // drawing a smaller square starting at the endpoint.

        // let start_time = Instant::now();

        // Start at the top left and draw a square, first going down across the left edge.
        let coord = GridCoord::new(0, 0);
        let direction = Direction::Down;
        let length = self.size as f32;
        self.square(coord, direction, length, false);
        if self.algorithm == CarpetAlgorithm::Wedge {
            //self.grid.complete_from_wedge();
        }

        // let duration = Instant::now() - start_time;
        // let pct_in_check_wedge = self.time_check_square_in_wedge.div_duration_f64(duration);
        // rintln!("Carpet::go(): overall = {:?}; check wedge count = {}; check wedge time = {:?}; pct check wedge = {}",
        //         duration, fc(self.count_check_square_in_wedge), self.time_check_square_in_wedge, pct_in_check_wedge);
    }

    fn square(&mut self, mut coord: GridCoord, mut direction: Direction, length: f32, mut in_wedge: bool) {
        if self.algorithm == CarpetAlgorithm::Wedge {
            if !in_wedge {
                self.count_check_square_in_wedge += 1;
                let start_time = Instant::now();
                // First see if any part of the planned square falls within the wedge.
                let length_int = length.round() as usize;
                let ln = length_int - 1;
                let (x1, y1, x2, y2) = match direction {
                    Direction::Up => (coord.x - ln, coord.y - ln, coord.x, coord.y),
                    Direction::Left => (coord.x - ln, coord.y, coord.x, coord.y + ln),
                    Direction::Down => (coord.x, coord.y, coord.x + ln, coord.y + ln),
                    Direction::Right => (coord.x, coord.y - ln, coord.x + ln, coord.y),
                };
                in_wedge = self.grid.rectangle_inside_wedge_xy(x1, y1, x2, y2);
                if in_wedge {
                    self.time_check_square_in_wedge += Instant::now() - start_time;
                } else {
                    let intersects_wedge = self.grid.rectangle_intersects_wedge_xy(x1, y1, x2, y2);
                    self.time_check_square_in_wedge += Instant::now() - start_time;
                    if !intersects_wedge {
                        return
                    }
                }
            }
        }
        self.count_square += 1;
        debug_assert!(self.grid.coord_is_in_grid(coord));
        for _ in 0..4 {
            coord = self.side(coord,direction, length, in_wedge);
            direction = direction.ccw();
        }
    }

    fn side(&mut self, coord1: GridCoord, direction: Direction, length: f32, in_wedge: bool) -> GridCoord {
        self.count_side += 1;
        debug_assert!(self.grid.coord_is_in_grid(coord1));
        let length_int = length.round() as usize;
        let ln = length_int - 1;
        let (x1, y1) = (coord1.x, coord1.y);
        let (x2, y2) = match direction {
            Direction::Up    => (x1,      y1 - ln),
            Direction::Left  => (x1 - ln, y1     ),
            Direction::Down  => (x1,      y1 + ln),
            Direction::Right => (x1 + ln, y1     ),
        };
        let coord2 = GridCoord::new(x2, y2);
        debug_assert!(self.grid.coord_is_in_grid(coord2));
        if self.record_events {
            let mut event = GridEvent::new();
            self.touch_rect(&mut event, coord1, coord2);
            self.grid.add_event(event);
        } else {
            self.touch_rect_no_event(coord1, coord2);
        }

        // Draw a smaller square starting at the endpoint of the side and turning counter-clockwise.
        let next_length = length * self.mult;
        if next_length.round() >= self.min_length as f32 {
            self.square(coord2, direction.ccw(), next_length, in_wedge);
        }
        coord2
    }

    fn touch_rect(&mut self, event: &mut GridEvent<usize>, mut coord1: GridCoord, mut coord2: GridCoord) {
        debug_assert!(self.grid.coord_is_in_grid(coord1));
        debug_assert!(self.grid.coord_is_in_grid(coord2));
        Point::fix_top_left_bottom_right(&mut coord1, &mut coord2);
        for y in coord1.y..=coord2.y {
            for x in coord1.x..=coord2.x {
                let current_cell_value = self.grid.get_xy(x, y);
                event.set_cell(GridCoord::new(x, y), current_cell_value + 1);
                // event.cells.push(GridEventCell::new(GridCoord::new(x, y), current_cell_value + 1));
            }
        }
    }

    fn touch_rect_no_event(&mut self, mut coord1: GridCoord, mut coord2: GridCoord) {
        self.count_touch_rect += 1;
        debug_assert!(self.grid.coord_is_in_grid(coord1));
        debug_assert!(self.grid.coord_is_in_grid(coord2));
        Point::fix_top_left_bottom_right(&mut coord1, &mut coord2);
        for y in coord1.y..=coord2.y {
            for x in coord1.x..=coord2.x {
                let current_cell_value = self.grid.get_xy(x, y);
                self.grid.set_coord(GridCoord::new(x, y), current_cell_value + 1);
            }
        }
    }

    fn go_flat_square(&mut self) {
        // Algorithm: Draw a square around the edges of the carpet, then draw four smaller squares
        // within that. Minimize function calls.

        // let start_time = Instant::now();

        self.cells.reserve(self.size * self.size);
        for _ in 0..self.size * self.size {
            self.cells.push(0);
        }

        let mut square_sizes = vec![];
        let mut one_size = self.size as f32;
        let mut prev_rounded_size = 0;
        while one_size >= self.min_length as f32 {
            let rounded_size = one_size.round() as usize;
            if rounded_size == prev_rounded_size {
                break;
            }
            prev_rounded_size = rounded_size;
            square_sizes.push(rounded_size);
            one_size *= self.mult;
        }

        self.flat_square(0, &square_sizes, false, 0, 0, self.size - 1, self.size - 1);

        for y in 0..self.size {
            for x in 0..self.size {
                self.grid.cell_values[y][x] = self.cells[(y * self.size) + x];
            }
        }
        self.grid.complete_from_wedge();

        //rintln!("Carpet::go_flat_square(): overall = {:?}", Instant::now() - start_time);
    }

    fn flat_square(&mut self, depth: usize, square_sizes: &Vec<usize>, mut inside_wedge: bool, x1: usize, y1: usize, x2: usize, y2: usize) {
        if !inside_wedge {
            inside_wedge = self.grid.rectangle_inside_wedge_xy(x1, y1, x2, y2);
            if !inside_wedge {
                let intersects_wedge = self.grid.rectangle_intersects_wedge_xy(x1, y1, x2, y2);
                if !intersects_wedge {
                    return
                }
            }
        }
        // Top and bottom of the square.
        for x in x1..=x2 {
            self.cells[(y1 * self.size) + x] += 1;
            self.cells[(y2 * self.size) + x] += 1;
        }
        // Left and right edges of the square.
        for y in y1..=y2 {
            self.cells[(y * self.size) + x1] += 1;
            self.cells[(y * self.size) + x2] += 1;
        }

        let next_depth = depth + 1;
        if next_depth < square_sizes.len() {
            let offset = square_sizes[next_depth] - 1;
            // Smaller square at top-left of the current square.
            self.flat_square(next_depth, square_sizes, inside_wedge, x1, y1, x1 + offset, y1 + offset);
            // Smaller square at top-right of the current square.
            self.flat_square(next_depth, square_sizes, inside_wedge, x2 - offset, y1, x2, y1 + offset);
            // Smaller square at bottom-right of the current square.
            self.flat_square(next_depth, square_sizes, inside_wedge, x2 - offset, y2 - offset, x2, y2);
            // Smaller square at bottom-left of the current square.
            self.flat_square(next_depth, square_sizes, inside_wedge, x1, y2 - offset, x1 + offset, y2);
        }
    }

    fn full_file_name(size: usize, min_length: usize, mult: f32, label: Option<&str>) -> String {
        let label = label.map_or("".to_string(), |label| format!(" {}", label));
        format!("{}/carpet_{}_{}_{}{}.txt", PATH_IMAGE_FILES, size, min_length, (mult * 1_000.0) as usize, label)
    }

    pub fn write_grid(&self) {
        let full_file_name = Self::full_file_name(self.size, self.min_length, self.mult, None);
        //let start_time = Instant::now();
        self.grid.write(&full_file_name);
        //rintln!("Carpet::write_grid({}): {:?}", full_file_name, Instant::now() - start_time);
    }

    pub fn write_grid_labeled(&self, label: &str) {
        let full_file_name = Self::full_file_name(self.size, self.min_length, self.mult, Some(label));
        //let start_time = Instant::now();
        self.grid.write(&full_file_name);
        //rintln!("Carpet::write_grid({}): {:?}", full_file_name, Instant::now() - start_time);
    }

    pub fn read_grid_optional(size: usize, min_length: usize, mult: f32) -> Option<Grid<usize>> {
        let full_file_name = Self::full_file_name(size, min_length, mult, None);
        Grid::read_optional(&full_file_name)
    }

    pub fn read_or_make_grid(size: usize, min_length: usize, mult: f32) -> Grid<usize> {
        match Carpet::read_grid_optional(size, min_length, mult) {
            Some(grid) => {
                //rintln!("Carpet::read_or_make_grid({}): found", full_file_name);
                grid
            },
            None => {
                let algorithm = CarpetAlgorithm::Simple;
                //et start_time = Instant::now();
                let mut carpet = Carpet::new(size, min_length, mult, &algorithm, false);
                carpet.go();
                //rintln!("Carpet::read_or_make_grid({}): not found, created carpet: {:?}", full_file_name, Instant::now() - start_time);
                let full_file_name = Self::full_file_name(size, min_length, mult, None);
                carpet.grid.write(&full_file_name);
                carpet.grid
            }
        }
    }

    pub fn grid_exists(size: usize, min_length: usize, mult: f32) -> bool {
        let full_file_name = Carpet::full_file_name(size, min_length, mult, None);
        Path::new(&full_file_name).exists()
    }

    pub fn to_bit_vec(&self) -> BitVec {
        let mut bits = BitVec::from_elem(self.size * self.size, false);
        for y in 0..self.size {
            for x in 0..self.size {
                if count_to_true_for_white(&self.grid.get_xy(x, y)) {
                    let index = x_y_to_cell_index_usize(self.size, x, y);
                    bits.set(index, true);
                }
            }
        }
        bits
    }

    pub fn draw(&self, display_width_mult: f64) {
        // let start_time = Instant::now();
        let display_width = self.size as f64 * display_width_mult;
        let display_height = display_width;
        let frame_seconds = 0.1;
        // let start_time = Instant::now();
        // let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, count_to_color_black_white);
        // let func: FnOnce(&usize) -> Color1 = |count| count_to_color_gray(count, min, max);
        let frames = self.grid.to_final_frame(display_width, display_height, frame_seconds, &|count| count_to_color_black_white(count));
        // println!("create frames seconds = {}", (Instant::now() - start_time).as_secs());

        let back_color = count_to_color_black_white(&0);
        let additive = false;
        Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
    }

}


/*
pub fn go(&mut self) {
    let x_left = 0;
    let x_right = self.size - 1;
    let y_top = 0;
    let y_bottom = self.size - 1;
    let length = self.size as f32;
    // Left side.
    self.side(x_right, y_bottom, Direction::Up, length);
    // Top.
    self.side(x_right, y_top,    Direction::Left, length);
    // Right side.
    self.side(x_left,  y_top,    Direction::Down, length);
    // Bottom.
    self.side(x_left,  y_bottom, Direction::Right, length);
}

fn side(&mut self, mut x: usize, mut y: usize, direction: Direction, length: f32) {
    let length_int = length.round() as usize;
    let ln = length_int - 1;
    let mut event = GridEvent::new();
    for i in 0..length_int {
        debug_assert!(x < self.size);
        debug_assert!(y < self.size);
        let current_cell_value = self.grid.get(x, y);
        event.cells.push(GridEventCell::new(x, y, current_cell_value + 1));
        //bg!(i, x, y);
        if i < length_int - 1 {
            match direction {
                Direction::Up => { y -= 1; }
                Direction::Left => { x -= 1; },
                Direction::Down => { y += 1; },
                Direction::Right => { x += 1; },
            };
        }
    }
    self.grid.add_event(event);

    let next_length = length * self.mult;
    if next_length.round() >= self.min_length as f32 {
        self.side(x, y, direction.ccw(), next_length);
    }
}
*/

/*
pub fn create_image_file(size: usize, min_length: usize, mult: f32, algorithm: CarpetAlgorithm) {
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, algorithm, record_events);
    carpet.go();

}
*/

pub fn create_one(size: usize, min_length: usize, mult: f32, algorithm: &CarpetAlgorithm) -> Carpet {
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, algorithm, record_events);
    carpet.go();
    carpet
}

#[allow(dead_code)]
fn draw_one(size: usize, display_width_mult: f64, min_length: usize, mult: f32, algorithm: &CarpetAlgorithm) {
    let start_time = Instant::now();
    let carpet = create_one(size, min_length, mult, algorithm);
    println!("Create carpet: {:?}; count_square = {}, count_side = {}, count_touch_rect = {}",
             Instant::now() - start_time, fc(carpet.count_square),
             fc(carpet.count_side), fc(carpet.count_touch_rect));
    carpet.draw(display_width_mult);
}

pub fn draw_grid_normal(grid: &Grid<usize>, display_width_mult: f64) {
    grid.draw(display_width_mult, &|count| count_to_color_black_white(count))
}

#[allow(dead_code)]
fn first() {
    let size: usize = 800;
    let display_width_mult = 1.0;
    let min_length = 5;
    let mult = 0.68;
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, &CarpetAlgorithm::Simple, record_events);

    let start_time = Instant::now();
    carpet.go();
    println!("create grid seconds = {}, count_square = {}, count_side = {}, count_touch_rect = {}",
        (Instant::now() - start_time).as_secs(),
        fc(carpet.count_square),
        fc(carpet.count_side),
        fc(carpet.count_touch_rect));

    // let char_grid = Grid::new_from(&carpet.grid, count_to_char(&0), count_to_char);
    // let char_grid = Grid::new_from(&carpet.grid, count_to_char_black_white);
    // char_grid.print("A");
    // let color_grid = Grid::new_from(&carpet_grid, count_to_color_black_white);

    let (min, max) = carpet.grid.min_max();
    println!("min = {}, max = {}", min, max);

    let frame_count = 100;
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let frame_seconds = 0.1;

    let start_time = Instant::now();
    // let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, count_to_color_black_white);
    // let func: FnOnce(&usize) -> Color1 = |count| count_to_color_gray(count, min, max);
    let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, &|count| count_to_color_gray(count, min, max));
    println!("create frames seconds = {}", (Instant::now() - start_time).as_secs());

    //bg!(&frames[1]);
    //bg_frame("1", &frames[1]);
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn dbg_frame(label: &str, frame: &Frame) {
    println!("\n{}", label);
    for shape in frame.shapes.iter() {
        match shape {
            Shape::Rectangle { top_left, bottom_right: _, color} => {
                let x = top_left.x.round() as usize;
                let y = top_left.y.round() as usize;
                let color = if color.r > 0.5 { "white" } else { "black" };
                println!("{}, {}: {}", x, y, color);
            },
            _ => unimplemented!(),
        }
    }
}

#[allow(dead_code)]
fn try_draw_wedge() {
    let size = 400;
    let display_width_mult = 2.0;
    let min_length = 7;
    let mult = 0.68;
    draw_one(size, display_width_mult, min_length, mult,&CarpetAlgorithm::Simple);
    draw_one(size, display_width_mult, min_length, mult,&CarpetAlgorithm::Wedge);
}

#[allow(dead_code)]
fn try_animation() {
    // animate_mult(200, 4.0, 2.0, 7, 0.675, 0.685, 0.001)
    // animate_mult(200, 2.0, 2.0, 4, 0.67, 0.69, 0.0001)
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.65, 0.70, 0.0003)
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.60, 0.65, 0.001)
    // animate_mult_parallel(800, 2, 1.0, 1.0, 3, 0.63, 0.68, 0.001, 50);
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.5, 0.60, 0.001)
    animate_mult_parallel(400, 2, 2.0, 2.0, 3, 0.65, 0.8, 0.001, 50);
    // animate_show_existing(400, 2.0, 2.0, 3, 0.7, 0.9, 0.001);
    // animate_mult_parallel(200, 5, 4.0, 1.0, 7, 0.6, 0.8, 0.002, 1_000);
    // animate_mult_parallel(200, 5, 2.0, 1.0, 7, 0.8, 0.9, 0.002, 1_000);
    // animate_show_existing(200, 5, 4.0, 1.5, 7, 0.8, 0.9, 0.002);
    // animate_mult_parallel(200, 4, 2.0, 0.75, 7, 0.53, 0.8, 0.002, 1_000);
    // animate_mult_parallel(100, 4, 4.0, 1.0, 7, 0.53, 0.9, 0.002, 1_000);
    // animate_show_existing(100, 4, 4.0, 1.0, 7, 0.53, 0.9, 0.002);
}

#[allow(dead_code)]
fn animate_mult(size: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: f32, mult_max: f32, mult_step: f32) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let mut frames = vec![];
    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }
    let mut prev_grid = None;
    let mut skipped_count = 0;
    let start_time = Instant::now();
    for mult in mults.iter() {
        let carpet = create_one(size, min_length, *mult,&CarpetAlgorithm::Wedge);
        if prev_grid.is_none() || prev_grid.unwrap() != carpet.grid {
            frames.push(carpet.grid.as_frame(display_width, display_height, frame_seconds, &|count| count_to_color_black_white(count)));
        } else {
            skipped_count += 1;
        }
        prev_grid = Some(carpet.grid.clone());
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(skipped_count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn animate_mult_parallel(size: usize, black_white_modulus: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: f32, mult_max: f32, mult_step: f32, threads_max: usize) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let start_time = Instant::now();
    let (tx, rx) = mpsc::channel();
    let mut threads = Vec::new();
    let mut frame_index = 0;

    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }

    let mut thread_count = 0;
    for mult in mults.iter() {
        let mult = *mult;
        let grid_exists = Carpet::grid_exists(size, min_length, mult);
        if !grid_exists {
            if thread_count < threads_max {
                let file_name = Carpet::full_file_name(size, min_length, mult, None);
                println!("Starting to build {}", file_name);
            }
        }
        if grid_exists || thread_count < threads_max {
            let thread_tx = tx.clone();
            let thread = thread::spawn(move || {
                // let carpet = create_one(size, min_length, mult,CarpetAlgorithm::Wedge);
                // thread_tx.send((frame_index, carpet.grid)).unwrap();
                let grid = Carpet::read_or_make_grid(size, min_length, mult);
                thread_tx.send((frame_index, grid)).unwrap();
            });
            threads.push(thread);
        }
        frame_index += 1;
        if !grid_exists {
            thread_count += 1;
            if thread_count == threads_max {
                println!("Reached max threads.");
            }
        }
    }

    // Here, all the messages are collected.
    let mut grids = BTreeMap::new();
    for i in 0..threads.len() {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        let (frame_index, grid) = rx.recv().unwrap();
        grids.insert(frame_index, grid);
        println!("frame_index = {}; remaining frames = {}", fc(frame_index), fc(mults.len() - (i + 1)));
    }

    // Wait for the threads to complete any remaining work.
    for thread in threads {
        thread.join().unwrap();
    }

    let mut frames = vec![];
    let mut prev_grid = None;
    let mut skipped_count = 0;
    for grid in grids.values() {
        if prev_grid.is_none() || prev_grid.unwrap() != *grid {
            // let (min, max) = grid.min_max();
            frames.push(grid.as_frame(display_width, display_height, frame_seconds,
                                      // &|count| count_to_color_black_white(count)));
                &|count| count_to_color_black_white_mod(count, black_white_modulus)));
            //&|count| count_to_color_gray(count, min, max)));
        } else {
            skipped_count += 1;
        }
        prev_grid = Some(grid.clone());
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(skipped_count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;

    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn animate_show_existing(size: usize, black_white_modulus: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: f32, mult_max: f32, mult_step: f32) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let start_time = Instant::now();

    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }
    let mut grids = vec![];
    for mult in mults.iter() {
        if let Some(grid) = Carpet::read_grid_optional(size, min_length, *mult) {
            grids.push(grid);
        };
    }

    let mut frames = vec![];
    for grid in grids.iter() {
        frames.push(grid.as_frame(display_width, display_height, frame_seconds,
            // &|count| count_to_color_black_white(count)));
            &|count| count_to_color_black_white_mod(count, black_white_modulus)));
            //&|count| count_to_color_gray(count, min, max)));
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(mults.len() - frames.len()));
    let back_color = count_to_color_black_white(&0);
    let additive = false;

    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn try_write_and_read_grid() {
    /*
    let carpet = create_one(400, 5, 0.68,CarpetAlgorithm::Wedge);
    let reference_grid = carpet.grid.clone();
    carpet.write_grid();

    let found_grid = Carpet::read_or_make_grid(400, 5, 0.68);
    assert!(reference_grid == found_grid);
    found_grid.write(&format!("{}/Test_Grid.txt", PATH_IMAGE_FILES));
     */

    Carpet::read_or_make_grid(400, 5, 0.681);
    Carpet::read_or_make_grid(400, 5, 0.681);
}

#[allow(dead_code)]
fn optimize_build_grid() {
    // Carpet::new(400, 3, 0.68, CarpetAlgorithm::Wedge, false).go();
    draw_one(400, 2.0, 7, 0.68, &CarpetAlgorithm::Wedge);
}

#[allow(dead_code)]
fn try_algorithms() {
    // draw_one(400, 2.0, 7, 0.68, CarpetAlgorithm::FlatSquare);
    // draw_one(20, 20.0, 10, 0.68, CarpetAlgorithm::FlatSquare);

    let size = 400;
    let min_length = 3;
    let mult = 0.68;
    for algorithm in [CarpetAlgorithm::Simple, CarpetAlgorithm::Wedge, CarpetAlgorithm::FlatSquare].iter() {
        let start_time = Instant::now();
        let mut carpet = Carpet::new(size, min_length, mult, algorithm, false);
        carpet.go();
        println!("{}: {:?}", algorithm.to_name(), Instant::now() - start_time);
        carpet.grid.write(&format!("T:/Compare/{}", algorithm.to_name()));
    }
}

#[allow(dead_code)]
fn try_combine_carpets() {
    // let color_func = &|count| count_to_color_black_white(count);
    let size = 300;
    let display_mult = 2.0;
    let min_length_a = 7;
    let min_length_b = 7;
    let mult_a = 0.68;
    let mult_b = 0.69;
    let algorithm = &CarpetAlgorithm::Wedge;
    let carpet_a = create_one(size, min_length_a, mult_a, algorithm);
    // carpet_a.draw(2.0);
    // carpet_a.write_grid();
    let carpet_b = create_one(size, min_length_b, mult_b, algorithm);
    // carpet_b.write_grid();
    // let bits_a = carpet_a.to_bit_vec();
    // let bits_b = carpet_b.to_bit_vec();
    // dbg!(&bits_a);
    // carpet_a.copy_set_from_bits(&bits_a).write_grid_labeled("from bits");
    // carpet_a.copy_set_from_bits(&bits_a).draw(2.0);
    //carpet_a.negative().draw(2.0);
    let combo = carpet_a.xor(&carpet_b);
    let layout_grid= Grid::arrange(3, 0, 8, &vec![carpet_a.grid, carpet_b.grid, combo.grid]);
    // draw_grid(&layout_grid, display_mult, color_func);
    draw_grid_normal(&layout_grid, display_mult);
}

#[allow(dead_code)]
fn make_gallery() {
    /*
    let size = 100;
    let margin_size = 5;
    let display_mult = 2.0;
    let min_length = 3;
    let mult_min = 0.63;
    let mult_max = 0.80;
    let carpet_count = 36;
    let col_count = 9;
     */
    /*
    let size = 200;
    let margin_size = 10;
    let display_mult = 1.0;
    let min_length = 5;
    let mult_min = 0.61;
    let mult_max = 0.75;
    let carpet_count = 36;
    let col_count = 9;
    */
    let size = 800;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 7;
    let mult_min = 0.680;
    let mult_max = 0.682;
    let carpet_count = 2;
    let col_count = 2;
    let algorithm = &CarpetAlgorithm::Wedge;
    let mult_inc = (mult_max - mult_min) / (carpet_count - 1) as f32;
    let mut grids = Vec::with_capacity(carpet_count);
    let mut mult = mult_min;
    for _ in 0..carpet_count {
        grids.push(create_one(size, min_length, mult, algorithm).grid);
        mult += mult_inc;
    }
    let layout_grid = Grid::arrange(col_count, 0, margin_size, &grids);
    draw_grid_normal(&layout_grid, display_mult);
}

#[allow(dead_code)]
fn make_gallery_other_mod() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 2.0;
    let min_length = 3;
    let mult= 0.67;
    let col_count = 4;
    let size = 200;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);

    let mut grids = vec![];
    for modulus in 2..=5 {
        grids.push(carpet.grid.copy_with_value_function(&|count| count % modulus != 0, false));
    }
    let mut negative_grids = vec![];
    for grid in grids.iter() {
        negative_grids.push(grid.copy_negative());
    }
    grids.append(&mut negative_grids);
    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mult_mod_xor() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 2.0;
    let min_length = 3;
    let mult= 0.67;
    let col_count = 4;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);

    let mut grids = vec![];
    for modulus in 2..=5 {
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus != 0, false);
        let grid = grid.copy_negative();
        grids.push(grid);
    }

    grids.push(grids[0].copy_xor(&grids[1]));
    grids.push(grids[1].copy_xor(&grids[2]));
    grids.push(grids[2].copy_xor(&grids[3]));
    grids.push(grids[3].copy_xor(&grids[0]));

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_large() {
    let size = 800;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 7;
    let mult= 0.67;
    let col_count = 2;
    let algorithm = &CarpetAlgorithm::Wedge;
    let mods = vec![4, 5];

    let carpet = create_one(size, min_length, mult, algorithm);
    let mut grids = Vec::with_capacity(mods.len());
    for modulus in mods.iter() {
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_medium() {
    let size = 400;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 4;
    let mult= 0.69;
    let col_count = 4;
    let grid_count = 8;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);
    let mut grids = Vec::with_capacity(grid_count);
    for i in 0..grid_count {
        let modulus = i + 2;
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_4_combo_1() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 4;
    let mult_min= 0.67;
    let mult_max= 0.73;
    let col_count = 9;
    let grid_count = col_count * 4;
    let algorithm = &CarpetAlgorithm::Wedge;

    let mult_inc = (mult_max - mult_min) / (col_count - 1) as f32;

    let mut grids = Vec::with_capacity(grid_count);
    let mut mult = mult_min;
    for _ in 0..col_count {
        let carpet = create_one(size, min_length, mult, algorithm);
        let modulus = 4;
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
        mult += mult_inc;
    }
    for offset in 1..=3 {
        for i in 0..col_count {
            let mut second_grid_index = i + offset;
            if second_grid_index >= col_count {
                second_grid_index -= col_count;
            }
            let grid = grids[i].copy_xor(&grids[second_grid_index]);
            grids.push(grid);
        }
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_4_combo() {
    let label = "add_display_mod_4";
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 1.0;
    // let min_length = 4;
    let min_length = 3;
    let mult_min= 0.67;
    let mult_max= 0.73;
    let modulus = 4;
    let col_count = 9;
    let grid_count = col_count * 4;
    // let algorithm = &CarpetAlgorithm::Simple;

    let mult_inc = (mult_max - mult_min) / (col_count - 1) as f32;

    let mut ref_grids = Vec::with_capacity(col_count);
    let mut grids = Vec::with_capacity(grid_count);
    let mut mult = mult_min;
    for _ in 0..col_count {
        let grid = Carpet::read_or_make_grid(size, min_length, mult);
        ref_grids.push(grid.clone());
        // These are the original grids with no modulus effect and thus the full counts.
        let grid = grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
        mult += mult_inc;
    }

    // Make three more rows of derived grids. The first row combines reference grid 0 with 1, 1
    // with 2, and so on. The next row combines 0 with 2, 1 with 3, and so on.
    for offset in 1..=3 {
        for i in 0..col_count {
            let mut second_grid_index = i + offset;
            if second_grid_index >= col_count {
                second_grid_index -= col_count;
            }
            let grid = ref_grids[i]
                .copy_add(&ref_grids[second_grid_index])
                .copy_with_value_function(&|count| count % modulus == 0, false);
            grids.push(grid);
        }
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);

    let file_name = format!("{}/carpet_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, (mult_min * 1_000.0) as usize, (mult_max * 1_000.0) as usize, label);
    image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);

    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn debug_edge_issue() {
    // let label = "debug_edge_issues";
    let size = 40;
    let margin_size = 5;
    let display_mult = 10.0;
    let min_length = 4;
    let mult= 0.67;
    let modulus = 2;
    let col_count = 3;
    let grid_count = col_count + 1;

    let mut carpets = Vec::with_capacity(grid_count);
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::Simple));
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::Wedge));
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::FlatSquare));

    let mut grids = carpets.iter()
        .map(|carpet| carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false))
        .collect::<Vec<_>>();
    let xor_grid = grids[0].copy_xor(&grids[1]);
    grids.push(xor_grid);
    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);

    // let file_name = format!("{}/carpet_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, (mult_min * 1_000.0) as usize, (mult_max * 1_000.0) as usize, label);
    // image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);

    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

fn generate_grids_parallel() {
    let size = 200;
    let min_length = 5;
    let mult_min = 0.700;
    let mult_inc = 0.001;
    let grid_count = 200;

    let mut mult = mult_min;
    let mut handles = vec![];
    for _ in 0..grid_count {
        handles.push(thread::spawn(move || { Carpet::read_or_make_grid(size, min_length, mult) }));
        mult += mult_inc;
    }
    for handle in handles {
        let _ = handle.join();
    }
}