use std::collections::BTreeMap;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

pub fn main() {
    // try_union_find();
    compare_performance();
}

pub struct UnionFind {
    pub nodes: Vec<usize>,
    pub union_time: Duration,
}

impl UnionFind {
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

fn try_union_find() {
    let mut rng = rand::thread_rng();
    let size = 10;
    let mut uf = UnionFind::new(size);
    let mut qu = QuickUnion::new(size);
    uf.print_components();
    qu.print_components();
    for _ in 0..size {
        let p = rng.gen_range(0..size);
        let q = rng.gen_range(0..size);
        uf.union(p, q);
        qu.union(p, q);
        println!("\nunion ({}, {})", p, q);
        uf.print_components();
        qu.print_components();
    }
}

fn compare_performance() {
    let mut rng = rand::thread_rng();
    for size in [50_000, 100_000, 200_000, 400_000].iter() {
        let size = *size;
        let mut connected_count = 0;
        let mut uf = UnionFind::new(size);
        let mut qu = QuickUnion::new(size);
        for _ in 0..size {
            let p = rng.gen_range(0..size);
            let q = rng.gen_range(0..size);
            uf.union(p, q);
            qu.union(p, q);
            let p = rng.gen_range(0..size);
            let q = rng.gen_range(0..size);
            debug_assert_eq!(uf.is_connected(p, q), qu.is_connected(p, q));
            if qu.is_connected(p, q) {
                connected_count += 1;
            }
        }
        // uf.print_components();
        // qu.print_components();
        println!("union-find union = {:?}; quick-union union = {:?}; quick-union-connected = {:?}, connected_count = {}",
                 uf.union_time, qu.union_time, qu.is_connected_time, connected_count);
    }
}
