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
    // try_animation_fast();
    animate_precalc();
}

pub struct PercolationGrid {
    pub width: usize,
    pub height: usize,
    pub wrap_top_bottom: bool,
    pub wrap_left_right: bool,
    pub start_node_index: usize,
    pub end_node_index: usize,
    pub grid: Grid<bool>,
    pub connections: WeightedQuickUnion,
}

pub enum PercolationBlockState {
    Blocked,
    Open,
    Filled,
}

#[derive(Clone)]
pub enum PercolationType {
    TopBottom,
    TopLeftBottomRight { radius: usize },
    CenterOut { radius: usize },
}

impl PercolationGrid {
    pub fn new(width: usize, height: usize, type_: PercolationType) -> Self {
        let mut grid = Grid::new(width, height,false);
        grid.record_events = false;
        let connections = WeightedQuickUnion::new((width * height) + 2, true);
        // Connect the top virtual node to every square in the top row, and the bottom virtual node
        // to every square in the bottom row.
        let start_node_index = width * height;
        let end_node_index= start_node_index + 1;
        let mut perc = Self {
            width,
            height,
            wrap_top_bottom: false,
            wrap_left_right: false,
            grid,
            connections,
            start_node_index,
            end_node_index,
        };

        let bottom_row_first_node_index = (height - 1) * width;
        match type_ {
            PercolationType::TopBottom => {
                perc.wrap_left_right = true;
                perc.connections.union(start_node_index, 0);
                perc.open_top_row();
                perc.connections.union(end_node_index, bottom_row_first_node_index);
                perc.open_bottom_row();
            },
            PercolationType::TopLeftBottomRight { radius } => {
                perc.wrap_left_right = true;
                perc.connections.union(start_node_index, 0);
                perc.open_circle(0, 0, radius);
                perc.connections.union(end_node_index, (height * width) - 1);
                perc.open_circle(width - 1, height - 1, radius);
            },
            PercolationType::CenterOut { radius } => {
                perc.wrap_top_bottom = true;
                perc.wrap_left_right = true;
                // Center square.
                let center_x = (width / 2) - radius;
                let center_y = (height / 2) - radius;
                perc.connections.union(start_node_index, perc.node_index(center_x, center_y));
                //rintln!("center node = {}, x = {}, y = {}, square_size = {}", center_node_index, x, y, square_size);
                perc.open_circle(center_x, center_y, radius);
                // Top and bottom edges.
                perc.connections.union(end_node_index, 0);
                perc.open_top_row();
                // perc.connections.union(end_node_index, bottom_row_first_node_index);
                // perc.open_bottom_row();
                // Left edge. Right edge is not necessary because of wrapping.
                perc.open_left_edge();
            },
        }
        perc
    }

    fn open_top_row(&mut self) {
        self.open_rectangle(0, 0, self.width, 1);
    }

    fn open_bottom_row(&mut self) {
        self.open_rectangle(0, self.height - 1, self.width, 1);
    }

    fn open_left_edge(&mut self) {
        self.open_rectangle(0, 0, 1, self.height);
    }

    pub fn open_rectangle(&mut self, left: usize, top: usize, width: usize, height: usize) {
        for y in top..top + height {
            for x in left..left + width {
                //rintln!("open_rectangle() for x = {}; y = {}; node = {}", x, y, self.node_index(x, y));
                self.open(x, y);
            }
        }
    }

