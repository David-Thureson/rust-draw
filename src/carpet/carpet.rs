use std::time::Instant;

use crate::*;
use renderer_3::*;
use crate::grid::*;
// use std::sync::mpsc;
use std::mem;
// use std::collections::BTreeMap;
use std::path::Path;
use bit_vec::BitVec;
#[allow(unused_imports)]
use rand::{thread_rng, Rng};

const PATH_IMAGE_FILES: &str = r"C:\Graphics\Carpet";

pub fn main() {
    // first();
    // draw_one(200, 2.0, 7, 0.68, &CarpetAlgorithm::Simple);
    // test_point_in_wedge();
    // try_draw_wedge();
    // try_animation();
    // try_write_and_read_grid();
    // optimize_build_grid();
    // try_algorithms();
    // try_combine_carpets();
    // make_gallery();
    // make_gallery_other_mod();
    // make_gallery_mult_mod_xor();
    // make_gallery_mod_large();
    // make_gallery_mod_medium();
    // make_gallery_mod_4_combo_1();
    // make_gallery_mod_4_combo();
    // debug_edge_issue();
    // debug_corner_algorithm();
    // time_corner_algorithm_vs_read_file();
    // time_corner_algorithm_vary_mult();
    // try_grayscale_256();
    // generate_grids_parallel();
    // draw_big_gallery();
    // draw_big_gallery_256();
    // draw_combo_gallery();
    anim_flow();
}

pub struct Carpet {
    size: usize,
    min_length: usize,
    mult: usize,
    grid: Grid<usize>,
    modulus: Option<usize>,
}

impl Carpet {
    pub fn new(size: usize, min_length: usize, mult: usize, modulus: Option<usize>) -> Self {
        let grid = Grid::new(1, 1,0);
        Self {
            size,
            min_length,
            mult,
            grid,
            modulus,
        }
    }

    pub fn copy_set_from_bits(&self, bits: &BitVec) -> Self {
        let mut grid = Grid::new(self.size, self.size, 0);
        for y in 0..self.size {
            for x in 0..self.size {
                let index = x_y_to_cell_index_usize(self.size, x, y);
                grid.set_xy(x, y, true_for_white_to_count(bits.get(index).unwrap()));
            }
        }
        Self {
            size: self.size,
            min_length: self.min_length,
            mult: self.mult,
            grid,
            modulus: None,
        }
    }

    pub fn negative(&self) -> Self {
        let mut bits = self.to_bit_vec();
        bits.negate();
        self.copy_set_from_bits(&bits)
    }

    pub fn xor(&self, other: &Carpet) -> Self {
        let bits_a = self.to_bit_vec();
        let bits_b = other.to_bit_vec();
        let mut bits = BitVec::from_elem(bits_a.len(), false);
        for i in 0..bits.len() {
            if bits_a.get(i).unwrap() != bits_b.get(i).unwrap() {
                bits.set(i, true);
            }
        }
        self.copy_set_from_bits(&bits)
    }

    pub fn go(&mut self) {
        // Algorithm: Metaphorically, draw a square around the edges of the carpet, then draw four
        // smaller squares within that, then four even smaller squares within each smaller square
        // and so on.
        //
        // But really draw the smallest possible square in the upper left corner, then use the
        // resulting data to repeat that square three times, forming the second-smallest square.
        // Then repeat that second-smallest square three times to form the third-smallest square
        // and so on. Each repeat means adding the numbers in the corresponding positions from the
        // source to the destination square.

        // let start_time = Instant::now();

        // self.cells.reserve(self.size * self.size);
        // for _ in 0..self.size * self.size {
        //     self.cells.push(0);
        //}

        let mut square_sizes = vec![];
        let mut one_size = self.size as f32;
        let mut prev_rounded_size = 0;
        while one_size.round() as usize >= self.min_length {
            let rounded_size = one_size.round() as usize;
            if rounded_size == prev_rounded_size {
                break;
            }
            prev_rounded_size = rounded_size;
            square_sizes.push(rounded_size);
            one_size *= self.mult as f32 / 1_000.0;
        }

        square_sizes.reverse();

        let mut prev_square_grid: Option<Grid<usize>> = None;
        for size in square_sizes.iter() {
            // Drow the outline, with a value of one for every grid position along the edges except
            // for the corners which have two.
            let mut square_grid = self.corner_square_outline_grid(*size);
            if let Some(prev_square_grid) = prev_square_grid {
                // Paste the previous, smaller grid onto this grid four times, in each of the
                // corners.
                let prev_size = prev_square_grid.width;
                let size_diff = size - prev_size;
                // Top left.
                square_grid.add_grid_at_x_y(0, 0,&prev_square_grid, self.modulus);
                // Top right.
                square_grid.add_grid_at_x_y(size_diff, 0, &prev_square_grid, self.modulus);
                // Bottom right.
                square_grid.add_grid_at_x_y(size_diff, size_diff,&prev_square_grid, self.modulus);
                // Bottom left.
                square_grid.add_grid_at_x_y(0, size_diff, &prev_square_grid, self.modulus);
            }
            prev_square_grid = Some(square_grid);
        }
        mem::swap(&mut self.grid, &mut prev_square_grid.unwrap());
    }

