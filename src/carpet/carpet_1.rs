use crate::*;

pub fn main() {
    first();
}

struct Grid {
    width: usize,
    height: usize,
    back_color: Color1,
    cells: Vec<Vec<Color1>>,
    events: Vec<GridEvent>,
}

struct GridEvent {
    cells: Vec<GridEventCell>,
}

struct GridEventCell {
    x: usize,
    y: usize,
    color: Color1,
}

impl Grid {
    pub fn new(width: usize, height: usize, back_color: Color1) -> Self {
        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(back_color);
            }
            cells.push(row);
        }
        Self {
            width,
            height,
            back_color,
            cells,
            events: vec![],
        }
    }

    pub fn add_event(&mut self, event: GridEvent) {
        for event_cell in event.cells.iter() {
            self.cells[event_cell.y][event_cell.x] = event_cell.color;
        }
    }
}

impl GridEvent {
    pub fn new() -> Self {
        Self {
            cells: vec![],
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, color: Color1) {
        self.cells.push(GridEventCell(x, y, color));
    }

    // pub fn set_rect(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: Color1) {

}

impl GridEventCell {
    pub fn new(x: usize, y: usize, color: Color1) -> Self {
        Self {
            x,
            y,
            color,
        }
    }
}



fn first() {

}

