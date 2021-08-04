// Based on https://gamedevelopment.tutsplus.com/tutorials/generate-random-cave-levels-using-cellular-automata--gamedev-9664
// The B3S1234 and similar algorithms are from https://en.wikipedia.org/wiki/Maze_generation_algorithm
// Similar approaches here: http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels

use rand::{Rng, thread_rng};

use crate::grid::{Grid, GridNeighborType, GridWrappingStyle};
use crate::{Color1, Frame, cell_index_to_x_y_isize, x_y_to_cell_index_isize};
use crate::renderer_3::Renderer;

const CELL_CLOSED: usize = 0;
const CELL_OPEN: usize = 1;

pub fn main() {
    run_animation();
}

#[derive(Clone)]
enum CellAutoAlgorithm {
    Original,
    B3S1234,
    B3S12345,
    GameOfLife,
}

#[allow(dead_code)]
enum CellAutoStartFill {
    Random { pct: f64 },
    Glider { count: usize },
}

#[derive(Clone)]
struct CellAutoGrid {
    width: usize,
    height: usize,
    algorithm: CellAutoAlgorithm,
    neighbor_type: GridNeighborType,
    wrapping_style: GridWrappingStyle,
    cells: Vec<CellAutoCell>,
}

#[derive(Clone)]
struct CellAutoCell {
    open: bool,
    neighbors: Vec<usize>,
    neighbor_open_count: usize,
}

struct CellAutoShape {
    pub grid: Grid<bool>,
}

impl CellAutoAlgorithm {
    pub fn should_open(&self, neighbor_count: usize) -> bool {
        match self {
            Self::Original => neighbor_count > 4,
            Self::B3S1234 | Self::B3S12345 | Self::GameOfLife => neighbor_count == 3,
        }
    }

    pub fn should_close(&self, neighbor_count: usize) -> bool {
        match self {
            Self::Original => neighbor_count < 3,
            Self::B3S1234 => neighbor_count < 1 || neighbor_count > 4,
            Self::B3S12345 => neighbor_count < 1 || neighbor_count > 5,
            Self::GameOfLife => neighbor_count < 2 || neighbor_count > 3,
        }
    }
}

impl CellAutoGrid {
    pub fn new(width: usize, height: usize, algorithm: CellAutoAlgorithm, neighbor_type: GridNeighborType, wrapping_style: GridWrappingStyle, start_fill: CellAutoStartFill) -> Self {
        let cell_count = width * height;
        let mut cells = Vec::with_capacity(cell_count);
        for cell_index in 0..cell_count {
            let neighbors = Self::neighbors(width, height, &neighbor_type, &wrapping_style, cell_index);
            cells.push(CellAutoCell::new(neighbors));
        }
        let mut grid = Self {
            width,
            height,
            algorithm,
            neighbor_type,
            wrapping_style,
            cells,
        };
        match start_fill {
            CellAutoStartFill::Random { pct } => {
                grid.open_random(pct);
            },
            CellAutoStartFill::Glider { count } => {
                grid.add_gliders(count);
            },
        }
        grid
    }

    fn open_random(&mut self, pct: f64) {
        let mut rng = thread_rng();
        for cell_index in 0..self.cells.len() {
            if rng.gen_range(0.0..1.0) < pct {
                self.open(cell_index);
            }
        }
    }

    fn add_gliders(&mut self, count: usize) {
        self.add_shapes(vec![(0, 0), (1, 1), (2, 1), (0, 2), (1, 2)], count);
    }

    fn add_shapes(&mut self, open_cells: Vec<(usize, usize)>, count: usize) {
        let shape = CellAutoShape::new(open_cells);
        let shape_variations = shape.grid.get_all_flips_and_rotations();
        debug_assert_eq!(8, shape_variations.len());

        let mut rng = thread_rng();
        let (width, height) = (self.width as isize, self.height as isize);
        for _ in 0..count {
            let cell_index = rng.gen_range(0..self.cells.len());
            let variant_index = rng.gen_range(0..shape_variations.len());
            let (x_start, y_start) = cell_index_to_x_y_isize(width, cell_index);
            for (x_offset, y_offset) in shape_variations[variant_index].matching_cells(|cell| cell).iter() {
                let (x, y) = (x_start + *x_offset as isize, y_start + *y_offset as isize);
                let neighbor_index= Self::resolve_neighbor(width, height, &self.wrapping_style, x, y).unwrap();
                self.open(neighbor_index);
            }
        }
    }

