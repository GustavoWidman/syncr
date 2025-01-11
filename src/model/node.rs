use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::utils::naivify_file_size;
use indexmap::IndexSet;
use log::debug;
use rand::{rngs::OsRng, seq::SliceRandom};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::Xxh3;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeNode {
    pub file_size: usize, // file size
    pub rate: f32,
    block_size: u32, // this is the power of two, 2^block_size
    naive: bool,
}

// bitwise nerd shit
impl TreeNode {
    pub fn new(file_size: usize, rate: f32, block_size: u32, naive: bool) -> Self {
        Self {
            file_size,
            rate,
            naive,
            block_size: block_size.trailing_zeros(),
        }
    }

    pub fn block_size(&self) -> u32 {
        1 << self.block_size
    }

    pub fn naive(&self) -> Option<Self> {
        if self.naive {
            return None;
        }

        Some(Self {
            file_size: naivify_file_size(self.file_size),
            rate: self.rate,
            block_size: self.block_size,
            naive: true,
        })
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = Xxh3::default();
        self.file_size.hash(&mut hasher);
        self.rate.to_bits().hash(&mut hasher);
        self.block_size.hash(&mut hasher);
        self.naive.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for TreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_size.hash(state);
        self.rate.to_bits().hash(state);
        self.block_size.hash(state);
        self.naive.hash(state);
    }
}
impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.file_size == other.file_size
            && self.rate == other.rate
            && self.block_size == other.block_size
            && self.naive == other.naive
    }
}
impl Eq for TreeNode {}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeList {
    inner: RwLock<IndexSet<TreeNode>>,

    pub optimal: HashMap<usize, (usize, u64)>, // points from file size to index in inner
}

impl NodeList {
    //? Constructor Methods
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(IndexSet::new()),
            optimal: HashMap::new(),
        }
    }

    //? Mutex Utilities
    pub fn read(&self) -> anyhow::Result<RwLockReadGuard<IndexSet<TreeNode>>> {
        self.inner
            .read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    }
    pub fn write(&self) -> anyhow::Result<RwLockWriteGuard<IndexSet<TreeNode>>> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    }

    //? NodeList Methods
    pub fn push(&mut self, node: TreeNode) -> anyhow::Result<()> {
        let mut lock = self
            .inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        if lock.contains(&node) {
            return Ok(());
        }

        let index = lock.len();

        self.optimal
            .entry(node.file_size)
            .and_modify(|(idx, hash)| {
                if lock.get_index(*idx).map_or(true, |existing| {
                    existing.hash() == *hash && node.rate >= existing.rate
                }) {
                    *idx = index;
                    *hash = node.hash();
                }
            })
            .or_insert((index, node.hash()));

        lock.insert(node);

        Ok(())
    }
    pub fn find(&self, file_size: usize) -> Option<u32> {
        let lock = self.read().ok()?;

        match self.optimal.get(&file_size) {
            Some((idx, hash)) => {
                let existing = &lock[*idx];
                (existing.hash() == *hash).then_some(existing.block_size())
            }
            None => None,
        }
    }
    pub fn wonderful_find(&mut self, file_size: usize) -> Option<u32> {
        let wonder: bool = rand::random();
        let found = self.find(file_size)?;

        match wonder {
            true => {
                debug!("Wondering");
                self.wonder(file_size, found)
            }
            false => Some(found),
        }
    }

    // allows the model to wonder a value upwards or downwards of the current most optimal value
    // the value is then tested to find it's compression rate and we can conclude if it is better or worse
    // if the model has wondered once upwards and once downwards and has not improved,
    // it is considered to be stuck and the most optimal value is considered to be found
    // for that specific file size
    //
    // "and i wonder..." - kanye west
    pub fn wonder(&self, file_size: usize, currently_predicted_size: u32) -> Option<u32> {
        let wonder_up_value = currently_predicted_size * 2;
        let wonder_down_value = std::cmp::max(currently_predicted_size / 2, 1);

        // attempts to find either wonder values in deduped data in an attempt
        // to see if we've already wandered there
        let (wonder_up, wonder_down) = self
            .read()
            .ok()?
            .par_iter()
            .filter(|node| node.file_size == file_size)
            .map(|node| {
                let wonder_up_hit = node.block_size() == wonder_up_value;
                let wonder_down_hit = node.block_size() == wonder_down_value;
                (wonder_up_hit, wonder_down_hit)
            })
            .reduce(
                || (false, false), // identity: no wonders seen yet
                |acc, (wonder_up_hit, wonder_down_hit)| {
                    (
                        acc.0 || wonder_up_hit,   // Combine results for wonder_up
                        acc.1 || wonder_down_hit, // Combine results for wonder_down
                    )
                },
            );

        debug!("Wondering up: {:?}", wonder_up);
        debug!("Wondering down: {:?}", wonder_down);

        match (wonder_up, wonder_down) {
            // has not wondered up or down, choose randomly
            (false, false) => {
                Some(
                    *[wonder_up_value, wonder_down_value]
                        .choose(&mut OsRng)
                        .unwrap(), // unwrap is safe here, list is never empty
                )
            }
            // has not wondered up
            (false, true) => Some(wonder_up_value),
            // has not wondered down
            (true, false) => Some(wonder_down_value),
            // already wondered enough, return current prediction
            (true, true) => Some(currently_predicted_size),
        }
    }
}
