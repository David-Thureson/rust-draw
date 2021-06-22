use num::{One, Unsigned, Zero};
use std::collections::BTreeMap;
use std::cmp::Reverse;
use core::iter::Step;
use core::ops::AddAssign;
use core::fmt::Display;
use itertools::Itertools;
use crate::algorithms::union_find::{WeightedQuickUnion, random_x_y_pairs};
use rand::Rng;
use std::fmt::Debug;
use std::time::{Instant, Duration};
use std::convert::TryFrom;

use crate::*;

pub fn main() {
    // test_correctness();
    test_performance();
}

pub struct GenericUnion<T>
    where
        T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize>,
        usize: TryFrom<T>,
        <usize as std::convert::TryFrom<T>>::Error: Debug,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pub size: T,
    pub nodes: Vec<T>,
    pub sizes: Vec<T>,
}

impl <T> GenericUnion<T>
    where
        T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize>,
        usize: TryFrom<T>,
        <usize as std::convert::TryFrom<T>>::Error: Debug,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pub fn new(size: T) -> Self {
        let size_usize = usize::try_from(size).unwrap();
        let mut nodes = Vec::with_capacity(size_usize);
        let mut sizes = Vec::with_capacity(size_usize);
        let one = T::one();
        for i in T::zero()..size {
            nodes.push(i);
            sizes.push(one);
        }
        Self {
            size,
            nodes,
            sizes,
        }
    }

    #[inline]
    pub fn union(&mut self, p: T, q: T) {
        let (root_p, root_q) = (self.root(p), self.root(q));
        if root_p != root_q {
            let (root_p_index, root_q_index) = (usize::try_from(root_p).unwrap(), usize::try_from(root_q).unwrap());
            let (p_size, q_size) = (self.sizes[root_p_index], self.sizes[root_q_index]);
            let (new_root, new_subordinate, new_root_index, new_subordinate_index) = if p_size >= q_size {
                (root_p, root_q, root_p_index, root_q_index)
            } else {
                (root_q, root_p, root_q_index, root_p_index)
            };
            debug_assert_ne!(new_root, new_subordinate);
            debug_assert_ne!(new_root_index, new_subordinate_index);
            self.sizes[new_root_index] = p_size + q_size;
            self.nodes[new_subordinate_index] = new_root;
            self.sizes[new_subordinate_index] = T::zero();
            self.compress_path(p, new_root);
            self.compress_path(q, new_root);
        }
    }

    #[inline]
    pub fn is_connected(&self, p: T, q: T) -> bool {
        let is_connected = self.root(p) == self.root(q);
        is_connected
    }

    #[inline]
    pub fn root(&self, mut p: T) -> T {
        loop {
            let p_parent= self.nodes[usize::try_from(p).unwrap()];
            if p_parent == p {
                return p;
            }
            p = p_parent;
        }
    }

    #[inline]
    fn compress_path(&mut self, mut i: T, root: T) {
        loop {
            let i_index = usize::try_from(i).unwrap();
            let i_parent = self.nodes[i_index];
            // if root_i == i {
            if i_parent == root {
                return;
            }
            self.nodes[i_index] = root;
            i = i_parent;
        }
    }

    pub fn union_list(&mut self, pairs: &[(usize, usize)]) {
        pairs.iter().for_each(|(p, q)| self.union(T::try_from(*p).unwrap(),T::try_from(*q).unwrap()));
    }

    pub fn count_is_connected_list(&mut self, pairs: &[(usize, usize)]) -> usize {
        pairs.iter().map(|(p, q)| if self.is_connected(T::try_from(*p).unwrap(), T::try_from(*q).unwrap()) { 1 } else { 0 }).sum()
    }

    /*
    pub fn union_list(&mut self, pairs: &[(T, T)]) {
        pairs.iter().for_each(|(p, q)| self.union(*p, *q));
    }

    pub fn count_is_connected_list(&mut self, pairs: &[(T, T)]) -> usize {
        pairs.iter().map(|(p, q)| if self.is_connected(*p, *q) { 1 } else { 0 }).sum()

     */

    pub fn tree_depth_mean(&self) -> f32 {
        let mut sum = 0;
        for i in T::zero()..self.size {
            let mut p: T = i;
            loop {
                let p_parent = self.nodes[usize::try_from(p).unwrap()];
                if p_parent == p {
                    break;
                }
                sum += 1;
                p = p_parent;
            }
        }
        sum as f32 / self.nodes.len() as f32
    }

    pub fn get_components(&self) -> Vec<Vec<T>> {
        let mut map = BTreeMap::new();
        for i in T::zero()..self.size {
            map.entry(self.root(i)).or_insert(vec![]).push(i);
        }
        map.values().map(|v| v.clone()).collect()
    }

    pub fn get_roots_of_largest_components(&self, limit: usize) -> Vec<T> {
        let one = T::one();
        let mut components = vec![];
        for index in T::zero()..self.size {
            let size = self.sizes[usize::try_from(index).unwrap()];
            if size > one {
                components.push((index, size));
            }
        }
        components.sort_by_cached_key(|(_index, size)| Reverse(*size));
        components.iter().take(limit).map(|(index, _size)| *index).collect()
    }

    pub fn print_components(&self) {
        let mut map = BTreeMap::new();
        for i in T::zero()..self.size {
            map.entry(self.root(i)).or_insert(vec![]).push(i);
        }
        println!("{}", self.get_components().iter()
            .map(|component| format!("{{ {} }}", component.iter().join(", ")))
            .join(" "));
    }
}