    /*
    fn add_shapes(&mut self, shape: Vec<(usize, usize)>, count: usize) {
        let mut rng = thread_rng();
        let (width, height) = (self.width as isize, self.height as isize);
        for _ in 0..count {
            let cell_index = rng.gen_range(0..self.cells.len());
            let (x_start, y_start) = cell_index_to_x_y_isize(width, cell_index);
            for (x_offset, y_offset) in shape.iter() {
                let (x, y) = (x_start + *x_offset as isize, y_start + *y_offset as isize);
                let neighbor_index= Self::resolve_neighbor(width, height, &self.wrapping_style, x, y).unwrap();
                self.open(neighbor_index);
            }
        }
    }
    */

    fn open(&mut self, cell_index: usize) {
        if !self.cells[cell_index].open {
            self.cells[cell_index].open = true;
            for neighbor_index in self.cells[cell_index].neighbors.clone().iter() {
                self.cells[*neighbor_index].neighbor_open_count += 1;
            }
        }
    }

    fn close(&mut self, cell_index: usize) {
        if self.cells[cell_index].open {
            self.cells[cell_index].open = false;
            for neighbor_index in self.cells[cell_index].neighbors.clone().iter() {
                self.cells[*neighbor_index].neighbor_open_count -= 1;
            }
        }
    }

    #[inline]
    pub fn neighbors(width: usize, height: usize, neighbor_type: &GridNeighborType, wrapping_style: &GridWrappingStyle, cell_index: usize) -> Vec<usize> {
        let (width, height) = (width as isize, height as isize);
        let (x, y) = cell_index_to_x_y_isize(width, cell_index);
        let mut v = vec![];
        match neighbor_type {
            GridNeighborType::Moore { range } => {
                // Surrounding cells.
                let range = *range as isize;
                for y_candidate in y - range..=y + range {
                    for x_candidate in x - range..=x + range {
                        if !(y_candidate == y && x_candidate == x) {
                            if let Some(cell_index) = Self::resolve_neighbor(width, height, wrapping_style, x_candidate, y_candidate) {
                                v.push(cell_index);
                            }
                        }
                    }
                }
            },
            GridNeighborType::VonNeuman { range } => {
                // Cross.
                let range = *range as isize;
                // Vertical cells.
                for y_candidate in y - range..=y + range {
                    if y_candidate != y {
                        if let Some(cell_index) = Self::resolve_neighbor(width, height, wrapping_style, x, y_candidate) {
                            v.push(cell_index);
                        }
                    }
                }
                // Horizontal cells.
                for x_candidate in x - range..=x + range {
                    if x_candidate != x {
                        if let Some(cell_index) = Self::resolve_neighbor(width, height, wrapping_style, x_candidate, y) {
                            v.push(cell_index);
                        }
                    }
                }
            }
        }
        v
    }

    #[inline]
    fn resolve_neighbor(width: isize, height: isize, wrapping_style: &GridWrappingStyle, x: isize, y: isize) -> Option<usize> {
        let x = if x < 0 {
            match wrapping_style {
                GridWrappingStyle::Horizontal | GridWrappingStyle::Toroidal => Some(x + width),
                _ => None,
            }
        } else if x >= width {
            match wrapping_style {
                GridWrappingStyle::Horizontal | GridWrappingStyle::Toroidal => Some(x - width),
                _ => None,
            }
        } else {
            Some(x)
        };
        if x.is_none() {
            return None;
        }
        let y = if y < 0 {
            match wrapping_style {
                GridWrappingStyle::Vertical | GridWrappingStyle::Toroidal => Some(y + height),
                _ => None,
            }
        } else if y >= height {
            match wrapping_style {
                GridWrappingStyle::Vertical | GridWrappingStyle::Toroidal => Some(y - height),
                _ => None,
            }
        } else {
            Some(y)
        };
        match (x, y) {
            (Some(x), Some(y)) => Some(x_y_to_cell_index_isize(width, x, y)),
            _ => None,
        }
    }

