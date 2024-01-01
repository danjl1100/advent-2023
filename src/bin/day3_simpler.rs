use std::collections::BTreeMap;

use advent_2023::{
    print::{ConsolePrinter, Highlight},
    CharIndex, CharIndexEnd, CharIndices,
};

const DEBUG_PRINT: bool = cfg!(test);

fn main() -> anyhow::Result<()> {
    println!("hello engine-fixing elf!");

    let input = advent_2023::get_input_string()?;

    let term = console::Term::buffered_stderr();

    let Stats {
        part_numbers_sum,
        gear_ratios_sum,
    } = interpret_engine_schematic(&input, term)?;

    println!("Sum of part numbers: {part_numbers_sum}");
    println!("Sum of gear ratios: {gear_ratios_sum}");

    Ok(())
}

#[derive(Debug)]
struct Stats {
    part_numbers_sum: u32,
    gear_ratios_sum: u32,
}
fn interpret_engine_schematic(
    input: &str,
    console_printer: impl Into<ConsolePrinter>,
) -> anyhow::Result<Stats> {
    interpret_engine_schematic_inner(input, console_printer.into())
}
fn interpret_engine_schematic_inner(
    input: &str,
    mut console_printer: ConsolePrinter,
) -> anyhow::Result<Stats> {
    if DEBUG_PRINT {
        println!("------------------------------");
    }
    let mut lines = input.lines().enumerate().peekable();

    let mut part_numbers_sum = 0;
    let mut part_numbers_by_asterisk: BTreeMap<InputPosition, Vec<u32>> = BTreeMap::new();

    let mut line_prev = None;
    while let Some((row, line)) = lines.next() {
        if DEBUG_PRINT {
            println!("{line:?}");
        }
        let mut chars = CharIndices::new(line);
        let line_next = lines.peek().copied();

        let mut debug_line_spans = vec![];
        let mut before_number_start = None;
        loop {
            let mut number_start = None;
            while let Some((col, c)) = chars.next() {
                if is_digit(c) {
                    number_start = Some(col);
                    break;
                }
                before_number_start = Some((col, c));
            }
            let Some(number_start) = number_start else {
                break;
            };

            let mut number_end = None;
            while let Some((col, c)) = chars.next() {
                if !is_digit(c) {
                    number_end = Some((CharIndexEnd::Position(col), Some(c)));
                    break;
                }
            }
            let after_number_end = chars.peek().map(|(col, _second_after_char)| col);
            let number_end = number_end.unwrap_or_else(|| {
                chars
                    .take_end_char_sequence()
                    .map(|col| (CharIndexEnd::End(col), None))
                    .expect("exhausted chars")
            });

            let number_str = number_start
                .slice_string(number_end.0, line)
                .expect("in bounds of current line");
            let number = parse_int(number_str)?;

            let mut asterisk_positions = vec![];

            // eval start/end
            let count_this_line = before_number_start.map_or(0, |(_, c)| count_symbol(c))
                + number_end.1.map_or(0, |c| count_symbol(c));

            if let Some((col, '*')) = before_number_start {
                asterisk_positions.push(InputPosition {
                    row,
                    col: Some(col),
                });
            }
            if let (col_end, Some('*')) = number_end {
                let col = match col_end {
                    CharIndexEnd::Position(col) => Some(col),
                    CharIndexEnd::End(_) => None,
                };
                asterisk_positions.push(InputPosition { row, col });
            }

            let line_search_start = before_number_start.map_or(number_start, |(col, _)| col);
            let line_search_end = after_number_end.map_or(number_end.0, CharIndexEnd::Position);

            // search previous line
            let (span_prev_line, count_prev_line) = count_symbols(
                line_prev,
                line_search_start,
                line_search_end,
                &mut asterisk_positions,
            )
            .unwrap_or(("N/A", 0));

            // search next line
            let (span_next_line, count_next_line) = count_symbols(
                line_next,
                line_search_start,
                line_search_end,
                &mut asterisk_positions,
            )
            .unwrap_or(("N/A", 0));

            let count = count_this_line + count_prev_line + count_next_line;
            let included = count > 0;
            if included {
                part_numbers_sum += number;
            }

            let debug_style = if included {
                console::Style::new().cyan()
            } else {
                console::Style::new().red()
            };
            debug_line_spans.push(Highlight {
                start: number_start,
                end: number_end.0,
                style: debug_style,
            });

            if DEBUG_PRINT {
                let symbol_before = before_number_start.map(|(_, c)| c);
                let symbol_after = number_end.1;

                let show_prev_next_lines = line_prev.is_some() || line_next.is_some();
                if show_prev_next_lines {
                    println!("        {span_prev_line}");
                }
                println!(
                    "Number: {char_before}{number_str}{char_after}",
                    char_before = opt_char_to_str(symbol_before),
                    char_after = opt_char_to_str(symbol_after),
                );
                if show_prev_next_lines {
                    println!("        {span_next_line}");
                }
                println!(" -> value {number}, {count} symbols,  ==> sum {part_numbers_sum}");
            }

            if let Some(pos) = asterisk_positions.pop() {
                assert!(asterisk_positions.is_empty());
                let list = part_numbers_by_asterisk.entry(pos).or_insert(vec![]);
                list.push(number);
            }

            // prep for next iteration
            before_number_start = match number_end {
                (CharIndexEnd::Position(col), Some(c)) => Some((col, c)),
                _ => None,
            };
        }

        console_printer.print_line(line, &debug_line_spans)?;
        debug_line_spans.clear();

        // prep for next LINE
        line_prev = Some((row, line));
    }

    for (pos, numbers) in &part_numbers_by_asterisk {
        if numbers.len() < 2 {
            continue;
        }
        let InputPosition { row, col } = pos;
        let col = col.map_or(0, |c_index| c_index.sequence());
        println!("Line {row}, col {col} connects numbers {numbers:?}");
    }

    let gear_ratios_sum = part_numbers_by_asterisk
        .into_iter()
        .map(|(pos, numbers)| match numbers.len() {
            0 | 1 => 0,
            2 => numbers.into_iter().product(),
            count => panic!("{count} (more than 2) numbers around asterisk at {pos:?}"),
        })
        .sum();

    Ok(Stats {
        part_numbers_sum,
        gear_ratios_sum,
    })
}

