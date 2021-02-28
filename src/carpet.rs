use itertools::Itertools;

use crate::*;
use renderer_3::*;

pub fn main() {
    first();
}

type GridCoord = Point<usize>;

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Clone)]
struct Grid<T>
    where T: Clone + Sized
{
    width: usize,
    height: usize,
    default_value: T,
    cell_values: Vec<Vec<T>>,
    events: Vec<GridEvent<T>>,
    record_events: bool,
}

#[derive(Clone)]
struct GridEvent<T>
    where T: Clone
{
    cells: Vec<GridEventCell<T>>,
}

#[derive(Clone)]
struct GridEventCell<T>
    where T: Clone
{
    coord: GridCoord,
    value: T,
}

impl <T> Grid<T>
    where T: Clone
{
    pub fn new(width: usize, height: usize, default_value: T) -> Self {
        let mut grid = Self {
            width,
            height,
            default_value,
            cell_values: vec![],
            events: vec![],
            record_events: true,
        };
        grid.create_cells();
        grid
    }

    fn create_cells(&mut self) {
        self.cell_values = Vec::with_capacity(self.height);
        for _ in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            for _ in 0..self.width {
                row.push(self.default_value.clone());
            }
            self.cell_values.push(row);
        }
    }

    pub fn new_from<U>(source_grid: &Grid<U>, default_value: T, value_func: fn(&U) -> T) -> Self
        where U: Clone
    {
        let mut grid = Self {
            width: source_grid.width,
            height: source_grid.height,
            default_value,
            cell_values: vec![],
            events: vec![],
            record_events: source_grid.record_events,
        };
        for source_row in source_grid.cell_values.iter() {
            let row = source_row.iter().map(|x| value_func(x)).collect::<Vec<T>>();
            grid.cell_values.push(row);
        }
        for source_event in source_grid.events.iter() {
            let mut event: GridEvent<T> = GridEvent::new();
            for source_event_cell in source_event.cells.iter() {
                let event_cell = GridEventCell::new(source_event_cell.coord,value_func(&source_event_cell.value));
                event.cells.push(event_cell);
            }
            grid.events.push(event);
        }
        grid
    }

    pub fn add_event(&mut self, event: GridEvent<T>) {
        debug_assert!(self.record_events);
        self.apply_event(&event);
        self.events.push(event);
    }

    fn apply_event(&mut self, event: &GridEvent<T>) {
        debug_assert!(self.record_events);
        for event_cell in event.cells.iter() {
            self.cell_values[event_cell.coord.y][event_cell.coord.x] = event_cell.value.clone();
        }
    }

    pub fn get_xy(&self, x: usize, y: usize) -> T {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        self.cell_values[y][x].clone()
    }

    pub fn get_coord(&self, coord: GridCoord) -> T {
        self.get_xy(coord.x, coord.y)
    }

    pub fn set_coord(&mut self, coord: GridCoord, value: T) {
        debug_assert!(!self.record_events);
        self.cell_values[coord.y][coord.x] = value;
    }

    pub fn coord_is_in_grid(&self, coord: GridCoord) -> bool {
        coord.x < self.width && coord.y < self.height
    }

    pub fn events_to_frames(&self, frame_count: usize, display_width: f64, display_height: f64, frame_seconds: f64, value_func: fn(&T) -> Color1) -> Vec<Frame> {
        let block_width = display_width / self.width as f64;
        let block_height = display_height / self.height as f64;
        let mut frames = vec![];
        // let mut working_grid = Grid::new(self.width, self.height, self.default_value.clone());
        //frames.push(working_grid.as_frame(block_width, block_height, frame_seconds, value_func));
        // for event in self.events.iter() {
        //    working_grid.apply_event(event);
            //frames.push(working_grid.as_frame(block_width, block_height, frame_seconds, value_func));
        //}
        frames.push(self.as_frame(block_width, block_height, frame_seconds, value_func));
        frames
    }

    fn as_frame(&self, block_width: f64, block_height: f64, frame_seconds: f64, value_func: fn(&T) -> Color1) -> Frame {
        let mut shapes = vec![];
        let mut block_x = 0.0;
        let mut block_y = 0.0;
        for y in 0..self.height {
            for x in 0..self.width {
                let top_left = PointF64::new(block_x, block_y);
                let bottom_right = PointF64::new(block_x + block_width, block_y + block_width);
                let color = value_func(&self.get_xy(x, y));
                shapes.push(Shape::rectangle(top_left, bottom_right, color));
                block_x += block_width;
            }
            block_y += block_height;
            block_x = 0.0;
        }
        let frame = Frame::new(shapes, frame_seconds);
        frame
    }

}

impl Grid<char> {
    pub fn print(&self, label: &str) {
        println!("\n{}", label);
        for row in self.cell_values.iter() {
            let line = row.iter().join("  ");
            println!("{}", line);
        }
        println!();
    }
}

impl <T> GridEvent<T>
    where T: Clone
{
    pub fn new() -> Self {
        Self {
            cells: vec![],
        }
    }

    pub fn set_cell(&mut self, coord: GridCoord, value: T) {
        self.cells.push(GridEventCell::new(coord, value));
    }

    // pub fn set_rect(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: Color1) {

}

impl <T> GridEventCell<T>
    where T: Clone
{
    pub fn new(coord: GridCoord, value: T) -> Self {
        Self {
            coord,
            value,
        }
    }
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

struct Carpet {
    size: usize,
    min_length: usize,
    mult: f32,
    record_events: bool,
    grid: Grid<usize>,
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
        debug_assert!(self.grid.coord_is_in_grid(coord));
        for _ in 0..4 {
            coord = self.side(coord,direction, length);
            direction = direction.ccw();
        }
    }

    fn side(&mut self, coord1: GridCoord, direction: Direction, length: f32) -> GridCoord {
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

fn first() {
    let size: usize = 20;
    let min_length = 6;
    let mult = 0.7;
    let record_events = false;
    let mut carpet = Carpet::new(800, 3, 0.68, record_events);
    carpet.go();
    let char_grid = Grid::new_from(&carpet.grid, count_to_char(&0), count_to_char);
    // let char_grid = Grid::new_from(&carpet.grid, count_to_char_black_white);
    // char_grid.print("A");
    // let color_grid = Grid::new_from(&carpet_grid, count_to_color_black_white);

    let frame_count = 100;
    let display_width = 800.0;
    let display_height = display_width;
    let frame_seconds = 0.1;
    let mut frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, count_to_color_black_white);
    //bg!(&frames[1]);
    //bg_frame("1", &frames[1]);
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

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

fn count_to_char_black_white(count: &usize) -> char {
    if count % 2 == 0 {
        '░'
    } else {
        '▓'
    }
}

fn count_to_color_black_white(count: &usize) -> Color1 {
    if count % 2 == 0 {
        Color1::black()
    } else {
        Color1::white()
    }
}

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
