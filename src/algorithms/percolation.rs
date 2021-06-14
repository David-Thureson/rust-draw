use crate::grid::Grid;
use crate::algorithms::union_find::WeightedQuickUnion;
use rand::Rng;
use crate::renderer_3::Renderer;
use crate::color::Color1;
use util::*;
use std::time::{Instant, Duration};

const COLOR_INDEX_BLACK: usize = 0;
const COLOR_INDEX_WHITE: usize = 1;
const COLOR_INDEX_BLUE: usize = 2;
const COLOR_INDEX_RED: usize = 3;
const COLOR_INDEX_FIRST_EXTRA: usize = 4;


pub fn main() {
    // try_percolation();
    // try_animation();
    try_animation_fast();
}

pub struct PercolationGrid {
    pub width: usize,
    pub height: usize,
    pub grid: Grid<bool>,
    pub connections: WeightedQuickUnion,
}

pub enum PercolationBlockState {
    Blocked,
    Open,
    Filled,
}

impl PercolationGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = Grid::new(width, height,false);
        grid.record_events = false;
        let mut connections = WeightedQuickUnion::new((width * height) + 2, true);
        // Connect the top virtual node to every square in the top row, and the bottom virtual node
        // to every square in the bottom row.
        let top_node_index = 0;
        let bottom_node_index = Self::bottom_node_index(width, height);
        for i in 0..width {
            connections.union(top_node_index, i + 1);
            connections.union(bottom_node_index, bottom_node_index - (i + 1));
        }
        Self {
            width,
            height,
            grid,
            connections,
        }
    }

    pub fn open(&mut self, x: usize, y: usize) -> bool {
        if self.grid.get_xy(x, y) {
            // The square is already open
            return false;
        }
        self.grid.set_xy(x, y, true);
        let connection_index = self.node_index(x, y);
        // Up.
        if y > 0 && self.grid.get_xy(x, y - 1) {
            self.connections.union(connection_index, connection_index - self.width);
        }
        // Right.
        if x < self.width - 1 && self.grid.get_xy(x + 1, y)  {
            self.connections.union(connection_index, connection_index + 1);
        }
        // Down.
        if y < self.height - 1 && self.grid.get_xy(x, y + 1) {
            self.connections.union(connection_index, connection_index + self.width);
        }
        // Left.
        if x > 0 && self.grid.get_xy(x - 1, y) {
            self.connections.union(connection_index, connection_index - 1);
        }
        true
    }

    pub fn percolates(&mut self) -> bool {
        self.connections.is_connected(0, Self::bottom_node_index(self.width, self.height))
    }

    #[inline]
    pub fn bottom_node_index(width: usize, height: usize) -> usize {
        (width * height) + 1
    }

    #[inline]
    fn node_index(&self, x: usize, y: usize) -> usize {
        (y * self.width) + x + 1
    }

    pub fn print(&mut self) {
        for y in 0..self.height {
            let mut line = "".to_string();
            for x in 0..self.width {
                line.push_str(match self.block_state(x, y) {
                    PercolationBlockState::Blocked => "#",
                    PercolationBlockState::Open => ".",
                    PercolationBlockState::Filled => "%",
                });
            }
            println!("{}", line);
        }
    }

    pub fn block_state(&mut self, x: usize, y: usize) -> PercolationBlockState {
        if self.grid.get_xy(x, y) {
            if self.connections.is_connected(0, self.node_index(x, y)) {
                PercolationBlockState::Filled
            } else {
                PercolationBlockState::Open
            }
        } else {
            PercolationBlockState::Blocked
        }
    }

}

#[allow(dead_code)]
fn try_percolation() {
    let mut rng = rand::thread_rng();
    let width = 5;
    let height = 5;
    let mut perc = PercolationGrid::new(width, height);
    println!();
    perc.connections.print_components();
    perc.print();
    while !perc.percolates() {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if perc.open(x, y) {
            println!("\nOpen {}, {}\n", x, y);
            perc.connections.print_components();
            perc.print();
        }
    }
}

