use anyhow::Context;

fn main() -> anyhow::Result<()> {
    println!("hello, boat racers!");

    let input = advent_2023::get_input_string()?;
    let product = races_stats(&input, Kerning::Bad)?;

    println!("Product of all combinations for each race: {product}");

    Ok(())
}

/// https://en.wikipedia.org/wiki/Kerning
enum Kerning {
    #[allow(unused)]
    /// Normal, spaces mean spaces
    Normal,
    /// You realize the piece of paper just has very bad kerning...
    Bad,
}

fn races_stats(input: &str, kerning: Kerning) -> anyhow::Result<u64> {
    let races = parse_race_info(input, kerning)?;
    dbg!(&races);

    let product: u64 = races
        .iter()
        .copied()
        .map(RaceInfo::count_win_options)
        .product();

    Ok(product)
}

fn parse_race_info(input: &str, kerning: Kerning) -> anyhow::Result<Vec<RaceInfo>> {
    const BASE_10: u32 = 10;

    let mut lines = input.lines();

    let Some(line_time) = lines.next() else {
        anyhow::bail!("expected first line for time")
    };
    let Some(line_distance) = lines.next() else {
        anyhow::bail!("expected first line for time")
    };

    match lines.next() {
        Some(extra) if !extra.is_empty() => {
            anyhow::bail!("extra line: {extra:?}");
        }
        _ => {}
    }

    let parse_int = |s| u64::from_str_radix(s, BASE_10).with_context(|| s.to_string());

    let mut times = line_time.trim_start_matches("Time:").to_string();
    let mut distances = line_distance.trim_start_matches("Distance:").to_string();

    match kerning {
        Kerning::Normal => {}
        Kerning::Bad => {
            times = times.replace(char::is_whitespace, "");
            distances = distances.replace(char::is_whitespace, "");
        }
    }

    let mut times = times.split_whitespace().map(parse_int);
    let mut distances = distances.split_whitespace().map(parse_int);

    let mut races = vec![];

    loop {
        match (times.next().transpose()?, distances.next().transpose()?) {
            (None, None) => break,
            (Some(time), None) => {
                anyhow::bail!("no distance present for time {time}")
            }
            (None, Some(distance)) => {
                anyhow::bail!("no time present for distance {distance}")
            }
            (Some(duration), Some(min_distance)) => {
                races.push(RaceInfo {
                    duration,
                    min_distance,
                });
            }
        }
    }

    Ok(races)
}

#[derive(Clone, Copy, Debug)]
struct RaceInfo {
    duration: u64,
    min_distance: u64,
}

impl RaceInfo {
    /// The distance traveled in a race is the area of a rectangle between (0, 0) and (x, y),
    /// for the curve `y = duration - x`
    ///
    /// ```text
    /// \
    ///  \  -  Line defined by 7 second duration  (y = 7 - x)
    ///   \
    ///    \
    /// ____O  - Hold button for 4 seconds, travels for 3 seconds
    /// ****|\
    /// *12*| \
    /// ****|  \
    /// ```
    ///
    fn count_win_options(self) -> u64 {
        let Self {
            duration,
            min_distance,
        } = self;

        let middle = duration / 2;

        let first_fail = {
            let mut current = middle;
            while current * (duration - current) > min_distance {
                // beats record
                current = current
                    .checked_sub(1)
                    .expect("nonzero current can decrement");
            }
            current
        };

        let smallest_success = first_fail + 1;
        let success = smallest_success..=(duration - smallest_success);
        let success_count = u64::try_from(success.clone().count()).expect("no overflow");
        dbg!(
            self,
            middle,
            first_fail,
            smallest_success,
            success,
            success_count
        );
        success_count
    }
}

#[cfg(test)]
mod tests {
    use crate::{races_stats, Kerning, RaceInfo};

    macro_rules! tests {
        (
            $(
                [$duration:expr, $min_distance:expr] => $expected:expr;
            )+
        ) => {
            $({
                let duration = $duration;
                let min_distance = $min_distance;
                let expected = $expected;
                assert_eq!(
                    RaceInfo {
                        duration,
                        min_distance,
                    }
                    .count_win_options(),
                    expected,
                    "[{duration} {min_distance}] => {expected}"
                );
            })+
        };
    }

    #[test]
    fn race_impossible() {
        tests! {
            [7, 4*4] => 0;
            [8, 4*4] => 0;
            [9, 4*5] => 0;
        };
    }
    #[test]
    fn race_one() {
        tests! {
            [6, 8] => 1;
        };
    }
    #[test]
    fn race_two() {
        tests! {
            [5, 5] => 2;
        };
    }
    #[test]
    fn race_four() {
        tests! {
            [7, 9] => 4;
            [9, 4*4] => 4;
        };
    }

    #[test]
    fn sample_races() {
        tests! {
            [7, 9] => 4;
            [15, 40] => 8;
            [30, 200] => 9;
        }
    }

    #[test]
    fn sample_input() {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        let result = races_stats(input, Kerning::Normal).expect("valid input");
        assert_eq!(result, 4 * 8 * 9);
    }
}
