use crate::*;
use crate::algorithms::percolation::*;
use crate::algorithms::generic_percolation::GenericPercolation;
use std::convert::TryFrom;
use rand::Rng;
use std::time::Duration;
use crate::grid::Grid;
use crate::renderer_3::Renderer;

pub fn main() {
    animate_precalc();
    // try_captions();
    // animate_hex();
}

#[allow(dead_code)]
fn animate_precalc() {
    // This is the equivalent of percolation::animate_precalc(), but using the GenericPercolation
    // and GenericUnion.
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let run_to_completion_max_seconds = 30;
    let extra_colors_max = 200;
    let animation_seconds = 30;
    let frame_seconds_min = 0.25;
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopBottom);
    let (width, height, start_render_threshold, percolation_type) = (600, 600, 0.95, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopLeftBottomRight { radius: 50 });
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::CenterOut { radius: 50 });
    let (width_typed, height_typed) = (from_usize_32(width), from_usize_32(height));
    println!("run_to_completion_max_seconds = {}, animation_seconds = {}, frame_seconds_min = {}",
             run_to_completion_max_seconds, animation_seconds, ff(frame_seconds_min, 2));
    println!("width = {}, height = {}, start_render_threshold = {}, percolation_type = {}",
             fc(width), fc(height), ff(start_render_threshold, 3), percolation_type.to_name());

    let largest_dimension = width.max(height);
    let display_width_mult = if largest_dimension >= 800 {
        1.0
    } else {
        (800.0 / largest_dimension as f64).floor()
    };
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    println!("display_width_mult = {}, display_width = {}, display_height = {}",
             display_width_mult as usize, fc(display_width as usize), fc(display_height as usize));

    // Precalculate the number of steps.
    let mut perc = GenericPercolation::new(width_typed, height_typed);
    let unions = perc.run_to_completion(run_to_completion_max_seconds);
    let step_count = unions.len();
    let start_render_step = (step_count as f64 * start_render_threshold) as usize;
    let render_step_count = (step_count - start_render_step) + 1;
    let frame_count_max = (animation_seconds as f64 / frame_seconds_min) as usize;
    let steps_per_frame = (render_step_count as f64 / frame_count_max as f64).floor() as usize;
    println!("step_count = {}, start_render_step = {}, render_step_count = {}, frame_count_max = {}, steps_per_frame = {}",
             fc(step_count), fc(start_render_step),
             fc(render_step_count), fc(frame_count_max),
             fc(steps_per_frame));

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

    let mut perc = GenericPercolation::clone_new(&perc);
    for (step_index, (x, y)) in unions.iter().enumerate() {
        perc.open(*x, *y);
        let is_last_frame = step_index == step_count - 1;
        if is_last_frame || (step_index >= start_render_step && (steps_per_frame == 0 || step_index % steps_per_frame == 0)) {
            if largest_roots.is_empty() {
                largest_roots = perc.union.get_roots_of_largest_components(extra_colors_max);
            }

            let color_grid_start_time = Instant::now();
            let mut color_grid = Grid::new(width, height, 0);
            color_grid.record_events = false;
            for color_y in 0..height_typed {
                for color_x in 0..width_typed {
                    color_grid.set_xy(usize::try_from(color_x).unwrap(), usize::try_from(color_y).unwrap(),
                                      perc.block_color_index(&largest_roots, color_x, color_y, is_last_frame));
                }
            }
            color_grid_elapsed += Instant::now() - color_grid_start_time;

            let frame_start_time = Instant::now();

            let frame= color_grid.as_frame_color_index(display_width, display_height, frame_seconds_min as f64);
            frames.push(frame);
            frame_elapsed += Instant::now() - frame_start_time;
        }
    }
    // Adjust the frame seconds for the actual number of frames.
    let frame_count = frames.len();
    let frame_seconds = animation_seconds as f64 / frame_count as f64;
    println!("frame_count = {}; frame_seconds = {}", fc(frame_count), ff(frame_seconds, 3));
    frames.iter_mut().for_each(|frame| frame.seconds_to_next = frame_seconds);

    let post_precalc_elapsed = Instant::now() - post_precalc_start_time;
    let overall_elapsed= Instant::now() - start_time;
    println!("overall = {:?}; post-precalc = {:?}, color grids = {:?}; frames = {:?}",
             overall_elapsed, post_precalc_elapsed, color_grid_elapsed, frame_elapsed);

    let additive = false;
    let back_color = Color1::black();
    Renderer::display_additive_with_colors("Percolation", display_width, display_height, back_color, frames, additive, colors.clone());
}

