use advent_2023::{
    print::{ConsolePrinter, Highlight},
    CharIndex, CharIndexEnd, CharIndices,
};

const DEBUG_PRINT: bool = cfg!(test);

fn main() -> anyhow::Result<()> {
    println!("hello engine-fixing elf!");

    let input = advent_2023::get_input_string()?;

    let term = console::Term::buffered_stderr();

    let Stats { part_numbers_sum } = interpret_engine_schematic(&input, term)?;

    println!("Sum of part numbers: {part_numbers_sum}");

    Ok(())
}

#[derive(Debug)]
struct Stats {
    part_numbers_sum: u32,
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
    let mut lines = input.lines().peekable();

    let mut part_numbers_sum = 0;

    let mut line_prev = None;
    while let Some(line) = lines.next() {
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

            // eval start/end
            let count_this_line = before_number_start.map_or(0, |(_, c)| count_symbol(c))
                + number_end.1.map_or(0, |c| count_symbol(c));

            let line_search_start = before_number_start.map_or(number_start, |(col, _)| col);
            let line_search_end = after_number_end.map_or(number_end.0, CharIndexEnd::Position);

            // search previous line
            let (span_prev_line, count_prev_line) =
                count_symbols(line_prev, line_search_start, line_search_end).unwrap_or(("N/A", 0));

            // search next line
            let (span_next_line, count_next_line) =
                count_symbols(line_next, line_search_start, line_search_end).unwrap_or(("N/A", 0));

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
                dbg!((number_end, after_number_end));
                if show_prev_next_lines {
                    println!("        {span_next_line}");
                }
                println!(" -> value {number}, {count} symbols,  ==> sum {part_numbers_sum}");
            }

            before_number_start = match number_end {
                (CharIndexEnd::Position(col), Some(c)) => Some((col, c)),
                _ => None,
            };
        }

        console_printer.print_line(line, &debug_line_spans)?;
        debug_line_spans.clear();

        // prep for next LINE
        line_prev = Some(line);
    }

    Ok(Stats { part_numbers_sum })
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
fn count_symbols(line: Option<&str>, start: CharIndex, end: CharIndexEnd) -> Option<(&str, u32)> {
    let haystack = start.slice_string(end, line?)?;
    let sum = haystack.chars().map(count_symbol).sum();
    Some((haystack, sum))
}
fn opt_char_to_str(c: Option<char>) -> std::borrow::Cow<'static, str> {
    c.as_ref().map_or(std::borrow::Cow::Borrowed(""), |c| {
        std::borrow::Cow::Owned(c.to_string())
    })
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
            ".1*2*.." => 3;
            ".1*2..." => 3;
            "....3*2" => 5;
            ".123*40" => 123+40;
        };
    }
}
