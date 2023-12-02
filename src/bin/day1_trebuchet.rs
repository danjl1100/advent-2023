use clap::Parser;
use std::{io::Read, path::PathBuf};

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

fn sum_calibration_amended_by_a_very_young_elf(input: &str) -> u32 {
    const BASE_10: u32 = 10;
    input
        .lines()
        .filter_map(|line| {
            // NOTE: a single digit on the line means *BOTH* first and last digits are that digit
            let mut digits = line.chars().filter_map(|char| char.to_digit(BASE_10));
            let first_digit = digits.next()?;
            let last_digit = digits.next_back().unwrap_or(first_digit);

            let number = first_digit * BASE_10 + last_digit;
            dbg!((first_digit, last_digit, number));
            Some(number)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::sum_calibration_amended_by_a_very_young_elf;

    #[test]
    fn simple() {
        let input = "this1
2is3
4a5
6digit7
8sum9";

        assert_eq!(
            sum_calibration_amended_by_a_very_young_elf(input),
            11 + 23 + 45 + 67 + 89
        );
    }

    #[test]
    fn more_digits() {
        let input = "0th7is1
2isignoring5exta4numbers3
4while524doing0234a4full5
6digit7
8sum9";

        assert_eq!(
            sum_calibration_amended_by_a_very_young_elf(input),
            1 + 23 + 45 + 67 + 89
        );
    }
}
