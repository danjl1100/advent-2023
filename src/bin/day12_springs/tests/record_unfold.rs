use crate::{Part, Record, Segment, FACTOR_5};
use advent_2023::vec_nonempty;
use std::num::NonZeroUsize;

macro_rules! test_record_unfold {
    (
        symbols = $symbols:expr;
        original = $original:expr;
        unfolded = $unfolded:expr;
    ) => {{
        use advent_2023::nonempty::NonEmptyVec;

        let symbols: &'static str = $symbols;
        let original: NonEmptyVec<Segment> = $original;
        let unfolded: NonEmptyVec<Segment> = $unfolded;

        let record = Record::new(&format!("{symbols} 1")).unwrap();
        assert_eq!(
            record
                .known_counts()
                .iter()
                .copied()
                .map(NonZeroUsize::get)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(record.segments(), &original);

        let record_unfolded = record.unfold(FACTOR_5);

        assert_eq!(
            record_unfolded
                .known_counts()
                .iter()
                .copied()
                .map(NonZeroUsize::get)
                .collect::<Vec<_>>(),
            vec![1, 1, 1, 1, 1]
        );
        assert_eq!(record_unfolded.segments(), &unfolded);

        let expected = Record::new(&format!(
            "{symbols}?{symbols}?{symbols}?{symbols}?{symbols} 1,1,1,1,1"
        ))
        .unwrap();
        assert_eq!(record_unfolded, expected);
    }};
}

#[test]
fn leading_sep() {
    test_record_unfold!(
        symbols = ".#";
        original = vec_nonempty![Segment(vec_parts![Absolute(1)])];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}
#[test]
fn leading_sep_multi() {
    test_record_unfold!(
        symbols = ".#.#";
        original = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1), Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}

#[test]
fn trailing_sep() {
    test_record_unfold!(
        symbols = "#.";
        original = vec_nonempty![Segment(vec_parts![Absolute(1)])];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            //
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Unknown(1), Absolute(1)]),
        ];
    );
}
#[test]
fn trailing_sep_multi() {
    test_record_unfold!(
        symbols = "#.#.";
        original = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            //
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            //
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            //
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            //
            Segment(vec_parts![Unknown(1), Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}

#[test]
fn no_sep() {
    test_record_unfold!(
        symbols = "#";
        original = vec_nonempty![Segment(vec_parts![Absolute(1)])];
        unfolded = vec_nonempty![Segment(vec_parts![
            Absolute(1),
            Unknown(1),
            //
            Absolute(1),
            Unknown(1),
            //
            Absolute(1),
            Unknown(1),
            //
            Absolute(1),
            Unknown(1),
            //
            Absolute(1),
        ])];
    );
}
#[test]
fn no_sep_multi() {
    test_record_unfold!(
        symbols = "#.#";
        original = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![
                Absolute(1),
                Unknown(1),
                Absolute(1),
            ]),
            Segment(vec_parts![
                Absolute(1),
                Unknown(1),
                Absolute(1),
            ]),
            Segment(vec_parts![
                Absolute(1),
                Unknown(1),
                Absolute(1),
            ]),
            Segment(vec_parts![
                Absolute(1),
                Unknown(1),
                Absolute(1),
            ]),
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}

#[test]
fn all_seps() {
    test_record_unfold!(
        symbols = ".#.";
        original = vec_nonempty![Segment(vec_parts![Absolute(1)])];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}
#[test]
fn all_seps_multi() {
    test_record_unfold!(
        symbols = ".#.#.";
        original = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
        unfolded = vec_nonempty![
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Unknown(1)]),
            //
            Segment(vec_parts![Absolute(1)]),
            Segment(vec_parts![Absolute(1)]),
        ];
    );
}
