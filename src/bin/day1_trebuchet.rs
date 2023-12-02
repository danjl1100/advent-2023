use clap::Parser;
use std::{
    collections::{BTreeMap, VecDeque},
    io::Read,
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    filename: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    println!("hello, trebuchet!");
    let args = Args::parse();

    let input = if let Some(filename) = args.filename {
        std::fs::read_to_string(filename)?
    } else {
        println!("Awaiting instructions from stdin:");
        let mut input_buf = String::new();
        std::io::stdin().read_to_string(&mut input_buf)?;
        input_buf
    };

    let sum = sum_calibration_amended_by_a_very_young_elf(&input);
    println!("Sum is: {sum}");

    Ok(())
}

struct CharScanner<'a> {
    line: &'a str,
    char_indices: std::iter::Peekable<std::str::CharIndices<'a>>,
    last_indices: VecDeque<usize>,
    number_words: BTreeMap<&'static str, usize>,
}
const LOOKBACK_LEN: usize = 5;
const LOOKBACK_MIN: usize = 3;
const LOOKBACK_MIN_INDEX: usize = LOOKBACK_MIN - 1;
const BASE_10: u32 = 10;

impl<'a> CharScanner<'a> {
    fn new(line: &'a str) -> Self {
        let char_indices = line.char_indices().peekable();

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
            line,
            char_indices,
            last_indices: VecDeque::with_capacity(LOOKBACK_LEN + 1),
            number_words,
        }
    }
}
impl Iterator for CharScanner<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((current_index, current_char)) = self.char_indices.next() {
            let next_index = self.char_indices.peek().map(|(index, _char)| index);
            self.last_indices.truncate(LOOKBACK_LEN - 1);
            self.last_indices.push_front(current_index);

            if let Some(digit) = current_char.to_digit(BASE_10) {
                self.last_indices.clear();
                return Some(digit);
            }

            for lookback in LOOKBACK_MIN_INDEX..LOOKBACK_LEN {
                if let Some(&start_index) = self.last_indices.get(lookback) {
                    let last_part = next_index
                        .map(|&next| {
                            self.line
                                .get(start_index..next)
                                .expect("slicing on char boundaries")
                        })
                        .unwrap_or(self.line.get(start_index..).expect("slicing on start char"));
                    if let Some(&number) = self.number_words.get(last_part) {
                        return Some(u32::try_from(number).expect("single digit fits in u32"));
                    }
                } else {
                    break;
                }
            }
        }
        // exhausted char_indices
        None
    }
}

fn sum_calibration_amended_by_a_very_young_elf(input: &str) -> u32 {
    input
        .lines()
        .filter_map(|line| {
            // NOTE: a single digit on the line means *BOTH* first and last digits are that digit
            let mut digits = CharScanner::new(line);
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
}
