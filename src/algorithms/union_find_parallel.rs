use std::collections::BTreeMap;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

use util::*;
use std::cmp::Reverse;
use std::sync::{Arc, Mutex};

pub fn main() {
    try_union_single_thread();
}

pub struct ParallelQuickUnion {
    pub nodes: Vec<Arc<Mutex<ParallelQuickUnionNode>>>,
}

struct ParallelQuickUnionNode {
    root: usize,
    size: usize,
}

impl ParallelQuickUnion {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(Arc::new(Mutex::new(ParallelQuickUnionNode { root: i, size: 1 })));
        }
        Self {
            nodes,
        }
    }

    #[inline]
    pub fn union(&mut self, p: usize, q: usize) -> bool {
        while p != self.nodes[p] {
            p = self.nodes[p];
        }

        let (root_p, root_q) = (self.root(p), self.root(q));
        if root_p != root_q {
            let root = if self.sizes[root_p] < self.sizes[root_q] {
                self.nodes[root_p] = root_q;
                self.sizes[root_q] += self.sizes[root_p];
                self.sizes[root_p] = 0;
                root_q
            } else {
                self.nodes[root_q] = root_p;
                self.sizes[root_p] += self.sizes[root_q];
                self.sizes[root_q] = 0;
                root_p
            };
            if self.path_compression {
                self.compress_path(p, root);
                self.compress_path(q, root);
            }
        }
        // self.union_time += Instant::now() - start_time;
    }

    #[inline]
    pub fn is_connected(&mut self, p: usize, q: usize) -> bool {
        // let start_time = Instant::now();
        let is_connected = self.root(p) == self.root(q);
        // self.is_connected_time += Instant::now() - start_time;
        is_connected
    }

    #[inline]
    pub fn root(&self, mut p: usize) -> usize {
        while p != self.nodes[p] {
            p = self.nodes[p];
        }
        p
    }

    /*
    #[inline]
    fn root_with_compression(&mut self, p: usize) -> usize {
        let mut root = p;
        let mut depth = 0;
        while root != self.nodes[root] {
            depth += 1;
            root = self.nodes[root];
        }
        if depth > 1 {
            let mut i = p;
            while i != self.nodes[i] {
                let next_i = self.nodes[i];
                self.nodes[i] = root;
                i = next_i;
            }
        }
        root
    }
     */

    #[inline]
    fn compress_path(&mut self, mut i: usize, root: usize) {
        while i != self.nodes[i] {
            let next_i = self.nodes[i];
            self.nodes[i] = root;
            i = next_i;
        }
    }

    pub fn tree_depth_mean(&self) -> f32 {
        let mut sum = 0;
        for i in 0..self.nodes.len() {
            let mut p = i;
            while p != self.nodes[p] {
                sum += 1;
                p = self.nodes[p];
            }
        }
        sum as f32 / self.nodes.len() as f32
    }

    pub fn get_components(&self) -> Vec<Vec<usize>> {
        let mut map = BTreeMap::new();
        for i in 0..self.nodes.len() {
            map.entry(self.root(i)).or_insert(vec![]).push(i);
        }
        map.values().map(|v| v.clone()).collect()
    }

    pub fn get_roots_of_largest_components(&self, limit: usize) -> Vec<usize> {
        let mut components: Vec<(usize, usize)> = self.sizes.iter().enumerate()
            .filter(|(_index, size)| **size > 1)
            .map(|(index, size)| (index, *size))
            .collect::<Vec<_>>();
        components.sort_by_cached_key(|(_index, size)| Reverse(*size));
        components.truncate(limit);
        components.iter().take(limit).map(|(index, _size)| *index).collect()
    }

    pub fn print_components(&self) {
        let mut map = BTreeMap::new();
        for i in 0..self.nodes.len() {
            map.entry(self.root(i)).or_insert(vec![]).push(i);
        }
        println!("{}", self.get_components().iter()
            .map(|component| format!("{{ {} }}", component.iter().join(", ")))
            .join(" "));
    }
}

