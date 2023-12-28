use advent_2023::nonempty::NonEmptyVec;
use anyhow::Context;

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
    known_counts: Vec<usize>,
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
    fn count_possibilities(&self, expected_counts: &[usize]) -> usize {
        let parts = &self.0;

        let Some((&count_first, counts_rest)) = expected_counts.split_first() else {
            // empty counts, nonempty parts (impossible)
            return 0;
        };

        let (&part_first, parts_rest) = parts.split_first();

        if parts_rest.is_empty() && counts_rest.is_empty() {
            match dbg!(part_first, count_first) {
                (Part::Absolute(count), expected) if count == expected => 1,
                (Part::Absolute(_), _) => 0,
                (Part::Unknown(count), expected) => {
                    if count == expected {
                        1
                    } else if let Some(blank_area) = count.checked_sub(expected) {
                        // blanked area can slide, with one side containing: 0..=blank_area
                        dbg!(blank_area + 1)
                    } else {
                        // expecting more than possible
                        0
                    }
                }
            }
        } else {
            // TODO
            todo!()
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Absolute(usize),
    Unknown(usize),
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

#[cfg(test)]
mod tests {
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
        assert_eq!(record.known_counts, vec![1, 5, 222, 99]);
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
        println!("------------------------------");
        let record = Segment::new_from_str(symbols)
            .expect("valid line")
            .expect("nonempty symbols input");
        let count = record.count_possibilities(counts);
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
            test_segment_count(&INPUT[0..count], &[count - 1], 0);
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
