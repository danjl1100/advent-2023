use advent_2023::nonempty::NonEmptyVec;
use anyhow::Context;
use std::num::NonZeroUsize;

fn main() -> anyhow::Result<()> {
    println!("hello, springy springs!");

    let input = advent_2023::get_input_string()?;
    let records = Record::parse_lines(&input)?;

    let sum = sum_counts(&records);
    println!("Sum of possibility counts: {sum}");

    Ok(())
}

fn sum_counts(records: &[Record]) -> usize {
    records.iter().map(Record::count_possibilities).sum()
}

struct Record {
    segments: NonEmptyVec<Segment>,
    known_counts: Vec<NonZeroUsize>,
}
impl Record {
    fn parse_lines(input: &str) -> anyhow::Result<Vec<Self>> {
        input
            .lines()
            .map(|line| Record::new(line).with_context(|| format!("on line {line:?}")))
            .collect()
    }
    fn new(line: &str) -> anyhow::Result<Self> {
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
        // TODO unclear how to functionalize this...
        // let segments =
        //     std::iter::repeat_with(|| Segment::new(symbols)).collect::<Result<Vec<_>, _>>()?;

        let Some(segments) = NonEmptyVec::new(segments) else {
            anyhow::bail!("empty segments")
        };
        Ok(Self {
            segments,
            known_counts,
        })
    }
    fn count_possibilities(&self) -> usize {
        let (segment_first, segments_rest) = self.segments.split_first();

        let Some((&count_first, counts_rest)) = self.known_counts.split_first() else {
            // no counts to match non-empty segments
            return 0;
        };

        // TODO - need more sophistication for splitting a segment
        let options = segment_first.count_possibilities(&[count_first]);

        if options == 0 {
            0
        } else {
            let Some(segments_rest) = NonEmptyVec::new(segments_rest.to_vec()) else {
                // TODO
                todo!()
            };
            let rest = Self {
                segments: segments_rest,
                known_counts: counts_rest.to_vec(),
            };
            let options_rest = rest.count_possibilities();

            options * options_rest
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Segment(NonEmptyVec<Part>);

impl Segment {
    #[allow(unused)] // for tests
    fn new_from_str(symbols: &str) -> anyhow::Result<Option<Self>> {
        let symbols = &mut symbols.chars().peekable();
        Self::new(symbols)
    }
    fn new(
        symbols: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    ) -> anyhow::Result<Option<Self>> {
        // ignore duplicate separators
        while let Some('.') = symbols.peek() {
            let _ = symbols.next();
        }

        let mut builder = SegmentBuilder::default();

        for symbol in symbols {
            let new = match symbol {
                '#' => Part::Absolute(1),
                '?' => Part::Unknown(1),
                '.' => {
                    break;
                }
                extra => {
                    anyhow::bail!("unknown character {extra:?}")
                }
            };
            builder.push(new);
        }
        Ok(builder.finish())
    }
}

#[derive(Default)]
struct SegmentBuilder {
    parts: Vec<Part>,
    prev: Option<Part>,
}
impl SegmentBuilder {
    fn push(&mut self, new: Part) {
        self.prev = match self.prev.take() {
            None => Some(new),
            Some(prev) => {
                let (finished, combined) = prev + new;
                if let Some(finished) = finished {
                    self.parts.push(finished);
                }
                Some(combined)
            }
        };
    }
    fn finish(self) -> Option<Segment> {
        let Self { mut parts, prev } = self;

        // append unfinished part
        if let Some(prev) = prev {
            parts.push(prev);
        }

        NonEmptyVec::new(parts).map(Segment)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Absolute(usize),
    Unknown(usize),
}
impl Part {
    fn is_nullable(self) -> bool {
        match self {
            Part::Absolute(inner) => inner == 0,
            Part::Unknown(_) => true,
        }
    }
    //     fn is_empty(self) -> bool {
    //         match self {
    //             Part::Absolute(inner) | Part::Unknown(inner) => inner == 0,
    //         }
    //     }
}
impl std::ops::Add for Part {
    type Output = (Option<Part>, Part);
    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Absolute(count_old), Self::Absolute(count_new)) => {
                (None, Self::Absolute(count_old + count_new))
            }
            (Self::Unknown(count_old), Self::Unknown(count_new)) => {
                (None, Self::Unknown(count_old + count_new))
            }
            (Self::Absolute(_), Self::Unknown(_)) | (Self::Unknown(_), Self::Absolute(_)) => {
                (Some(self), other)
            }
        }
    }
}
// impl std::ops::Sub<usize> for Part {
//     type Output = Option<Self>;
//     fn sub(self, other: usize) -> Self::Output {
//         match self {
//             Part::Absolute(inner) => inner.checked_sub(other).map(Part::Absolute),
//             Part::Unknown(inner) => inner.checked_sub(other).map(Part::Unknown),
//         }
//     }
// }
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Absolute(count) => write!(f, "#x{count}"),
            Part::Unknown(count) => write!(f, "?x{count}"),
        }
    }
}

