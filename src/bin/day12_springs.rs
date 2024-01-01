use std::{num::NonZeroUsize, time::Instant};

const DEBUG_CACHE: bool = false;
const DEBUG_PROGRESS: bool = true;

pub use crate::day12_springs::{
    record::Record,
    segment::{DebugParts, Part, Segment, SegmentBuilder},
};

mod day12_springs {
    pub mod record;

    pub mod segment;

    mod analysis;

    #[cfg(test)]
    mod tests;

    pub mod cache {
        use std::hash::Hash;
        use std::{collections::HashMap, num::NonZeroUsize};

        pub struct Cache<T: Hash + Eq> {
            map: HashMap<Key<T>, usize>,
            lookup_count: usize,
        }
        impl<T: Hash + Eq> Cache<T> {
            pub fn lookup(&mut self, key: &Key<T>) -> Option<usize> {
                self.lookup_count += 1;
                self.map.get(key).copied()
            }
            /// Panics if there is already a value stored
            pub fn save_new(&mut self, key: Key<T>, result: usize) {
                let prev = self.map.insert(key, result);
                assert_eq!(prev, None);
            }
            pub fn summary<'a>(&self, label: &'a str) -> impl std::fmt::Display + 'a {
                let Self {
                    ref map,
                    lookup_count,
                } = *self;
                let len = map.len();
                Summary {
                    label,
                    len,
                    lookup_count,
                }
            }
        }
        impl<T: Hash + Eq> Default for Cache<T> {
            fn default() -> Self {
                Self {
                    map: HashMap::default(),
                    lookup_count: 0,
                }
            }
        }
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct Key<T> {
            pub value: T,
            pub counts: Vec<NonZeroUsize>,
        }

        struct Summary<'a> {
            label: &'a str,
            len: usize,
            lookup_count: usize,
        }
        impl std::fmt::Display for Summary<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let Self {
                    label,
                    len,
                    lookup_count,
                } = *self;
                let hit_ratio = (len as f64) / (lookup_count as f64) * 100.0;
                write!(
                    f,
                    "CACHE {label}: {len} stored across {lookup_count} lookups {hit_ratio:.1}%"
                )
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    println!("hello, springy springs!");

    let input = advent_2023::get_input_string()?;
    let records = Record::parse_lines(&input)?;

    let records_unfolded = records
        .into_iter()
        .map(|record| record.unfold(FACTOR_5))
        .collect::<Vec<_>>();

    let sum_start = Instant::now();

    let sum = sum_counts(&records_unfolded);

    let total_duration = sum_start.elapsed();
    eprintln!("Sum of possibility counts: {sum}, in {total_duration:?}");

    Ok(())
}

fn sum_counts(records: &[Record]) -> usize {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let line = index + 1;
            if DEBUG_PROGRESS {
                eprint!("{line} ");
            }

            let start = Instant::now();
            let result = record.count_possibilities();
            let elapsed = start.elapsed();
            if DEBUG_PROGRESS {
                if elapsed.as_secs() > 0 {
                    eprintln!("\nLINE {line:03} CALCULATED IN {elapsed:?}");
                }
            } else {
                eprintln!("{result}");
            }
            result
        })
        .sum()
}

const ONE: NonZeroUsize = match NonZeroUsize::new(1) {
    Some(v) => v,
    None => [][0],
};

const FACTOR_5: NonZeroUsize = match NonZeroUsize::new(5) {
    Some(v) => v,
    None => [][0],
};