#[allow(dead_code)]
fn test_correctness() {
    let size = 10u16;
    let mut rng = rand::thread_rng();
    let mut union = WeightedQuickUnion::new(size.into(), true);
    let mut gen_union = GenericUnion::new(size);
    let mut connected_count = 0;
    for _ in 0..size {
        let (p, q) = (rng.gen_range(0..size), rng.gen_range(0..size));
        union.union(p.into(), q.into());
        gen_union.union(p, q);
        debug_assert!(union.is_connected(p.into(), q.into()));
        debug_assert!(gen_union.is_connected(p, q));

        let (p, q) = (rng.gen_range(0..size), rng.gen_range(0..size));
        let union_is_connected = union.is_connected(p.into(), q.into());
        let gen_union_is_connected = gen_union.is_connected(p, q);
        if union_is_connected {
            connected_count += 1;
        }
        debug_assert_eq!(union_is_connected, gen_union_is_connected);
    }
    println!("\nconnected_count = {}", fc(connected_count));
    union.print_components();
    // Try every possible combination.
    let mut connected_count = 0;
    for p in 0..size {
        for q in 0..size {
            let union_is_connected = union.is_connected(p.into(), q.into());
            let gen_union_is_connected = gen_union.is_connected(p, q);
            //rintln!("p = {}, q = {}, union_is_connected = {:?}, gen_union_is_connected = {:?}", fc(p), fc(q), union_is_connected, gen_union_is_connected);
            if union_is_connected {
                connected_count += 1;
            }
            debug_assert_eq!(union_is_connected, gen_union_is_connected);
        }
    }
    println!("connected_count = {}", fc(connected_count));
}

