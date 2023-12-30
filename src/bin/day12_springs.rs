use std::{num::NonZeroUsize, time::Instant};

use crate::day12_springs::{
    record::Record,
    segment::{DebugParts, Part, Segment},
};

mod day12_springs {
    pub mod record;

    pub mod segment;

    mod analysis;

    #[cfg(test)]
    mod tests;
}

fn main() -> anyhow::Result<()> {
    println!("hello, springy springs!");

    let input = advent_2023::get_input_string()?;
    let records = Record::parse_lines(&input)?;

    let records_unfolded = records
        .into_iter()
        .map(|record| record.unfold(FACTOR_5))
        .collect::<Vec<_>>();

    let sum = sum_counts(&records_unfolded);
    eprintln!("Sum of possibility counts: {sum}");

    Ok(())
}

fn sum_counts(records: &[Record]) -> usize {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let start = Instant::now();
            let result = record.count_possibilities();
            let elapsed = start.elapsed();
            eprintln!("RECORD {index:03} CALCULATED IN {elapsed:?}");
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
