use std::collections::BTreeMap;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

pub fn main() {
    try_union_find();
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

fn try_union_find() {
    let mut rng = rand::thread_rng();
    let size = 10;
    let mut uf = UnionFind::new(size);
    uf.print_components();
    for _ in 0..size {
        let p = rng.gen_range(0..size);
        let q = rng.gen_range(0..size);
        uf.union(p, q);
        println!("\nunion ({}, {})", p, q);
        uf.print_components();
    }
}