#[allow(dead_code)]
fn try_union_find() {
    let mut rng = rand::thread_rng();
    let size = 10;
    let mut qf = QuickFind::new(size);
    let mut qu = QuickUnion::new(size);
    let mut wqu = WeightedQuickUnion::new(size, false);
    let mut wqup = WeightedQuickUnion::new(size, true);
    qf.print_components();
    qu.print_components();
    wqu.print_components();
    wqup.print_components();
    for _ in 0..size {
        let p = rng.gen_range(0..size);
        let q = rng.gen_range(0..size);
        qf.union(p, q);
        qu.union(p, q);
        wqu.union(p, q);
        wqup.union(p, q);
        println!("\nunion ({}, {})", p, q);
        qf.print_components();
        qu.print_components();
        wqu.print_components();
        wqup.print_components();
    }
}

#[allow(dead_code)]
fn compare_performance() {
    let mut rng = rand::thread_rng();
    for size in [50_000, 100_000, 200_000, 400_000].iter() {
        let size = *size;
        let mut connected_count = 0;
        let mut qf = QuickFind::new(size);
        let mut qu = QuickUnion::new(size);
        let mut wqu = WeightedQuickUnion::new(size, false);
        let mut wqup = WeightedQuickUnion::new(size, true);
        for _ in 0..size {
            let p = rng.gen_range(0..size);
            let q = rng.gen_range(0..size);
            qf.union(p, q);
            qu.union(p, q);
            wqu.union(p, q);
            wqup.union(p, q);
            let p = rng.gen_range(0..size);
            let q = rng.gen_range(0..size);
            let qf_is_connected = qf.is_connected(p, q);
            assert_eq!(qf_is_connected, qu.is_connected(p, q));
            assert_eq!(qf_is_connected, wqu.is_connected(p, q));
            assert_eq!(qf_is_connected, wqup.is_connected(p, q));
            if qf_is_connected {
                connected_count += 1;
            }
        }
        // uf.print_components();
        // qu.print_components();
        println!("uf union = {:?}; qu union = {:?}; qu con = {:?}, qu depth = {}, wqu union = {:?}; wqu con = {:?}, wqu depth = {}, wqup union = {:?}; wqup con = {:?}, wqup depth = {}, connected_count = {}",
                 qf.union_time,
                 qu.union_time, qu.is_connected_time, format::format_float(qu.tree_depth_mean(), 2),
                 wqu.union_time, wqu.is_connected_time, format::format_float(wqu.tree_depth_mean(), 2),
                 wqup.union_time, wqup.is_connected_time, format::format_float(wqup.tree_depth_mean(), 2),
                 connected_count);
    }
}

#[allow(dead_code)]
fn compare_performance_fastest() {
    let mut rng = rand::thread_rng();
    let mut sizes = vec![];
    let mut size = 10;
    let mult = 10;
    for _ in 0..12 {
        sizes.push(size);
        size *= mult;
    }
    for size in sizes.iter() {
        let size = *size;
        // let mut connected_count = 0;
        let mut inputs = Vec::with_capacity(size);
        // let start_time = Instant::now();
        for _ in 0..size {
            inputs.push((rng.gen_range(0..size), rng.gen_range(0..size), rng.gen_range(0..size), rng.gen_range(0..size)));
        }
        // let inputs_elapsed = Instant::now() - start_time;
        let start_time = Instant::now();
        let mut wqu = WeightedQuickUnion::new(size, true);
        for i in 0..size {
            wqu.union(inputs[i].0, inputs[i].1);
            // if wqu.is_connected(inputs[i].2, inputs[i].3) {
            //     connected_count += 1;
            //}
        }
        // println!("size = {}; inputs_elapsed = {:?}; elapsed = {:?}; connected_count = {}", format::format_count(size), inputs_elapsed, Instant::now() - start_time, format::format_count(connected_count));
        let elapsed = Instant::now() - start_time;
        let per_union = elapsed / size as u32;
        println!("size = {}; per union = {:?}; elapsed = {:?}", format::format_count(size), per_union, elapsed);
    }
    /*
    size = 10; inputs_elapsed = 7.4µs; elapsed = 9.6µs; connected_count = 4
    size = 100; inputs_elapsed = 6.3µs; elapsed = 6.8µs; connected_count = 16
    size = 1,000; inputs_elapsed = 31.1µs; elapsed = 32.7µs; connected_count = 167
    size = 10,000; inputs_elapsed = 615.5µs; elapsed = 318.7µs; connected_count = 1,578
    size = 100,000; inputs_elapsed = 4.215ms; elapsed = 4.7196ms; connected_count = 16,089
    size = 1,000,000; inputs_elapsed = 30.7133ms; elapsed = 63.9324ms; connected_count = 162,187
    size = 10,000,000; inputs_elapsed = 579.3957ms; elapsed = 1.586015s; connected_count = 1,619,168
    size = 100,000,000; inputs_elapsed = 4.2080786s; elapsed = 37.9052902s; connected_count = 16,185,211
    size = 1,000,000,000; inputs_elapsed = 29.9572552s; elapsed = 469.9692364s; connected_count = 161,908,437
    It died from memory attempting 10 billion. One billion was using about 45GB.
    I could use u32 and have up to 4,294,967,296 nodes, but four billion nodes would want about
    90 GB (half size per node but four times as many nodes as the last successful test above).
     */

}

