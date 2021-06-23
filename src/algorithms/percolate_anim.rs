use crate::*;
use crate::algorithms::percolation::*;
use crate::algorithms::generic_percolation::GenericPercolation;
use std::convert::TryFrom;
use rand::Rng;
use std::time::Duration;
use crate::grid::{Grid, GridLayout};
use crate::renderer_3::Renderer;
use crate::algorithms::group_color::GroupColor;

pub fn main() {
    // animate_precalc();
    // try_captions();
    // animate_hex();
    // test_decelerate();
    // animate_decelerate();
    // test_group_color();
    animate_group_color();
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

#[allow(dead_code)]
fn animate_hex() {
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let run_to_completion_max_seconds = 30;
    let extra_colors_max = 200;
    let animation_seconds = 60;
    let frame_seconds_min = 0.25;
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.97, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (1_600, 800, 0.95, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (100, 100, 0.0, PercolationType::TopBottom);
    // let (width, height, start_render_threshold, percolation_type) = (40, 40, 0.0, PercolationType::TopBottom);
    let (width, height, start_render_threshold, percolation_type) = (200, 200, 0.8, PercolationType::TopBottom);
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
    let block_half_width = (display_width / width as f64) / 2.0;
    println!("display_width_mult = {}, display_width = {}, display_height = {}",
             display_width_mult as usize, fc(display_width as usize), fc(display_height as usize));

    // Precalculate the number of steps.
    let mut perc = GenericPercolation::new_layout(width_typed, height_typed, GridLayout::Hex);
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

            let frame= color_grid.as_frame_color_index_layout(display_width, display_height, GridLayout::Hex, frame_seconds_min as f64);
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
    Renderer::display_additive_with_colors("Percolation", display_width + block_half_width, display_height, back_color, frames, additive, colors.clone());
}

#[allow(dead_code)]
fn animate_decelerate() {
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let run_to_completion_max_seconds = 30;
    let extra_colors_max = 200;
    let frame_seconds_min = 0.25;
    let (width, height, start_render_threshold, animation_seconds, deceleration_ratio) = (800, 800, 0.95, 30.0, 100.0);
    let (width_typed, height_typed) = (from_usize_32(width), from_usize_32(height));
    println!("run_to_completion_max_seconds = {}, animation_seconds = {}, frame_seconds_min = {}",
             run_to_completion_max_seconds, animation_seconds, ff(frame_seconds_min, 2));
    println!("width = {}, height = {}, start_render_threshold = {}, deceleration_ratio = {}",
             fc(width), fc(height), ff(start_render_threshold, 3), ff(deceleration_ratio, 1));

    let display_width_mult = if height >= 800 {
        1.0
    } else {
        (800.0 / height as f64).floor()
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

    let frame_step_indexes = group_decelerate(deceleration_ratio, render_step_count, start_render_step, frame_count_max);

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
    let mut next_frame_step = frame_step_indexes[0];
    for (step_index, (x, y)) in unions.iter().enumerate() {
        perc.open(*x, *y);
        let is_last_frame = step_index == step_count - 1;
        if is_last_frame || step_index == next_frame_step {
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

            if frames.len() < frame_step_indexes.len() {
                next_frame_step = frame_step_indexes[frames.len()];
            }
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
fn animate_group_color() {
    let start_time = Instant::now();
    let run_to_completion_max_seconds = 30;
    // let extra_colors_max = 200;
    let extra_colors_max = 200;
    let (color_min, color_max) = (0.2, 0.8);
    let frame_seconds_min = 0.25;
    let (width, height, start_render_threshold, animation_seconds, deceleration_ratio) = (800, 800, 0.95, 30.0, 1.0);
    let color_threshold = 0.95;
    let open_as_white = false;
    let (width_typed, height_typed) = (from_usize_32(width), from_usize_32(height));
    println!("run_to_completion_max_seconds = {}, animation_seconds = {}, frame_seconds_min = {}",
             run_to_completion_max_seconds, animation_seconds, ff(frame_seconds_min, 2));
    println!("width = {}, height = {}, start_render_threshold = {}, deceleration_ratio = {}",
             fc(width), fc(height), ff(start_render_threshold, 3), ff(deceleration_ratio, 1));

    let display_width_mult = if height >= 800 {
        1.0
    } else {
        (800.0 / height as f64).floor()
    };
    let display_width = width as f64 * display_width_mult;
    let display_height = height as f64 * display_width_mult;
    println!("display_width_mult = {}, display_width = {}, display_height = {}",
             display_width_mult as usize, fc(display_width as usize), fc(display_height as usize));

    // Precalculate the number of steps.
    let mut perc = GenericPercolation::new(width_typed, height_typed);
    let unions = perc.run_to_completion(run_to_completion_max_seconds);
    let step_count = unions.len();
    let color_threshold_step = (step_count as f64 * color_threshold) as usize;
    let start_render_step = (step_count as f64 * start_render_threshold) as usize;
    let render_step_count = (step_count - start_render_step) + 1;
    let frame_count_max = (animation_seconds as f64 / frame_seconds_min) as usize;
    let steps_per_frame = (render_step_count as f64 / frame_count_max as f64).floor() as usize;
    println!("step_count = {}, start_render_step = {}, render_step_count = {}, frame_count_max = {}, steps_per_frame = {}",
             fc(step_count), fc(start_render_step),
             fc(render_step_count), fc(frame_count_max),
             fc(steps_per_frame));

    let frame_step_indexes = group_decelerate(deceleration_ratio as f64, render_step_count, start_render_step, frame_count_max);

    let mut frames = vec![];
    let post_precalc_start_time = Instant::now();

    let mut group_color = GroupColor::new(color_min, color_max, open_as_white);

    let mut color_grid_elapsed = Duration::zero();
    let mut frame_elapsed = Duration::zero();

    let mut perc = GenericPercolation::clone_new(&perc);
    let mut next_frame_step = frame_step_indexes[0];
    for (step_index, (x, y)) in unions.iter().enumerate() {
        perc.open(*x, *y);
        let is_last_frame = step_index == step_count - 1;
        if is_last_frame || step_index == next_frame_step {
            let largest_roots = perc.union.get_roots_of_largest_components(extra_colors_max);
            if step_index >= color_threshold_step {
                group_color.update(largest_roots, &perc.union);
            }

            let color_grid_start_time = Instant::now();
            let mut color_grid = Grid::new(width, height, 0);
            color_grid.record_events = false;
            for color_y in 0..height_typed {
                for color_x in 0..width_typed {
                    let node_index = perc.x_y_to_index(color_x, color_y);
                    let block_state = perc.block_state(color_x, color_y);
                    let color_index = group_color.block_color_index(node_index, perc.end_node_index, block_state, is_last_frame, &perc.union);
                    let (color_x_usize, color_y_usize) = (perc.to_usize(color_x), perc.to_usize(color_y));
                    color_grid.set_xy(color_x_usize, color_y_usize, color_index);
                }
            }
            color_grid_elapsed += Instant::now() - color_grid_start_time;

            let frame_start_time = Instant::now();

            let frame= color_grid.as_frame_color_index(display_width, display_height, frame_seconds_min as f64);
            frames.push(frame);
            frame_elapsed += Instant::now() - frame_start_time;

            if frames.len() < frame_step_indexes.len() {
                next_frame_step = frame_step_indexes[frames.len()];
            }
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
    Renderer::display_additive_with_colors("Percolation", display_width, display_height, back_color, frames, additive, group_color.get_colors());
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

pub fn group_decelerate(ratio_first_to_second_half: f64, render_step_count: usize, start_render_step: usize, frame_count: usize) -> Vec<usize> {
    let base = ratio_first_to_second_half.powf(2.0 / frame_count as f64);
    //rintln!("\ngroup_decelerate: ratio_first_to_second_half = {}; render_step_count = {}, start_render_step: {}, frame_count = {}, base = {}",
    //         ff(ratio_first_to_second_half, 1), fc(render_step_count), fc(start_render_step), fc(frame_count), ff(base, 5));
    let mut base_sum= 0.0;
    let mut bases = vec![];
    for i in 0..frame_count {
        let val = base.powf(i as f64);
        bases.push(val);
        base_sum += val;
    }
    assert_eq!(frame_count, bases.len());
    //rintln!("group_decelerate: bases = {}", bases.iter().map(|x| ff(*x, 5)).join("\t"));

    let mult_normalize = render_step_count as f64 / base_sum;
    let norms = bases.iter_mut().map(|x| *x * mult_normalize).collect::<Vec<_>>();
    assert_eq!(frame_count, norms.len());
    //rintln!("group_decelerate: norms = {}", norms.iter().map(|x| ff(*x, 5)).join("\t"));

    let mut count_sum = 0;
    let mut counts = vec![];
    for (index, norm) in norms.iter().enumerate() {
        let count = if index == norms.len() - 1 {
            //bg!(render_step_count, count_sum);
            render_step_count - count_sum
        } else {
            if index % 2 == 0 {
                norm.ceil() as usize
            } else {
                norm.floor() as usize
            }
        };
        counts.push(count);
        count_sum += count;
    }
    //rintln!("group_decelerate: counts = {}", counts.iter().map(|x| fc(*x)).join("\t"));

    counts.reverse();
    //rintln!("group_decelerate: counts = {}", counts.iter().map(|x| fc(*x)).join("\t"));
    assert_eq!(frame_count, counts.len());
    debug_assert_eq!(render_step_count, counts.iter().sum());

    let mut this_index = 0;
    let mut indexes = vec![];
    for (index, count) in counts.iter().enumerate() {
        if index == 0 {
            this_index = start_render_step + (count - 1);
        } else {
            this_index += count;
        }
        indexes.push(this_index);
    }
    //rintln!("group_decelerate: indexes = {}", indexes.iter().map(|x| fc(*x)).join("\t"));

    indexes
}

#[allow(dead_code)]
fn test_decelerate() {
    for step_count in [25, 100].iter() {
        for frame_count in [2, 3, 5].iter() {
            for ratio in [2.0, 4.0, 10.0].iter() {
                for start_render_step in [0, 10].iter() {
                    let render_step_count = step_count - start_render_step;
                    group_decelerate(*ratio, render_step_count, *start_render_step, *frame_count);
                }
            }
        }
    }
    /*
    let step_count = 500_000;
    let frame_count = 60;
    for start_render_step in [0, 400,000].iter() {
        let render_step_count = step_count - start_render_step;
        for ratio in [0.5, 1.0, 2.0, 4.0, 10.0].iter() {
            group_decelerate(*ratio, render_step_count, *start_render_step, frame_count);
        }
    }
     */
}

#[inline]
pub fn from_usize_32(val: usize) -> u32 {
    u32::try_from(val).unwrap()
}

#[inline]
pub fn from_usize_16(val: usize) -> u16 {
    u16::try_from(val).unwrap()
}