fn unsplit_to_vec<T: Copy>(first: T, rest: &[T]) -> Vec<T> {
    std::iter::once(first).chain(rest.iter().copied()).collect()
}
struct DebugParts(Vec<Part>);
impl std::fmt::Display for DebugParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = Some(());
        for &part in &self.0 {
            if is_first.take().is_none() {
                write!(f, " ")?;
            }
            let (symbol, count) = match part {
                Part::Absolute(count) => ('#', count),
                Part::Unknown(count) => ('?', count),
            };
            for _ in 0..count {
                write!(f, "{symbol}")?;
            }
        }
        write!(f, "}}")
    }
}

mod analysis {
    //! Privacy barrier for analysis-specific functions/types
    use std::num::NonZeroUsize;

    use advent_2023::nonempty::NonEmptyVec;

    use crate::{unsplit_to_vec, DebugParts, Part, Segment};

    trait SplitFirstCopyOption<T: Copy> {
        fn split_first_copy(&self) -> Option<(T, &[T])>;
    }
    impl<'a, T: Copy> SplitFirstCopyOption<T> for &'a [T] {
        fn split_first_copy(&self) -> Option<(T, &[T])> {
            self.split_first().map(|(&first, rest)| (first, rest))
        }
    }
    trait SplitFirstCopy<T: Copy> {
        fn split_first_copy(&self) -> (T, &[T]);
    }
    impl<T: Copy> SplitFirstCopy<T> for NonEmptyVec<T> {
        fn split_first_copy(&self) -> (T, &[T]) {
            let (&first, rest) = self.split_first();
            (first, rest)
        }
    }

    trait NonZeroCheckedSub<T>: Sized {
        fn checked_sub(self, rhs: T) -> Option<Self>;
    }
    impl NonZeroCheckedSub<usize> for NonZeroUsize {
        fn checked_sub(self, rhs: usize) -> Option<Self> {
            self.get().checked_sub(rhs).and_then(Self::new)
        }
    }

    impl Segment {
        pub(crate) fn count_possibilities(&self, expected_counts: &[NonZeroUsize]) -> usize {
            println!("------------------------------");
            let Some(counts_split) = expected_counts.split_first_copy() else {
                // empty counts, nonempty parts (impossible)
                return 0;
            };

            let parts_split = self.0.split_first_copy();

            // SegmentAnalysis {
            //     parts_split,
            //     counts_split,
            //     force_left_align: None,
            //     debug_indent: 0,
            // }
            SegmentAnalysis::new(parts_split, counts_split).count()
        }
    }

