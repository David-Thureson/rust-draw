// Based on https://gamedevelopment.tutsplus.com/tutorials/generate-random-cave-levels-using-cellular-automata--gamedev-9664
// The B3S1234 and similar algorithms are from https://en.wikipedia.org/wiki/Maze_generation_algorithm
// Similar approaches here: http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
// Orthogonal town generation: https://pvigier.github.io/2020/03/15/vagabond-city-generation.html

use rand::{Rng, thread_rng, RngCore};

use crate::grid::Grid;
use crate::algorithms::group_color::GroupColor;
use crate::Color1;
use crate::renderer_3::Renderer;

const CELL_CLOSED: usize = 0;
const CELL_OPEN: usize = 1;

pub fn main() {
    run();
}

enum Algorithm {
    Original,
    B3S1234,
    B3S12345,
    GameOfLife,
}

impl Algorithm {
    pub fn live_min(&self) -> usize {
        match self {
            Self::B3S1234 | Self::B3S12345 => 1,
            Self::GameOfLife => 2,
            _ => panic!(),
        }
    }
    pub fn live_max(&self) -> usize {
        match self {
            Self::B3S1234 => 4,
            Self::B3S12345 => 5,
            Self::GameOfLife => 3,
            _ => panic!(),
        }
    }
    pub fn born(&self) -> usize {
        match self {
            Self::B3S1234 | Self::B3S12345 | Self::GameOfLife => 3,
            _ => panic!(),
        }
    }
}

fn run() {
    let mut rng = thread_rng();
    let (death_limit, birth_limit) = (3, 4);
    // let (width, height, steps, animation_seconds) = (400, 200, 4, 30);
    // let algorithm = Algorithm::B3S12345;
    let algorithm = Algorithm::GameOfLife;
    // let (width, height, steps, animation_seconds) = (800, 400, 20, 30);
    //let (width, height, steps, animation_seconds) = (400, 200, 30, 60);
    let (width, height, steps, animation_seconds) = (400, 200, 600, 120);
    // let (width, height, steps, animation_seconds) = (40, 20, 120, 30);
    // let initial_chance = 0.4;
    let initial_chance = 0.5;
    let frame_seconds = animation_seconds as f64 / steps as f64;
    let display_width_mult = if height >= 800 {
        1.0
    } else {
        (800.0 / height as f64).floor()
    };
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;

    let mut grid = Grid::new(width, height, CELL_CLOSED);
    for y in 0..height {
        for x in 0..width {
            if rng.gen_range(0.0..1.0) < initial_chance {
                grid.set_xy(x, y, CELL_OPEN);
            }
        }
    }

    /*
    grid.set_xy(0, 0, CELL_OPEN);
    grid.set_xy(1, 1, CELL_OPEN);
    grid.set_xy(2, 1, CELL_OPEN);
    grid.set_xy(0, 2, CELL_OPEN);
    grid.set_xy(1, 2, CELL_OPEN);
    */

    /*
    grid.set_xy(1, 0, CELL_OPEN);
    grid.set_xy(1, 1, CELL_OPEN);
    grid.set_xy(1, 2, CELL_OPEN);
    */

    let mut frames = vec![];
    let frame = grid.as_frame_color_index(display_width, display_height, frame_seconds);
    frames.push(frame);

    for _ in 0..steps {
        let mut new_grid = grid.clone();
        // Count open/alive neighbors.
        for y in 0..height {
            for x in 0..width {
                let count = count_open_neighbors(&grid, x, y);
                match algorithm {
                    Algorithm::Original => {
                        match grid.get_xy(x, y) {
                            CELL_CLOSED => {
                                if count > birth_limit {
                                    new_grid.set_xy(x, y, CELL_OPEN);
                                }
                            },
                            CELL_OPEN => {
                                if count < death_limit {
                                    new_grid.set_xy(x, y, CELL_CLOSED);
                                }
                            },
                            _ => panic!(),
                        }
                    },
                    Algorithm::B3S1234 | Algorithm::B3S12345 | Algorithm::GameOfLife => {
                        match grid.get_xy(x, y) {
                            CELL_CLOSED => {
                                if count == algorithm.born() {
                                    new_grid.set_xy(x, y, CELL_OPEN);
                                }
                            },
                            CELL_OPEN => {
                                if count < algorithm.live_min() || count > algorithm.live_max() {
                                    new_grid.set_xy(x, y, CELL_CLOSED);
                                }
                            },
                            _ => panic!(),
                        }
                    },
                }
            }
        }
        std::mem::replace(&mut grid, new_grid);
        let frame = grid.as_frame_color_index(display_width, display_height, frame_seconds);
        frames.push(frame);
    }

    let additive = false;
    let back_color = Color1::black();
    Renderer::display_additive_with_colors("Cave Cell", display_width, display_height, back_color, frames, additive, vec![Color1::black(), Color1::white()]);
}

fn count_open_neighbors(grid: &Grid<usize>, x: usize, y: usize) -> usize {
    // grid.neighbor_values(x, y).iter()
    grid.neighbor_values_moore_toroidal(x, y).iter()
        .map(|x| if *x == CELL_OPEN { 1 } else { 0 }).sum()
}
