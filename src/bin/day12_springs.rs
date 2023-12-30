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

        let Some(segments) = NonEmptyVec::new(segments) else {
            anyhow::bail!("empty segments")
        };
        Ok(Self {
            segments,
            known_counts,
        })
    }
    fn count_possibilities(&self) -> usize {
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
                None
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

#[derive(Clone, PartialEq, Eq)]
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
        const ONE: NonZeroUsize = match NonZeroUsize::new(1) {
            Some(v) => v,
            None => [][0],
        };

        // ignore duplicate separators
        while let Some('.') = symbols.peek() {
            let _ = symbols.next();
        }

        let mut builder = SegmentBuilder::default();

        for symbol in symbols {
            let new = match symbol {
                '#' => Part::Absolute(ONE),
                '?' => Part::Unknown(ONE),
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
    fn is_nullable(&self) -> bool {
        self.0.iter().copied().all(Part::is_nullable)
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
    Absolute(NonZeroUsize),
    Unknown(NonZeroUsize),
}
impl Part {
    fn is_nullable(self) -> bool {
        match self {
            Part::Absolute(_) => false,
            Part::Unknown(_) => true,
        }
    }
}
impl std::ops::Add for Part {
    type Output = (Option<Part>, Part);
    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Absolute(count_old), Self::Absolute(count_new)) => {
                let sum =
                    NonZeroUsize::new(count_old.get() + count_new.get()).expect("nonzerousize add");
                (None, Self::Absolute(sum))
            }
            (Self::Unknown(count_old), Self::Unknown(count_new)) => {
                let sum =
                    NonZeroUsize::new(count_old.get() + count_new.get()).expect("nonzerousize add");
                (None, Self::Unknown(sum))
            }
            (Self::Absolute(_), Self::Unknown(_)) | (Self::Unknown(_), Self::Absolute(_)) => {
                (Some(self), other)
            }
        }
    }
}
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Part::Absolute(count) => {
                if count.get() == 1 {
                    write!(f, "#")
                } else {
                    write!(f, "#x{count}")
                }
            }
            Part::Unknown(count) => {
                if count.get() == 1 {
                    write!(f, "?")
                } else {
                    write!(f, "?x{count}")
                }
            }
        }
    }
}
impl std::fmt::Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <NonEmptyVec<Part> as std::fmt::Debug>::fmt(&self.0, f)
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
            for _ in 0..count.get() {
                write!(f, "{symbol}")?;
            }
        }
        write!(f, "}}")
    }
}

mod analysis {
    //! Privacy barrier for analysis-specific functions/types
    use crate::{unsplit_to_vec, DebugParts, Part, Segment};
    use advent_2023::nonempty::NonEmptyVec;
    use std::num::NonZeroUsize;