    struct SegmentAnalysis<'a> {
        part_first: Part,
        parts_rest: &'a [Part],
        count_first: NonZeroUsize,
        counts_rest: &'a [NonZeroUsize],
        force_left_align: Option<ForceLeftAlign>,
        debug_indent: usize,
    }
    impl<'a> SegmentAnalysis<'a> {
        fn new(
            parts_split: (Part, &'a [Part]),
            counts_split: (NonZeroUsize, &'a [NonZeroUsize]),
        ) -> Self {
            let (part_first, parts_rest) = parts_split;
            let (count_first, counts_rest) = counts_split;
            Self {
                part_first,
                parts_rest,
                count_first,
                counts_rest,
                force_left_align: None,
                debug_indent: 0,
            }
        }
        fn recurse(
            &self,
            debug_msg: &'static str,
            (part_first, parts_rest): (Part, &'a [Part]),
            (count_first, counts_rest): (NonZeroUsize, &'a [NonZeroUsize]),
            force_left_align: Option<ForceLeftAlign>,
        ) -> Self {
            println!("{:width$}-> {debug_msg} --v", "", width = self.debug_indent);
            Self {
                part_first,
                parts_rest,
                count_first,
                counts_rest,
                force_left_align,
                debug_indent: self.debug_indent + 4,
            }
        }
        /// Counts the number of possibilities for the Segment covering ALL counts
        ///
        /// Segment = Vec<Part>, and multiple parts can cover one count
        ///
        /// |-------------SEGMENT--------------|
        /// |-PART-|-PART-|-PART-|-PART-|-PART-|
        /// ====================================
        /// |--COUNT--| |--COUNT--|  |--COUNT--|
        /// ====================================
        ///            ^-----------^^----Unknowns chosen to be NONE
        ///
        fn count(self) -> usize {
            let Self {
                part_first,
                parts_rest,
                count_first,
                counts_rest,
                force_left_align,
                debug_indent,
            } = self;

            print!("{:width$}", "", width = debug_indent);
            print!("* count_possibilities_slice");
            print!("(parts: ({part_first:?}, {parts_rest:?})");
            print!(", counts: ({count_first:?}, {counts_rest:?})");
            print!(
                ", {}",
                force_left_align.as_ref().map_or("none", |_| "Align")
            );
            let debug_parts = DebugParts(unsplit_to_vec(part_first, parts_rest));
            let debug_counts = unsplit_to_vec(count_first, counts_rest);
            println!(")\tpartial-record {debug_parts} {debug_counts:?}",);

            let (branch, (result, reason)): (_, (usize, &'static str)) =
                if parts_rest.is_empty() && counts_rest.is_empty() {
                    (
                        "case_singleton",
                        Self::case_singleton(part_first, count_first, force_left_align),
                    )
                } else {
                    // Non-singleton case
                    match part_first {
                        Part::Absolute(count_part_absolute) => {
                            ("case_absolute", self.case_absolute(count_part_absolute))
                        }
                        Part::Unknown(count_part_unknown) => {
                            ("case_unknown", self.case_unknown(count_part_unknown))
                        }
                    }
                };
            println!(
                "{:width$}{result} <- {reason} ({branch})",
                "",
                width = debug_indent
            );
            result
        }
        fn case_singleton(
            part_first: Part,
            count_first: NonZeroUsize,
            force_left_align: Option<ForceLeftAlign>,
        ) -> (usize, &'static str) {
            // ONE part in segment, and ONE count
            let expected = count_first.get();
            match part_first {
                // Absoulute - must match completely
                Part::Absolute(count_part) if count_part == expected => {
                    (1, "absolute perfect match")
                }
                Part::Absolute(_) => (0, "absolute NOT perfect match"),
                // Unknown - possible to slide around
                Part::Unknown(count_part) => {
                    if let Some(blank_area) = count_part.checked_sub(expected) {
                        if force_left_align.is_some() {
                            (1, "unknowns left-aligned handles remaining expected")
                        } else {
                            // blanked area can slide, with one side containing: 0..=blank_area
                            (blank_area + 1, "unknowns slide")
                        }
                    } else {
                        // expecting more than possible
                        (0, "unknowns too short")
                    }
                }
            }
        }
        fn case_absolute(&self, count_part_absolute: usize) -> (usize, &'static str) {
            // anchored by first
            //
            // For a match, `part_first` MUST fully contain `count_first`
            let counts_next = if self.count_first.get() == count_part_absolute {
                // Pop count
                if let Some(counts_next) = self.counts_rest.split_first_copy() {
                    counts_next
                } else {
                    // verify remainder is all Unknowns
                    let possible = self.parts_rest.iter().copied().all(Part::is_nullable);
                    return if possible {
                        (1, "absolute exhausted count, remainder is nullable")
                    } else {
                        (0, "absolute exhausted count, but remainder is not nullable")
                    };
                }
            } else if let Some(count_remaining) = self.count_first.checked_sub(count_part_absolute)
            {
                (count_remaining, self.counts_rest)
            } else {
                // REJECT, Part::Absolute larger than count_first
                return (0, "absolute too long for current count");
            };

            let Some(parts_next) = self.parts_rest.split_first_copy() else {
                // TODO
                todo!()
            };

            let result = self
                .recurse(
                    "part pop_front, count reduced",
                    parts_next,
                    counts_next,
                    Some(ForceLeftAlign),
                )
                .count();
            (result, "recurse")
        }
        fn case_unknown(&self, count_part_unknown: usize) -> (usize, &'static str) {
            // if count_first == 0 {
            //     let Some(parts_next) = parts_rest.split_first_copy() else {
            //         // TODO
            //         todo!()
            //     };

            //     self.recurse(
            //         "part pop_front",
            //         parts_next,
            //         (count_first, counts_rest),
            //         force_left_align,
            //     )
            //     .count();
            // } else

            if self.force_left_align.is_some()
                && self.counts_rest.is_empty()
                && count_part_unknown >= self.count_first.get()
                && !self.parts_rest.iter().copied().all(Part::is_nullable)
            {
                return (
                    0,
                    "unknowns longer than final count, but force-left-align with remainder not nullable",
                );
            }
            // if self.counts_rest.is_empty() && self.force_left_align.is_some() {
            //     return (0, "questionable... not sure why");
            // }
            // TODO resume FUNCTION-ification here, to clean up and also pretty-print results for ALL paths

            // FIXME avoid N-based recurse for Unknowns...
            // For now, input is not excessively long so recurse for each question-mark choice

            // TODO make all parts `NonZeroUsize`
            let parts_reduced = if count_part_unknown == 1 {
                // TODO when nonzerousize, this is IDENTICAL to the final `else` case below
                // remove empty first
                let Some(parts_next) = self.parts_rest.split_first_copy() else {
                    // TODO
                    todo!()
                };
                parts_next
            } else if let Some(part_reduced) = count_part_unknown.checked_sub(1) {
                let part_reduced = Part::Unknown(part_reduced);
                // -1 to first
                (part_reduced, self.parts_rest)
            } else {
                // remove empty first
                let Some(parts_next) = self.parts_rest.split_first_copy() else {
                    // TODO
                    todo!()
                };
                parts_next
            };

            // split into two possibilities
            let unknown_on = if self.count_first.get() == 1 {
                if let Some(counts_next) = self.counts_rest.split_first_copy() {
                    self.recurse(
                        "assume Unknown = ON, reduce count",
                        parts_reduced,
                        counts_next,
                        Some(ForceLeftAlign),
                    )
                    .count()
                } else {
                    // verify remainder is all Unknowns
                    let possible = self.parts_rest.iter().copied().all(Part::is_nullable);
                    if possible {
                        // unknown exhasted count, remainder is nullable
                        1
                    } else {
                        // unknown exhasted count, but remainder is not nullable
                        0
                    }
                }
            } else if let Some(count_reduced) = self.count_first.checked_sub(1) {
                self.recurse(
                    "assume Unknown = ON, reduce count",
                    parts_reduced,
                    (count_reduced, self.counts_rest),
                    Some(ForceLeftAlign),
                )
                .count()
            } else {
                // cannot reduce count, impossible for ON
                0
            };
            let unknown_off = if self.force_left_align.is_some() {
                // cannot assume off, forced left align
                0
            } else {
                // let counts_next = counts_rest.split_first_copy().unwrap_or((0, &[]));
                self.recurse(
                    "assume Unknown = OFF, count pop_front",
                    parts_reduced,
                    (self.count_first, self.counts_rest),
                    self.force_left_align,
                )
                .count()
            };
            let sum = unknown_on + unknown_off;
            (sum, "sum of ON and OFF options")
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct ForceLeftAlign;
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use advent_2023::vec_nonempty;

    use crate::{Part, Record, Segment};

    #[test]
    fn parse_segment() {
        let symbols = ".......####??###.";
        let segment = Segment::new_from_str(symbols)
            .expect("characters valid")
            .expect("nonempty");
        assert_eq!(
            segment.0,
            vec_nonempty![Part::Absolute(4), Part::Unknown(2), Part::Absolute(3),]
        );
    }
    #[test]
    fn parse_record() {
        let line = ".#.##.#??.??##.? 1,5,222,99";
        let record = Record::new(line).unwrap();
        assert_eq!(
            record
                .known_counts
                .iter()
                .copied()
                .map(NonZeroUsize::get)
                .collect::<Vec<_>>(),
            vec![1, 5, 222, 99]
        );
        assert_eq!(
            record.segments,
            vec_nonempty![
                Segment(vec_nonempty![Part::Absolute(1)]),
                Segment(vec_nonempty![Part::Absolute(2)]),
                Segment(vec_nonempty![Part::Absolute(1), Part::Unknown(2)]),
                Segment(vec_nonempty![Part::Unknown(2), Part::Absolute(2)]),
                Segment(vec_nonempty![Part::Unknown(1)]),
            ]
        );
    }

    fn test_segment_count(symbols: &str, counts: &[usize], expected: usize) {
        let segment = Segment::new_from_str(symbols)
            .expect("valid line")
            .expect("nonempty symbols input");
        println!(
            "-- TEST SEGMENT COUNT: {parts:?}, expected: {expected}",
            parts = &&*segment.0
        );
        let Some(counts) = counts
            .iter()
            .copied()
            .map(NonZeroUsize::new)
            .collect::<Option<Vec<_>>>()
        else {
            panic!("invalid 0 in specified counts: {counts:?}")
        };
        let count = segment.count_possibilities(&counts);
        assert_eq!(count, expected, "symbols {symbols:?}, counts {counts:?}");
    }

    #[test]
    fn segment_count_absolute() {
        const INPUT: &str = "######";
        for count in 1..INPUT.len() {
            let expected = 1;
            test_segment_count(&INPUT[0..count], &[count], expected);
            // degenerate cases
            test_segment_count(&INPUT[0..count], &[count + 1], 0);
            if count > 1 {
                test_segment_count(&INPUT[0..count], &[count - 1], 0);
            }
        }
    }
    #[test]
    fn segment_count_unknown_zero() {
        const INPUT: &str = "#??????";
        for count in 1..INPUT.len() {
            let expected = 1;
            test_segment_count(&INPUT[0..count], &[1], expected);
        }
    }
    #[test]
    fn segment_count_unknown_block() {
        const INPUT: &str = "??????";
        for count in 1..INPUT.len() {
            let expected = 1; //2usize.pow(u32::try_from(count).unwrap() - 1);
            test_segment_count(&INPUT[0..count], &[count], expected);
            // degenerate case
            test_segment_count(&INPUT[0..count], &[count + 1], 0);
        }
    }
    #[test]
    fn segment_count_unknown_sliding() {
        const INPUT: &str = "??????";
        for count in 1..INPUT.len() {
            let expected = count;
            test_segment_count(&INPUT[0..count], &[1], expected);
        }
    }

    #[test]
    fn segment_count_toggle() {
        const INPUT: &str = "???????";
        for multiple in 1..(INPUT.len() - 1) {
            for count in multiple..INPUT.len() {
                test_segment_count(&INPUT[0..count], &[multiple], count + 1 - multiple);
            }
        }
    }

    #[test]
    fn segment_first_known() {
        const INPUT: &str = "#?????";
        const INPUT_IMPOSSIBLE: &str = "#?????#";
        for count in 1..(INPUT.len() + 1) {
            test_segment_count(INPUT, &[count], 1);
            test_segment_count(INPUT_IMPOSSIBLE, &[count], 0);
        }
    }
    #[test]
    fn segment_pivots_around_knowns() {
        test_segment_count("??#??", &[1], 1);
        test_segment_count("??#??", &[2], 2);
        test_segment_count("??#??", &[3], 3);
        test_segment_count("??#??", &[4], 2);
        test_segment_count("??#??", &[5], 1);
    }

    // TODO
    // fn test_record_count(line: &str, expected: usize) {
    //     let record = Record::new(line).expect("valid line");
    //     let count = record.count_possibilities();
    //     assert_eq!(count, expected);
    // }
    // #[test]
    // fn sample_record0() {
    //     test_record_count("#.#.### 1,1,3", 1);
    // }
    // #[test]
    // fn sample_record1() {
    //     test_record_count("???.### 1,1,3", 1);
    // }

    // TODO
    //     #[test]
    //     fn sample_input() {
    //         let input = "???.### 1,1,3
    // .??..??...?##. 1,1,3
    // ?#?#?#?#?#?#?#? 1,3,1,6
    // ????.#...#... 4,1,1
    // ????.######..#####. 1,6,5
    // ?###???????? 3,2,1
    // ";
    //         let records = Record::parse_lines(input).unwrap();
    //         let sum = sum_counts(&records);
    //         assert_eq!(sum, 21);
    //     }
}
