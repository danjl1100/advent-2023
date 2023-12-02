fn main() -> anyhow::Result<()> {
    println!("hello, so many cubes!");
    let input = advent_2023::get_input_string()?;

    let Stats {
        sum_valid_game_ids,
        sum_of_powers,
    } = play_cube_game(&input);
    println!("Sum of IDs of valid games: {sum_valid_game_ids}");
    println!("Sum of power for the minimal sets for all games: {sum_of_powers}");

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stats {
    sum_valid_game_ids: u32,
    sum_of_powers: u32,
}

fn play_cube_game(input: &str) -> Stats {
    const LIMIT: ColorCounts = ColorCounts {
        red: 12,
        green: 13,
        blue: 14,
    };

    let games: Vec<_> = input
        .lines()
        .map(|line| {
            let (input, game) = parse::game(line).expect("valid game line");
            assert!(input.is_empty());
            game
        })
        .collect();

    let sum_of_powers = games.iter().map(|game| game.miminal_counts().power()).sum();

    let sum_valid_game_ids = games
        .iter()
        .filter_map(|game| game.within_limits(LIMIT).then_some(game.game_id))
        .sum();

    Stats {
        sum_valid_game_ids,
        sum_of_powers,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Game {
    pub game_id: u32,
    reveals: Vec<ColorCounts>,
}
impl Game {
    fn within_limits(&self, limit: ColorCounts) -> bool {
        self.reveals.iter().all(|color_counts| {
            color_counts.red <= limit.red && // format
                color_counts.green <= limit.green &&
                color_counts.blue <= limit.blue
        })
    }
    fn miminal_counts(&self) -> ColorCounts {
        let mut min = ColorCounts::default();
        for &reveal in &self.reveals {
            min = min.elem_max(reveal);
        }
        min
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ColorCounts {
    red: u32,
    green: u32,
    blue: u32,
}
impl ColorCounts {
    fn try_from_iter<I>(parts: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = ColorCount>,
    {
        let mut new = ColorCounts::default();
        for part in parts {
            match part {
                ColorCount::Red(red) if new.red == 0 => {
                    new.red = red;
                }
                ColorCount::Green(green) if new.green == 0 => {
                    new.green = green;
                }
                ColorCount::Blue(blue) if new.blue == 0 => {
                    new.blue = blue;
                }
                _ => {
                    return Err(format!("duplicate definintion encountered: {part:?}"));
                }
            }
        }
        Ok(new)
    }
    fn power(&self) -> u32 {
        self.red * self.green * self.blue
    }
    fn elem_max(self, other: Self) -> Self {
        let Self { red, green, blue } = self;
        Self {
            red: red.max(other.red),
            green: green.max(other.green),
            blue: blue.max(other.blue),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorCount {
    Red(u32),
    Green(u32),
    Blue(u32),
}

mod parse {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::digit1;
    use nom::combinator::{map, map_res};
    use nom::multi::separated_list1;
    use nom::sequence::tuple;
    use nom::IResult;

    use crate::{ColorCount, ColorCounts, Game};

    pub fn game(input: &str) -> IResult<&str, Game> {
        let colon = tag(": ");
        map(tuple((game_id, colon, reveals)), |(game_id, _, reveals)| {
            Game { game_id, reveals }
        })(input)
    }

    fn game_id(input: &str) -> IResult<&str, u32> {
        let (input, _) = tag("Game ")(input)?;
        map_res(digit1, str::parse)(input)
    }

    fn reveals(input: &str) -> IResult<&str, Vec<ColorCounts>> {
        separated_list1(tag("; "), reveal)(input)
    }
    fn reveal(input: &str) -> IResult<&str, ColorCounts> {
        map_res(
            separated_list1(tag(", "), reveal_part),
            ColorCounts::try_from_iter,
        )(input)
    }

    fn reveal_part(input: &str) -> IResult<&str, ColorCount> {
        const COLOR_RED: &str = "red";
        const COLOR_GREEN: &str = "green";
        const COLOR_BLUE: &str = "blue";

        let (input, count) = map_res(digit1, str::parse)(input)?;
        let (input, _) = tag(" ")(input)?;
        let match_colors = alt((tag(COLOR_RED), tag(COLOR_GREEN), tag(COLOR_BLUE)));
        map(match_colors, move |color| {
            if color == COLOR_RED {
                ColorCount::Red(count)
            } else if color == COLOR_GREEN {
                ColorCount::Green(count)
            } else if color == COLOR_BLUE {
                ColorCount::Blue(count)
            } else {
                panic!("color {color:?} matched tags, but not matching")
            }
        })(input)
    }

    #[cfg(test)]
    mod tests {
        use crate::{ColorCount, ColorCounts, Game};

        #[test]
        fn game_id_number() {
            assert_eq!(super::game_id("Game 22"), Ok(("", 22)));
        }

        #[test]
        fn reveal_part() {
            assert_eq!(super::reveal_part("6 red"), Ok(("", ColorCount::Red(6))));
        }

        #[test]
        fn reveal() {
            assert_eq!(
                super::reveal("6 red, 1 blue, 3 green"),
                Ok((
                    "",
                    ColorCounts {
                        red: 6,
                        green: 3,
                        blue: 1,
                    }
                ))
            );
        }

        #[test]
        fn reveal_1() {
            assert_eq!(
                super::reveal("1 blue, 2 red"),
                Ok((
                    "",
                    ColorCounts {
                        red: 2,
                        green: 0,
                        blue: 1
                    },
                ))
            );
        }
        #[test]
        fn reveal_2() {
            assert_eq!(
                super::reveal("2 green, 3 red"),
                Ok((
                    "",
                    ColorCounts {
                        red: 3,
                        green: 2,
                        blue: 0
                    },
                ))
            );
        }
        #[test]
        fn reveals() {
            assert_eq!(
                super::reveals("1 blue, 2 red, 3 green; 2 green, 3 red"),
                Ok((
                    "",
                    vec![
                        ColorCounts {
                            red: 2,
                            green: 3,
                            blue: 1
                        },
                        ColorCounts {
                            red: 3,
                            green: 2,
                            blue: 0
                        },
                    ],
                ))
            );
        }
        #[test]
        fn game() {
            assert_eq!(
                super::game("Game 27: 1 blue, 2 red, 3 green; 2 green, 3 red"),
                Ok((
                    "",
                    Game {
                        game_id: 27,
                        reveals: vec![
                            ColorCounts {
                                red: 2,
                                green: 3,
                                blue: 1
                            },
                            ColorCounts {
                                red: 3,
                                green: 2,
                                blue: 0
                            },
                        ],
                    }
                ))
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Stats;

    #[test]
    fn example() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

        let result = super::play_cube_game(input);
        assert_eq!(
            result,
            Stats {
                sum_valid_game_ids: 8,
                sum_of_powers: 2286,
            }
        );
    }
}
