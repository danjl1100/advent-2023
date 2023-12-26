use std::collections::BTreeMap;

fn main() -> anyhow::Result<()> {
    let input = advent_2023::get_input_string()?;
    let parsed = parse_input(&input)?;

    let start = "AAA".parse().expect("valid start");
    let end = "ZZZ".parse().expect("valid start");
    let shortest = parsed.find_shortest_count(start, end)?;

    println!("Shortest path from {start:?} to {end:?} is: {shortest}");

    let parallel_lcm = parsed.find_shortest_parallel('A', 'Z')?;

    println!("Parallel shortest path: {parallel_lcm}");

    Ok(())
}

fn parse_input(input: &str) -> anyhow::Result<Parsed> {
    let mut lines = input.lines();

    let Some(instructions_line) = lines.next() else {
        anyhow::bail!("missing instructions line")
    };

    let Some(blank_line) = lines.next() else {
        anyhow::bail!("missing blank separator line")
    };
    if !blank_line.is_empty() {
        anyhow::bail!("blank line is not blank: {blank_line:?}");
    }

    let instructions = instructions_line
        .chars()
        .map(Instruction::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|c| anyhow::anyhow!("invalid char for instruction {c}"))?;

    let maps = lines.map(parse_map_line).collect::<Result<_, _>>()?;

    Ok(Parsed { instructions, maps })
}

fn parse_map_line(line: &str) -> anyhow::Result<(Key, (Key, Key))> {
    let Some((key, values)) = line.split_once(" = (") else {
        anyhow::bail!("missing key/value delimiter on line {line:?}")
    };
    let key: Key = key
        .parse()
        .map_err(|s| anyhow::anyhow!("invalid key {s:?}"))?;

    let Some((value1, value2)) = values.split_once(", ") else {
        anyhow::bail!("missing comma delimiter between values {values:?} on line {line:?}")
    };
    let value2 = value2.strip_suffix(')').unwrap_or(value2);

    let value1: Key = value1
        .parse()
        .map_err(|s| anyhow::anyhow!("invalid value1 {s:?}"))?;

    let value2: Key = value2
        .parse()
        .map_err(|s| anyhow::anyhow!("invalid value2 {s:?}"))?;

    Ok((key, (value1, value2)))
}

#[derive(Debug)]
struct Parsed {
    instructions: Vec<Instruction>,
    maps: BTreeMap<Key, (Key, Key)>,
}
impl Parsed {
    pub fn find_shortest_count(&self, start: Key, end: Key) -> anyhow::Result<usize> {
        self.find_count(start, |current| current == end)
    }
    fn find_count(&self, start: Key, accept_fn: impl Fn(Key) -> bool) -> anyhow::Result<usize> {
        let mut instructions = std::iter::repeat(self.instructions.iter()).flatten();

        let mut current = start;
        let mut count = 0;
        while !(accept_fn)(current) {
            let Some((value1, value2)) = self.maps.get(&current).copied() else {
                anyhow::bail!("mapping not found for current {current:?}")
            };

            let Some(instruction) = instructions.next() else {
                anyhow::bail!("instructions iter empty")
            };

            current = instruction.choose(value1, value2);
            count += 1;
        }
        Ok(count)
    }

    pub fn find_shortest_parallel(
        &self,
        start_third: char,
        end_third: char,
    ) -> anyhow::Result<usize> {
        let repetition_counts = self
            .maps
            .keys()
            .copied()
            .filter(|key| key.ends_with(start_third))
            .map(|start| self.find_count(start, |current| current.ends_with(end_third)))
            .collect::<Result<Vec<_>, _>>()?;

        let parallel_lcm = repetition_counts
            .iter()
            .copied()
            .fold(1, advent_2023::math::lcm);

        println!("{repetition_counts:?} -> {parallel_lcm}");

        Ok(parallel_lcm)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Key([char; 3]);
impl Key {
    fn ends_with(self, expected: char) -> bool {
        let Self([_, _, end]) = self;
        end == expected
    }
}
impl std::str::FromStr for Key {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let c1 = chars.next();
        let c2 = chars.next();
        let c3 = chars.next();
        let extra = chars.next();
        if let Some(((c1, c2), c3)) = c1.zip(c2).zip(c3) {
            if extra.is_none() {
                return Ok(Key([c1, c2, c3]));
            }
        }
        Err(s.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instruction {
    Left,
    Right,
}
impl Instruction {
    fn choose<T>(self, left: T, right: T) -> T {
        match self {
            Instruction::Left => left,
            Instruction::Right => right,
        }
    }
}
impl TryFrom<char> for Instruction {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_input, Instruction};

    #[test]
    fn sample_input_parse() {
        let input = "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)";
        let parsed = parse_input(input).unwrap();
        assert_eq!(
            parsed.instructions,
            vec![Instruction::Left, Instruction::Left, Instruction::Right]
        );

        let aaa = "AAA".parse().unwrap();
        let bbb = "BBB".parse().unwrap();
        let zzz = "ZZZ".parse().unwrap();
        assert_eq!(parsed.maps.get(&aaa), Some(&(bbb, bbb)));
        assert_eq!(parsed.maps.get(&bbb), Some(&(aaa, zzz)));
        assert_eq!(parsed.maps.get(&zzz), Some(&(zzz, zzz)));

        let shortest = parsed.find_shortest_count(aaa, zzz).unwrap();
        assert_eq!(shortest, 6);
    }

    #[test]
    fn sample_input_moderate() {
        let input = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";
        let parsed = parse_input(input).unwrap();
        assert_eq!(
            parsed.instructions,
            vec![Instruction::Right, Instruction::Left]
        );

        let aaa = "AAA".parse().unwrap();
        let bbb = "BBB".parse().unwrap();
        let ccc = "CCC".parse().unwrap();
        let ddd = "DDD".parse().unwrap();
        let eee = "EEE".parse().unwrap();
        let ggg = "GGG".parse().unwrap();
        let zzz = "ZZZ".parse().unwrap();
        assert_eq!(parsed.maps.get(&aaa), Some(&(bbb, ccc)));
        assert_eq!(parsed.maps.get(&bbb), Some(&(ddd, eee)));
        assert_eq!(parsed.maps.get(&ccc), Some(&(zzz, ggg)));
        assert_eq!(parsed.maps.get(&ddd), Some(&(ddd, ddd)));
        assert_eq!(parsed.maps.get(&eee), Some(&(eee, eee)));
        assert_eq!(parsed.maps.get(&ggg), Some(&(ggg, ggg)));
        assert_eq!(parsed.maps.get(&zzz), Some(&(zzz, zzz)));

        let shortest = parsed.find_shortest_count(aaa, zzz).unwrap();
        assert_eq!(shortest, 2);
    }

    #[test]
    fn sample_input_parallel() {
        let input = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";

        let parsed = parse_input(input).unwrap();
        let length = parsed.find_shortest_parallel('A', 'Z').unwrap();
        assert_eq!(length, 6);
    }
}
