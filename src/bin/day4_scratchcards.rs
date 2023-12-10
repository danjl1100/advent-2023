use std::str::FromStr;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    println!("hello, tell the elf what they've won...");
    let input = advent_2023::get_input_string()?;

    let Stats {
        total_score,
        number_of_cards,
    } = evaluate_scatchcards(&input)?;
    println!("Elf's total score: {total_score}");
    println!("Total number of cards (there is no actual score): {number_of_cards}");

    Ok(())
}

struct Stats {
    total_score: u32,
    number_of_cards: usize,
}
fn evaluate_scatchcards(input: &str) -> anyhow::Result<Stats> {
    let card_wins = input
        .lines()
        .map(|line| {
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
            Ok(matching_count)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let card_wins_len = card_wins.len();
    let mut card_multipliers: Vec<_> = std::iter::repeat(1).take(card_wins_len).collect();
    for (this_game, score) in card_wins.iter().enumerate() {
        let score = usize::try_from(*score).expect("score fits in usize");
        let this_multiplier = card_multipliers[this_game];
        println!("card {this_game}: {score}");
        for next_game in (0..score).filter_map(|relative| {
            let absolute = relative + 1 + this_game;
            (absolute < card_wins_len).then_some(absolute)
        }) {
            let orig = card_multipliers[next_game];
            println!("\tcard {next_game} += {this_multiplier} (was {orig})");
            card_multipliers[next_game] += this_multiplier;
        }
    }

    let card_scores: Vec<_> = card_wins
        .iter()
        .map(|&matching_count| {
            if let Some(matches_less_one) = matching_count.checked_sub(1) {
                2u32.pow(matches_less_one)
            } else {
                0
            }
        })
        .collect();
    let total_score = card_scores.iter().sum();

    let number_of_cards = card_multipliers.iter().sum();

    Ok(Stats {
        total_score,
        number_of_cards,
    })
}
