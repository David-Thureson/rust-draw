use rand::Rng;
use std::time::{Instant, Duration};
use num::{Unsigned, Zero, One};
use std::iter::Step;
use std::ops::AddAssign;
use std::fmt::{Display, Debug};
use std::convert::TryFrom;
use crate::algorithms::generic_union::GenericUnion;
use crate::algorithms::percolation::*;
use rand::distributions::uniform::SampleUniform;
use crate::algorithms::union_find::random_x_y_pairs;
use itertools::Itertools;

use crate::*;
use std::sync::mpsc;
use std::thread;

const PERCOLATION_THRESHOLD_EXPECTED: f64 = 0.592746;

pub fn main() {
    // try_percolation();
    // test_correctness();
    // find_mismatch();
    // find_largest_possible();
    // find_largest_practical_for_parallel();
    find_percolation_threshold();
}

pub struct GenericPercolation<T>
    where
        T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize> + SampleUniform,
        usize: TryFrom<T>,
        <usize as std::convert::TryFrom<T>>::Error: Debug,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pub width: T,
    pub height: T,
    pub width_usize: usize,
    pub height_usize: usize,
    pub start_node_index: T,
    pub end_node_index: T,
    pub cell_count: T,
    pub cells: Vec<bool>,
    pub union: GenericUnion<T>,
}

