use itertools::Itertools;
use rand::{Rng, thread_rng};

use crate::*;
use crate::renderer_3::Renderer;
use crate::carpet::carpet::count_to_color_black_white;
use std::fs;

pub type GridCoord = Point<usize>;

#[derive(Clone)]
pub struct Grid<T>
    where T: Clone + Sized
{
    pub width: usize,
    pub height: usize,
    pub default_value: T,
    pub cell_values: Vec<Vec<T>>,
    pub events: Vec<GridEvent<T>>,
    pub record_events: bool,
}

#[derive(Clone)]
pub struct GridEvent<T>
    where T: Clone
{
    cells: Vec<GridEventCell<T>>,
}

#[derive(Clone)]
pub struct GridEventCell<T>
    where T: Clone
{
    coord: GridCoord,
    value: T,
}

pub struct GridRectangle {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
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
                let event_cell = GridEventCell::new(source_event_cell.coord, value_func(&source_event_cell.value));
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

    pub fn set_xy(&mut self, x: usize, y: usize, value: T) {
        debug_assert!(!self.record_events);
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        self.cell_values[y][x] = value;
    }

    pub fn set_coord(&mut self, coord: GridCoord, value: T) {
        debug_assert!(!self.record_events);
        self.cell_values[coord.y][coord.x] = value;
    }

    pub fn coord_is_in_grid(&self, coord: GridCoord) -> bool {
        coord.x < self.width && coord.y < self.height
    }