    fn corner_square_outline_grid(&self, size: usize) -> Grid<usize> {
        // This will leave a value of 1 in all points along the edge except for the corners which
        // will have a value of 2.
        let mut grid = Grid::new(size, size, 0);
        // Top and bottom of the square.
        for x in 0..size {
            grid.add_xy(x, 0, 1, self.modulus);
            grid.add_xy(x, size - 1, 1, self.modulus);
        }
        // Left and right edges of the square.
        for y in 0..size {
            grid.add_xy(0, y, 1, self.modulus);
            grid.add_xy(size - 1, y, 1, self.modulus);
        }
        grid
    }

    fn full_file_name(size: usize, min_length: usize, mult: usize, modulus: Option<usize>, label: Option<&str>) -> String {
        let label = label.map_or("".to_string(), |label| format!(" {}", label));
        let modulus = modulus.map_or("".to_string(), |modulus| format!("_{}", modulus));
        format!("{}/carpet_{}_{}_{}{} {}.txt", PATH_IMAGE_FILES, size, min_length, mult, modulus, label)
    }

    pub fn write_grid(&self) {
        let full_file_name = Self::full_file_name(self.size, self.min_length, self.mult,self.modulus,None);
        //let start_time = Instant::now();
        self.grid.write(&full_file_name);
        //rintln!("Carpet::write_grid({}): {:?}", full_file_name, Instant::now() - start_time);
    }

    pub fn write_grid_labeled(&self, label: &str) {
        let full_file_name = Self::full_file_name(self.size, self.min_length, self.mult, self.modulus, Some(label));
        //let start_time = Instant::now();
        self.grid.write(&full_file_name);
        //rintln!("Carpet::write_grid({}): {:?}", full_file_name, Instant::now() - start_time);
    }

    pub fn read_grid_optional(size: usize, min_length: usize, mult: usize, modulus: Option<usize>) -> Option<Grid<usize>> {
        let full_file_name = Self::full_file_name(size, min_length, mult, modulus,None);
        Grid::read_optional(&full_file_name)
    }

    pub fn read_or_make_grid(size: usize, min_length: usize, mult: usize, modulus: Option<usize>) -> Grid<usize> {
        match Carpet::read_grid_optional(size, min_length, mult, modulus) {
            Some(grid) => {
                //rintln!("Carpet::read_or_make_grid({}): found", full_file_name);
                grid
            },
            None => {
                let mut carpet = Carpet::new(size, min_length, mult, modulus);
                carpet.go();
                let full_file_name = Self::full_file_name(size, min_length, mult, modulus, None);
                carpet.grid.write(&full_file_name);
                carpet.grid
            }
        }
    }

    pub fn grid_exists(size: usize, min_length: usize, mult: usize, modulus: Option<usize>) -> bool {
        let full_file_name = Carpet::full_file_name(size, min_length, mult, modulus,None);
        Path::new(&full_file_name).exists()
    }

    pub fn to_bit_vec(&self) -> BitVec {
        let mut bits = BitVec::from_elem(self.size * self.size, false);
        for y in 0..self.size {
            for x in 0..self.size {
                if count_to_true_for_white(&self.grid.get_xy(x, y)) {
                    let index = x_y_to_cell_index_usize(self.size, x, y);
                    bits.set(index, true);
                }
            }
        }
        bits
    }

    pub fn draw(&self, display_width_mult: f64) {
        // let start_time = Instant::now();
        let display_width = self.size as f64 * display_width_mult;
        let display_height = display_width;
        let frame_seconds = 0.1;
        // let start_time = Instant::now();
        // let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, count_to_color_black_white);
        // let func: FnOnce(&usize) -> Color1 = |count| count_to_color_gray(count, min, max);
        let frames = self.grid.to_final_frame(display_width, display_height, frame_seconds, &|count| count_to_color_black_white(count));
        // println!("create frames seconds = {}", (Instant::now() - start_time).as_secs());

        let back_color = count_to_color_black_white(&0);
        let additive = false;
        Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
    }

}

pub fn create_one(size: usize, min_length: usize, mult: usize, modulus: Option<usize>) -> Carpet {
    let mut carpet = Carpet::new(size, min_length, mult, modulus);
    carpet.go();
    carpet
}

#[allow(dead_code)]
fn draw_one(size: usize, display_width_mult: f64, min_length: usize, mult: usize, modulus: Option<usize>) {
    let carpet = create_one(size, min_length, mult, modulus);
    carpet.draw(display_width_mult);
}

