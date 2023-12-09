use std::str::FromStr;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    println!("hello, tell the elf what they've won...");
    let input = advent_2023::get_input_string()?;

    let Stats { total_score } = evaluate_scatchcards(&input)?;
    println!("Elf's total score: {total_score}");

    Ok(())
}

struct Stats {
    total_score: u32,
}
fn evaluate_scatchcards(input: &str) -> anyhow::Result<Stats> {
    let mut total_score = 0;
    for line in input.lines() {
        let Some(colon_position) = line.find(":") else {
            panic!("no colon, invalid line {line:?}");
        };
        let Some(pipe_position) = line.find("|") else {
            panic!("no pipe, invalid line {line:?}");
        };
        let winning_numbers = {
            let winning_numbers_str = &line[(colon_position + 1)..pipe_position];
            let mut winning_numbers = winning_numbers_str
                .split_whitespace()
                .map(u32::from_str)
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| winning_numbers_str.to_string())?;
            winning_numbers.sort();
            winning_numbers
        };

        let current_numbers = {
            let current_numbers_str = &line[(pipe_position + 1)..];
            current_numbers_str
                .split_whitespace()
                .map(u32::from_str)
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| current_numbers_str.to_string())?
        };

        let matching_count = current_numbers
            .iter()
            .filter(|&n| winning_numbers.binary_search(n).is_ok())
            .count();
        let matching_count = u32::try_from(matching_count).expect("matching_count fits in u32");

        let score = if let Some(matches_less_one) = matching_count.checked_sub(1) {
            2u32.pow(matches_less_one)
        } else {
            0
        };
        total_score += score;
    }
    Ok(Stats { total_score })
}