#[allow(dead_code)]
fn test_performance() {
    // let mut size = 64;
    let mut size = 134_217_728;
    let size_mult = 2;
    let size_max = 8_000_000_000;
    let union_mult = 1;

    while size < size_max {
        let mut connected_count = 0;
        let union_count = size * union_mult;
        println!("\nsize = {}; union_count = {}", fc(size), fc(union_count));

        let start_time = Instant::now();
        let union_pairs = random_x_y_pairs(size, union_count);
        let is_connected_pairs = random_x_y_pairs(size, union_count);
        println!("pairs: {:?}", Instant::now() - start_time);

        // Non-generic.

        let start_time = Instant::now();
        let mut union = WeightedQuickUnion::new(size, true);
        let new_elapsed = Instant::now() - start_time;

        let start_time = Instant::now();
        union.union_list(&union_pairs[..size]);
        let union_elapsed = Instant::now() - start_time;

        let start_time = Instant::now();
        let standard_connected_count = union.count_is_connected_list(&is_connected_pairs[..size]);
        connected_count += standard_connected_count;
        let connected_elapsed = Instant::now() - start_time;

        print_elapsed_one_run("union", size, new_elapsed, union_elapsed, connected_elapsed);

        // Generic with u16.
        if size < u16::max_value() as usize {
            let start_time = Instant::now();
            let mut union = GenericUnion::new(u16::try_from(size).unwrap());
            let new_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            union.union_list(&union_pairs[..size]);
            let union_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            let connected_count = union.count_is_connected_list(&is_connected_pairs[..size]);
            assert_eq!(connected_count, standard_connected_count);
            let connected_elapsed = Instant::now() - start_time;

            print_elapsed_one_run("u16", size, new_elapsed, union_elapsed, connected_elapsed);
        }

        // Generic with u32.
        if size < u32::max_value() as usize {
            let start_time = Instant::now();
            let size_32 = u32::try_from(size).unwrap();
            let mut union = GenericUnion::new(size_32);
            let new_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            union.union_list(&union_pairs[..size]);
            let union_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            let connected_count = union.count_is_connected_list(&is_connected_pairs[..size]);
            assert_eq!(connected_count, standard_connected_count);
            let connected_elapsed = Instant::now() - start_time;

            print_elapsed_one_run("u32", size, new_elapsed, union_elapsed, connected_elapsed);
        }

        // Generic with u64.
        if size < u64::max_value() as usize {
            let start_time = Instant::now();
            let mut union = GenericUnion::new(u64::try_from(size).unwrap());
            let new_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            union.union_list(&union_pairs[..size]);
            let union_elapsed = Instant::now() - start_time;

            let start_time = Instant::now();
            let connected_count = union.count_is_connected_list(&is_connected_pairs[..size]);
            assert_eq!(connected_count, standard_connected_count);
            let connected_elapsed = Instant::now() - start_time;

            print_elapsed_one_run("u64", size, new_elapsed, union_elapsed, connected_elapsed);
        }


        // union16 = if size < u16::max_value() as usize { Some(GenericUnion::new(u16::from(size))) } else { None };
        // union32 = if size < u32::max_value() as usize { Some(GenericUnion::new(u32::from(size))) } else { None };
        // union64 = if size < u64::max_value() as usize { Some(GenericUnion::new(u64::from(size))) } else { None };

        println!("connected_count = {}", fc(connected_count));

        size *= size_mult;
    }
}

fn print_elapsed_one_run(label: &str, size: usize, new_elapsed: Duration, union_elapsed: Duration, connected_elapsed: Duration) {
    let size = u32::try_from(size).unwrap_or(0);
    let (new_each, union_each, connected_each) = if size > 0 {
        (new_elapsed / size, union_elapsed / size, connected_elapsed / size)
    } else {
        (Duration::zero(), Duration::zero(), Duration::zero())
    };
    println!("{}: new {:?} / {:?}; union {:?} / {:?}; connected {:?} / {:?}",
             label,
             new_each, new_elapsed,
             union_each, union_elapsed,
             connected_each, connected_elapsed);
}

#[allow(dead_code)]
fn converted_pairs<T>(pairs: &[(usize, usize)]) -> Vec<(T, T)>
    where T: TryFrom<usize>,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pairs.iter().map(|(p, q)| (T::try_from(*p).unwrap(), T::try_from(*q).unwrap())).collect()
}


