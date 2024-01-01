use advent_2023::{nonempty::NonEmptyVec, vec_nonempty};
use anyhow::Context;
use std::num::NonZeroUsize;

use crate::{day12_springs::cache, Part, Segment, SegmentBuilder, ONE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    inner: RecordInner,
    separators: (bool, bool),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordInner {
    segments: NonEmptyVec<Segment>,
    known_counts: Vec<NonZeroUsize>,
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

        println!("------------------------------ symbols={symbols_str:?}");

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
        let mut separator_leading = false;
        let mut separator_trailing = false;

        let mut first = Some(());
        while let Some((leading, segment, trailing)) = Segment::new(&mut symbols)? {
            if first.take().is_some() {
                separator_leading = leading.is_some();
            }
            separator_trailing = trailing.is_some();

            segments.push(segment);
        }

        let Some(segments) = NonEmptyVec::new(segments) else {
            anyhow::bail!("empty segments")
        };
        Ok(Self {
            inner: RecordInner {
                segments,
                known_counts,
            },
            separators: (separator_leading, separator_trailing),
        })
    }
    pub fn segments(&self) -> &NonEmptyVec<Segment> {
        &self.inner.segments
    }
    pub fn known_counts(&self) -> &[NonZeroUsize] {
        &self.inner.known_counts
    }
    pub fn separators(&self) -> (bool, bool) {
        self.separators
    }
    pub fn unfold(self, factor: NonZeroUsize) -> Self {
        let Self {
            inner:
                RecordInner {
                    segments,
                    known_counts,
                },
            separators,
        } = self;

        let segments = unfold_segments(segments, separators, factor);

        let known_counts = std::iter::repeat_with(|| known_counts.iter().copied())
            .take(factor.get())
            .flatten()
            .collect();
        Self {
            inner: RecordInner {
                segments,
                known_counts,
            },
            separators,
        }
    }
    pub fn count_possibilities(&self) -> usize {
        let mut caches = Caches::default();
        let result = self.inner.count_possibilities_inner(0, &mut caches);
        if crate::DEBUG_CACHE {
            eprintln!("{}", caches.segment.summary("SEGMENT"));
            eprintln!("{}", caches.part.summary("PART"));
        }
        result
    }
}

#[derive(Default)]
struct Caches {
    segment: cache::Cache<Segment>,
    part: super::analysis::Cache,
}