const BASE_10: u32 = 10;
fn is_digit(c: char) -> bool {
    c.is_digit(BASE_10)
}
fn parse_int(s: &str) -> anyhow::Result<u32> {
    Ok(u32::from_str_radix(s, BASE_10)?)
}
fn count_symbol(c: char) -> u32 {
    if !is_digit(c) && c != '.' {
        1
    } else {
        0
    }
}
fn count_symbols<'a>(
    row_line: Option<(usize, &'a str)>,
    start: CharIndex,
    end: CharIndexEnd,
    asterisk_positions: &mut Vec<InputPosition>,
) -> Option<(&'a str, u32)> {
    let (row, line) = row_line?;

    let haystack = start.slice_string(end, line).expect("valid indices");
    let line_truncated = CharIndex::default()
        .slice_string(end, line)
        .expect("valid index");

    let mut sum = 0;
    for (col, c) in CharIndices::new(line_truncated) {
        if col < start {
            continue;
        }
        sum += count_symbol(c);

        if c == '*' {
            asterisk_positions.push(InputPosition {
                row,
                col: Some(col),
            });
        }
    }
    Some((haystack, sum))
}
fn opt_char_to_str(c: Option<char>) -> std::borrow::Cow<'static, str> {
    c.as_ref().map_or(std::borrow::Cow::Borrowed(""), |c| {
        std::borrow::Cow::Owned(c.to_string())
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InputPosition {
    row: usize,
    col: Option<CharIndex>,
}

#[cfg(test)]
mod tests {
    use crate::interpret_engine_schematic;

    #[test]
    fn sample_test() {
        // without the pesky indentation:
        // 467..114..
        // ...*......
        // ..35..633.
        // ......#...
        // 617*......
        // .....+.58.
        // ..592.....
        // ......755.
        // ...$.*....
        // .664.598..
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
        let stats = interpret_engine_schematic(input, None).unwrap();
        assert_eq!(stats.part_numbers_sum, 4361);
    }
    #[test]
    fn sample_test_gear_ratios() {
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
        let stats = interpret_engine_schematic(input, None).unwrap();
        assert_eq!(stats.gear_ratios_sum, 467835);
    }

    #[test]
    fn outside_right_corners() {
        let input = "........
...*...*
.5...7..
...*..6*";
        let stats = interpret_engine_schematic(input, None).unwrap();
        assert_eq!(stats.part_numbers_sum, 6);
    }

    macro_rules! test_line {
        ($(
                $line:expr => $expected:expr
        );+ $(;)?) => {
            $({
                let line: &'static str = $line;
                let expected: u32 = $expected;
                let stats = interpret_engine_schematic(line, None).unwrap();
                assert_eq!(stats.part_numbers_sum, expected, "{line:?} => {}", stringify!($expected));
            })+
        };
    }
    #[test]
    fn numbers_sandwiching_symbol() {
        test_line! {
            ".1*2..." => 3;
            "....3*2" => 5;
            ".123*40" => 123+40;
        };
    }
}