impl <T> GenericPercolation<T>
    where
        T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize> + SampleUniform,
        usize: TryFrom<T>,
        <usize as std::convert::TryFrom<T>>::Error: Debug,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pub fn new(width: T, height: T) -> Self {
        let cell_count = width * height;
        let mut cells= Vec::with_capacity(Self::to_usize(cell_count));
        let zero = T::zero();
        for _ in zero..cell_count {
            cells.push(false);
        }
        let one = T::one();
        let union_size = cell_count + one + one;
        let union = GenericUnion::new(union_size);
        let start_node_index = cell_count;
        let end_node_index= cell_count + one;
        let mut perc = Self {
            width,
            height,
            width_usize: usize::try_from(width).unwrap(),
            height_usize: usize::try_from(height).unwrap(),
            start_node_index,
            end_node_index,
            cell_count,
            cells,
            union,
        };

        // Connect the top virtual node to every square in the top row, and the bottom virtual node
        // to every square in the bottom row.
        let bottom_row_first_node_index = (height - one) * width;
        /*
        perc.union.union(start_node_index, zero);
        perc.open_top_row();
        perc.union.union(end_node_index, bottom_row_first_node_index);
        perc.open_bottom_row();
         */
        for x in zero..width {
            perc.union.union(start_node_index, x);
            perc.union.union(end_node_index, bottom_row_first_node_index + x);
        }
        perc
    }

    pub fn clone_new(other: &Self) -> Self {
        Self::new(other.width, other.height)
    }

    #[allow(dead_code)]
    fn open_top_row(&mut self) {
        self.open_rectangle(T::zero(),T::zero(), self.width, T::one());
    }

    #[allow(dead_code)]
    fn open_bottom_row(&mut self) {
        self.open_rectangle(T::zero(), self.height - T::one(), self.width, T::one());
    }

    pub fn open_rectangle(&mut self, left: T, top: T, width: T, height: T) {
        for y in top..top + height {
            for x in left..left + width {
                self.open(x, y);
            }
        }
    }

    pub fn open_usize(&mut self, x: usize, y: usize) -> bool {
        self.open(Self::from_usize(x), Self::from_usize(y))
    }

    pub fn open(&mut self, x: T, y: T) -> bool {
        let index_usize = self.x_y_to_index_usize(x, y);
        if self.cells[index_usize] {
            // The square is already open
            return false;
        }
        self.cells[index_usize] = true;
        let index = self.x_y_to_index(x, y);
        let (zero, one) = (T::zero(), T::one());
        // Up.
        if y > zero && self.is_open_x_y(x, y - one) {
            self.union.union(index, index - self.width);
        }
        // Right.
        if x < self.width - one && self.is_open_x_y(x + one, y)  {
            self.union.union(index, index + one);
        }
        // Down.
        if y < self.height - one && self.is_open_x_y(x, y + one) {
            self.union.union(index, index + self.width);
        }
        // Left.
        if x > zero && self.is_open_x_y(x - one, y) {
            self.union.union(index, index - one);
        }
        true
    }

    #[inline]
    pub fn x_y_to_index(&self, x: T, y: T) -> T {
        (y * self.width) + x
    }

    #[inline]
    pub fn x_y_to_index_usize(&self, x: T, y: T) -> usize {
        usize::try_from((y * self.width) + x).unwrap()
    }

    #[inline]
    pub fn is_open_x_y(&self, x: T, y: T) -> bool {
        self.cells[self.x_y_to_index_usize(x, y)]
    }

    #[inline]
    pub fn to_usize(val: T) -> usize {
        usize::try_from(val).unwrap()
    }

    #[inline]
    pub fn from_usize(val: usize) -> T {
        T::try_from(val).unwrap()
    }

    pub fn get_steps_to_percolation(&mut self, pairs: &[(usize, usize)]) -> Result<usize, String> {
        let mut step_count = 0;
        for (x, y) in pairs.iter() {
            if self.open(Self::from_usize(*x), Self::from_usize(*y)) {
                step_count += 1;
                if self.percolates() {
                    return Ok(step_count)
                }
            }
        }
        Err("Ran out of pairs before percolation.".to_string())
    }

    pub fn get_steps_to_percolation_generate_pairs(&mut self) -> usize {
        let mut rng = rand::thread_rng();
        let zero = T::zero();
        let mut step_count = 0;
        loop {
            let (x, y) = (rng.gen_range(zero..self.width), rng.gen_range(zero..self.height));
            if self.open(x, y) {
                step_count += 1;
                if self.percolates() {
                    return step_count
                }
            }
        }
    }

    pub fn percolates(&mut self) -> bool {
        self.union.is_connected(self.start_node_index, self.end_node_index)
    }

    pub fn print(&self) {
        let zero = T::zero();
        for y in zero..self.height {
            let mut line = "".to_string();
            for x in zero..self.width {
                line.push_str(match self.block_state(x, y) {
                    PercolationBlockState::Blocked => "#",
                    PercolationBlockState::Open => ".",
                    PercolationBlockState::Filled => "%",
                });
            }
            println!("{}", line);
        }
    }

    pub fn block_state(&self, x: T, y: T) -> PercolationBlockState {
        let index_usize = self.x_y_to_index_usize(x, y);
        if self.cells[index_usize] {
            let index = self.x_y_to_index(x, y);
            if self.union.is_connected(self.start_node_index, index) {
                PercolationBlockState::Filled
            } else {
                PercolationBlockState::Open
            }
        } else {
            PercolationBlockState::Blocked
        }
    }

    #[inline]
    pub(crate) fn block_color_index(&self, largest_roots: &Vec<T>, x: T, y: T, is_last_frame: bool) -> usize
    {
        match self.block_state(x, y) {
            PercolationBlockState::Blocked => COLOR_INDEX_BLACK,
            PercolationBlockState::Open => {
                let node_index = self.x_y_to_index(x, y);
                let node_root_index = self.union.root(node_index);
                if self.union.is_connected(self.end_node_index, node_index) {
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

    pub fn run_to_completion(&mut self, max_seconds: usize) -> Vec<(T, T)> {
        let start_time = Instant::now();
        let stop_time = start_time + Duration::from_secs_f64(max_seconds as f64);
        let mut rng = rand::thread_rng();
        let mut unions = vec![];
        let zero = T::zero();
        while !self.percolates() && Instant::now() < stop_time {
            let x = rng.gen_range(zero..self.width);
            let y = rng.gen_range(zero..self.height);
            if self.open(x, y) {
                unions.push((x, y));
            }
        }
        let time_msg = if self.percolates() { "complete" } else { "RAN OUT OF TIME" };
        println!("run_to_completion(): [{}]; duration = {:?}; unions = {}", time_msg, Instant::now() - start_time, unions.len());
        unions
    }

    pub fn get_state(&self) -> (Vec<bool>, Vec<usize>) {
        (self.cells.clone(), self.union.nodes.iter().map(|x| Self::to_usize(*x)).collect())
    }
}

#[allow(dead_code)]
fn try_percolation() {
    let mut rng = rand::thread_rng();
    let width = 10u8;
    let height = 10u8;
    let mut perc = GenericPercolation::new(width, height);
    println!();
    perc.union.print_components();
    perc.print();
    while !perc.percolates() {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if perc.open(x, y) {
            println!("\nOpen {}, {}\n", x, y);
            perc.union.print_components();
            perc.print();
        }
    }
}

#[allow(dead_code)]
fn test_correctness() {
    let width = 10_000;
    let height = 10_000;
    // let width = 20;
    // let height = 20;
    let width32 = u32::try_from(width).unwrap();
    let height32 = u32::try_from(height).unwrap();
    let mut perc = PercolationGrid::new(width, height, PercolationType::TopBottom);
    perc.wrap_top_bottom = false;
    perc.wrap_left_right = false;
    let mut perc32 = GenericPercolation::new(width32, height32);
    let pairs= random_x_y_pairs(width, width * height);

    let start_time = Instant::now();
    let result = perc.get_steps_to_percolation(&pairs);
    match result {
        Ok(step_count) => { println!("perc: {} steps; {:?}", fc(step_count), Instant::now() - start_time); },
        Err(msg) => { println!("perc: {}", msg); },
    }

    let start_time = Instant::now();
    let result= perc32.get_steps_to_percolation(&pairs);
    match result {
        Ok(step_count) => { println!("perc32: {} steps; {:?}", fc(step_count), Instant::now() - start_time); },
        Err(msg) => { println!("perc32: {}", msg); },
    }
}

#[allow(dead_code)]
fn find_mismatch() {
    // let width = 1_000;
    // let height = 1_000;
    let width = 10;
    let height = 10;
    let width32 = u32::try_from(width).unwrap();
    let height32 = u32::try_from(height).unwrap();
    let mut perc = PercolationGrid::new(width, height, PercolationType::TopBottom);
    perc.wrap_top_bottom = false;
    perc.wrap_left_right = false;
    let mut perc32 = GenericPercolation::new(width32, height32);
    compare_perc_perc32(0, 0, &perc, &perc32);
    let pairs= random_x_y_pairs(width, width * height);
    for (x, y) in pairs.iter() {
        perc.open(*x, *y);
        perc32.open_usize(*x, *y);
        compare_perc_perc32(*x, *y, &perc, &perc32);
    }
}

fn compare_perc_perc32(x: usize, y: usize, perc: &PercolationGrid, perc32: &GenericPercolation<u32>) {
    println!("\nopen({}, {})\n", fc(x), fc(y));
    perc.print();
    println!();
    perc32.print();
    println!();
    let (cells, roots) = perc.get_state();
    let (cells32, roots32) = perc32.get_state();
    println!("{}", cells.iter().map(|x| if *x { "." } else { "#" }).join(""));
    println!("{}", cells32.iter().map(|x| if *x { "." } else { "#" }).join(""));
    println!();
    println!("{}", roots.iter().map(|x| x.to_string()).join(" "));
    println!("{}", roots32.iter().map(|x| x.to_string()).join(" "));
    println!();
    let mut quit = false;
    if cells != cells32 {
        println!("Mismatch in cells.");
        quit = true;
    }
    if roots != roots32 {
        println!("Mismatch in roots.");
        quit = true;
    }
    if quit {
        panic!()
    }
}

#[allow(dead_code)]
fn find_largest_possible() {
    // Starting with 106 GB free.
    let mut size_f64 = 4_000_000_000f64;
    let size_mult = 1.5;
    loop {
        let size_sqrt = size_f64.sqrt();
        let (width, height) = (size_sqrt.ceil() as u64, size_sqrt.floor() as u64);
        println!("size = {}; width = {}; height = {}", fc(width * height), fc(width), fc(height));
        let start_time = Instant::now();
        let _perc = GenericPercolation::new(width, height);
        println!("new() = {:?}", Instant::now() - start_time);
        size_f64 *= size_mult;
    }
}

#[allow(dead_code)]
fn find_largest_practical_for_parallel() {
    // Should be 0.592746.
    // Starting with 110 GB free.
    let mut size_f64 = 100_000_000f64;
    let size_mult = 1.5;
    while size_f64 < u32::max_value() as f64 {
        let size_sqrt = size_f64.sqrt();
        let (width, height) = (size_sqrt.ceil() as u32, size_sqrt.floor() as u32);
        let size = width * height;
        println!("\nsize = {}; width = {}; height = {}", fc(size), fc(width), fc(height));

        let start_time = Instant::now();
        let mut perc = GenericPercolation::new(width, height);
        println!("new = {:?}", Instant::now() - start_time);

        let start_time = Instant::now();
        let step_count = perc.get_steps_to_percolation_generate_pairs();
        let step_pct = step_count as f64 / size as f64;
        println!("percolate = {:?}; step_count = {}; step_pct = {}", Instant::now() - start_time, fc(step_count), ff(step_pct, 5));

        size_f64 *= size_mult;
    }
}

#[allow(dead_code)]
fn find_percolation_threshold() {
    // [10,000,000/100,000,000]: expected = 0.592746; actual = 0.592743; diff = -0.000003; pct = -0.000006
    let trial_count = 100_000_000;
    let thread_count = 50;
    let trials_per_thread = 50;
    let chunk_count = trial_count / (thread_count * trials_per_thread);
    let width = 1_000;
    let height = width as u32;
    let size = width * height;

    let (tx, rx) = mpsc::channel();

    let mut step_count_sum = 0;

    for chunk_index in 0..chunk_count {
        let mut threads = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let thread_tx = tx.clone();
            let thread = thread::spawn(move || {
                let mut step_count = 0;
                for _ in 0..trials_per_thread {
                    let mut perc = GenericPercolation::new(width, height);
                    step_count += perc.get_steps_to_percolation_generate_pairs();
                }
                thread_tx.send(step_count).unwrap();
            });
            threads.push(thread);
        }
        for _ in 0..threads.len() {
            // The `recv` method picks a message from the channel
            // `recv` will block the current thread if there are no messages available
            step_count_sum += rx.recv().unwrap();
        }
        let trials_so_far = thread_count * trials_per_thread * (chunk_index + 1);
        let calc_threshold = step_count_sum as f64 / (size as f64 * trials_so_far as f64);
        let diff = calc_threshold - PERCOLATION_THRESHOLD_EXPECTED;
        let pct = diff / PERCOLATION_THRESHOLD_EXPECTED;
        println!("[{}/{}]: expected = {}; actual = {}; diff = {}; pct = {}",
                 fc(trials_so_far), fc(trial_count), ff(PERCOLATION_THRESHOLD_EXPECTED, 6),
                 ff(calc_threshold, 6), ff(diff, 6), ff(pct, 6));

        // Wait for the threads to complete any remaining work.
        for thread in threads {
            thread.join().unwrap();
        }
    }
    println!("\nDone.");
}

#[allow(dead_code)]
fn find_percolation_threshold_1() {
    // Final result was: expected = 0.592746; actual = 0.592741; diff = -0.000005; pct = -0.000009
    // using one million trials of 1,000 by 1,000 grids.
    let trial_count = 1_000_000;
    let width= 1_000;
    let height = width as u32;
    let size = width * height;

    let (tx, rx) = mpsc::channel();
    let mut threads = Vec::new();

    for _ in 0..trial_count {
        let thread_tx = tx.clone();
        let thread = thread::spawn(move || {
            let mut perc = GenericPercolation::new(width, height);
            let step_count = perc.get_steps_to_percolation_generate_pairs();
            thread_tx.send(step_count).unwrap();
        });
        threads.push(thread);
    }
    println!("\nThreads created.");

    let mut step_counts = Vec::with_capacity(trial_count);

    for i in 0..threads.len() {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        let step_count = rx.recv().unwrap();
        step_counts.push(step_count);
        let step_count_sum = step_counts.iter().sum::<usize>();
        let calc_threshold = step_count_sum as f64 / (size as f64 * step_counts.len() as f64);
        let diff = calc_threshold - PERCOLATION_THRESHOLD_EXPECTED;
        let pct = diff / PERCOLATION_THRESHOLD_EXPECTED;
        println!("[{}/{}]: expected = {}; actual = {}; diff = {}; pct = {}",
                 fc(i), fc(trial_count), ff(PERCOLATION_THRESHOLD_EXPECTED, 6),
                 ff(calc_threshold, 6), ff(diff, 6), ff(pct, 6));
    }

    // Wait for the threads to complete any remaining work.
    for thread in threads {
        thread.join().unwrap();
    }
    println!("\nDone.");
}

/*
find_largest_practical_for_parallel()

size = 100,000,000; width = 10,000; height = 10,000
new = 499.3972ms
percolate = 28.3092141s; step_count = 59,239,615; step_pct = 0.59240

size = 150,001,256; width = 12,248; height = 12,247
new = 728.5498ms
percolate = 48.5348071s; step_count = 88,874,470; step_pct = 0.59249

size = 225,000,000; width = 15,000; height = 15,000
new = 1.2382906s
percolate = 77.3597489s; step_count = 133,374,780; step_pct = 0.59278

size = 337,512,012; width = 18,372; height = 18,371
new = 1.7260331s
percolate = 111.77263s; step_count = 200,099,367; step_pct = 0.59287

size = 506,250,000; width = 22,500; height = 22,500
new = 2.5096143s
percolate = 184.2833539s; step_count = 300,173,741; step_pct = 0.59294

size = 759,360,692; width = 27,557; height = 27,556
new = 3.8598415s
percolate = 297.6558581s; step_count = 450,111,039; step_pct = 0.59275

size = 1,139,062,500; width = 33,750; height = 33,750
new = 5.6995546s
percolate = 481.4363099s; step_count = 675,292,846; step_pct = 0.59285

size = 1,708,623,560; width = 41,336; height = 41,335
new = 8.5412516s
percolate = 723.3059033s; step_count = 1,012,644,371; step_pct = 0.59267

size = 2,562,890,625; width = 50,625; height = 50,625
new = 13.0710212s
percolate = 1087.7479362s; step_count = 1,519,110,420; step_pct = 0.59273

size = 3,844,310,006; width = 62,003; height = 62,002
new = 20.3549162s
percolate = 1650.930519s; step_count = 2,278,751,216; step_pct = 0.59276
*/