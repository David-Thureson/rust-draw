use crate::*;
use itertools::Itertools;

pub fn main() {
    first();
}

enum Direction {
    Up,
    Left,
    Down,
    Right,
}

struct Grid<T>
    where T: Clone
{
    width: usize,
    height: usize,
    cell_values: Vec<Vec<T>>,
    events: Vec<GridEvent<T>>,
}

struct GridEvent<T>
    where T: Clone
{
    cells: Vec<GridEventCell<T>>,
}

struct GridEventCell<T>
    where T: Clone
{
    x: usize,
    y: usize,
    value: T,
}

impl <T> Grid<T>
    where T: Clone
{
    pub fn new(width: usize, height: usize, default_value: T) -> Self {
        let mut cell_values = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(default_value.clone());
            }
            cell_values.push(row);
        }
        Self {
            width,
            height,
            cell_values,
            events: vec![],
        }
    }

    pub fn new_from<U>(source_grid: &Grid<U>, value_func: fn(&U) -> T) -> Self
        where U: Clone
    {
        let mut grid = Self {
            width: source_grid.width,
            height: source_grid.height,
            cell_values: vec![],
            events: vec![],
        };
        for source_row in source_grid.cell_values.iter() {
            let row = source_row.iter().map(|x| value_func(x)).collect::<Vec<T>>();
            grid.cell_values.push(row);
        }
        for source_event in source_grid.events.iter() {
            let mut event: GridEvent<T> = GridEvent::new();
            for source_event_cell in source_event.cells.iter() {
                let event_cell = GridEventCell::new(source_event_cell.x, source_event_cell.y, value_func(&source_event_cell.value));
                event.cells.push(event_cell);
            }
            grid.events.push(event);
        }
        grid
    }

    pub fn add_event(&mut self, event: GridEvent<T>) {
        for event_cell in event.cells.iter() {
            self.cell_values[event_cell.y][event_cell.x] = event_cell.value.clone();
        }
    }

    pub fn get(&self, x: usize, y: usize) -> T {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        self.cell_values[y][x].clone()
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

    pub fn set_cell(&mut self, x: usize, y: usize, value: T) {
        self.cells.push(GridEventCell::new(x, y, value));
    }

    // pub fn set_rect(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: Color1) {

}

impl <T> GridEventCell<T>
    where T: Clone
{
    pub fn new(x: usize, y: usize, value: T) -> Self {
        Self {
            x,
            y,
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
    grid: Grid<usize>,
}

impl Carpet {
    pub fn new(size: usize, min_length: usize, mult: f32) -> Self {
        Self {
            size,
            min_length,
            mult,
            grid: Grid::new(size, size, 0),
        }
    }

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
}

fn first() {
    let mut carpet = Carpet::new(30, 2, 0.7);
    carpet.go();
    let char_grid = Grid::new_from(&carpet.grid, count_to_char);
    char_grid.print("A");
}

fn count_to_char(count: &usize) -> char {
    match count {
        0 => '\'',
        // 1..9 => count.to_string().chars()[0],
        // 10..35 => char::
        1..=36 => char::from_digit(*count as u32, 36).unwrap(),
        _ => '#',
    }
}