    // pub fn events_to_frames(&self, _frame_count: usize, display_width: f64, display_height: f64, frame_seconds: f64, value_func: fn(&T) -> Color1) -> Vec<Frame> {
    pub fn events_to_frames<F>(&self, _frame_count: usize, display_width: f64, display_height: f64, frame_seconds: f64, value_func: &F) -> Vec<Frame>
        where F: Fn(&T) -> Color1
    {
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

    pub fn to_final_frame<F>(&self, display_width: f64, display_height: f64, frame_seconds: f64, value_func: &F) -> Vec<Frame>
        where F: Fn(&T) -> Color1
    {
        vec![self.as_frame(display_width, display_height, frame_seconds, value_func)]
    }

    pub fn as_frame<F>(&self, display_width: f64, display_height: f64, frame_seconds: f64, value_func: &F) -> Frame
        where F: Fn(&T) -> Color1
    {
        let block_width = display_width / self.width as f64;
        let block_height = display_height / self.height as f64;
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

    pub fn display<F>(&self, title: &str, block_size: usize, back_color: Color1, value_func: &F)
        where F: Fn(&T) -> Color1
    {
        let display_width = (self.width * block_size) as f64;
        let display_height = (self.height * block_size) as f64;
        let additive = false;
        let frames = self.to_final_frame(display_width, display_height, 0.1, value_func);
        Renderer::display_additive(title, display_width, display_height, back_color, frames, additive);
    }

    #[inline(always)]
    fn point_in_wedge(&self, x: usize, y: usize) -> bool {
        debug_assert!(self.width == self.height, "This function is only for square grids, yet width = {} and height = {}", self.width, self.height);
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        debug_assert!(x < self.width, "x = {} and width = {}", x, self.width);
        debug_assert!(y < self.height, "y = {} and height = {}", x, self.height);
        y < self.height / 2 && y >= x
    }

    #[inline(always)]
    pub fn rectangle_inside_wedge_xy(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        debug_assert!(self.width == self.height, "This function is only for square grids, yet width = {} and height = {}", self.width, self.height);
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        debug_assert!(x2 < self.width, "x2 = {} and width = {}", x2, self.width);
        debug_assert!(y2 < self.height, "y2 = {} and height = {}", y2, self.height);
        debug_assert!(x1 <= x2, "x2 = {}, less than x1 = {} so this is not a proper rectangle.", x2, x1);
        debug_assert!(y1 <= y2, "y2 = {}, less than y1 = {} so this is not a proper rectangle.", y2, y1);
        self.point_in_wedge(x1, y1) && self.point_in_wedge(x2, y1) && self.point_in_wedge(x2, y2) && self.point_in_wedge(x1, y2)
    }

    #[inline(always)]
    pub fn rectangle_intersects_wedge_xy(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        debug_assert!(self.width == self.height, "This function is only for square grids, yet width = {} and height = {}", self.width, self.height);
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        debug_assert!(x2 < self.width, "x2 = {} and width = {}", x2, self.width);
        debug_assert!(y2 < self.height, "y2 = {} and height = {}", y2, self.height);
        debug_assert!(x1 <= x2, "x2 = {}, less than x1 = {} so this is not a proper rectangle.", x2, x1);
        debug_assert!(y1 <= y2, "y2 = {}, less than y1 = {} so this is not a proper rectangle.", y2, y1);
        let half = self.height / 2;
        x1 < half && y1 < half && y2 >= x1
    }

    #[inline(always)]
    pub fn rectangle_inside_wedge(&self, rectangle: &GridRectangle) -> bool {
        self.rectangle_inside_wedge_xy(rectangle.x1, rectangle.y1, rectangle.x2, rectangle.y2)
    }

    #[inline(always)]
    pub fn rectangle_intersects_wedge(&self, rectangle: &GridRectangle) -> bool {
        self.rectangle_intersects_wedge_xy(rectangle.x1, rectangle.y1, rectangle.x2, rectangle.y2)
    }

    pub fn complete_from_wedge(&mut self) {
        self.reflect_copy_wedge();
        self.reflect_copy_top_left_quarter();
        self.reflect_copy_top_half();
    }

    pub fn reflect_copy_wedge(&mut self) {
        debug_assert!(self.width == self.height, "This function is only for square grids, yet width = {} and height = {}", self.width, self.height);
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        let half = self.height / 2;
        for y in 1..half {
            for x in 0..y {
                self.set_xy(y, x, self.get_xy(x, y));
            }
        }
    }

    pub fn reflect_copy_top_left_quarter(&mut self) {
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        let half_width = self.width / 2;
        let x_last = self.width - 1;
        for y in 1..self.height / 2 {
            for x in 0..half_width {
                self.set_xy(x_last - x, y, self.get_xy(x, y));
            }
        }
    }

    pub fn reflect_copy_top_half(&mut self) {
        debug_assert_eq!(0, self.width % 2, "This function is only for grids with an even width, yet width = {}", self.width);
        debug_assert_eq!(0, self.height % 2, "This function is only for grids with an even height, yet height = {}", self.height);
        let y_last = self.height - 1;
        for y in 1..self.height / 2 {
            for x in 0..self.width {
                self.set_xy(x, y_last - y, self.get_xy(x, y));
                //self.cell_values[y_last - y][x] = self.cell_values[y][x].clone();
            }
        }
    }

    #[inline(always)]
    fn contains_rectangle(&self, rectangle: &GridRectangle) -> bool {
        rectangle.x2 < self.width && rectangle.y2 < self.height
    }

    pub fn outline_rectangle(&mut self, rectangle: &GridRectangle, value: &T) {
        debug_assert!(self.contains_rectangle(rectangle));
        // Top and bottom edges.
        for x in rectangle.x1..=rectangle.x2 {
            self.set_xy(x, rectangle.y1, value.clone());
            self.set_xy(x, rectangle.y2, value.clone());
        }
        // Left and right edges.
        for y in rectangle.y1 + 1..rectangle.y2 {
            self.set_xy(rectangle.x1, y,value.clone());
            self.set_xy(rectangle.x2, y,value.clone());
        }
    }

    pub fn random_rectangle(&self) -> GridRectangle {
        let mut rng = thread_rng();

        let mut x1 = rng.gen_range(0..self.width);
        let mut y1 = rng.gen_range(0..self.height);
        let mut x2 = rng.gen_range(0..self.width);
        let mut y2 = rng.gen_range(0..self.height);
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }
        let rectangle = GridRectangle::new(x1, y1, x2, y2);
        debug_assert!(self.contains_rectangle(&rectangle));
        rectangle
    }

    pub fn random_rectangle_limit(&self, max_width: usize, max_height: usize) -> GridRectangle {
        let mut rng = thread_rng();

        let x1 = rng.gen_range(0..self.width);
        let y1 = rng.gen_range(0..self.height);
        let x_add_max = ((self.width - x1) -1).min(max_width - 1);
        let y_add_max = ((self.height - y1) -1).min(max_height - 1);
        let x2 = x1 + rng.gen_range(0..=x_add_max);
        let y2 = y1 + rng.gen_range(0..=y_add_max);
        let rectangle = GridRectangle::new(x1, y1, x2, y2);
        debug_assert!(self.contains_rectangle(&rectangle));
        rectangle
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

impl Grid<usize> {
    pub fn min_max(&self) -> (usize, usize) {
        let mut min = usize::MAX;
        let mut max = usize::MIN;
        for row in self.cell_values.iter() {
            for value in row.iter() {
                min = min.min(*value);
                max = max.max(*value);
            }
        }
        (min, max)
    }

    pub fn as_frame_color_index(&self, display_width: f64, display_height: f64, frame_seconds: f64) -> Frame {
        let block_width = display_width / self.width as f64;
        let block_height = display_height / self.height as f64;
        let mut shapes = vec![];
        let mut block_x = 0.0;
        let mut block_y = 0.0;
        for y in 0..self.height {
            for x in 0..self.width {
                shapes.push(Shape::rectangle_fast(block_x, block_y, block_width, block_height, self.get_xy(x, y)));
                block_x += block_width;
            }
            block_y += block_height;
            block_x = 0.0;
        }
        let frame = Frame::new(shapes, frame_seconds);
        frame
    }

}

impl <T> PartialEq for Grid<T>
    where T: Clone + PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
        && self.height == other.height
        && self.cell_values == other.cell_values
    }
}

impl <T> Eq for Grid<T>
    where T: Clone + PartialEq
{
}

impl Grid<usize> {
    pub fn write(&self, full_file_name: &str) {
        // let start_time = Instant::now();
        let content = format!("{}\n{}\n{}", self.width, self.height,
            self.cell_values.iter()
                .map(|row| row.iter().join("\t"))
                .join("\n"));
        fs::write(full_file_name, content).unwrap();
        //rintln!("Grid::write({}): {:?}", full_file_name, Instant::now() - start_time);
    }