    fn animate(&mut self, steps: usize, animation_seconds: usize) {
        let frame_seconds = animation_seconds as f64 / steps as f64;
        let display_width_mult = if self.height >= 800 {
            1.0
        } else {
            (800.0 / self.height as f64).floor()
        };
        let display_width = self.width as f64 * display_width_mult;
        let display_height = self.height as f64 * display_width_mult;

        let mut frames = vec![];
        let frame = self.as_frame_color_index(display_width, display_height, frame_seconds);
        frames.push(frame);

        for _ in 0..steps {
            self.step();
            let frame = self.as_frame_color_index(display_width, display_height, frame_seconds);
            frames.push(frame);
        }
        let additive = false;
        let back_color = Color1::black();
        Renderer::display_additive_with_colors("Cave Cell", display_width, display_height, back_color, frames, additive, vec![Color1::black(), Color1::white()]);
    }

    fn step(&mut self) {
        let old_grid = self.clone();
        for cell_index in 0..self.cells.len() {
            let open = old_grid.cells[cell_index].open;
            let open_neighbor_count = old_grid.cells[cell_index].neighbor_open_count;
            if open {
                if self.algorithm.should_close(open_neighbor_count) {
                    self.close(cell_index);
                }
            } else {
                if self.algorithm.should_open(open_neighbor_count) {
                    self.open(cell_index);
                }
            }
        }
    }

    fn as_frame_color_index(&self, display_width: f64, display_height: f64, frame_seconds: f64) -> Frame {
        let mut color_grid = Grid::new(self.width, self.height, CELL_CLOSED);
        for cell_index in 0..self.cells.len() {
            if self.cells[cell_index].open {
                color_grid.set_by_index(cell_index, CELL_OPEN)
            }
        }
        color_grid.as_frame_color_index(display_width, display_height, frame_seconds)
    }
}

impl CellAutoCell {
    pub fn new(neighbors: Vec<usize>) -> Self {
        Self {
            open: false,
            neighbors,
            neighbor_open_count: 0,
        }
    }
    /*
    pub fn new(neighbor_type: GridNeighborType) -> Self {
        let neighbor_count = match neighbor_type {
            GridNeighborType::Moore { range } => {
                let r = (range * 2) + 1;
                (r * r) - 1
            },
            GridNeighborType::VonNeuman { range } => {
                range * 4
            },
        };
        Self {
            value: CELL_CLOSED,
            neighbors: Vec::with_capacity(neighbor_count),
            neighbor_open_count: 0,
        }
    }
     */
}

impl CellAutoShape {
    pub fn new(open_cells: Vec<(usize, usize)>) -> Self {
        let width = open_cells.iter().map(|cell| cell.0).max().unwrap() + 1;
        let height = open_cells.iter().map(|cell| cell.1).max().unwrap() + 1;
        let mut grid = Grid::new(width, height, false);
        for (x, y) in open_cells.iter() {
            grid.set_xy(*x, *y, true);
        }
        Self {
            grid,
        }
    }
}

fn run_animation() {
    // Lots of gliders.
    // let mut grid = CellAutoGrid::new(400, 200, CellAutoAlgorithm::GameOfLife, GridNeighborType::Moore { range: 1 }, GridWrappingStyle::Toroidal, CellAutoStartFill::Glider { count: 500 });
    // grid.animate(120, 30);

    // Cave.
    let mut grid = CellAutoGrid::new(400, 200, CellAutoAlgorithm::Original, GridNeighborType::Moore { range: 1 }, GridWrappingStyle::None, CellAutoStartFill::Random { pct: 0.4 });
    grid.animate(12, 30);

    // Game of Life with random start.
    // let mut grid = CellAutoGrid::new(400, 200, CellAutoAlgorithm::GameOfLife, GridNeighborType::Moore { range: 1 }, GridWrappingStyle::Toroidal, CellAutoStartFill::Random { pct: 0.5 } );
    // grid.animate(120, 30);
}
