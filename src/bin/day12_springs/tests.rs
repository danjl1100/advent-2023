use crate::{Part, Record, Segment};
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

mod record_unfold;

mod sample_input;

mod oddly_specific;

#[test]
fn parse_segment() {
    let symbols = ".......####??###.";
    let (_lead, segment, _trail) = Segment::new_from_str(symbols)
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
            .known_counts()
            .iter()
            .copied()
            .map(NonZeroUsize::get)
            .collect::<Vec<_>>(),
        vec![1, 5, 222, 99]
    );
    assert_eq!(
        record.segments(),
        &vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(2)]),
            Segment(vec_parts![Absolute(1), Unknown(2)]),
            Segment(vec_parts![Unknown(2), Absolute(2)]),
            Segment(vec_parts![Unknown(1)]),
        ]
    );
}
#[test]
fn parse_record_separators() {
    macro_rules! test {
        ($str:expr, $expected:expr) => {
            assert_eq!(Record::new($str).unwrap().separators(), $expected);
        };
    }
    test!("#.##.?? 1", (false, false));
    test!(".#.##.?? 1", (true, false));
    test!("#.##.??. 1", (false, true));
    test!(".#.##.??. 1", (true, true));
}

fn test_segment_count(symbols: &str, counts: &[usize], expected: usize) {
    let (_lead, segment, _trail) = Segment::new_from_str(symbols)
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

fn test_record_count(line: &str, expected: usize) {
    let record = Record::new(line).expect("valid line");
    let count = record.count_possibilities();
    assert_eq!(count, expected);
}

#[test]
fn segment_robust_1() {
    test_segment_count("????#?#?", &[7, 2], 0);
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