    pub fn read_optional(full_file_name: &str) -> Option<Grid<usize>> {
        // let start_time = Instant::now();
        let read_result = fs::read_to_string(full_file_name);
        match read_result {
            Ok(content) => {
                //rintln!("Grid::read_optional({}): read file: {:?}", full_file_name, Instant::now() - start_time);
                // let start_time = Instant::now();
                let lines = content.split("\n").collect::<Vec<_>>();
                let width = lines[0].parse::<usize>().unwrap();
                let height= lines[1].parse::<usize>().unwrap();
                let values = lines[2..].iter()
                    .map(|line| line.split("\t").map(|value| value.parse::<usize>().unwrap()).collect::<Vec<_>>())
                    .collect::<Vec<_>>();
                let mut grid = Grid::new(width, height, 0);
                grid.cell_values = values;
                //rintln!("Grid::read_optional({}): build grid: {:?}", full_file_name, Instant::now() - start_time);
                Some(grid)
            },
            Err(_) => {
                //rintln!("Grid::read_optional({}): not found.", full_file_name);
                None
            },
        }
    }
}

impl<T> GridEvent<T>
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

impl GridRectangle {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        debug_assert!(x1 <= x2, "x1 = {}, greater than x2 = {}; not a proper rectangle.", x1, x2);
        debug_assert!(y1 <= y2, "y1 = {}, greater than y2 = {}; not a proper rectangle.", y1, y2);
        Self {
            x1,
            y1,
            x2,
            y2,
        }
    }
}

pub fn main() {
    // test_point_in_wedge();
    // test_rectangle_intersects_wedge();
    // test_reflect_copy();
    // test_reflect_copy_non_square();
    // test_compare();
}

