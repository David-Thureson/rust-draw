use std::collections::BTreeMap;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

use util::*;
use std::cmp::Reverse;

pub fn main() {
    // try_union_find();
    // compare_performance();
    compare_performance_fastest();
}

pub struct QuickFind {
    pub nodes: Vec<usize>,
    pub union_time: Duration,
}

impl QuickFind {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(i);
        }
        Self {
            nodes,
            union_time: Duration::zero(),
        }
    }

    pub fn union(&mut self, p: usize, q: usize) {
        let start_time = Instant::now();
        let comp_p = self.nodes[p];
        let comp_q = self.nodes[q];
        if comp_p != comp_q {
            for i in 0..self.nodes.len() {
                if self.nodes[i] == comp_p {
                    self.nodes[i] = comp_q;
                }
            }
        }
        self.union_time += Instant::now() - start_time;
    }

    pub fn is_connected(&self, p: usize, q: usize) -> bool {
        self.nodes[p] == self.nodes[q]
    }

    pub fn print_components(&self) {
        let mut map = BTreeMap::new();
        for (i, value) in self.nodes.iter().enumerate() {
            map.entry(value).or_insert(vec![]).push(i);
        }
        println!("{}", map.values()
            .map(|component| format!("{{ {} }}", component.iter().join(", ")))
            .join(" "));
    }
}

pub struct QuickUnion {
    pub nodes: Vec<usize>,
    pub union_time: Duration,
    pub is_connected_time: Duration,
}

impl QuickUnion {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(i);
        }
        Self {
            nodes,
            union_time: Duration::zero(),
            is_connected_time: Duration::zero(),
        }
    }

    pub fn union(&mut self, p: usize, q: usize) {
        let start_time = Instant::now();
        let root_p = self.root(p);
        let root_q = self.root(q);
        if root_p != root_q {
            self.nodes[root_p] = root_q;
        }
        self.union_time += Instant::now() - start_time;
    }

    //#[inline]
    pub fn is_connected(&mut self, p: usize, q: usize) -> bool {
        let start_time = Instant::now();
        let is_connected = self.root(p) == self.root(q);
        self.is_connected_time += Instant::now() - start_time;
        is_connected
    }

    #[inline]
    fn root(&self, mut p: usize) -> usize {
        while p != self.nodes[p] {
            p = self.nodes[p];
        }
        p
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

    pub fn print_components(&self) {
        let mut map = BTreeMap::new();
        for i in 0..self.nodes.len() {
            map.entry(self.root(i)).or_insert(vec![]).push(i);
        }
        println!("{}", map.values()
            .map(|component| format!("{{ {} }}", component.iter().join(", ")))
            .join(" "));
    }
}

pub struct WeightedQuickUnion {
    pub nodes: Vec<usize>,
    pub sizes: Vec<usize>,
    pub path_compression: bool,
    pub union_time: Duration,
    pub is_connected_time: Duration,
}

impl WeightedQuickUnion {
    pub fn new(size: usize, path_compression: bool) -> Self {
        let mut nodes = Vec::with_capacity(size);
        let mut sizes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(i);
            sizes.push(1);
        }
        Self {
            nodes,
            sizes,
            path_compression,
            union_time: Duration::zero(),
            is_connected_time: Duration::zero(),
        }
    }

    pub fn union(&mut self, p: usize, q: usize) {
        // if p == 100 || p == 101 || q == 100 || q == 101 {
        //     dbg!(p, q);
        // }

        // let start_time = Instant::now();
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

    //#[inline]
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
        let mut connected_count = 0;
        let mut inputs = Vec::with_capacity(size);
        let start_time = Instant::now();
        for _ in 0..size {
            inputs.push((rng.gen_range(0..size), rng.gen_range(0..size), rng.gen_range(0..size), rng.gen_range(0..size)));
        }
        let inputs_elapsed = Instant::now() - start_time;
        let start_time = Instant::now();
        let mut wqu = WeightedQuickUnion::new(size, true);
        for i in 0..size {
            wqu.union(inputs[i].0, inputs[i].1);
            if wqu.is_connected(inputs[i].2, inputs[i].3) {
                connected_count += 1;
            }
        }
        println!("size = {}; inputs_elapsed = {:?}; elapsed = {:?}; connected_count = {}", format::format_count(size), inputs_elapsed, Instant::now() - start_time, format::format_count(connected_count));
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
