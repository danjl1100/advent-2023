use advent_2023::nonempty::NonEmptyVec;
use anyhow::Context;
use std::num::NonZeroUsize;

use crate::{Part, Segment, ONE};

pub struct Record {
    pub segments: NonEmptyVec<Segment>,
    pub known_counts: Vec<NonZeroUsize>,
}
impl Record {
    pub fn parse_lines(input: &str) -> anyhow::Result<Vec<Self>> {
        input
            .lines()
            .map(|line| Record::new(line).with_context(|| format!("on line {line:?}")))
            .collect()
    }
    pub fn new(line: &str) -> anyhow::Result<Self> {
        const BASE_10: u32 = 10;

        let Some((symbols_str, list_str)) = line.split_once(' ') else {
            anyhow::bail!("space delimiter not found")
        };

        let known_counts = list_str
            .split(',')
            .map(|s| {
                usize::from_str_radix(s, BASE_10)
                    .map_err(|s| anyhow::anyhow!("invalid number: {s:?}"))
                    .and_then(|n| {
                        NonZeroUsize::new(n).ok_or_else(|| anyhow::anyhow!("invalid count: zero"))
                    })
            })
            .collect::<Result<_, _>>()?;

        let mut symbols = symbols_str.chars().peekable();
        let mut segments = vec![];
        while let Some(segment) = Segment::new(&mut symbols)? {
            segments.push(segment);
        }

        let Some(segments) = NonEmptyVec::new(segments) else {
            anyhow::bail!("empty segments")
        };
        Ok(Self {
            segments,
            known_counts,
        })
    }
    pub fn unfold(self, factor: NonZeroUsize) -> Self {
        let Self {
            segments,
            known_counts,
        } = self;

        let segments_for_end = segments.clone();

        let segments_for_begin_middle = {
            let (mut segments, segment_last) = segments.into_split_last();
            let segment_last = {
                let mut segment_last = segment_last.into_builder();
                segment_last.push(Part::Unknown(ONE));
                segment_last
                    .finish()
                    .expect("nonempty, added to existing part")
            };
            segments.push(segment_last);

            NonEmptyVec::new(segments).expect("nonempty, pushed segment_last")
        };

        let segments = std::iter::repeat_with(|| segments_for_begin_middle.clone().into_iter())
            .take(factor.get() - 1)
            .flatten()
            .chain(segments_for_end.into_iter())
            .collect();
        let segments =
            NonEmptyVec::new(segments).expect("nonempty, repeated nonempty vecs a bunch");

        let known_counts = std::iter::repeat_with(|| known_counts.iter().copied())
            .take(factor.get())
            .flatten()
            .collect();
        Self {
            segments,
            known_counts,
        }
    }
    pub fn count_possibilities(&self) -> usize {
        self.count_possibilities_inner(0)
    }
    fn count_possibilities_inner(&self, debug_indent: usize) -> usize {
        print!("{:width$}", "", width = debug_indent);
        let debug_indent_next = debug_indent + 4;
        println!(
            "[Record::count_possibilities] segments={segments:?} known_counts={known_counts:?}",
            segments = self.segments,
            known_counts = self.known_counts
        );
        let (segment_first, segments_rest) = self.segments.split_first();

        let start_count = if segment_first.is_nullable() { 0 } else { 1 };

        let mut total_options = 0;
        for take_count in start_count..(self.known_counts.len() + 1) {
            let counts_taken = &self.known_counts[..take_count];
            let counts_rest = &self.known_counts[take_count..];

            if segments_rest.is_empty() && !counts_rest.is_empty() {
                continue;
            }

            let options = if counts_taken.is_empty() {
                segment_first.is_nullable().then_some(1)
            } else {
                let options = segment_first.count_possibilities(counts_taken, debug_indent_next);
                Some(options)
            };

            match options {
                Some(0) => {
                    print!("{:width$}", "", width = debug_indent);
                    println!("options = 0 for that run");
                }
                None | Some(_) => {
                    if let Some(segments_rest) = NonEmptyVec::new(segments_rest.to_vec()) {
                        let rest = Self {
                            segments: segments_rest,
                            known_counts: counts_rest.to_vec(),
                        };
                        let options_rest = rest.count_possibilities_inner(debug_indent_next);

                        let options_num = options.unwrap_or(1);
                        total_options += options_num * options_rest;
                        print!("{:width$}", "", width = debug_indent);
                        println!("options += {options_num} * {options_rest} => {total_options}");
                    } else if counts_rest.is_empty() {
                        // no more segments, and satisfied all counts
                        let options_num = options.unwrap_or(0);
                        total_options += options_num;
                        print!("{:width$}", "", width = debug_indent);
                        println!("options += {options_num} (no more counts) => {total_options}");
                    } else {
                        unreachable!(
                            "ALREADY CHECKED FOR: options not allowed, segments will be empty while counts is nonempty"
                        );
                    }
                }
            }
        }
        print!("{:width$}", "", width = debug_indent);
        println!("returning total {total_options}");
        total_options
    }
}
