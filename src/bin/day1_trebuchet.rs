use advent_2023::CharScanner;
use clap::Parser;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    filename: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    println!("hello, trebuchet!");
    let input = advent_2023::get_input_string()?;

    let sum = sum_calibration_amended_by_a_very_young_elf(&input);
    println!("Sum is: {sum}");

    Ok(())
}

struct NumberScanner<'a> {
    scanner: CharScanner<'a, u32>,
    number_words: BTreeMap<&'static str, usize>,
}
const LOOKBACK_RANGE: (usize, usize) = (3, 5);
const BASE_10: u32 = 10;

impl<'a> NumberScanner<'a> {
    fn new(line: &'a str) -> Self {
        let number_words = BTreeMap::from_iter([
            ("zero", 0),
            ("one", 1),
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
            ("six", 6),
            ("seven", 7),
            ("eight", 8),
            ("nine", 9),
        ]);
        Self {
            scanner: CharScanner::new(line, Some(LOOKBACK_RANGE)),
            number_words,
        }
    }
}
impl Iterator for NumberScanner<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let f_single_char = |current_char: char, _current_index| current_char.to_digit(BASE_10);
        let f_lookback_str = |last_part: &str, _index_range| {
            self.number_words
                .get(last_part)
                .map(|&number| u32::try_from(number).expect("single digit fits in u32"))
        };
        self.scanner
            .find_next(Some(f_single_char), Some(f_lookback_str))
    }
}

fn sum_calibration_amended_by_a_very_young_elf(input: &str) -> u32 {
    input
        .lines()
        .filter_map(|line| {
            // NOTE: a single digit on the line means *BOTH* first and last digits are that digit
            let mut digits = NumberScanner::new(line);
            // let mut digits = line.chars().filter_map(|char| char.to_digit(BASE_10));
            let first_digit = digits.next()?;
            let last_digit = digits.last().unwrap_or(first_digit);

            let number = first_digit * BASE_10 + last_digit;
            Some(number)
        })
        .sum()
}

#[cfg(test)]
mod tests {

    fn test_fn(input: &str) -> u32 {
        crate::sum_calibration_amended_by_a_very_young_elf(input)
    }

    #[test]
    fn simple() {
        let input = "this1
            2is3
            4a5
            6digit7
            8sum9";

        assert_eq!(test_fn(input), 11 + 23 + 45 + 67 + 89);
    }

    #[test]
    fn more_digits() {
        let input = "0th7is1
            2isignoring5exta4numbers3
            ignored4while524doing0234a4full5
            6digit7
            yeah8sum9";

        assert_eq!(test_fn(input), 1 + 23 + 45 + 67 + 89);
    }

    #[test]
    fn also_digit_words() {
        let input = "thisone
            also2accepts4
            digits5likeone
            twoandthree
            and3four";

        assert_eq!(test_fn(input), 11 + 24 + 51 + 23 + 34);
    }

    #[test]
    fn single_digit_0() {
        assert_eq!(test_fn("zero"), 0);
    }
    #[test]
    fn single_digit_1() {
        assert_eq!(test_fn("one"), 11);
    }
    #[test]
    fn single_digit_2() {
        assert_eq!(test_fn("two"), 22);
    }
    #[test]
    fn single_digit_3() {
        assert_eq!(test_fn("three"), 33);
    }
    #[test]
    fn single_digit_4() {
        assert_eq!(test_fn("four"), 44);
    }
    #[test]
    fn single_digit_5() {
        assert_eq!(test_fn("five"), 55);
    }
    #[test]
    fn single_digit_6() {
        assert_eq!(test_fn("six"), 66);
    }
    #[test]
    fn single_digit_7() {
        assert_eq!(test_fn("seven"), 77);
    }
    #[test]
    fn single_digit_8() {
        assert_eq!(test_fn("eight"), 88);
    }
    #[test]
    fn single_digit_9() {
        assert_eq!(test_fn("nine"), 99);
    }

    #[test]
    fn example_digits_only() {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";
        assert_eq!(test_fn(input), 142);
    }
    #[test]
    fn example_spelled_out() {
        let input = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";
        assert_eq!(test_fn(input), 281);
    }
}