    impl Segment {
        pub(crate) fn count_possibilities(
            &self,
            expected_counts: &[NonZeroUsize],
            debug_indent: usize,
        ) -> usize {
            print!("{:width$}", "", width = debug_indent);
            println!("[Segment::count_possibilities]");
            let Some(counts_split) = expected_counts.split_first_copy() else {
                // empty counts, nonempty parts (impossible)
                return 0;
            };

            let parts_split = self.0.split_first_copy();
            SegmentAnalysis::new(parts_split, counts_split, debug_indent).count()
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
            debug_indent: usize,
        ) -> Self {
            let (part_first, parts_rest) = parts_split;
            let (count_first, counts_rest) = counts_split;
            Self {
                part_first,
                parts_rest,
                count_first,
                counts_rest,
                force_left_align: None,
                debug_indent,
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
        /// `Segment` = `Vec<Part>`, and multiple parts can cover one count
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
            print!("* count");
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
            let expected = count_first;
            match part_first {
                // Absoulute - must match completely
                Part::Absolute(count_part) => {
                    if count_part == expected {
                        (1, "absolute perfect match")
                    } else {
                        (0, "absolute NOT perfect match")
                    }
                }
                // Unknown - possible to slide around
                Part::Unknown(count_part) => {
                    if let Some(blank_area) = count_part.get().checked_sub(expected.get()) {
                        if force_left_align.is_some() {
                            (1, "unknowns left-aligned covers remaining expected")
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
        fn case_absolute(&self, count_part_absolute: NonZeroUsize) -> (usize, &'static str) {
            // anchored by first
            //
            // For a match, the absolute part MUST fully contain `count_first`
            let Some(count_remaining_opt) = self
                .count_first
                .get()
                .checked_sub(count_part_absolute.get())
                .map(NonZeroUsize::new)
            else {
                // REJECT, Part::Absolute larger than count_first
                return (0, "absolute too long for current count");
            };

            let (counts_next, new_force_left_align) =
                if let Some(count_remaining) = count_remaining_opt {
                    // Absolute matches all, force align for remaining
                    ((count_remaining, self.counts_rest), Some(ForceLeftAlign))
                } else {
                    // Pop count
                    let Some(counts_next) = self.counts_rest.split_first_copy() else {
                        // verify remainder is all Unknowns
                        let possible = self.parts_rest.iter().copied().all(Part::is_nullable);
                        return if possible {
                            (1, "absolute exhausted count, remainder is nullable")
                        } else {
                            (0, "absolute exhausted count, but remainder is not nullable")
                        };
                    };
                    (counts_next, None)
                };

            let Some((part_next, parts_rest)) = self.parts_rest.split_first_copy() else {
                return (0, "absolute exhausted parts, but counts remain");
            };
            let parts_next: (Part, &[Part]) = if count_remaining_opt.is_some() {
                (part_next, parts_rest)
            } else {
                // perfect match, need to nullify immediate next
                match part_next {
                    Part::Unknown(part_count) => {
                        if let Some(part_count_reduced) =
                            part_count.get().checked_sub(1).and_then(NonZeroUsize::new)
                        {
                            (Part::Unknown(part_count_reduced), parts_rest)
                        } else {
                            let Some(parts_next_split2) = parts_rest.split_first_copy() else {
                                return (0, "absolute perfect match, but after consuming separator no parts remain to satify next counts");
                            };
                            parts_next_split2
                        }
                    }
                    Part::Absolute(_) => {
                        return (
                                0,
                                "absolute perfect match, but need separator and immediate next is absolute",
                            );
                    }
                }
            };

            let result = self
                .recurse(
                    "part pop_front, count reduced",
                    parts_next,
                    counts_next,
                    new_force_left_align,
                )
                .count();
            (result, "recurse")
        }
        fn case_unknown(&self, count_part_unknown: NonZeroUsize) -> (usize, &'static str) {
            if self.force_left_align.is_some()
                && self.counts_rest.is_empty()
                && count_part_unknown >= self.count_first
                && !self.parts_rest.iter().copied().all(Part::is_nullable)
            {
                return (
                    0,
                    "unknowns longer than final count, but remainder not nullable and force_left_align",
                );
            }

            // FIXME avoid N-based recurse for Unknowns...
            // For now, input is not excessively long so recurse for each question-mark choice

            let part_reduced_opt = count_part_unknown
                .get()
                .checked_sub(1)
                .map(NonZeroUsize::new);
            let parts_reduced = match part_reduced_opt {
                Some(Some(part_reduced)) => {
                    // if let Some(part_reduced) = count_part_unknown.checked_sub(1)
                    let part_reduced = Part::Unknown(part_reduced);
                    // -1 to first
                    (part_reduced, self.parts_rest)
                }
                Some(None) | None => {
                    // if count_part_unknown == ONE
                    // else

                    // remove empty first
                    let Some(parts_next) = self.parts_rest.split_first_copy() else {
                        return (0, "unknown exhausted parts, but counts remain");
                    };
                    parts_next
                }
            };
            let parts_reduced_2 = part_reduced_opt.flatten().and_then(|part_reduced| {
                let part_reduced_2 = part_reduced.get().checked_sub(1).map(NonZeroUsize::new);
                match part_reduced_2 {
                    Some(Some(part_reduced_2)) => {
                        let part_reduced_2 = Part::Unknown(part_reduced_2);
                        // -2 (total) to first
                        Some((part_reduced_2, self.parts_rest))
                    }
                    Some(None) | None => {
                        // remove empty first
                        self.parts_rest.split_first_copy()
                    }
                }
            });

            // split into two possibilities (ON, OFF)
            let unknown_on = {
                let count_reduced_opt =
                    self.count_first.get().checked_sub(1).map(NonZeroUsize::new);
                match count_reduced_opt {
                    Some(None) => {
                        // if self.count_first.get() == 1
                        if let Some(counts_next) = self.counts_rest.split_first_copy() {
                            if let Some(parts_reduced_2) = parts_reduced_2 {
                                // use `part_reduced_2` to add pseudo-separatorof Unknown OFF, after the assumed ON
                                self.recurse(
                                    "assume Unknown = ON, pop_front count, parts reduce x2",
                                    parts_reduced_2,
                                    counts_next,
                                    None,
                                )
                                .count()
                            } else {
                                // the assumed "Unknown ON" completes the count,
                                // but there remaining counts with no remaining part
                                0
                            }
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
                    }
                    Some(Some(count_reduced)) => {
                        // else if let Some(count_reduced) = self.count_first.checked_sub(1)
                        self.recurse(
                            "assume Unknown = ON, reduce count",
                            parts_reduced,
                            (count_reduced, self.counts_rest),
                            Some(ForceLeftAlign),
                        )
                        .count()
                    }
                    None => {
                        //else
                        // cannot reduce count, impossible for ON
                        dbg!(0)
                    }
                }
            };
            let unknown_off = if self.force_left_align.is_some() {
                // cannot assume off, forced left align
                dbg!(0)
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

    // Convenience functions (as traits, for foreign types)
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
}

#[cfg(test)]
mod tests {
    use crate::{sum_counts, Part, Record, Segment};
    use advent_2023::vec_nonempty;
    use std::num::NonZeroUsize;

    macro_rules! vec_parts {
        ($( $elem:ident ( $value:expr ) ),+ $(,)?) => {
            vec_nonempty![ $( vec_parts![@elem $elem ( $value ) ] ),+ ]
        };
        (@elem Absolute($value:expr)) => {{
            const VALUE: NonZeroUsize = match NonZeroUsize::new($value) {
                Some(v) => v,
                None => [][0],
            };
            Part::Absolute(VALUE)
        }};
        (@elem Unknown($value:expr)) => {{
            const VALUE: NonZeroUsize = match NonZeroUsize::new($value) {
                Some(v) => v,
                None => [][0],
            };
            Part::Unknown(VALUE)
        }};
    }

    #[test]
    fn parse_segment() {
        let symbols = ".......####??###.";
        let segment = Segment::new_from_str(symbols)
            .expect("characters valid")
            .expect("nonempty");
        assert_eq!(segment.0, vec_parts![Absolute(4), Unknown(2), Absolute(3)]);
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
                Segment(vec_parts![Absolute(1)]),
                Segment(vec_parts![Absolute(2)]),
                Segment(vec_parts![Absolute(1), Unknown(2)]),
                Segment(vec_parts![Unknown(2), Absolute(2)]),
                Segment(vec_parts![Unknown(1)]),
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
        let count = segment.count_possibilities(&counts, 0);
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
    fn segment_count_split_unknowns() {
        // 1: #.#
        test_segment_count("???", &[1, 1], 1);
        // 1: #.#.
        // 2: #..#
        // 3: .#.#
        test_segment_count("????", &[1, 1], 3);
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

    #[test]
    fn segment_robust_1() {
        test_segment_count("????#?#?", &[7, 2], 0);
    }

    fn test_record_count(line: &str, expected: usize) {
        let record = Record::new(line).expect("valid line");
        let count = record.count_possibilities();
        assert_eq!(count, expected);
    }
    #[test]
    fn sample_record_perfect() {
        test_record_count("#.#.### 1,1,3", 1);
    }
    #[test]
    fn sample_record_impossible() {
        test_record_count("???.### 1,1,3", 1);
    }
    #[test]
    fn sample_record_sliding_multicount() {
        // 1: #.#.
        // 2: .#.#
        // ---
        // 3: #..#
        test_record_count("???? 1,1", 2 + 1);
        // 1. #.#.#.
        // 2. .#.#.#
        // ---
        // 3: #..#.#
        // 4: #.#..#
        test_record_count("?????? 1,1,1", 2 + 2);
        // 1. #.#.#.#.
        // 2. .#.#.#.#
        // ---
        // 3. #..#.#.#
        // 4. #.#..#.#
        // 5. #.#.#..#
        test_record_count("???????? 1,1,1,1", 2 + 3);
        // 1. #.#.#.#.#.
        // 2. .#.#.#.#.#
        // ---
        // 3. #..#.#.#.#
        // 4. #.#..#.#.#
        // 5. #.#.#..#.#
        // 6. #.#.#.#..#
        test_record_count("?????????? 1,1,1,1,1", 2 + 4);
    }

    #[test]
    fn sample_input_record_1() {
        test_record_count("???.### 1,1,3", 1);
    }

    #[test]
    fn sample_input_record_2() {
        test_record_count(".??..??...?##. 1,1,3", 4);
    }

    #[test]
    fn sample_input_record_3_pretest() {
        //    #?#?#?
        // 1. ######
        test_record_count("#?#?#? 6", 1);
    }
    #[test]
    fn sample_input_record_3_pretest_2() {
        //    #?#?#?#?
        // 1. #.######
        test_record_count("#?#?#?#? 1,6", 1);
    }
    #[test]
    fn sample_input_record_3_pretest_3() {
        //    #?#?#?#?#?
        // 1. #.#.######
        test_record_count("#?#?#?#?#? 1,1,6", 1);
    }
    #[test]
    fn sample_input_record_3() {
        //    ?#?#?#?#?#?#?#?
        // 1. .#.###.#.######
        test_record_count("?#?#?#?#?#?#?#? 1,3,1,6", 1);
    }

    #[test]
    fn sample_input_record_4() {
        test_record_count("????.#...#... 4,1,1", 1);
    }

    #[test]
    fn sample_input_record_5() {
        test_record_count("????.######..#####. 1,6,5", 4);
    }

    #[test]
    fn sample_input_record_6_pretest() {
        // 1. ##.#...
        // 2. .##.#..
        // 3. ..##.#.
        // 4. ...##.#
        // ---
        // 5. ##..#..
        // 6. .##..#.
        // 7. ..##..#
        // ---
        // 8. ##...#.
        // 9. .##...#
        // ---
        // 10 ##....#
        test_record_count("??????? 2,1", 10)
    }

    #[test]
    fn sample_input_record_6() {
        test_record_count("?###???????? 3,2,1", 10);
    }

    #[test]
    fn sample_input() {
        let input = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
";
        let records = Record::parse_lines(input).unwrap();
        let sum = sum_counts(&records);
        assert_eq!(sum, 21);
    }

    #[test]
    fn test_record_38() {
        //     ?###.?#?#????#????? 4,1,1,1,3,2
        //  0. ####..#.#.
        //
        //  Reduces to:
        //     ???#????? 1,3,2
        //  1. #.###.##.
        //  2. .#.###.##
        //  ---
        //  3. #..###.##
        //  ---
        //     ???#????? 1,3,2
        //  4. #.###..##
        //
        test_record_count("?###.?#?#????#????? 4,1,1,1,3,2", 4);
    }
    #[test]
    fn test_record_99() {
        //     ???#????? 1,3
        //  1. #.###....
        //  2. .#.###...
        //  ---
        //  3. #..###...
        //  ---
        //  4. ...#.###.
        //  5. ...#..###
        test_record_count(".???#????? 1,3", 5);
    }
    #[test]
    fn test_record_140() {
        //     #??#????#??#?##??? 1,1,1,1,6,1
        //  0. #..#.
        //  ---
        //     ???#??#?##??? 1,1,6,1
        //  1. #..#.######.#
        //  2. .#.#.######.#
        test_record_count("#??#????#??#?##??? 1,1,1,1,6,1", 2);
    }

    #[test]
    fn test_record_951() {
        //     ???.????#????????? 1,7,5
        //  1. #...#######.#####.
        //  2. .#..#######.#####.
        //  3. ..#.#######.#####.
        //  ---
        //  4. #....#######.#####
        //  5. .#...#######.#####
        //  6. ..#..#######.#####
        //  ---
        //  7. #...#######..#####
        //  8. .#..#######..#####
        //  9. ..#.#######..#####
        //  ---
        test_record_count("???.????#?????????. 1,7,5", 9);
    }
    #[test]
    fn test_record_952_precheck_1() {
        //     ?.#?.?#? 1,2
        //  1. ..#..##.
        //  2. ..#...##
        test_record_count("?.#?.?#? 1,2", 2);
    }
    #[test]
    fn test_record_952() {
        //     ?.?.#?.?#? 1,2
        //  1. ....#..##.
        //  2. ....#...##
        test_record_count("?.?.#?.?#? 1,2", 2);
    }
}
