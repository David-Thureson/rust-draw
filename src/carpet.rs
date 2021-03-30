use std::time::Instant;

use crate::*;
use renderer_3::*;
use crate::grid::*;
use util::format;

pub fn main() {
    // first();
    draw_one(200, 2.0, 7, 0.68);
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

struct Carpet {
    size: usize,
    min_length: usize,
    mult: f32,
    record_events: bool,
    grid: Grid<usize>,
    count_square: usize,
    count_side: usize,
    count_touch_rect: usize,
}

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
    pub fn new(size: usize, min_length: usize, mult: f32, record_events: bool) -> Self {
        let mut grid = Grid::new(size, size, 0);
        grid.record_events = record_events;
        Self {
            size,
            min_length,
            mult,
            record_events,
            grid,
            count_square: 0,
            count_side: 0,
            count_touch_rect: 0,
        }
    }

    pub fn go(&mut self) {
        // Algorithm: Draw a square around the edges of the carpet. Drawing a square means drawing
        // each side going counter-clockwise. Drawing a side means doing the side itself and then
        // drawing a smaller square starting at the endpoint.

        // Start at the top left and draw a square, first going down across the left edge.
        let coord = GridCoord::new(0, 0);
        let direction = Direction::Down;
        let length = self.size as f32;
        self.square(coord, direction, length);
    }

    fn square(&mut self, mut coord: GridCoord, mut direction: Direction, length: f32) {
        self.count_square += 1;
        debug_assert!(self.grid.coord_is_in_grid(coord));
        for _ in 0..4 {
            coord = self.side(coord,direction, length);
            direction = direction.ccw();
        }
    }

    fn side(&mut self, coord1: GridCoord, direction: Direction, length: f32) -> GridCoord {
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
            self.square(coord2, direction.ccw(), next_length);
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

fn draw_one(size: usize, display_width_mult: f64, min_length: usize, mult: f32) {
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, record_events);

    let start_time = Instant::now();
    carpet.go();
    dbg!(Instant::now() - start_time);
    /*
    println!("create grid seconds = {}, count_square = {}, count_side = {}, count_touch_rect = {}",
             (Instant::now() - start_time).as_secs(),
             format::format_count(carpet.count_square),
             format::format_count(carpet.count_side),
             format::format_count(carpet.count_touch_rect));
    */
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
    let frames = carpet.grid.to_final_frame(display_width, display_height, frame_seconds, &|count| count_to_color_black_white(count));
    println!("create frames seconds = {}", (Instant::now() - start_time).as_secs());

    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

fn first() {
    let size: usize = 800;
    let display_width_mult = 1.0;
    let min_length = 5;
    let mult = 0.68;
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, record_events);

    let start_time = Instant::now();
    carpet.go();
    println!("create grid seconds = {}, count_square = {}, count_side = {}, count_touch_rect = {}",
        (Instant::now() - start_time).as_secs(),
        format::format_count(carpet.count_square),
        format::format_count(carpet.count_side),
        format::format_count(carpet.count_touch_rect));

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
fn count_to_char(count: &usize) -> char {
    //bg!(*count, *count as u32);
    match *count {
        0 => '\'',
        // 1..9 => count.to_string().chars()[0],
        // 10..35 => char::
        1..=35 => char::from_digit(*count as u32, 36).unwrap(),
        _ => '#',
    }
}

#[allow(dead_code)]
fn count_to_char_black_white(count: &usize) -> char {
    if count % 2 == 0 {
        '░'
    } else {
        '▓'
    }
}

#[allow(dead_code)]
fn count_to_color_black_white(count: &usize) -> Color1 {
    if count % 2 == 0 {
        Color1::black()
    } else {
        Color1::white()
    }
}

#[allow(dead_code)]
fn count_to_color_gray(count: &usize, min: usize, max: usize) -> Color1 {
    // Normalize the count to be within the range 0..1.
    let level = ((count - min) as f32 / (max - min) as f32);
    //rintln!("count = {}, min = {}, max = {}, level = {}", count, min, max, level);
    debug_assert!(level <= 255.0);
    // Color1::from_rgb(level, level, level)
    match(count % 2) {
        0 => Color1::from_rgb(level, 0.0, 0.0),
        //1 => Color1::from_rgb(0.0, level, 0.0),
        1 => Color1::from_rgb(0.0, 0.0, level),
        _ => panic!(),
    }
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