#[allow(dead_code)]
fn compare_performance_fixed_size() {
    // About 10ns per union or 100 million per second.
    // 8ns per union if union() is inlined, or 125 million per second.
    // let mut rng = rand::thread_rng();
    let size = 1_000;
    // let union_count = 100_000;
    // let union_count = 100_000_000;
    let union_count = 10_000_000_000;
    let steps = 50;
    let union_chunk_size = union_count / steps;
    let mut wqu = WeightedQuickUnion::new(size, true);
    for _ in 0..steps {
        let pairs = random_x_y_pairs(size, union_chunk_size);
        let start_time = Instant::now();
        for i in 0..union_chunk_size {
            wqu.union(pairs[i].0, pairs[i].1);
        }
        let elapsed = Instant::now() - start_time;
        let per_union = elapsed / union_chunk_size as u32;
        let unions_per_second = (1.0 / (elapsed.as_secs_f64() / union_chunk_size as f64)) as usize;
        println!("per union = {:?}; elapsed = {:?}; union/sec = {}", per_union, elapsed, format::format_count(unions_per_second));
    }
}

#[allow(dead_code)]
fn time_mutex() {
    let size = 1_000;
    let union_count = 100_000;
    // let union_count = 100_000_000;
    // let union_count = 10_000_000_000;
    println!("size = {}; union_count = {}", format::format_count(size), format::format_count(union_count));

    let mut union = Vec::with_capacity(size);
    let start_time = Instant::now();
    for i in 0..size {
        // 64ns for each item.
        union.push(Arc::new(Mutex::new(i))); //
        // union.push(i);
    }
    let elapsed_per_item = (Instant::now() - start_time) / size as u32;
    println!("create union (per item) = {:?}", elapsed_per_item);

    let pairs = random_x_y_pairs(size, union_count);
    let start_time = Instant::now();
    for pair in pairs {
        // 19ns to lock both.
        // union[pair.0].lock().unwrap();
        // union[pair.1].lock().unwrap();
        match union[pair.0].try_lock() {
            Ok(a) => {},
            Err(_) => { continue; },
        }
        match union[pair.1].try_lock() {
            Ok(a) => {},
            Err(_) => { continue; },
        }
    }
    let elapsed_per_pair = (Instant::now() - start_time) / union_count as u32;
    println!("lock (per pair) = {:?}", elapsed_per_pair);
}

#[allow(dead_code)]
fn random_x_y_pairs(size: usize, pair_count: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::thread_rng();
    let mut pairs = Vec::with_capacity(pair_count);
    for _ in 0..pair_count {
        pairs.push((rng.gen_range(0..size), rng.gen_range(0..size)));
    }
    pairs
}