#[allow(dead_code)]
fn try_animation() {
    let mut rng = rand::thread_rng();
    // let total_seconds = 30.0;
    let total_seconds = 5.0;
    // let (width, height, display_width_mult, steps_per_frame) = (100, 100, 8.0, 20);
    // let (width, height, display_width_mult, steps_per_frame) = (200, 200, 4.0, 25);
    let (width, height, display_width_mult, steps_per_frame) = (400, 400, 2.0, 100);
    // let (width, height, display_width_mult, steps_per_frame) = (500, 500, 2.0, 5_000);
    // let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 500);
    // let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 250);
    let start_render_threshold = 0.7;
    // let start_render_threshold = 0.97;
    let extra_colors_max = 200;
    let approx_steps = (width * height) as f64 * 0.593;
    let start_render_step = (approx_steps * start_render_threshold) as usize;
    let approx_frames = (approx_steps / steps_per_frame as f64) * (1.0 - start_render_threshold);
    let frame_seconds = total_seconds / approx_frames;
    println!("width = {}, height = {}, approx_steps = {}, approx_frames = {}, frame_seconds = {}",
             format::format_count(width), format::format_count(height), approx_steps, format::format_float(approx_frames, 0), frame_seconds);
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    let mut perc = PercolationGrid::new(width, height);
    let mut frames = vec![];
    let mut color_grid_time = Duration::zero();
    let mut color_grid_union_time = Duration::zero();
    let mut frame_time = Duration::zero();
    let start_time = Instant::now();
    let mut step_count = 0;
    let mut extra_colors = Vec::with_capacity(extra_colors_max);
    let color_min = 0.0;
    let color_max = 1.0;
    for _ in 0..extra_colors_max {
        extra_colors.push(Color1::from_rgb(rng.gen_range(color_min..color_max),rng.gen_range(color_min..color_max),rng.gen_range(color_min..color_max)));
    }
    let mut component_colors= vec![];
    while !perc.percolates() {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if perc.open(x, y) {
            // if extra_colors.len() < extra_colors_max {
            //     extra_colors.push((perc.node_index(x, y), Color1::from_rgb(rng.gen_range(0.5..1.0),rng.gen_range(0.5..1.0),rng.gen_range(0.5..1.0))));
            //}
            step_count += 1;
            if step_count >= start_render_step {
                let is_last_frame = perc.percolates();
                if step_count % steps_per_frame == 0 || is_last_frame {
                    if component_colors.is_empty() {
                        component_colors = perc.connections.get_roots_of_largest_components(extra_colors_max).iter().enumerate()
                            .map(|(index, root)| (*root, extra_colors[index].clone())).collect::<Vec<_>>();
                    }
                    let color_grid_start_time = Instant::now();
                    let color_grid_start_union_duration = perc.connections.union_time;
                    let mut color_grid = Grid::new(width, height, Color1::black());
                    color_grid.record_events = false;
                    for color_y in 0..height {
                        for color_x in 0..width {
                            color_grid.set_xy(color_x, color_y, block_color(&mut perc, &component_colors, color_x, color_y, is_last_frame));
                        }
                    }
                    let color_grid_end_union_duration = perc.connections.union_time;
                    color_grid_union_time += color_grid_end_union_duration - color_grid_start_union_duration;
                    color_grid_time += Instant::now() - color_grid_start_time;
                    let frame_start_time = Instant::now();
                    frames.push(color_grid.as_frame(display_width, display_height, frame_seconds, &|color| *color));
                    frame_time += Instant::now() - frame_start_time;
                }
            }
        }
    }
    let back_color = Color1::black();
    let additive = false;
    println!("overall = {:?}; union = {:?}, connected (build) = {:?}, connected (draw) = {:?}, color grids = {:?}; frames = {:?}",
             Instant::now() - start_time, perc.connections.union_time - color_grid_union_time, color_grid_union_time, perc.connections.is_connected_time, color_grid_time, frame_time);
    Renderer::display_additive("Percolation", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn try_animation_fast() {
    let mut rng = rand::thread_rng();
    // let total_seconds = 30.0;
    let total_seconds = 60.0;
    // let (width, height, display_width_mult, steps_per_frame) = (100, 100, 8.0, 20);
    // let (width, height, display_width_mult, steps_per_frame) = (200, 200, 4.0, 25);
    // let (width, height, display_width_mult, steps_per_frame) = (400, 400, 2.0, 100);
    // let (width, height, display_width_mult, steps_per_frame) = (500, 500, 2.0, 5_000);
    // let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 500);
    let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 100);
    // let start_render_threshold = 0.7;
    let start_render_threshold = 0.97;
    let extra_colors_max = 200;
    let approx_steps = (width * height) as f64 * 0.593;
    let start_render_step = (approx_steps * start_render_threshold) as usize;
    let approx_frames = (approx_steps / steps_per_frame as f64) * (1.0 - start_render_threshold);
    let frame_seconds = total_seconds / approx_frames;
    println!("width = {}, height = {}, approx_steps = {}, approx_frames = {}, frame_seconds = {}",
             format::format_count(width), format::format_count(height), approx_steps, format::format_float(approx_frames, 0), frame_seconds);
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    let mut perc = PercolationGrid::new(width, height);
    let mut frames = vec![];
    let mut color_grid_time = Duration::zero();
    let mut color_grid_union_time = Duration::zero();
    let mut frame_time = Duration::zero();
    let start_time = Instant::now();
    let mut step_count = 0;
    let mut colors = Vec::with_capacity(extra_colors_max);
    colors.push(Color1::black());
    colors.push(Color1::white());
    colors.push(Color1::blue());
    colors.push(Color1::red());
    let color_min = 0.0;
    let color_max = 1.0;
    for _ in 0..extra_colors_max {
        colors.push(Color1::from_rgb(rng.gen_range(color_min..color_max),rng.gen_range(color_min..color_max),rng.gen_range(color_min..color_max)));
    }
    let mut largest_roots= vec![];
    while !perc.percolates() {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if perc.open(x, y) {
            // if extra_colors.len() < extra_colors_max {
            //     extra_colors.push((perc.node_index(x, y), Color1::from_rgb(rng.gen_range(0.5..1.0),rng.gen_range(0.5..1.0),rng.gen_range(0.5..1.0))));
            //}
            step_count += 1;
            if step_count >= start_render_step {
                let is_last_frame = perc.percolates();
                if step_count % steps_per_frame == 0 || is_last_frame {
                    if largest_roots.is_empty() {
                        largest_roots = perc.connections.get_roots_of_largest_components(extra_colors_max);
                    }
                    let color_grid_start_time = Instant::now();
                    let color_grid_start_union_duration = perc.connections.union_time;
                    let mut color_grid = Grid::new(width, height, 0);
                    color_grid.record_events = false;
                    for color_y in 0..height {
                        for color_x in 0..width {
                            color_grid.set_xy(color_x, color_y, block_color_index(&mut perc,&largest_roots, color_x, color_y, is_last_frame));
                        }
                    }
                    let color_grid_end_union_duration = perc.connections.union_time;
                    color_grid_union_time += color_grid_end_union_duration - color_grid_start_union_duration;
                    color_grid_time += Instant::now() - color_grid_start_time;
                    let frame_start_time = Instant::now();
                    frames.push(color_grid.as_frame_color_index(display_width, display_height, frame_seconds));
                    frame_time += Instant::now() - frame_start_time;
                }
            }
        }
    }
    let back_color = Color1::black();
    let additive = false;
    println!("overall = {:?}; union = {:?}, connected (build) = {:?}, connected (draw) = {:?}, color grids = {:?}; frames = {:?}",
             Instant::now() - start_time, perc.connections.union_time - color_grid_union_time, color_grid_union_time, perc.connections.is_connected_time, color_grid_time, frame_time);
    Renderer::display_additive_with_colors("Percolation", display_width, display_height, back_color, frames, additive, colors.clone());
}

#[inline]
fn block_color(perc: &mut PercolationGrid, extra_colors: &Vec<(usize, Color1)>, x: usize, y: usize, is_last_frame: bool) -> Color1 {
    match perc.block_state(x, y) {
        PercolationBlockState::Blocked => Color1::black(),
        PercolationBlockState::Open => {
            let node_index = perc.node_index(x, y);
            if perc.connections.is_connected(PercolationGrid::bottom_node_index(perc.width, perc.height), node_index) {
                Color1::red()
            } else {
                if !is_last_frame {
                    for i in 0..extra_colors.len() {
                        if perc.connections.is_connected(extra_colors[i].0, node_index) {
                            return extra_colors[i].1;
                        }
                    }
                }
                Color1::white()
            }
        },
        PercolationBlockState::Filled => Color1::blue(),
    }
}

#[inline]
fn block_color_index(perc: &mut PercolationGrid, largest_roots: &Vec<usize>, x: usize, y: usize, is_last_frame: bool) -> usize {
    match perc.block_state(x, y) {
        PercolationBlockState::Blocked => COLOR_INDEX_BLACK,
        PercolationBlockState::Open => {
            let node_index = perc.node_index(x, y);
            let node_root_index = perc.connections.root(node_index);
            if perc.connections.is_connected(PercolationGrid::bottom_node_index(perc.width, perc.height), node_index) {
                COLOR_INDEX_RED
            } else {
                if !is_last_frame {
                    for i in 0..largest_roots.len() {
                        //if perc.connections.is_connected(largest_roots[i], node_root_index) {
                        if node_root_index == largest_roots[i] {
                            return COLOR_INDEX_FIRST_EXTRA + i;
                        }
                    }
                }
                COLOR_INDEX_WHITE
            }
        },
        PercolationBlockState::Filled => COLOR_INDEX_BLUE,
    }
}