impl RecordInner {
    fn count_possibilities_inner(&self, debug_indent: usize, caches: &mut Caches) -> usize {
        print!("{:width$}", "", width = debug_indent);
        let debug_indent_next = debug_indent + 4;
        println!(
            "[Record::count_possibilities] segments={segments:?} known_counts={known_counts:?}",
            segments = self.segments,
            known_counts = self.known_counts
        );
        let (segment_first, segments_rest) = self.segments.split_first();

        // verify all segments are compatible with known_counts
        if let Some(Impossible) =
            Self::heurestic_segments_impossible((segment_first, segments_rest), &self.known_counts)
        {
            print!("{:width$}", "", width = debug_indent_next);
            println!("ignore impossible");
            return 0;
        }

        let start_count = if segments_rest.is_empty() {
            // no more segments, skip to accepting ALL counts
            self.known_counts.len()
        } else if segment_first.is_nullable() {
            0
        } else {
            1
        };

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
                // verify segment_first is compatible with counts
                if let Some(Impossible) =
                    Self::heurestic_segments_impossible((segment_first, &[]), counts_taken)
                {
                    print!("{:width$}", "", width = debug_indent_next);
                    println!(
                        "ignore impossible take_count={take_count} of {total}",
                        total = self.known_counts.len(),
                    );
                    continue;
                }
                let key = cache::Key {
                    value: segment_first.clone(),
                    counts: counts_taken.to_vec(),
                };
                let options = caches.segment.lookup(&key).unwrap_or_else(|| {
                    let result = key.value.count_possibilities(
                        &key.counts,
                        debug_indent_next,
                        &mut caches.part,
                    );
                    caches.segment.save_new(key, result);
                    result
                });
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
                        let options_rest =
                            rest.count_possibilities_inner(debug_indent_next, caches);

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
    /// Returns `true` if the segments cannot possibly match the known counts
    fn heurestic_segments_impossible(
        (segment_first, segments_rest): (&Segment, &[Segment]),
        known_counts: &[NonZeroUsize],
    ) -> Option<Impossible> {
        let segments_iter = std::iter::once(segment_first).chain(segments_rest);

        // verify largest segment run is within max count
        let min_run = segments_iter
            .clone()
            .map(Segment::get_largest_min_run)
            .max()
            .expect("nonempty by construction");

        match known_counts.iter().copied().max() {
            Some(largest_count) if min_run > largest_count.get() => {
                return Some(Impossible);
            }
            _ => {}
        }

        // verify segments are long enough for remaining counts
        let max_total = segments_iter.map(Segment::get_maximum_count).sum();
        let sum_remaining_counts: usize = known_counts.iter().copied().map(NonZeroUsize::get).sum();

        if sum_remaining_counts > max_total {
            return Some(Impossible);
        }

        None
    }
}

#[derive(Debug)]
struct Impossible;

fn unfold_segments(
    segments: NonEmptyVec<Segment>,
    separators: (bool, bool),
    factor: NonZeroUsize,
) -> NonEmptyVec<Segment> {
    const SEPARATOR_UNKNOWN: Part = Part::Unknown(ONE);

    let (segment_first, segments_rest) = segments.split_first();
    match (segments_rest, separators) {
        (&[], (false, false)) => {
            // intersperse within the single segment
            //
            // e.g.  '#'  -> #?#?#?#?#
            //
            // strategy:
            // - given record A single segment
            // - output (A)(?A)(?A)(?A)(?A) single segment

            let mut first = Some(());
            // chain:   [seg], ?[seg], ?[seg], ?[seg], ?[seg]
            let builder: SegmentBuilder = std::iter::repeat_with(|| {
                let allow_sep = if first.take().is_some() { 0 } else { 1 };
                std::iter::once(SEPARATOR_UNKNOWN)
                    .take(allow_sep)
                    .chain(segment_first.iter())
            })
            .take(factor.get())
            .flatten()
            .collect::<SegmentBuilder>();

            let segment = builder
                .finish()
                .expect("nonempty segment, added to existing parts");

            vec_nonempty![segment]
        }
        (_nonempty, (false, false)) => {
            // separator creates "mid" segments that are double the input
            //
            // e.g. '#.#' -> #.#?#.#?#.#?#.#?#.#
            //
            // strategy:
            // - given record A, B..B, C   (B..B may be empty, but guaranteed A != C)
            // - calculate middle case C?A,
            // - output (A, B..B), (C?A B..B), (C?A B..B), (C?A B..B), (C?A B..B), C

            let (segment_last, segments_mid) = segments_rest
                .split_last()
                .expect("nonempty case, empty case handled above");

            let combined_last_then_first = segment_last
                .iter()
                .chain(std::iter::once(SEPARATOR_UNKNOWN))
                .chain(segment_first.iter())
                .collect::<SegmentBuilder>()
                .finish()
                .expect("nonempty segment, added to existing parts");

            let segments = std::iter::once(segment_first.clone())
                .chain(segments_mid.iter().cloned())
                .chain(
                    std::iter::repeat_with(|| {
                        std::iter::once(combined_last_then_first.clone())
                            .chain(segments_mid.iter().cloned())
                    })
                    .take(factor.get() - 1)
                    .flatten(),
                )
                .chain(std::iter::once(segment_last.clone()))
                .collect();
            NonEmptyVec::new(segments).expect("nonempty segment, added to existing parts")
        }
        (_, (true, true)) => {
            // separator itself is an independent segment
            //
            // e.g. '.#.' -> .#.?.#.?.#.?.#.?.#.
            //
            // strategy:
            // - given record A..Z
            // - output (A..Z) ? (A..Z) ? (A..Z) ? (A..Z) ? (A..Z)

            let segment_sep = std::iter::once(SEPARATOR_UNKNOWN)
                .collect::<SegmentBuilder>()
                .finish()
                .expect("nonempty");

            let mut first = Some(());
            // chain: [segments], ? [segments], ...
            let segments = std::iter::repeat_with(|| {
                let allow_sep = if first.take().is_some() { 0 } else { 1 };
                std::iter::once(segment_sep.clone())
                    .take(allow_sep)
                    .chain(segments.clone())
            })
            .take(factor.get())
            .flatten()
            .collect();
            NonEmptyVec::new(segments).expect("nonempty, repeated nonempty vecs a bunch")
        }
        (_, (false, true) | (true, false)) => {
            // separator "joins" on the ends of the existing segments
            // combine sets: "start" / 3x "mid" / "end"
            //
            // e.g. '.#' -> .#?.#?.#?.#?.#
            //
            // e.g. '#.' -> #.?#.?#.?#.?#.
            //
            // special care for "first" and "last" modifications, which may refer to the same
            // element (if sequence is empty)
            //
            // strategy:
            // - given record A, B..B, C  (where B..B may be empty, or single element A = C)
            //       - record has ONE of leading/trailing separators
            // - calculate A' := ?A (if no leading separator) else A
            // - calculate C' := C? (if no trailing separator) else C
            // - output (A, B..B, C'), (A', B..B, C'), (A', B..B, C'), (A', B..B, C'), (A', B..B, C)

            let (leading_has_sep, trailing_has_sep) = separators;
            let add_sep_to_front = !leading_has_sep;
            let add_sep_to_back = !trailing_has_sep;

            let segment_last = segments.last();
            let single_segment = segments_rest.is_empty();

            let segment_first_modified_opt = add_sep_to_front.then(|| {
                std::iter::once(SEPARATOR_UNKNOWN)
                    .chain(segment_first.clone())
                    .collect::<SegmentBuilder>()
                    .finish()
                    .expect("nonempty, added to segment")
            });

            let segment_last_modified_opt = add_sep_to_back.then(|| {
                segment_last
                    .iter()
                    .chain(std::iter::once(SEPARATOR_UNKNOWN))
                    .collect::<SegmentBuilder>()
                    .finish()
                    .expect("nonempty, added to segment")
            });

            let (segment_first_modified, segment_last_modified) = if single_segment {
                let single_modified = segment_first_modified_opt
                    .as_ref()
                    .or(segment_last_modified_opt.as_ref())
                    .expect("mutually exclusive case");
                (single_modified, single_modified)
            } else {
                (
                    segment_first_modified_opt.as_ref().unwrap_or(segment_first),
                    segment_last_modified_opt.as_ref().unwrap_or(segment_last),
                )
            };

            let (segment_first_for_start, segment_first_for_end) = if add_sep_to_back {
                // modified back, so apply to start
                (segment_first_modified, segment_first)
            } else {
                // modified front, so applyt to end
                (segment_first, segment_first_modified)
            };

            let mut first = Some(());
            let segments = std::iter::repeat_with(|| {
                let first = if first.take().is_some() {
                    segment_first_for_start
                } else {
                    segment_first_modified
                };

                std::iter::once(first.clone())
                    .chain(
                        segments_rest
                            .iter()
                            .take(segments_rest.len().saturating_sub(1))
                            .cloned(),
                    )
                    .chain(
                        std::iter::once_with(|| segment_last_modified.clone())
                            .take(if segments_rest.is_empty() { 0 } else { 1 }),
                    )
            })
            .take(factor.get() - 1)
            .flatten()
            .chain(
                std::iter::once(segment_first_for_end.clone()).chain(segments_rest.iter().cloned()),
            )
            .collect();
            NonEmptyVec::new(segments).expect("nonempty")
        }
    }
}
