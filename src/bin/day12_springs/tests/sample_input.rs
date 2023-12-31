use crate::{day12_springs::record::Record, sum_counts, FACTOR_5};

fn test_record_counts_only_unfolded(line: &str, expect_unfolded: usize) {
    let record = Record::new(line).expect("valid line");
    // let count = record.count_possibilities();
    // assert_eq!(count, expected, "record (prior to unfold)");

    let record_unfolded = record.unfold(FACTOR_5);
    let count_unfolded = record_unfolded.count_possibilities();
    assert_eq!(count_unfolded, expect_unfolded, "unfolded record");
}

fn test_record_counts(line: &str, (expected, expect_unfolded): (usize, usize)) {
    let record = Record::new(line).expect("valid line");
    let count = record.count_possibilities();
    assert_eq!(count, expected, "record (prior to unfold)");

    let record_unfolded = record.unfold(FACTOR_5);
    let count_unfolded = record_unfolded.count_possibilities();
    assert_eq!(count_unfolded, expect_unfolded, "unfolded record");
}

#[test]
fn sample_input_record_1() {
    test_record_counts("???.### 1,1,3", (1, 1));
}

#[test]
fn sample_input_record_2() {
    test_record_counts(".??..??...?##. 1,1,3", (4, 16384));
}

#[test]
fn sample_input_record_3_pretest() {
    //    #?#?#?
    // 1. ######
    test_record_counts("#?#?#? 6", (1, 1));
    // NOTE: not free to "slide" on the extra ?, since that is set ?=0 as a separator
}
#[test]
fn sample_input_record_3_pretest_2() {
    //    #?#?#?#?
    // 1. #.######
    test_record_counts("#?#?#?#? 1,6", (1, 1));
}
#[test]
fn sample_input_record_3_pretest_3() {
    //    #?#?#?#?#?
    // 1. #.#.######
    test_record_counts("#?#?#?#?#? 1,1,6", (1, 1));
}
#[test]
fn sample_input_record_3() {
    //    ?#?#?#?#?#?#?#?
    // 1. .#.###.#.######
    test_record_counts("?#?#?#?#?#?#?#? 1,3,1,6", (1, 1));
}

#[test]
fn sample_input_record_4() {
    test_record_counts("????.#...#... 4,1,1", (1, 16));
}

#[test]
fn sample_input_record_5() {
    super::test_record_count("????.######..#####. 1,6,5", 4);
    // TODO
    // let expected = todo!();
    // test_record_counts("????.######..#####. 1,6,5", (4, expected));
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
    super::test_record_count("??????? 2,1", 10);
    // TODO
    // todo!();
    // // unfolded length is 7*5 + 4 = 39
    // // 1. ##.# (len 4, possibilities 0..=35 -> 36)
    // // 2. ##..# (len 5, possibilities 0..=34 -> 35)
    // // so 36 + 35 + 34 + .. + 1 = 36 * 35 / 2 = 630
    // test_record_counts("??????? 2,1", (10, 630));
}

#[test]
fn sample_input_record_6() {
    // vvvv--- `true` for BACON usage
    if true {
        super::test_record_count("?###???????? 3,2,1", 10);
        // TODO
        // test_record_counts("?###???????? 3,2,1", (10, 506250));
    } else {
        // TODO resume HERE
        test_record_counts_only_unfolded("?###???????? 3,2,1", 506250);
    }
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

//     TODO
//     #[test]
//     fn sample_input_unfolded() {
//         let input = "???.### 1,1,3
// .??..??...?##. 1,1,3
// ?#?#?#?#?#?#?#? 1,3,1,6
// ????.#...#... 4,1,1
// ????.######..#####. 1,6,5
// ?###???????? 3,2,1
// ";
//         let records = Record::parse_lines(input).unwrap();
//         let unfolded = records
//             .into_iter()
//             .map(|record| record.unfold(FACTOR_5))
//             .collect::<Vec<_>>();
//         let sum = sum_counts(&unfolded);
//         assert_eq!(sum, 525152);
//     }
