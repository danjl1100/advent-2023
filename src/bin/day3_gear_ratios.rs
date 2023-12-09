use entity_scanner::EntityScanner;

fn main() -> anyhow::Result<()> {
    println!("hello engine-fixing elf!");

    let input = advent_2023::get_input_string()?;
    let stats = interpret_engine_schematic(&input);
    dbg!(stats);
    Ok(())
}

#[derive(Debug)]
struct Stats {}
fn interpret_engine_schematic(input: &str) -> Stats {
    let entities: Vec<_> = input
        .lines()
        .enumerate()
        .flat_map(|(row, line)| EntityScanner::new(row, line))
        .collect();
    dbg!(entities);
    Stats {}
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
                        let char_last = chars.next_back()?; // TODO handle 1-digit line case
                        let first_is_non_digit = !is_digit(char_first);
                        let last_is_non_digit = !is_digit(char_last);
                        let (include_first, include_last) = if start_index.is_zero()
                            && end_index.is_end()
                        {
                            // case: entire line
                            Some((true, true))
                        } else if start_index.is_zero() && last_is_non_digit {
                            // case: beginning
                            Some((true, false))
                        } else if first_is_non_digit && last_is_non_digit && chars.peek().is_some()
                        {
                            // case: middle
                            Some((false, false))
                        } else if first_is_non_digit && end_index.is_end() {
                            // case: end
                            Some((false, true))
                        } else {
                            None
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entity {
    Number(Number),
    Symbol(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Number {
    value: u32,
    region: Region,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symbol {
    location: Location,
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
