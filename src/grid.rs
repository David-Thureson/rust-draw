use itertools::Itertools;

use crate::*;

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
        let block_width = display_width / self.width as f64;
        let block_height = display_height / self.height as f64;
        vec![self.as_frame(block_width, block_height, frame_seconds, value_func)]
    }

    fn as_frame<F>(&self, block_width: f64, block_height: f64, frame_seconds: f64, value_func: &F) -> Frame
        where F: Fn(&T) -> Color1
    {
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

