use anyhow::Context;
use std::collections::HashMap;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    println!("hello farmer");

    let input = advent_2023::get_input_string()?;
    let mut input_lines = input.lines();

    let seeds_line = input_lines.next().expect("seeds line");
    let blank_line = input_lines.next().expect("at least one map");
    assert!(blank_line.is_empty());

    let seeds = parse_seeds(seeds_line).with_context(|| format!("seeds line {seeds_line:?}"))?;
    let maps = parse_maps(input_lines)?;

    assert!(!seeds.is_empty());

    let locations = seeds
        .iter()
        .map(|&seed| {
            let mut current_value = seed;
            let mut current_type = "seed";
            while current_type != "location" {
                let Some(map) = maps.get(current_type) else {
                    anyhow::bail!("map not found for {current_type:?}");
                };
                let (new_type, new_value) = map.lookup_value(current_value)?;
                current_value = new_value;
                current_type = new_type;
            }
            Ok(current_value)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let closest_location = locations.iter().min().expect("at least one seed provided");

    println!("closest location: {closest_location}");

    Ok(())
}

fn parse_seeds(line: &str) -> anyhow::Result<Vec<u64>> {
    const SEEDS_COLON: &str = "seeds: ";
    let Some((empty, numbers_str)) = line.split_once(SEEDS_COLON) else {
        anyhow::bail!("not found: {SEEDS_COLON:?}");
    };
    anyhow::ensure!(empty.is_empty(), "extra text before seeds label");
    Ok(numbers_str
        .split_whitespace()
        .map(u64::from_str)
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_maps(mut input_lines: std::str::Lines<'_>) -> anyhow::Result<HashMap<String, Map>> {
    let mut maps = HashMap::new();
    while let Some(title_line) = input_lines.next() {
        let (title_from, title_to) =
            parse_title_line(title_line).with_context(|| format!("title line {title_line:?}"))?;

        let mut ranges = vec![];
        for line in input_lines.by_ref() {
            if line.is_empty() {
                break;
            }
            let range = parse_range(line).with_context(|| format!("range line {line:?}"))?;
            ranges.push(range);
        }
        maps.insert(
            title_from,
            Map {
                ranges,
                output_kind: title_to,
            },
        );
    }
    Ok(maps)
}
fn parse_title_line(title_line: &str) -> anyhow::Result<(String, String)> {
    const MAP_COLON: &str = "map:";
    let Some((title_map_str, map_colon)) = title_line.split_once(' ') else {
        anyhow::bail!("expected map declaration to have space");
    };
    if map_colon != MAP_COLON {
        anyhow::bail!("expected map declaration to end with {MAP_COLON:?}, found {map_colon:?}");
    }
    let mut title_parts = title_map_str.split('-');
    let Some(a) = title_parts.next() else {
        anyhow::bail!("missing map part A");
    };
    match title_parts.next() {
        Some("to") => {}
        Some(unexpected) => anyhow::bail!("unexpected token in map title: {unexpected:?}"),
        None => anyhow::bail!("incomplete map title"),
    }
    let Some(b) = title_parts.next() else {
        anyhow::bail!("missing map part B");
    };
    Ok((a.to_string(), b.to_string()))
}

fn parse_range(line: &str) -> anyhow::Result<Range> {
    let numbers = line
        .split_whitespace()
        .map(u64::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    anyhow::ensure!(numbers.len() == 3, "expected 3 numbers to define range");
    let dest_start = numbers[0];
    let source_start = numbers[1];
    let len = numbers[2];

    let offset = {
        let Ok(dest_start) = i64::try_from(dest_start) else {
            anyhow::bail!("destination start {dest_start} too large");
        };
        let Ok(source_start) = i64::try_from(source_start) else {
            anyhow::bail!("source start {source_start} too large");
        };

        dest_start - source_start
    };

    let sources = {
        dbg!(source_start, len);
        source_start..(source_start + len)
    };

    Ok(Range { sources, offset })
}

struct Map {
    ranges: Vec<Range>,
    output_kind: String,
}
impl Map {
    fn lookup_value(&self, value: u64) -> anyhow::Result<(&str, u64)> {
        let mut output_value = value;
        for range in &self.ranges {
            if range.sources.contains(&value) {
                let Ok(value) = i64::try_from(value) else {
                    anyhow::bail!("value {value} exceeds i64")
                };
                let with_offset = value + range.offset;
                let Ok(with_offset) = u64::try_from(with_offset) else {
                    anyhow::bail!("offset value {with_offset} exceeds u64")
                };
                output_value = with_offset;
                break;
            }
        }
        Ok((&self.output_kind, output_value))
    }
}
struct Range {
    sources: std::ops::Range<u64>,
    offset: i64,
}