    pub fn open_circle(&mut self, center_x: usize, center_y: usize, radius: usize) {
        let center_x = center_x as isize;
        let center_y = center_y as isize;
        let radius = radius as isize;
        let start_x = (center_x - radius).max(0);
        let start_y = (center_y - radius).max(0);
        let end_x = (center_x + radius).min(self.width as isize - 1);
        let end_y = (center_y + radius).min(self.height as isize - 1);
        let r_squared = (radius * radius) as f32;
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                //rintln!("open_rectangle() for x = {}; y = {}; node = {}", x, y, self.node_index(x, y));
                let dist_squared = ((x - center_x).pow(2) + (y - center_y).pow(2)) as f32;
                if dist_squared <= r_squared {
                    self.open(x as usize, y as usize);
                }
            }
        }
    }

    pub fn open(&mut self, x: usize, y: usize) -> bool {
        if self.grid.get_xy(x, y) {
            // The square is already open
            return false;
        }
        self.grid.set_xy(x, y, true);
        let node_index = self.node_index(x, y);
        /*
        // Up.
        if y > 0 && self.grid.get_xy(x, y - 1) {
            self.connections.union(node_index, node_index - self.width);
        }
        // Right.
        if x < self.width - 1 && self.grid.get_xy(x + 1, y)  {
            self.connections.union(node_index, node_index + 1);
        } else {
            // Try to wrap around to the left edge.
            if x == self.width - 1 && self.grid.get_xy(0, y) {
                self.connections.union(node_index, node_index - (self.width - 1));
            }
        }
        // Down.
        if y < self.height - 1 && self.grid.get_xy(x, y + 1) {
            self.connections.union(node_index, node_index + self.width);
        }
        // Left.
        if x > 0 && self.grid.get_xy(x - 1, y) {
            self.connections.union(node_index, node_index - 1);
        } else {
            // Try to wrap around to the right edge.
            if x == 0 && self.grid.get_xy(self.width - 1, y) {
                self.connections.union(node_index, node_index + (self.width - 1));
            }
        }
         */
        // Up, right, down, left.
        let x = x as isize;
        let y = y as isize;
        let mut adjacents = vec![];
        adjacents.push((x    , y - 1)); // N
        // adjacents.push((x + 1, y - 1)); // NE
        adjacents.push((x + 1, y    )); // E
        // adjacents.push((x + 1, y + 1)); // SE
        adjacents.push((x    , y + 1)); // S
        // adjacents.push((x - 1, y + 1)); // SW
        adjacents.push((x - 1, y    )); // W
        // adjacents.push((x - 1, y - 1)); // NW
        //bg!(x, y, &adjacents);
        for (adj_x, adj_y) in adjacents.iter() {
            if let Some((adj_x, adj_y)) = self.resolve_x_y(*adj_x, *adj_y) {
                //bg!(adj_x, adj_y);
                if self.grid.get_xy(adj_x, adj_y) {
                    self.connections.union(node_index, self.node_index(adj_x, adj_y))
                }
            }
        }
        true
    }

    #[inline]
    fn resolve_x_y(&self, mut x: isize, mut y: isize) -> Option<(usize, usize)> {
        let width = self.width as isize;
        if x < 0 {
            if self.wrap_left_right {
                x += width;
            } else {
                return None;
            }
        } else {
            if x >= width {
                if self.wrap_left_right {
                    x -= width;
                } else {
                    return None;
                }
            }
        }
        let height = self.height as isize;
        if y < 0 {
            if self.wrap_top_bottom {
                y += height;
            } else {
                return None;
            }
        } else {
            if y >= height {
                if self.wrap_top_bottom {
                    y -= height;
                } else {
                    return None;
                }
            }
        }
        Some((x as usize, y as usize))
    }
    
    pub fn percolates(&mut self) -> bool {
        self.connections.is_connected(self.start_node_index, self.end_node_index)
    }

    #[inline]
    fn node_index(&self, x: usize, y: usize) -> usize {
        (y * self.width) + x
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
            if self.connections.is_connected(self.start_node_index, self.node_index(x, y)) {
                PercolationBlockState::Filled
            } else {
                PercolationBlockState::Open
            }
        } else {
            PercolationBlockState::Blocked
        }
    }

    pub fn run_to_completion(&mut self, max_seconds: usize) -> Vec<(usize, usize)> {
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs_f64(max_seconds as f64);
        let mut rng = rand::thread_rng();
        let mut unions = vec![];
        while !self.percolates() && Instant::now() < end_time {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            if self.open(x, y) {
                unions.push((x, y));
            }
        }
        let time_msg = if self.percolates() { "complete" } else { "RAN OUT OF TIME" };
        println!("run_to_completion(): [{}]; duration = {:?}; unions = {}", time_msg, Instant::now() - start_time, unions.len());
        unions
    }
}