pub fn draw_grid_normal(grid: &Grid<usize>, display_width_mult: f64) {
    grid.draw(display_width_mult, &|count| count_to_color_black_white(count))
}

/*
#[allow(dead_code)]
fn first() {
    let size: usize = 800;
    let display_width_mult = 1.0;
    let min_length = 5;
    let mult = 680;
    let record_events = false;
    let mut carpet = Carpet::new(size, min_length, mult, &CarpetAlgorithm::Simple, record_events);

    let start_time = Instant::now();
    carpet.go();
    println!("create grid seconds = {}, count_square = {}, count_side = {}, count_touch_rect = {}",
        (Instant::now() - start_time).as_secs(),
        fc(carpet.count_square),
        fc(carpet.count_side),
        fc(carpet.count_touch_rect));

    // let char_grid = Grid::new_from(&carpet.grid, count_to_char(&0), count_to_char);
    // let char_grid = Grid::new_from(&carpet.grid, count_to_char_black_white);
    // char_grid.print("A");
    // let color_grid = Grid::new_from(&carpet_grid, count_to_color_black_white);

    let (min, max) = carpet.grid.min_max();
    println!("min = {}, max = {}", min, max);

    let frame_count = 100;
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let frame_seconds = 0.1;

    let start_time = Instant::now();
    // let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, count_to_color_black_white);
    // let func: FnOnce(&usize) -> Color1 = |count| count_to_color_gray(count, min, max);
    let frames = carpet.grid.events_to_frames(frame_count, display_width, display_height, frame_seconds, &|count| count_to_color_gray(count, min, max));
    println!("create frames seconds = {}", (Instant::now() - start_time).as_secs());

    //bg!(&frames[1]);
    //bg_frame("1", &frames[1]);
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn dbg_frame(label: &str, frame: &Frame) {
    println!("\n{}", label);
    for shape in frame.shapes.iter() {
        match shape {
            Shape::Rectangle { top_left, bottom_right: _, color} => {
                let x = top_left.x.round() as usize;
                let y = top_left.y.round() as usize;
                let color = if color.r > 0.5 { "white" } else { "black" };
                println!("{}, {}: {}", x, y, color);
            },
            _ => unimplemented!(),
        }
    }
}

#[allow(dead_code)]
fn try_draw_wedge() {
    let size = 400;
    let display_width_mult = 2.0;
    let min_length = 7;
    let mult = 680;
    draw_one(size, display_width_mult, min_length, mult,&CarpetAlgorithm::Simple);
    draw_one(size, display_width_mult, min_length, mult,&CarpetAlgorithm::Wedge);
}

#[allow(dead_code)]
fn try_animation() {
    // animate_mult(200, 4.0, 2.0, 7, 0.675, 0.685, 0.001)
    // animate_mult(200, 2.0, 2.0, 4, 0.67, 0.69, 0.0001)
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.65, 0.70, 0.0003)
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.60, 0.65, 0.001)
    // animate_mult_parallel(800, 2, 1.0, 1.0, 3, 0.63, 0.68, 0.001, 50);
    // animate_mult_parallel(400, 2.0, 1.0, 3, 0.5, 0.60, 0.001)
    animate_mult_parallel(400, 2, 2.0, 2.0, 3, 650, 800, 1, 50);
    // animate_show_existing(400, 2.0, 2.0, 3, 0.7, 0.9, 0.001);
    // animate_mult_parallel(200, 5, 4.0, 1.0, 7, 0.6, 0.8, 0.002, 1_000);
    // animate_mult_parallel(200, 5, 2.0, 1.0, 7, 0.8, 0.9, 0.002, 1_000);
    // animate_show_existing(200, 5, 4.0, 1.5, 7, 0.8, 0.9, 0.002);
    // animate_mult_parallel(200, 4, 2.0, 0.75, 7, 0.53, 0.8, 0.002, 1_000);
    // animate_mult_parallel(100, 4, 4.0, 1.0, 7, 0.53, 0.9, 0.002, 1_000);
    // animate_show_existing(100, 4, 4.0, 1.0, 7, 0.53, 0.9, 0.002);
}

#[allow(dead_code)]
fn animate_mult(size: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: usize, mult_max: usize, mult_step: usize) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let mut frames = vec![];
    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }
    let mut prev_grid = None;
    let mut skipped_count = 0;
    let start_time = Instant::now();
    for mult in mults.iter() {
        let carpet = create_one(size, min_length, *mult,&CarpetAlgorithm::Wedge);
        if prev_grid.is_none() || prev_grid.unwrap() != carpet.grid {
            frames.push(carpet.grid.as_frame(display_width, display_height, frame_seconds, &|count| count_to_color_black_white(count)));
        } else {
            skipped_count += 1;
        }
        prev_grid = Some(carpet.grid.clone());
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(skipped_count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn animate_mult_parallel(size: usize, black_white_modulus: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: usize, mult_max: usize, mult_step: usize, threads_max: usize) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let start_time = Instant::now();
    let (tx, rx) = mpsc::channel();
    let mut threads = Vec::new();
    let mut frame_index = 0;

    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }

    let mut thread_count = 0;
    for mult in mults.iter() {
        let mult = *mult;
        let grid_exists = Carpet::grid_exists(size, min_length, mult);
        if !grid_exists {
            if thread_count < threads_max {
                let file_name = Carpet::full_file_name(size, min_length, mult, None);
                println!("Starting to build {}", file_name);
            }
        }
        if grid_exists || thread_count < threads_max {
            let thread_tx = tx.clone();
            let thread = thread::spawn(move || {
                // let carpet = create_one(size, min_length, mult,CarpetAlgorithm::Wedge);
                // thread_tx.send((frame_index, carpet.grid)).unwrap();
                let grid = Carpet::read_or_make_grid(size, min_length, mult);
                thread_tx.send((frame_index, grid)).unwrap();
            });
            threads.push(thread);
        }
        frame_index += 1;
        if !grid_exists {
            thread_count += 1;
            if thread_count == threads_max {
                println!("Reached max threads.");
            }
        }
    }

    // Here, all the messages are collected.
    let mut grids = BTreeMap::new();
    for i in 0..threads.len() {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        let (frame_index, grid) = rx.recv().unwrap();
        grids.insert(frame_index, grid);
        println!("frame_index = {}; remaining frames = {}", fc(frame_index), fc(mults.len() - (i + 1)));
    }

    // Wait for the threads to complete any remaining work.
    for thread in threads {
        thread.join().unwrap();
    }

    let mut frames = vec![];
    let mut prev_grid = None;
    let mut skipped_count = 0;
    for grid in grids.values() {
        if prev_grid.is_none() || prev_grid.unwrap() != *grid {
            // let (min, max) = grid.min_max();
            frames.push(grid.as_frame(display_width, display_height, frame_seconds,
                                      // &|count| count_to_color_black_white(count)));
                &|count| count_to_color_black_white_mod(count, black_white_modulus)));
            //&|count| count_to_color_gray(count, min, max)));
        } else {
            skipped_count += 1;
        }
        prev_grid = Some(grid.clone());
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(skipped_count));
    let back_color = count_to_color_black_white(&0);
    let additive = false;

    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn animate_show_existing(size: usize, black_white_modulus: usize, display_width_mult: f64, frame_seconds: f64, min_length: usize, mult_min: usize, mult_max: usize, mult_step: usize) {
    let display_width = size as f64 * display_width_mult;
    let display_height = display_width;
    let start_time = Instant::now();

    let mut mults = vec![];
    let mut mult = mult_min;
    while mult <= mult_max {
        mults.push(mult);
        mult += mult_step;
    }
    let mut grids = vec![];
    for mult in mults.iter() {
        if let Some(grid) = Carpet::read_grid_optional(size, min_length, *mult) {
            grids.push(grid);
        };
    }

    let mut frames = vec![];
    for grid in grids.iter() {
        frames.push(grid.as_frame(display_width, display_height, frame_seconds,
            // &|count| count_to_color_black_white(count)));
            &|count| count_to_color_black_white_mod(count, black_white_modulus)));
            //&|count| count_to_color_gray(count, min, max)));
    }
    dbg!(Instant::now() - start_time);
    println!("frame count = {}, skipped_count = {}", fc(frames.len()), fc(mults.len() - frames.len()));
    let back_color = count_to_color_black_white(&0);
    let additive = false;

    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}

#[allow(dead_code)]
fn try_write_and_read_grid() {
    /*
    let carpet = create_one(400, 5, 0.68,CarpetAlgorithm::Wedge);
    let reference_grid = carpet.grid.clone();
    carpet.write_grid();

    let found_grid = Carpet::read_or_make_grid(400, 5, 0.68);
    assert!(reference_grid == found_grid);
    found_grid.write(&format!("{}/Test_Grid.txt", PATH_IMAGE_FILES));
     */

    Carpet::read_or_make_grid(400, 5, 681);
    Carpet::read_or_make_grid(400, 5, 681);
}

