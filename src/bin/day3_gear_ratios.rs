use std::collections::HashMap;

use entity_scanner::EntityScanner;

fn main() -> anyhow::Result<()> {
    println!("hello engine-fixing elf!");

    let input = advent_2023::get_input_string()?;
    let Stats { part_numbers_sum } = interpret_engine_schematic(&input);

    println!("Sum of part numbers: {part_numbers_sum}");

    Ok(())
}

#[derive(Debug)]
struct Stats {
    part_numbers_sum: u32,
}
fn interpret_engine_schematic(input: &str) -> Stats {
    let entities: Vec<_> = input
        .lines()
        .enumerate()
        .flat_map(|(row, line)| EntityScanner::new(row, line))
        .collect();

    // DEBUG print entities
    if false {
        let mut lines_iter = input.lines().enumerate().peekable();
        let print_line = |(line_number, line)| {
            println!("LINE {:03}: {line}", line_number);
        };
        for entity in &entities {
            while let Some(true) = lines_iter.peek().map(|&(index, _)| entity.row() >= index) {
                print_line(lines_iter.next().expect("peeked some"));
            }
            let entity_col_offset = entity.col_start();
            let padding = " ".to_string().repeat(entity_col_offset + 10);
            match entity {
                Entity::Number(number) => println!("{padding}{number}"),
                Entity::Symbol(symbol) => println!("{padding}{symbol}"),
            }
        }
        for line_elem in lines_iter {
            print_line(line_elem);
        }
    }

    let mut symbols_by_row: HashMap<usize, Vec<Symbol>> = HashMap::new();
    for symbol in entities.iter().copied().filter_map(|entity| match entity {
        Entity::Symbol(symbol) => Some(symbol),
        _ => None,
    }) {
        let row = symbol.location.row;
        symbols_by_row.entry(row).or_default().push(symbol);
    }

    // DEBUG print entities
    if true {
        let lines: Vec<_> = input.lines().collect();
        let print_lines = |row: usize| {
            for n in row.saturating_sub(1)..lines.len().min(row + 2) {
                let line = lines[n];
                println!("LINE {n:03}: {line}");
            }
        };
        let mut prev_line_printed = None;
        for entity in &entities {
            let entity_col_offset = entity.col_start();
            let padding = " ".to_string().repeat(entity_col_offset + 10);
            match entity {
                Entity::Number(number) if !number.region.adjacent_to_symbol(&symbols_by_row) => {
                    let row = entity.row();
                    match prev_line_printed {
                        Some(prev) if prev == row => {}
                        _ => {
                            print_lines(row);
                            prev_line_printed = Some(row);
                        }
                    }
                    let value = number.value;
                    println!("{padding}{value} NOT adjacent");
                }
                // Entity::Symbol(symbol) => println!("{padding}{symbol}"),
                _ => {}
            }
        }
    }

    let part_numbers_sum = entities
        .iter()
        .filter_map(|entity| match entity {
            Entity::Number(number) => {
                if number.region.adjacent_to_symbol(&symbols_by_row) {
                    Some(number.value)
                } else {
                    println!("NOT ADJACENT: {number}");
                    None
                }
            }
            _ => None,
        })
        .sum();

    Stats { part_numbers_sum }
}

mod entity_scanner {
    use advent_2023::{CharIndex, CharIndexEnd, CharScanner};

    use crate::{Entity, Location, Number, Region, Symbol};