impl PercolationType {
    pub fn to_name(&self) -> &str {
        match self {
            PercolationType::TopBottom => "top-bottom",
            PercolationType::TopLeftBottomRight { radius: _ } => "top-left-bottom-right",
            PercolationType::CenterOut { radius: _ } => "center-out",
        }
    }
}

#[allow(dead_code)]
fn try_percolation() {
    let mut rng = rand::thread_rng();
    let width = 5;
    let height = 5;
    let mut perc = PercolationGrid::new(width, height, PercolationType::TopBottom);
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
    let percolation_type = PercolationType::TopBottom;
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
    let mut perc = PercolationGrid::new(width, height, percolation_type);
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
    // let percolation_type = PercolationType::TopBottom;
    // let percolation_type = PercolationType::TopLeftBottomRight;
    let percolation_type = PercolationType::CenterOut { radius: 100 };
    // let total_seconds = 30.0;
    let total_seconds = 60.0;
    // let (width, height, display_width_mult, steps_per_frame) = (100, 100, 8.0, 20);
    // let (width, height, display_width_mult, steps_per_frame) = (200, 200, 4.0, 25);
    // let (width, height, display_width_mult, steps_per_frame) = (400, 400, 2.0, 100);
    // let (width, height, display_width_mult, steps_per_frame) = (500, 500, 2.0, 5_000);
    // let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 500);
    let (width, height, display_width_mult, steps_per_frame) = (1_000, 1_000, 1.0, 100);
    // let (width, height, display_width_mult, steps_per_frame) = (10, 10, 50.0, 1);
    // let start_render_threshold = 0.7;
    let start_render_threshold = 0.97;
    // let start_render_threshold = 0.0;
    let extra_colors_max = 200;
    let approx_steps = (width * height) as f64 * 0.593;
    let start_render_step = (approx_steps * start_render_threshold) as usize;
    let approx_frames = (approx_steps / steps_per_frame as f64) * (1.0 - start_render_threshold);
    // let max_frames = (approx_frames * 1.5) as usize;
    let max_frames = 500;
    let frame_seconds = total_seconds / approx_frames;
    println!("width = {}, height = {}, approx_steps = {}, approx_frames = {}, frame_seconds = {}",
             format::format_count(width), format::format_count(height), approx_steps, format::format_float(approx_frames, 0), frame_seconds);
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    let mut perc = PercolationGrid::new(width, height, percolation_type);
    // let percolates = perc.percolates();
    //rintln!("start node = {}; end node = {}; percolates = {}", perc.start_node_index, perc.end_node_index, percolates);
    //perc.connections.print_components();
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
    while !perc.percolates() && frames.len() < max_frames {
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
                    if frames.len() % 20 == 0 {
                        println!("frames = {}", format::format_count(frames.len()));
                    }
                }
            }
        }
    }
    println!("step count = {}", step_count);
    //perc.connections.print_components();
    let back_color = Color1::black();
    let additive = false;
    println!("overall = {:?}; union = {:?}, connected (build) = {:?}, connected (draw) = {:?}, color grids = {:?}; frames = {:?}",
             Instant::now() - start_time, perc.connections.union_time - color_grid_union_time, color_grid_union_time, perc.connections.is_connected_time, color_grid_time, frame_time);
    Renderer::display_additive_with_colors("Percolation", display_width, display_height, back_color, frames, additive, colors.clone());
}