#[allow(dead_code)]
fn optimize_build_grid() {
    // Carpet::new(400, 3, 0.68, CarpetAlgorithm::Wedge, false).go();
    draw_one(400, 2.0, 7, 680, &CarpetAlgorithm::Wedge);
}

#[allow(dead_code)]
fn try_algorithms() {
    // draw_one(400, 2.0, 7, 0.68, CarpetAlgorithm::FlatSquare);
    // draw_one(20, 20.0, 10, 0.68, CarpetAlgorithm::FlatSquare);

    let size = 400;
    let min_length = 3;
    let mult = 680;
    for algorithm in [CarpetAlgorithm::Simple, CarpetAlgorithm::Wedge, CarpetAlgorithm::FlatSquare].iter() {
        let start_time = Instant::now();
        let mut carpet = Carpet::new(size, min_length, mult, algorithm, false);
        carpet.go();
        println!("{}: {:?}", algorithm.to_name(), Instant::now() - start_time);
        carpet.grid.write(&format!("T:/Compare/{}", algorithm.to_name()));
    }
}

#[allow(dead_code)]
fn try_combine_carpets() {
    // let color_func = &|count| count_to_color_black_white(count);
    let size = 300;
    let display_mult = 2.0;
    let min_length_a = 7;
    let min_length_b = 7;
    let mult_a = 680;
    let mult_b = 690;
    let algorithm = &CarpetAlgorithm::Wedge;
    let carpet_a = create_one(size, min_length_a, mult_a, algorithm);
    // carpet_a.draw(2.0);
    // carpet_a.write_grid();
    let carpet_b = create_one(size, min_length_b, mult_b, algorithm);
    // carpet_b.write_grid();
    // let bits_a = carpet_a.to_bit_vec();
    // let bits_b = carpet_b.to_bit_vec();
    // dbg!(&bits_a);
    // carpet_a.copy_set_from_bits(&bits_a).write_grid_labeled("from bits");
    // carpet_a.copy_set_from_bits(&bits_a).draw(2.0);
    //carpet_a.negative().draw(2.0);
    let combo = carpet_a.xor(&carpet_b);
    let layout_grid= Grid::arrange(3, 0, 8, &vec![carpet_a.grid, carpet_b.grid, combo.grid]);
    // draw_grid(&layout_grid, display_mult, color_func);
    draw_grid_normal(&layout_grid, display_mult);
}
*/

