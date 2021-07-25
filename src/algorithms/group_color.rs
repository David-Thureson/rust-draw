use std::collections::{HashMap, BTreeSet, BTreeMap};
use num::{Unsigned, Zero, One};
use std::iter::{Step, FromIterator};
use std::ops::AddAssign;
use std::fmt::{Debug, Display};
use std::convert::TryFrom;
use std::hash::Hash;
use rand::Rng;
use num_format::ToFormattedStr;

use crate::*;
use crate::Color1;
use super::percolation::*;
use super::generic_union::GenericUnion;
use std::collections::btree_set::Difference;
use itertools::Itertools;

pub struct GroupColor<T> {
    groups: HashMap<T, usize>,
    colors: Vec<Color1>,
    color_min: f32,
    color_max: f32,
    open_as_white: bool,
}

impl <T> GroupColor<T>
    where
        T: Copy + Sized + Unsigned + Zero + One + Step + AddAssign + Ord + Display + Debug + TryFrom<usize> + Hash + ToFormattedStr,
        usize: TryFrom<T>,
        <usize as std::convert::TryFrom<T>>::Error: Debug,
        <T as std::convert::TryFrom<usize>>::Error: Debug,
{
    pub fn new(color_min: f32, color_max: f32, open_as_white: bool) -> Self {
        assert!(color_min >= 0.0);
        assert!(color_min <= 1.0);
        assert!(color_max >= 0.0);
        assert!(color_max <= 1.0);
        Self {
            groups: Default::default(),
            colors: vec![],
            color_min,
            color_max,
            open_as_white,
        }
    }

    fn set_up_first_groups(&mut self, roots: Vec<T>) {
        self.colors = vec![Color1::black(), Color1::white(), Color1::blue(), Color1::red()];
        let special_color_count = self.colors.len();
        self.gen_colors(roots.len());

        self.groups = HashMap::with_capacity(roots.len());
        let mut color_index = special_color_count;
        for root in roots.iter() {
            self.groups.insert(*root, color_index);
            color_index += 1;
        }
    }

    pub fn update(&mut self, roots: Vec<T>, union: &GenericUnion<T>) {
        if self.groups.is_empty() {
            self.set_up_first_groups(roots);
        } else {
            //rintln!("\n###########################################################################\n");
            let old_set = BTreeSet::from_iter(self.groups.keys().map(|x| *x));
            let new_set = BTreeSet::from_iter(roots);
            //Self::debug_print_set("old_set", &old_set);
            //Self::debug_print_set("new_set", &new_set);

            let added = new_set.difference(&old_set);
            //Self::debug_print_diff("added", &added);

            let mut added_map: BTreeMap<T, Option<usize>> = BTreeMap::new();
            added.for_each(|root| { added_map.insert(*root, None); });
            //Self::debug_print_map("added_map", &added_map);

            let removed = old_set.difference(&new_set);
            //Self::debug_print_diff("removed", &removed);

            let mut removed_map: BTreeMap<T, Option<usize>> = BTreeMap::new();
            removed.for_each(|root| {
                let removed_color = self.groups.remove(root).unwrap();
                removed_map.insert(*root, Some(removed_color));
            });
            //Self::debug_print_map("removed_map", &removed_map);

            for (removed_root, removed_color) in removed_map.iter_mut() {
                for (added_root, added_color) in added_map.iter_mut() {
                    if union.is_connected(*removed_root, *added_root) {
                        //rintln!("Found connected removed = ({}, {}), added_root = {}", fc(*removed_root), removed_color.unwrap(), fc(*added_root));
                        *added_color = *removed_color;
                        *removed_color = None;
                        continue;
                    }
                }
            }
            let mut removed_color_indexes = removed_map.values()
                .filter(|color_index| color_index.is_some())
                .map(|color_index| color_index.unwrap())
                .collect::<Vec<_>>();
            //Self::debug_print_color_indexes("removed_color_indexes",removed_color_indexes.clone());
            for (added_root, added_color_index) in added_map.iter() {
                let color_index = match added_color_index {
                    Some(color_index) => *color_index,
                    None => removed_color_indexes.remove(0),
                };
                self.groups.insert(*added_root, color_index);
                //rintln!("Inserted ({}, {})", fc(*added_root), color_index);
            }
        }
    }

    fn gen_colors(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            self.colors.push(
                Color1::from_rgb(rng.gen_range(self.color_min..self.color_max),
                                 rng.gen_range(self.color_min..self.color_max),
                                 rng.gen_range(self.color_min..self.color_max)));
        }
    }

    pub fn get_colors(&self) -> Vec<Color1> {
        self.colors.clone()
    }

    pub fn block_color_index(&self, node_index: T, end_node_index: T, block_state: PercolationBlockState, is_last_frame: bool, union: &GenericUnion<T>) -> usize
    {
        match block_state {
            PercolationBlockState::Blocked => COLOR_INDEX_BLACK,
            PercolationBlockState::Open => {
                let node_root_index = union.root(node_index);
                if union.is_connected(end_node_index, node_index) {
                    COLOR_INDEX_RED
                } else {
                    if !is_last_frame {
                        if let Some(color_index) = self.groups.get(&node_root_index) {
                            return *color_index;
                        }
                    }
                    if self.open_as_white { COLOR_INDEX_WHITE } else { COLOR_INDEX_BLACK }
                }
            },
            PercolationBlockState::Filled => {
                // println!("Color index = COLOR_INDEX_BLUE / {} / {:?}", COLOR_INDEX_BLUE, self.colors[COLOR_INDEX_BLUE]);
                // panic!()
                COLOR_INDEX_BLUE
            },
        }
    }

    #[allow(dead_code)]
    fn debug_print_set(label: &str, set: &BTreeSet<T>) {
        let indexes = set.iter().map(|x| *x).collect::<Vec<_>>();
        Self::debug_print_indexes(label, indexes);
    }

    #[allow(dead_code)]
    fn debug_print_diff(label: &str, diff: &Difference<T>) {
        let mut indexes = vec![];
        diff.clone().for_each(|x| { indexes.push(*x); });
        Self::debug_print_indexes(label, indexes);
    }

    #[allow(dead_code)]
    fn debug_print_indexes(label: &str, mut indexes: Vec<T>) {
        indexes.sort();
        println!("{}: {}", label, list_of_counts(&indexes));
    }

    #[allow(dead_code)]
    fn debug_print_color_indexes(label: &str, mut indexes: Vec<usize>) {
        indexes.sort();
        println!("{}: {}", label, list_of_counts(&indexes));
    }

    #[allow(dead_code)]
    fn debug_print_map(label: &str, map: &BTreeMap<T, Option<usize>>) {
        println!("{}: {}", label,
        map.iter()
            .map(|(root, color_index)| format!("({}, {})", root, color_index.map_or("_".to_string(), |x| x.to_string())))
            .join(" "));
    }

}