    const EMPTY_CHAR: char = '.';
    const BASE_10: u32 = 10;
    pub struct EntityScanner<'a> {
        char_scanner: CharScanner<'a, Entity>,
        row: usize,
    }
    impl<'a> EntityScanner<'a> {
        pub fn new(row: usize, line: &'a str) -> Self {
            let lookback_all = (1, line.len());
            Self {
                char_scanner: CharScanner::new(line, Some(lookback_all)),
                row,
            }
        }
    }
    impl Iterator for EntityScanner<'_> {
        type Item = Entity;
        fn next(&mut self) -> Option<Self::Item> {
            let Self { row, .. } = *self;
            let f_single_char = |current_char: char, col: CharIndex| {
                (current_char != EMPTY_CHAR && !current_char.is_digit(BASE_10)).then_some(
                    Entity::Symbol(Symbol {
                        value: current_char,
                        location: Location {
                            row: self.row,
                            col_sequence: col.sequence(),
                        },
                    }),
                )
            };
            let f_lookback_str =
                |last_str: &str, (start_index, end_index): (CharIndex, CharIndexEnd)| {
                    let is_digit = |c: char| c.is_digit(BASE_10);
                    let fold_chars_to_int =
                        |prev, c: char| c.to_digit(BASE_10).map(|current| prev * BASE_10 + current);
                    // Three cases:
                    let (value, (start_sequence, end_sequence)) = {
                        let mut chars = last_str.chars().peekable();
                        let char_first = chars.next()?;
                        let char_last = chars.next_back()?;
                        // NOTE 1-digit case is already handled (e.g. ".9." len=3)
                        let first_is_digit = is_digit(char_first);
                        let last_is_digit = is_digit(char_last);
                        let (include_first, include_last) = match (first_is_digit, last_is_digit) {
                            (true, true) if start_index.is_zero() && end_index.is_end() => {
                                // case: entire line
                                Some((true, true))
                            }
                            (true, false) if start_index.is_zero() => {
                                // case: beginning
                                Some((true, false))
                            }
                            (false, false) if chars.peek().is_some() => {
                                // case: middle
                                Some((false, false))
                            }
                            (false, true) if end_index.is_end() => {
                                // case: end
                                Some((false, true))
                            }
                            _ => None,
                        }?;
                        // check ALL cases
                        let first_iter =
                            std::iter::once(char_first).take(if include_first { 1 } else { 0 });
                        let last_iter =
                            std::iter::once(char_last).take(if include_last { 1 } else { 0 });
                        let value = first_iter
                            .chain(chars)
                            .chain(last_iter)
                            .try_fold(0, fold_chars_to_int)?;
                        let start_sequence = if include_first {
                            start_index.sequence()
                        } else {
                            start_index.sequence() + 1
                        };
                        let end_sequence = if include_last {
                            end_index.sequence()
                        } else {
                            end_index.sequence() - 1
                        };
                        (value, (start_sequence, end_sequence))
                    };
                    let entity = Entity::Number(Number {
                        value,
                        region: Region {
                            top_left: Location {
                                row,
                                col_sequence: start_sequence,
                            },
                            bottom_right: Location {
                                row,
                                col_sequence: end_sequence,
                            },
                        },
                    });
                    Some(entity)
                };
            self.char_scanner
                .find_next(Some(f_single_char), Some(f_lookback_str))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    /// Line number
    row: usize,
    /// Graphical column number (for humans 🙂)
    col_sequence: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Region {
    top_left: Location,
    bottom_right: Location,
}
impl Region {
    fn adjacent_to_symbol(&self, symbols_by_row: &HashMap<usize, Vec<Symbol>>) -> bool {
        let adjacent_rows = {
            let row_start = self.top_left.row.saturating_sub(1);
            let row_end = self.bottom_right.row.saturating_add(1);
            // ROWS are closed (inclusive of end)
            row_start..=row_end
        };

        let adjacent_cols = {
            let col_start = self.top_left.col_sequence.saturating_sub(1);
            let col_end = self.bottom_right.col_sequence.saturating_add(1);
            // COLUMNS are half-open (non-inclusive of end)
            col_start..col_end
        };

        // println!("Region checking for rows {adjacent_rows:?}, cols {adjacent_cols:?}");
        for row in adjacent_rows.clone() {
            let Some(row_symbols) = symbols_by_row.get(&row) else {
                continue;
            };
            for symbol in row_symbols {
                let col = symbol.location.col_sequence;
                if adjacent_cols.contains(&col) {
                    // println!("\tSymbol matches! {symbol}");
                    return true;
                    // } else {
                    //     println!("\tNOT adjacent.. {symbol}");
                }
            }
        }
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entity {
    Number(Number),
    Symbol(Symbol),
}
impl Entity {
    fn row(&self) -> usize {
        match self {
            Entity::Number(number) => number.region.top_left.row,
            Entity::Symbol(symbol) => symbol.location.row,
        }
    }

    fn col_start(&self) -> usize {
        match self {
            Entity::Number(number) => number.region.top_left.col_sequence,
            Entity::Symbol(symbol) => symbol.location.col_sequence,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Number {
    value: u32,
    region: Region,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symbol {
    value: char,
    location: Location,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { row, col_sequence } = *self;
        write!(f, "(r={row},c={col_sequence})")
    }
}
impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            value,
            region: Region {
                top_left,
                bottom_right,
            },
        } = *self;
        write!(f, "{value} from {top_left} to {bottom_right}")
    }
}
impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { value, location } = *self;
        write!(f, "{value} @{location}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{Entity, EntityScanner, Location, Number, Region, Symbol};

    fn region(row: usize, (col_start, col_end): (usize, usize)) -> Region {
        Region {
            top_left: Location {
                row,
                col_sequence: col_start,
            },
            bottom_right: Location {
                row,
                col_sequence: col_end,
            },
        }
    }

    #[test]
    fn scanner_empty() {
        for row in 0..5 {
            let line = "....";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(entities, vec![]);
        }
    }

    #[test]
    fn scanner_symbol1() {
        for row in 0..5 {
            let line = ".$..";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Symbol(Symbol {
                    value: '$',
                    location: Location {
                        row,
                        col_sequence: 1,
                    },
                })]
            );
        }
    }
    #[test]
    fn scanner_symbol2() {
        for row in 0..5 {
            let line = "....*";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Symbol(Symbol {
                    value: '*',
                    location: Location {
                        row,
                        col_sequence: 4,
                    },
                })]
            );
        }
    }

    #[test]
    fn number_begin() {
        for row in 0..5 {
            let line = "5678...";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Number(Number {
                    value: 5678,
                    region: region(row, (0, 4)),
                })]
            );
        }
    }
    #[test]
    fn number_middle() {
        for row in 0..5 {
            let line = "..5678..";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Number(Number {
                    value: 5678,
                    region: region(row, (2, 6)),
                })]
            );
        }
    }
    #[test]
    fn numbers_middle_2() {
        for row in 0..5 {
            let line = ".664.598..";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![
                    Entity::Number(Number {
                        value: 664,
                        region: region(row, (1, 4)),
                    }),
                    Entity::Number(Number {
                        value: 598,
                        region: region(row, (5, 8)),
                    }),
                ]
            );
        }
    }
    #[test]
    fn number_middle_single() {
        for row in 0..5 {
            let line = "..5..";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Number(Number {
                    value: 5,
                    region: region(row, (2, 3)),
                })]
            );
        }
    }
    #[test]
    fn number_end() {
        for row in 0..5 {
            let line = "....5678";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Number(Number {
                    value: 5678,
                    region: region(row, (4, 8)),
                })]
            );
        }
    }
    #[test]
    fn number_full() {
        for row in 0..5 {
            let line = "12345";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![Entity::Number(Number {
                    value: 12345,
                    region: region(row, (0, 5)),
                })]
            );
        }
    }
    #[test]
    fn number_end_by_symbol() {
        for row in 0..5 {
            let line = "..59%";
            let entities: Vec<_> = EntityScanner::new(row, line).collect();
            assert_eq!(
                entities,
                vec![
                    Entity::Symbol(Symbol {
                        value: '%',
                        location: Location {
                            row,
                            col_sequence: 4
                        }
                    }),
                    Entity::Number(Number {
                        value: 59,
                        region: region(row, (2, 4)),
                    })
                ]
            );
        }
    }
    #[test]
    fn complex_lines() {
        let input = "...$.
12345
..67*
12...
..83.
.....
.....";
        let entities: Vec<_> = input
            .lines()
            .enumerate()
            .flat_map(|(row, line)| EntityScanner::new(row, line))
            .collect();
        let top_dollar = Entity::Symbol(Symbol {
            value: '$',
            location: Location {
                row: 0,
                col_sequence: 3,
            },
        });
        let large_number = Entity::Number(Number {
            value: 12345,
            region: region(1, (0, 5)),
        });
        let right_67 = Entity::Number(Number {
            value: 67,
            region: region(2, (2, 4)),
        });
        let right_star = Entity::Symbol(Symbol {
            value: '*',
            location: Location {
                row: 2,
                col_sequence: 4,
            },
        });
        let left_number = Entity::Number(Number {
            value: 12,
            region: region(3, (0, 2)),
        });
        let right_number = Entity::Number(Number {
            value: 83,
            region: region(4, (2, 4)),
        });
        assert_eq!(
            entities,
            vec![
                top_dollar,
                large_number,
                right_star,
                right_67,
                left_number,
                right_number,
            ]
        );
    }
}

#[test]
fn sample_test() {
    let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";
    let stats = interpret_engine_schematic(input);
    assert_eq!(stats.part_numbers_sum, 4361);
}

#[test]
fn outside_right_corners() {
    let input = "........
...*...*
.5...7..
...*..6*";
    let stats = interpret_engine_schematic(input);
    assert_eq!(stats.part_numbers_sum, 6);
}