#[allow(dead_code)]
fn list_unique_grid_mults(size: usize, min_length: usize, mult_min: usize, mult_max: usize, mult_inc: usize) -> Vec<usize> {
    let mut mults = vec![];
    let mut mult = mult_min;
    let mut previous_found_mult = None;
    while mult <= mult_max {
        if previous_found_mult.map_or(true, |previous_found_mult|!equivalent_carpet(size, min_length, previous_found_mult, mult)) {
            // if Carpet::grid_exists(size, min_length, mult) {
                mults.push(mult);
                previous_found_mult = Some(mult);
            // }
        }
        mult += mult_inc;
    }
    mults
}

fn equivalent_carpet(size: usize, min_length: usize, mult_a: usize, mult_b: usize) -> bool {
    //bg!(size, min_length, mult_a, mult_b);
    let (mult_a, mult_b) = (mult_a as f32 / 1_000.0, mult_b as f32 / 1_000.0);
    let mut length_a = size as f32;
    let mut length_b = length_a;
    loop {
        let length_a_round = length_a.round() as usize;
        let length_b_round = length_b.round() as usize;
        //bg!(length_a, length_b);
        if length_a_round != length_b_round {
            return false;
        }
        if length_a_round < min_length || length_b_round < min_length {
            return true;
        }
        length_a *= mult_a;
        length_b *= mult_b;
    }
}

/*
#[allow(dead_code)]
fn try_grayscale_256() {
    let size = 300;
    let min_length = 5;
    let mult = 700;

    let display_mult= 2.0;

    let grid = Carpet::read_or_make_grid(size, min_length, mult);
    //bg!(&grid.cell_values);

    dbg!(grid.max_value());

    let grid = grid.copy_with_value_function(&|count| count & 100, 0);

    let grid = grid.copy_normalize(255);
    //bg!(&grid.cell_values);

    grid.draw(display_mult, &|value| grayscale_256_to_color_1(*value));
}

#[allow(dead_code)]
fn make_gallery() {
    /*
    let size = 100;
    let margin_size = 5;
    let display_mult = 2.0;
    let min_length = 3;
    let mult_min = 0.63;
    let mult_max = 0.80;
    let carpet_count = 36;
    let col_count = 9;
     */
    /*
    let size = 200;
    let margin_size = 10;
    let display_mult = 1.0;
    let min_length = 5;
    let mult_min = 0.61;
    let mult_max = 0.75;
    let carpet_count = 36;
    let col_count = 9;
    */
    let size = 800;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 7;
    let mult_min = 680;
    let mult_max = 682;
    let carpet_count = 2;
    let col_count = 2;
    let algorithm = &CarpetAlgorithm::Wedge;
    let mult_inc = (mult_max - mult_min) / (carpet_count - 1);
    let mut grids = Vec::with_capacity(carpet_count);
    let mut mult = mult_min;
    for _ in 0..carpet_count {
        grids.push(create_one(size, min_length, mult, algorithm).grid);
        mult += mult_inc;
    }
    let layout_grid = Grid::arrange(col_count, 0, margin_size, &grids);
    draw_grid_normal(&layout_grid, display_mult);
}