#[allow(dead_code)]
fn try_captions() {
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let run_to_completion_max_seconds = 30;
    let extra_colors_max = 200;
    let animation_seconds = 30;
    let frame_seconds_min = 0.25;
    let caption_height = 100.0;
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopBottom);
    let (width, height, start_render_threshold, percolation_type) = (600, 600, 0.95, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopLeftBottomRight { radius: 50 });
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::CenterOut { radius: 50 });
    let (width_typed, height_typed) = (from_usize_32(width), from_usize_32(height));
    println!("run_to_completion_max_seconds = {}, animation_seconds = {}, frame_seconds_min = {}",
             run_to_completion_max_seconds, animation_seconds, ff(frame_seconds_min, 2));
    println!("width = {}, height = {}, start_render_threshold = {}, percolation_type = {}",
             fc(width), fc(height), ff(start_render_threshold, 3), percolation_type.to_name());

    let largest_dimension = width.max(height);
    let display_width_mult = if largest_dimension >= 800 {
        1.0
    } else {
        (800.0 / largest_dimension as f64).floor()
    };
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    println!("display_width_mult = {}, display_width = {}, display_height = {}",
             display_width_mult as usize, fc(display_width as usize), fc(display_height as usize));

    // Precalculate the number of steps.
    let mut perc = GenericPercolation::new(width_typed, height_typed);
    let unions = perc.run_to_completion(run_to_completion_max_seconds);
    let step_count = unions.len();
    let start_render_step = (step_count as f64 * start_render_threshold) as usize;
    let render_step_count = (step_count - start_render_step) + 1;
    let frame_count_max = (animation_seconds as f64 / frame_seconds_min) as usize;
    let steps_per_frame = (render_step_count as f64 / frame_count_max as f64).floor() as usize;
    println!("step_count = {}, start_render_step = {}, render_step_count = {}, frame_count_max = {}, steps_per_frame = {}",
             fc(step_count), fc(start_render_step),
             fc(render_step_count), fc(frame_count_max),
             fc(steps_per_frame));

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

    let mut perc = GenericPercolation::clone_new(&perc);
    for (step_index, (x, y)) in unions.iter().enumerate() {
        perc.open(*x, *y);
        let is_last_frame = step_index == step_count - 1;
        if is_last_frame || (step_index >= start_render_step && (steps_per_frame == 0 || step_index % steps_per_frame == 0)) {
            if largest_roots.is_empty() {
                largest_roots = perc.union.get_roots_of_largest_components(extra_colors_max);
            }

            let color_grid_start_time = Instant::now();
            let mut color_grid = Grid::new(width, height, 0);
            color_grid.record_events = false;
            for color_y in 0..height_typed {
                for color_x in 0..width_typed {
                    color_grid.set_xy(usize::try_from(color_x).unwrap(), usize::try_from(color_y).unwrap(),
                                      perc.block_color_index(&largest_roots, color_x, color_y, is_last_frame));
                }
            }
            color_grid_elapsed += Instant::now() - color_grid_start_time;

            let frame_start_time = Instant::now();

            let frame= color_grid.as_frame_color_index_captioned(display_width, display_height, caption_height, frame_seconds_min as f64);
            //add_caption(&mut frame, display_width, caption_height, caption);

            frames.push(frame);
            frame_elapsed += Instant::now() - frame_start_time;
        }
    }
    // Adjust the frame seconds for the actual number of frames.
    let frame_count = frames.len();
    let frame_seconds = animation_seconds as f64 / frame_count as f64;
    println!("frame_count = {}; frame_seconds = {}", fc(frame_count), ff(frame_seconds, 3));
    frames.iter_mut().for_each(|frame| frame.seconds_to_next = frame_seconds);

    let post_precalc_elapsed = Instant::now() - post_precalc_start_time;
    let overall_elapsed= Instant::now() - start_time;
    println!("overall = {:?}; post-precalc = {:?}, color grids = {:?}; frames = {:?}",
             overall_elapsed, post_precalc_elapsed, color_grid_elapsed, frame_elapsed);

    let additive = false;
    let back_color = Color1::black();
    Renderer::display_additive_with_colors("Percolation", display_width, display_height + caption_height, back_color, frames, additive, colors.clone());
}

/*
#[inline]
fn block_color<T>(perc: &mut GenericPercolation<T>, extra_colors: &Vec<(usize, Color1)>, x: usize, y: usize, is_last_frame: bool) -> Color1
    where T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize> + SampleUniform
{
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
*/

#[inline]
pub fn from_usize_32(val: usize) -> u32 {
    u32::try_from(val).unwrap()
}

#[inline]
pub fn from_usize_16(val: usize) -> u16 {
    u16::try_from(val).unwrap()
}