/*
size = 4,194,304; union_count = 4,194,304
union: new 3ns / 15.2816ms; union 92ns / 387.2712ms; connected 46ns / 195.3241ms
u32: new 2ns / 9.5191ms; union 81ns / 341.6324ms; connected 37ns / 159.2281ms
u64: new 3ns / 15.2697ms; union 98ns / 412.3035ms; connected 43ns / 181.2661ms
connected_count = 7,989,606

size = 8,388,608; union_count = 8,388,608
union: new 4ns / 39.101ms; union 98ns / 825.4921ms; connected 52ns / 439.7642ms
u32: new 2ns / 22.1912ms; union 97ns / 819.2298ms; connected 44ns / 370.7409ms
u64: new 3ns / 26.9287ms; union 109ns / 916.93ms; connected 48ns / 407.4901ms
connected_count = 15,983,547

size = 16,777,216; union_count = 16,777,216
union: new 4ns / 71.0854ms; union 103ns / 1.7388085s; connected 55ns / 937.3746ms
u32: new 2ns / 40.102ms; union 107ns / 1.8216439s; connected 49ns / 836.0219ms
u64: new 4ns / 69.0693ms; union 114ns / 1.9377914s; connected 51ns / 863.2585ms
connected_count = 31,963,806

size = 33,554,432; union_count = 33,554,432
union: new 4ns / 145.9226ms; union 123ns / 4.1394089s; connected 58ns / 1.987916s
u32: new 2ns / 97.6309ms; union 114ns / 3.8603354s; connected 51ns / 1.7628664s
u64: new 4ns / 137.2077ms; union 136ns / 4.6038187s; connected 54ns / 1.8455795s
connected_count = 63,890,550

size = 67,108,864; union_count = 67,108,864
union: new 4ns / 279.0087ms; union 216ns / 14.5685787s; connected 117ns / 7.8746912s
u32: new 3ns / 224.8255ms; union 141ns / 9.5207312s; connected 55ns / 3.7673864s
u64: new 4ns / 290.3624ms; union 227ns / 15.2862021s; connected 120ns / 8.0993378s
connected_count = 127,845,123

size = 134,217,728; union_count = 134,217,728
union: new 4ns / 580.272ms; union 278ns / 37.5341487s; connected 157ns / 21.2258469s
u32: new 3ns / 409.5548ms; union 260ns / 35.0475761s; connected 128ns / 17.383549s
u64: new 4ns / 580.6529ms; union 296ns / 39.9003512s; connected 153ns / 20.5926343s
connected_count = 255,691,077

size = 268,435,456; union_count = 268,435,456
union: new 3ns / 1.1477993s; union 303ns / 81.5456893s; connected 158ns / 42.7917238s
u32: new 3ns / 938.1944ms; union 301ns / 81.2325096s; connected 156ns / 42.1190719s
u64: new 3ns / 1.227913s; union 316ns / 85.0294521s; connected 152ns / 41.2320363s
connected_count = 511,325,181

===================================================================================================
===================================================================================================
===================================================================================================

size = 134,217,728; union_count = 134,217,728
pairs: 8.8566119s
union: new 4ns / 576.9139ms; union 307ns / 41.3675757s; connected 160ns / 21.5665089s
u32: new 3ns / 408.0389ms; union 257ns / 34.5676657s; connected 119ns / 16.0857107s
u64: new 4ns / 589.8467ms; union 295ns / 39.7230787s; connected 146ns / 19.7740658s
connected_count = 85,217,123

size = 268,435,456; union_count = 268,435,456
pairs: 17.7302061s
union: new 3ns / 1.155209s; union 295ns / 79.3024168s; connected 157ns / 42.4116782s
u32: new 3ns / 930.6007ms; union 313ns / 84.5242457s; connected 159ns / 42.8650593s
u64: new 3ns / 1.2394369s; union 301ns / 80.9881913s; connected 157ns / 42.3237128s
connected_count = 170,433,505

size = 536,870,912; union_count = 536,870,912
pairs: 35.4191967s
union: new 3ns / 2.3163602s; union 296ns / 159.3907386s; connected 162ns / 87.4785216s
u32: new 3ns / 2.1346331s; union 321ns / 172.7656159s; connected 156ns / 84.5083372s
u64: new 3ns / 2.4098708s; union 315ns / 169.8082637s; connected 158ns / 85.0894605s
connected_count = 340,879,386

size = 1,073,741,824; union_count = 1,073,741,824
pairs: 70.8610657s
union: new 3ns / 4.583285s; union 310ns / 333.3275341s; connected 162ns / 175.2993051s
u32: new 2ns / 3.8535551s; union 323ns / 347.5648083s; connected 168ns / 181.8145825s
u64: new 3ns / 4.5773793s; union 321ns / 345.435231s; connected 168ns / 181.836759s
connected_count = 681,719,510

size = 2,147,483,648; union_count = 2,147,483,648
pairs: 142.9537805s
union: new 4ns / 9.8458007s; union 322ns / 693.5622424s; connected 172ns / 371.6632287s
u32: new 24ns / 53.1832289s; union 389ns / 837.9758407s; connected 178ns / 384.6169481s
u64: new 5ns / 11.916272s; union 339ns / 728.3968178s; connected 174ns / 374.6282047s
connected_count = 1,363,436,750

*/