#[allow(dead_code)]
fn test_point_in_wedge() {
    //let size = 20;
    //let block_size = 20;
    let size = 200;
    let block_size = 4;
    let back_color = count_to_color_black_white(&0);

    let mut grid = Grid::new(size, size, 0);
    grid.record_events = false;
    for y in 0..size {
        for x in 0..size {
            if grid.point_in_wedge(x, y) {
                grid.set_xy(x, y, 1);
            }
        }
    }
    grid.display("test_point_in_wedge()", block_size, back_color,&|count| count_to_color_black_white(count));

    /*
    let frames = grid.to_final_frame(display_size, display_size, frame_seconds, &|count| count_to_color_black_white(count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_size, display_size, back_color, frames, additive);
     */
}

#[allow(dead_code)]
fn test_rectangle_intersects_wedge() {
    // let size = 20;
    // let rectangle_max_size = 4;
    // let block_size = 20;

    let size = 1000;
    let rectangle_max_size = 200;
    let block_size = 1;

    let rectangle_count = 200;
    let back_color = Color1::white();

    let mut grid = Grid::new(size, size, Color1::white());
    grid.record_events = false;

    for _ in 0..rectangle_count {
        // let rectangle = grid.random_rectangle();
        let rectangle = grid.random_rectangle_limit(rectangle_max_size, rectangle_max_size);
        let color = if grid.rectangle_intersects_wedge(&rectangle) { Color1::random(1.0) } else { Color1::random(0.2) };
        grid.outline_rectangle(&rectangle, &color);
    }
    grid.display("test_rectangle_intersects_wedge()", block_size, back_color,&|value| *value);

    /*
    let frames = grid.to_final_frame(display_size, display_size, frame_seconds, &|count| count_to_color_black_white(count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_size, display_size, back_color, frames, additive);
     */
}

#[allow(dead_code)]
fn test_reflect_copy() {
    // let size = 20;
    // let rectangle_max_size = 4;
    // let block_size = 20;

    let size = 1000;
    let rectangle_max_size = 200;
    let block_size = 1;

    let rectangle_count = 200;
    let back_color = Color1::white();

    let mut grid = Grid::new(size, size, Color1::white());
    grid.record_events = false;

    print_elapsed_time("fill_grid_with_shapes", || fill_grid_with_shapes(&mut grid, rectangle_count, rectangle_max_size, rectangle_max_size));
    print_elapsed_time("fill_grid_non_wedge", || fill_grid_non_wedge(&mut grid, &Color1::black()));
    print_elapsed_time("reflect_copy_wedge", || grid.reflect_copy_wedge());
    print_elapsed_time("reflect_copy_top_left_quarter", || grid.reflect_copy_top_left_quarter());
    print_elapsed_time("reflect_copy_top_half", || grid.reflect_copy_top_half());

    grid.display("test_reflect_copy()", block_size, back_color,&|value| *value);
}

#[allow(dead_code)]
fn test_reflect_copy_non_square() {
    let width = 2_000;
    let height = 1_000;
    let rectangle_max_size = 200;
    let block_size = 1;

    let rectangle_count = 200;
    let back_color = Color1::white();

    let mut grid = Grid::new(width, height, Color1::white());
    grid.record_events = false;

    print_elapsed_time("fill_grid_with_shapes", || fill_grid_with_shapes(&mut grid, rectangle_count, rectangle_max_size, rectangle_max_size));
    print_elapsed_time("reflect_copy_top_left_quarter", || grid.reflect_copy_top_left_quarter());
    print_elapsed_time("reflect_copy_top_half", || grid.reflect_copy_top_half());

    grid.display("test_reflect_copy_non_square()", block_size, back_color,&|value| *value);
}

#[allow(dead_code)]

#[allow(dead_code)]
fn fill_grid_with_shapes(grid: &mut Grid<Color1>, rectangle_count: usize, rectangle_max_width: usize, rectangle_max_height: usize) {
    for _ in 0..rectangle_count {
        let rectangle = grid.random_rectangle_limit(rectangle_max_width, rectangle_max_height);
        grid.outline_rectangle(&rectangle, &Color1::random(0.5));
    }
}

#[allow(dead_code)]
fn fill_grid_non_wedge(grid: &mut Grid<Color1>, value: &Color1) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if !grid.point_in_wedge(x, y) {
                grid.set_xy(x, y, *value);
            }
        }
    }
}