#[allow(dead_code)]
fn animate_precalc() {
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let run_to_completion_max_seconds = 30;
    let extra_colors_max = 200;
    let animation_seconds = 30;
    let frame_seconds_min = 0.25;
    let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopLeftBottomRight { radius: 50 });
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::CenterOut { radius: 50 });
    println!("run_to_completion_max_seconds = {}, animation_seconds = {}, frame_seconds_min = {}",
             run_to_completion_max_seconds, animation_seconds, format::format_float(frame_seconds_min, 2));
    println!("width = {}, height = {}, start_render_threshold = {}, percolation_type = {}",
        format::format_count(width), format::format_count(height), format::format_float(start_render_threshold, 3), percolation_type.to_name());

    let largest_dimension = width.max(height);
    let display_width_mult = if largest_dimension >= 800 {
        1.0
    } else {
        (800.0 / largest_dimension as f64).floor()
    };
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    println!("display_width_mult = {}, display_width = {}, display_height = {}",
        display_width_mult as usize, format::format_count(display_width as usize), format::format_count(display_height as usize));

    // Precalculate the number of steps.
    let mut perc = PercolationGrid::new(width, height, percolation_type.clone());
    let unions = perc.run_to_completion(run_to_completion_max_seconds);
    let step_count = unions.len();
    let start_render_step = (step_count as f64 * start_render_threshold) as usize;
    let render_step_count = (step_count - start_render_step) + 1;
    let frame_count_max = (animation_seconds as f64 / frame_seconds_min) as usize;
    let steps_per_frame = (render_step_count as f64 / frame_count_max as f64).floor() as usize;
    println!("step_count = {}, start_render_step = {}, render_step_count = {}, frame_count_max = {}, steps_per_frame = {}",
             format::format_count(step_count), format::format_count(start_render_step),
             format::format_count(render_step_count), format::format_count(frame_count_max),
             format::format_count(steps_per_frame));

    let mut frames = vec![];
    let post_precalc_start_time = Instant::now();

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

    let mut color_grid_elapsed = Duration::zero();
    let mut frame_elapsed = Duration::zero();
    let mut largest_roots= vec![];

    let mut perc = PercolationGrid::new(width, height, percolation_type);
    for (step_index, (x, y)) in unions.iter().enumerate() {
        perc.open(*x, *y);
        let is_last_frame = step_index == step_count - 1;
        if is_last_frame || (step_index >= start_render_step && (steps_per_frame == 0 || step_index % steps_per_frame == 0)) {
            if largest_roots.is_empty() {
                largest_roots = perc.connections.get_roots_of_largest_components(extra_colors_max);
            }

            let color_grid_start_time = Instant::now();
            let mut color_grid = Grid::new(width, height, 0);
            color_grid.record_events = false;
            for color_y in 0..height {
                for color_x in 0..width {
                    color_grid.set_xy(color_x, color_y, block_color_index(&mut perc, &largest_roots, color_x, color_y, is_last_frame));
                }
            }
            color_grid_elapsed += Instant::now() - color_grid_start_time;

            let frame_start_time = Instant::now();
            frames.push(color_grid.as_frame_color_index(display_width, display_height, frame_seconds_min as f64));
            frame_elapsed += Instant::now() - frame_start_time;
        }
    }
    // Adjust the frame seconds for the actual number of frames.
    let frame_count = frames.len();
    let frame_seconds = animation_seconds as f64 / frame_count as f64;
    println!("frame_count = {}; frame_seconds = {}", format::format_count(frame_count), format::format_float(frame_seconds, 3));
    frames.iter_mut().for_each(|frame| frame.seconds_to_next = frame_seconds);

    let post_precalc_elapsed = Instant::now() - post_precalc_start_time;
    let overall_elapsed= Instant::now() - start_time;
    println!("overall = {:?}; post-precalc = {:?}, color grids = {:?}; frames = {:?}",
             overall_elapsed, post_precalc_elapsed, color_grid_elapsed, frame_elapsed);

    let additive = false;
    let back_color = Color1::black();
    Renderer::display_additive_with_colors("Percolation", display_width, display_height, back_color, frames, additive, colors.clone());
}

#[inline]
fn block_color(perc: &mut PercolationGrid, extra_colors: &Vec<(usize, Color1)>, x: usize, y: usize, is_last_frame: bool) -> Color1 {
    match perc.block_state(x, y) {
        PercolationBlockState::Blocked => Color1::black(),
        PercolationBlockState::Open => {
            let node_index = perc.node_index(x, y);
            if perc.connections.is_connected(perc.end_node_index, node_index) {
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
            if perc.connections.is_connected(perc.end_node_index, node_index) {
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
