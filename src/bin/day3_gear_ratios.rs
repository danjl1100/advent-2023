use advent_2023::CharScanner;

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
    let _ = input.lines().enumerate().map(|(row, line)| {
        let mut scanner = EntityScanner::new(line);
        // hey
        todo!()
    });
    Stats {}
}

struct EntityScanner<'a> {
    scanner: CharScanner<'a>,
}
impl<'a> EntityScanner<'a> {
    fn new(line: &'a str) -> Self {
        let lookback_all = (1, line.len());
        Self {
            scanner: CharScanner::new(line, Some(lookback_all)),
        }
    }
}
impl Iterator for EntityScanner<'_> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    /// Line number
    row: usize,
    /// Graphical column
    col: usize,
    /// Byte index for char
    char_index: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Region {
    top_left: Location,
    bottom_right: Location,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entity {
    Number(Number),
    Symbol(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Number {
    value: u32,
    region: Region,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Symbol {
    location: Location,
}