#[allow(dead_code)]
fn make_gallery_other_mod() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 2.0;
    let min_length = 3;
    let mult= 670;
    let col_count = 4;
    let size = 200;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);

    let mut grids = vec![];
    for modulus in 2..=5 {
        grids.push(carpet.grid.copy_with_value_function(&|count| count % modulus != 0, false));
    }
    let mut negative_grids = vec![];
    for grid in grids.iter() {
        negative_grids.push(grid.copy_negative());
    }
    grids.append(&mut negative_grids);
    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mult_mod_xor() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 2.0;
    let min_length = 3;
    let mult= 670;
    let col_count = 4;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);

    let mut grids = vec![];
    for modulus in 2..=5 {
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus != 0, false);
        let grid = grid.copy_negative();
        grids.push(grid);
    }

    grids.push(grids[0].copy_xor(&grids[1]));
    grids.push(grids[1].copy_xor(&grids[2]));
    grids.push(grids[2].copy_xor(&grids[3]));
    grids.push(grids[3].copy_xor(&grids[0]));

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_large() {
    let size = 800;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 7;
    let mult= 670;
    let col_count = 2;
    let algorithm = &CarpetAlgorithm::Wedge;
    let mods = vec![4, 5];

    let carpet = create_one(size, min_length, mult, algorithm);
    let mut grids = Vec::with_capacity(mods.len());
    for modulus in mods.iter() {
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_medium() {
    let size = 400;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 4;
    let mult= 690;
    let col_count = 4;
    let grid_count = 8;
    let algorithm = &CarpetAlgorithm::Wedge;

    let carpet = create_one(size, min_length, mult, algorithm);
    let mut grids = Vec::with_capacity(grid_count);
    for i in 0..grid_count {
        let modulus = i + 2;
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_4_combo_1() {
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 1.0;
    let min_length = 4;
    let mult_min= 670;
    let mult_max= 730;
    let col_count = 9;
    let grid_count = col_count * 4;
    let algorithm = &CarpetAlgorithm::Wedge;

    let mult_inc = (mult_max - mult_min) / (col_count - 1);

    let mut grids = Vec::with_capacity(grid_count);
    let mut mult = mult_min;
    for _ in 0..col_count {
        let carpet = create_one(size, min_length, mult, algorithm);
        let modulus = 4;
        let grid = carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
        mult += mult_inc;
    }
    for offset in 1..=3 {
        for i in 0..col_count {
            let mut second_grid_index = i + offset;
            if second_grid_index >= col_count {
                second_grid_index -= col_count;
            }
            let grid = grids[i].copy_xor(&grids[second_grid_index]);
            grids.push(grid);
        }
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn make_gallery_mod_4_combo() {
    let label = "add_display_mod_4";
    let size = 200;
    let margin_size = size / 20;
    let display_mult = 1.0;
    // let min_length = 4;
    let min_length = 3;
    let mult_min= 670;
    let mult_max= 730;
    let modulus = 4;
    let col_count = 9;
    let grid_count = col_count * 4;
    // let algorithm = &CarpetAlgorithm::Simple;

    let mult_inc = (mult_max - mult_min) / (col_count - 1);

    let mut ref_grids = Vec::with_capacity(col_count);
    let mut grids = Vec::with_capacity(grid_count);
    let mut mult = mult_min;
    for _ in 0..col_count {
        let grid = Carpet::read_or_make_grid(size, min_length, mult);
        ref_grids.push(grid.clone());
        // These are the original grids with no modulus effect and thus the full counts.
        let grid = grid.copy_with_value_function(&|count| count % modulus == 0, false);
        grids.push(grid);
        mult += mult_inc;
    }

    // Make three more rows of derived grids. The first row combines reference grid 0 with 1, 1
    // with 2, and so on. The next row combines 0 with 2, 1 with 3, and so on.
    for offset in 1..=3 {
        for i in 0..col_count {
            let mut second_grid_index = i + offset;
            if second_grid_index >= col_count {
                second_grid_index -= col_count;
            }
            let grid = ref_grids[i]
                .copy_add(&ref_grids[second_grid_index])
                .copy_with_value_function(&|count| count % modulus == 0, false);
            grids.push(grid);
        }
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);

    let file_name = format!("{}/carpet_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, mult_min, mult_max, label);
    image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);

    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn debug_edge_issue() {
    // let label = "debug_edge_issues";
    let size = 40;
    let margin_size = 5;
    let display_mult = 10.0;
    let min_length = 4;
    let mult= 670;
    let modulus = 2;
    let col_count = 3;
    let grid_count = col_count + 1;

    let mut carpets = Vec::with_capacity(grid_count);
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::Simple));
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::Wedge));
    carpets.push(create_one(size, min_length, mult, &CarpetAlgorithm::FlatSquare));

    let mut grids = carpets.iter()
        .map(|carpet| carpet.grid.copy_with_value_function(&|count| count % modulus == 0, false))
        .collect::<Vec<_>>();
    let xor_grid = grids[0].copy_xor(&grids[1]);
    grids.push(xor_grid);
    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);

    // let file_name = format!("{}/carpet_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, (mult_min * 1_000.0) as usize, (mult_max * 1_000.0) as usize, label);
    // image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);

    layout_grid.draw(display_mult, &|value| bool_to_color_black_white(*value));
}

#[allow(dead_code)]
fn debug_corner_algorithm() {
    let label = "debug_corner_algorithm";
    let size = 200;
    let margin_size = 2;
    let display_mult = 600.0 / size as f64;
    let min_length = 5;
    let mult= 750;
    // Timing for size = 200, min_length = 5, mult = 750, running as Release:
    //   Simple: 12.2780701s
    //   Corner: 752µs
    // Corner is 16,327 times faster.
    let col_count = 2;
    let grid_count = col_count;

    let mut carpets = Vec::with_capacity(grid_count);

    for algorithm in [CarpetAlgorithm::Simple, CarpetAlgorithm::Corner].iter() {
        let start_time = Instant::now();
        let carpet = create_one(size, min_length, mult, algorithm);
        println!("{}: {:?}", algorithm.to_name(), Instant::now() - start_time);
        let file_name = format!("{}/carpet_{}_{}_{}_{} {}.txt", PATH_IMAGE_FILES, size, min_length, mult, label, algorithm.to_name().to_lowercase());
        carpet.grid.write(&file_name);
        carpets.push(carpet);
    }

    carpets[0].grid.assert_equal(&carpets[1].grid);

    let grids = carpets.iter()
        .map(|carpet| carpet.grid.copy_normalize(255))
        .collect::<Vec<_>>();
    let layout_grid = Grid::arrange(col_count, 0, margin_size, &grids);

    // let file_name = format!("{}/carpet_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, (mult_min * 1_000.0) as usize, (mult_max * 1_000.0) as usize, label);
    // image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);

    layout_grid.draw(display_mult, &|value| grayscale_256_to_color_1(*value));
}

#[allow(dead_code)]
fn time_corner_algorithm_vs_read_file() {
    let label = "time_corner_algorithm";
    let size = 200;
    let min_length = 5;
    let mult= 750;

    let start_time = Instant::now();
    let carpet = create_one(size, min_length, mult, &CarpetAlgorithm::Corner);
    println!("Create: {:?}", Instant::now() - start_time);

    let file_name = format!("{}/carpet_{}.txt", PATH_IMAGE_FILES, label);
    let start_time = Instant::now();
    carpet.grid.write(&file_name);
    println!("Write: {:?}", Instant::now() - start_time);

    let start_time = Instant::now();
    let read_grid = Grid::read_optional(&file_name).unwrap();
    println!("Read: {:?}", Instant::now() - start_time);

    carpet.grid.assert_equal(&read_grid);
}
*/

#[allow(dead_code)]
fn time_corner_algorithm_vary_mult() {
    let size = 800;
    let min_length = 5;
    let mult_min= 550;
    let mult_max = 990;
    let mult_inc = 20;

    let mut mult = mult_min;
    while mult <= mult_max {
        let start_time = Instant::now();
        let grid = create_one(size, min_length, mult,None).grid;
        let max = grid.max_value();
        println!("{} (max {}): {:?}", mult, max, Instant::now() - start_time);
        mult += mult_inc;
    }
}

/*
#[allow(dead_code)]
fn generate_grids_parallel() {
    let size = 300;
    let min_length = 5;
    let mult_min = 550;
    let mult_max = 750;
    let mult_inc = 1;
    // let thread_limit = 30;
    let thread_limit = 2_000;

    let mut mult = mult_min;
    let mut handles = vec![];
    while mult <= mult_max {
        if !Carpet::grid_exists(size, min_length, mult) {
            println!("[{}] {}", handles.len(), mult);
            handles.push(thread::spawn(move || { Carpet::read_or_make_grid(size, min_length, mult) }));
            if handles.len() == thread_limit - 1 {
                break;
            }
        }
        mult += mult_inc;
    }
    for handle in handles {
        let _ = handle.join();
    }
}

#[allow(dead_code)]
fn draw_big_gallery() {
    // let size = 200;
    // let col_count = 10;
    let size = 300;
    let col_count = 7;
    let min_length = 5;
    let mult_min = 800;
    let mult_max = 990;
    let mult_inc = 1;

    let margin_size = size / 20;
    let modulus = 4;

    let mut grids = vec![];
    for mult in list_unique_grid_mults(size, min_length, mult_min, mult_max, mult_inc).iter() {
        println!("{}", mult);
        let grid = create_one(size, min_length, *mult, &CarpetAlgorithm::Corner).grid;
        grids.push(grid.copy_with_value_function(&|count| count % modulus == 0, false));
    }

    let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    let file_name = format!("{}/carpet_big_gallery_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, mult_min, mult_max);
    image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);
}
*/

#[allow(dead_code)]
fn draw_big_gallery_256() {
    // let size = 200;
    // let col_count = 10;
    let size = 300;
    let col_count = 7;
    let min_length = 5;
    let mult_min = 800;
    let mult_max = 990;
    let mult_inc = 1;

    let margin_size = size / 20;
    let modulus = 100;

    let mut grids = vec![];
    for mult in list_unique_grid_mults(size, min_length, mult_min, mult_max, mult_inc).iter() {
        println!("{}", mult);
        let grid = create_one(size, min_length, *mult, Some(modulus)).grid;
        println!("{}", grid.max_value());
        // let grid = grid.copy_with_value_function(&|count| count & modulus,0);
        let grid = grid.copy_normalize(255);
        grids.push(grid);
    }

    let layout_grid = Grid::arrange(col_count, 0, margin_size, &grids);
    let file_name = format!("{}/carpet_big_gallery_{}_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, mult_min, mult_max, modulus);
    image_util::save_grid(&layout_grid, &file_name, &|value| grayscale_256_to_color_256(*value), 0, None);
}

/*
#[allow(dead_code)]
fn draw_combo_gallery() {
    let mut rng = thread_rng();

    let size = 200;
    let col_count = 10;
    let combo_count = 1_000;
    // let size = 300;
    // let col_count = 7;
    // let combo_count = 100;
    let min_length = 5;
    let mult_min = 550;
    let mult_max = 900;
    let mult_inc = 1;

    let margin_size = size / 20;
    // let modulus = 4;

    let mults = list_unique_grid_mults(size, min_length, mult_min, mult_max, mult_inc);
    //bg!(mults.len());

    let mut grids = vec![];
    for i in 0..combo_count {
        let mult_a = mults[rng.gen_range(0..mults.len())];
        let mult_b = mults[rng.gen_range(0..mults.len())];
        let grid_a = Carpet::read_grid_optional(size, min_length, mult_a).unwrap();
        let grid_b = Carpet::read_grid_optional(size, min_length, mult_b).unwrap();

        // let grid = grid_a.copy_add(&grid_b).copy_with_value_function(&|count| count % modulus == 0, false);

        let grid = grid_a.copy_add(&grid_b);
        let grid = grid.copy_with_value_function(&|count| count & 100, 0);
        let grid = grid.copy_normalize(255);
        // grid.draw(display_mult, &|value| grayscale_256_to_color_1(*value));

        grids.push(grid);
        println!("[{}]: {} + {}", i, mult_a, mult_b);
    }
    // let layout_grid = Grid::arrange(col_count, false, margin_size, &grids);
    let layout_grid = Grid::arrange(col_count, 0, margin_size, &grids);
    let file_name = format!("{}/carpet_combo_gallery.png_{}_{}_{}_{}.png", PATH_IMAGE_FILES, size, min_length, mult_min, mult_max);
    // image_util::save_grid(&layout_grid, &file_name, &|value| bool_to_color_256_black_white(*value), 0, None);
    image_util::save_grid(&layout_grid, &file_name, &|value| grayscale_256_to_color_256(*value), 0, None);
}
*/

#[allow(dead_code)]
fn anim_flow() {
    let size = 200;
    // let min_length = 5;
    let mult = 680;

    let mod_min = 2;
    // let mod_max = 300;
    let mod_inc = 1;

    let margin_size = size / 20;
    // let display_width_mult = 2.0;
    let frame_seconds = 0.2;

    let size = 800;
    let min_length = 7;
    // let mod_max = 400;
    let display_width_mult = 1.0;

    let display_width = (size + (4 * margin_size)) as f64 * display_width_mult;
    let display_height = display_width;

    let ref_grid = Carpet::read_or_make_grid(size, min_length, mult, None);
    let mod_max = ref_grid.max_value() / 40;

    let mut modulus = mod_min;
    let mut frames = vec![];
    while modulus <= mod_max {
        let grid = ref_grid.copy_with_value_function(&|count| count % modulus, 0);
        let grid = grid.copy_normalize(255);
        let layout_grid = Grid::arrange(1, 0, margin_size, &vec![grid]);
        frames.push(layout_grid.as_frame(display_width, display_height, frame_seconds, &|value| grayscale_256_to_color_1(*value)));
        println!("frame {} / {}", modulus, mod_max);
        modulus += mod_inc;
    }

    let back_color = count_to_color_black_white(&0);
    let additive = false;
    Renderer::display_additive("Carpet", display_width, display_height, back_color, frames, additive);
}
