use std::collections::HashMap;

use advent_2023::either::Either;

fn main() -> anyhow::Result<()> {
    println!("hello, turns out some ROCKS will ROLL");

    let input = advent_2023::get_input_string()?;

    let Stats {
        sum_round_weight,
        sum_after_one_million,
    } = eval_input(&input)?;

    println!("Sum of round weights: {sum_round_weight}");
    println!("Sum of after 1 million... {sum_after_one_million}");

    Ok(())
}

struct Stats {
    sum_round_weight: usize,
    sum_after_one_million: usize,
}

fn eval_input(input: &str) -> anyhow::Result<Stats> {
    let mut sum_round_weight = 0;
    let mut sum_after_one_million = 0;
    let mut lines = input.lines();
    loop {
        let Some(grid) = Grid::new(lines.by_ref())? else {
            break;
        };
        let mut single_roll = grid.clone();
        single_roll.roll_stones(Direction::North);
        sum_round_weight += single_roll.get_round_weight();

        let after_million = grid.perform_cycles(1_000_000_000);
        sum_after_one_million += after_million.get_round_weight();
    }
    Ok(Stats {
        sum_round_weight,
        sum_after_one_million,
    })
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Grid {
    cells: Vec<Cell>,
    width: usize,
    max_row: usize,
    random_direction_for_cache_key: Option<Direction>,
}
impl Grid {
    #[allow(dead_code)] // for tests
    fn new_from_str(input: &str) -> anyhow::Result<Option<Self>> {
        let mut lines = input.lines();
        let result = Self::new(lines.by_ref());
        assert_eq!(lines.next(), None);
        result
    }
    fn new<'a>(lines: impl Iterator<Item = &'a str>) -> anyhow::Result<Option<Self>> {
        let mut cells = vec![];
        let mut width = None;
        let mut max_row = 0;
        for line in lines {
            if line.is_empty() {
                break;
            }
            let current_width = line.len();
            match width {
                None => {
                    width = Some(current_width);
                }
                Some(existing) if current_width != existing => {
                    anyhow::bail!("invalid width {current_width}, expected width {existing}")
                }
                Some(_match) => {}
            }

            let row = line
                .chars()
                .map(Cell::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|extra| anyhow::anyhow!("unknown character {extra:?}"))?;
            cells.extend(row);
            max_row += 1;
        }
        Ok(width.map(|width| Self {
            cells,
            width,
            max_row,
            random_direction_for_cache_key: None,
        }))
    }
    fn iter(&self) -> impl Iterator<Item = (Point, Cell)> + '_ {
        Iter {
            cells: &self.cells,
            width: self.width,
            next_index: Some(0),
        }
    }
    fn row_range(&self, row: usize) -> std::ops::Range<usize> {
        assert!(row <= self.max_row, "row {row}");
        let start = Point { row, col: 0 }.index_for(self.width);
        let end = Point {
            row: row + 1,
            col: 0,
        }
        .index_for(self.width);
        start..end
    }
    fn row(&self, row: usize) -> Option<&[Cell]> {
        self.cells.get(self.row_range(row))
    }
    // fn col(&self, col: usize) -> impl Iterator<Item = Cell> + Clone + '_ {
    //     if col >= self.width {
    //         Either::A(std::iter::empty())
    //     } else {
    //         Either::B(ColIter {
    //             cells: &self.cells,
    //             width: self.width,
    //             col,
    //             next_row: Some(0),
    //         })
    //     }
    // }
    fn col_indices(&self, col: usize) -> impl Iterator<Item = usize> + Clone {
        assert!(
            col < self.width,
            "col {col}, width {width}",
            width = self.width
        );
        if col >= self.width {
            Either::A(std::iter::empty())
        } else {
            Either::B(ColIterIndex {
                len: self.cells.len(),
                width: self.width,
                col,
                next_row: Some(0),
            })
        }
    }
    // fn series(&self, dimension: Dimension, n: usize) -> impl Iterator<Item = Cell> + Clone + '_ {
    //     match dimension {
    //         Dimension::Row => {
    //             let row = self.row(n).unwrap_or(&[]).iter().copied();
    //             Either::A(row)
    //         }
    //         Dimension::Col => Either::B(self.col(n)),
    //     }
    // }
    fn indices(&self, dimension: Dimension, n: usize) -> impl Iterator<Item = usize> + Clone {
        match dimension {
            Dimension::Row => Either::A(self.row_range(n)),
            Dimension::Col => Either::B(self.col_indices(n)),
        }
    }
    fn size(&self, dimension: Dimension) -> usize {
        match dimension {
            Dimension::Row => self.max_row,
            Dimension::Col => self.width,
        }
    }
    fn roll_stones(&mut self, direction: Direction) {
        let (roll_dim, roll_step) = direction.roll_dim();
        // dbg!(("ROLLING", direction, roll_dim, roll_step));

        let major_axis = self.size(roll_dim);
        for major in 0..major_axis {
            let elem_indices = self.indices(roll_dim, major);
            let mut elems: Vec<_> = elem_indices.clone().map(|i| self.cells[i]).collect();
            let elems_len = elems.len();

            // let DEBUG_elems = elems.clone();

            let (forward, reverse) = if roll_step < 0 {
                (elems_len, 0)
            } else {
                (0, elems_len)
            };
            let roll_start = (0..elems_len)
                .take(forward)
                .chain((0..elems_len).rev().take(reverse));

            let mut progress_made = true;
            while progress_made {
                progress_made = false;
                let mut dest = None;
                for src in roll_start.clone() {
                    if let Some(dest) = dest {
                        let dest_value = elems[dest];
                        let src_value = elems[src];

                        if let Cell::None = dest_value {
                            if let Cell::Some(Rock::Round) = src_value {
                                elems[dest] = src_value;
                                elems[src] = dest_value;
                                progress_made = true;
                            }
                        }
                    }
                    dest = Some(src);
                }
            }
            // dbg!("changed", major, &DEBUG_elems, &elems);

            for (i, elem) in elem_indices.zip(elems) {
                self.cells[i] = elem;
            }
        }
    }
    fn get_round_weight(&self) -> usize {
        let mut sum = 0;
        for (point, cell) in self.iter() {
            if cell == Cell::Some(Rock::Round) {
                let weight = self.max_row - point.row;
                sum += weight;
            }
        }
        sum
    }
    fn perform_cycles(self, total_cycles_count: usize) -> Self {
        // NOTE: A cycle is defined as 4 individual steps
        let steps_count = 4 * total_cycles_count;
        self.perform_steps(steps_count)
    }
    fn perform_steps(self, steps_count: usize) -> Self {
        let reference_clone = {
            let mut clone = self.clone();
            clone.random_direction_for_cache_key = Some(Direction::East);
            clone
        };
        self.perform_steps_inner(steps_count, Some(reference_clone), None)
    }
    fn perform_steps_inner(
        mut self,
        total_count: usize,
        find_cycle: Option<Self>,
        start_direction: Option<Direction>,
    ) -> Self {
        let mut past_cases = find_cycle.map(|original| {
            let mut map = HashMap::new();
            map.insert(original, 0);
            map
        });
        let mut direction = start_direction.unwrap_or_default();
        for count_index in 0..total_count {
            println!(
                "{count_index:9}\t{}\t{direction:?}",
                self.get_round_weight()
            );
            self.roll_stones(direction);
            self.random_direction_for_cache_key = Some(direction);
            if let Some(past_cases) = &mut past_cases {
                if let Some(past_count_index) = past_cases.get(&self) {
                    let cycle_length = count_index - past_count_index;
                    let remaining = (total_count - past_count_index - 1) % cycle_length;
                    dbg!(
                        "DETECTED CYCLE!!!",
                        count_index,
                        past_count_index,
                        cycle_length,
                        remaining
                    );
                    return self.perform_steps_inner(remaining, None, Some(direction.next()));
                } else {
                    past_cases.insert(self.clone(), count_index);
                }
            }
            direction = direction.next();
        }
        // dbg!(&self, self.get_round_weight());
        self
    }
}
impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Grid:")?;
        if let Some(direction) = &self.random_direction_for_cache_key {
            write!(f, "\t(last {direction:?})")?;
        }
        write!(f, "\n  ")?;
        for col in 0..self.width {
            write!(f, "{}", col % 10)?;
        }
        writeln!(f)?;

        for row in 0..self.max_row {
            write!(f, "{} ", row % 10)?;
            let Some(cells) = self.row(row) else {
                writeln!(f)?;
                break;
            };
            for cell in cells {
                write!(f, "{cell:?}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct ColIter<'a> {
    cells: &'a [Cell],
    width: usize,
    col: usize,
    next_row: Option<usize>,
}
impl Iterator for ColIter<'_> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.next_row?;
        let elem = {
            let point = Point { row, col: self.col };
            let index = point.index_for(self.width);
            self.cells.get(index).copied()
        };

        self.next_row = elem.is_some().then_some(row + 1);
        elem
    }
}

#[derive(Clone)]
struct ColIterIndex {
    len: usize,
    width: usize,
    col: usize,
    next_row: Option<usize>,
}
impl Iterator for ColIterIndex {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.next_row?;
        let index = {
            let point = Point { row, col: self.col };
            let index = point.index_for(self.width);
            (index < self.len).then_some(index)
        };

        self.next_row = index.is_some().then_some(row + 1);
        index
    }
}

#[derive(Clone)]
struct Iter<'a> {
    cells: &'a [Cell],
    width: usize,
    next_index: Option<usize>,
}
impl Iterator for Iter<'_> {
    type Item = (Point, Cell);
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.next_index.take()?;
        let point = Point::from_index_width(index, self.width);
        let cell = self.cells.get(index).copied()?;

        self.next_index = Some(index + 1);
        Some((point, cell))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Some(Rock),
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Rock {
    Round,
    Cube,
}
impl TryFrom<char> for Cell {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::None),
            'O' => Ok(Self::Some(Rock::Round)),
            '#' => Ok(Self::Some(Rock::Cube)),
            extra => Err(extra),
        }
    }
}
impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Some(Rock::Round) => 'O',
            Cell::Some(Rock::Cube) => '#',
            Cell::None => '.',
        };
        write!(f, "{c}")
    }
}

#[derive(Clone, Copy, Debug)]
enum Dimension {
    Row,
    Col,
}
// impl Dimension {
//     fn other(self) -> Self {
//         match self {
//             Self::Row => Self::Col,
//             Self::Col => Self::Row,
//         }
//     }
// }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Direction {
    #[default]
    North,
    West,
    South,
    East,
}
impl Direction {
    // fn all() -> impl Iterator<Item = Self> {
    //     const ALL: &'static [Direction] = &[
    //         Direction::North,
    //         Direction::West,
    //         Direction::South,
    //         Direction::East,
    //     ];
    //     std::iter::repeat(ALL).flatten().copied()
    // }
    fn roll_dim(self) -> (Dimension, isize) {
        match self {
            Self::North => (Dimension::Col, -1),
            Self::South => (Dimension::Col, 1),
            Self::West => (Dimension::Row, -1),
            Self::East => (Dimension::Row, 1),
        }
    }

    // fn cross_dim(self) -> Dimension {
    //     let (roll, _) = self.roll_dim();
    //     roll.other()
    // }

    fn next(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    row: usize,
    col: usize,
}
impl Point {
    fn index_for(self, width: usize) -> usize {
        assert!(width > 0);
        let Self { row, col } = self;
        (row * width) + col
    }
    fn from_index_width(index: usize, width: usize) -> Self {
        assert!(width > 0);
        Self {
            row: index / width,
            col: index % width,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{eval_input, Direction, Grid};

    macro_rules! grid {
        ($($line:expr),+ $(,)? $(; $direction:expr)?) => {{
            let lines = vec![
                $({
                    let line: &'static str = $line;
                    line
                }),+
            ];
            #[allow(unused_mut)]
            let mut grid = Grid::new(lines.into_iter())
                .expect("valid input")
                .expect("nonempty by macro construction");
            $(
                grid.random_direction_for_cache_key = Some($direction);
            )?
            grid
        }};
    }

    macro_rules! assert_roll {
        (
            original=$original:expr;
            roll $direction:expr;
            expected=$expected:expr;
        ) => {{
            let original: Grid = $original;
            dbg!(&original);
            let direction: Direction = $direction;
            let expected: Grid = $expected;
            dbg!(&expected);
            let modified = {
                let mut modified = original.clone();
                modified.roll_stones(direction);
                modified
            };
            assert_eq!(modified, expected, "{direction:?}");
            modified
        }};
    }

    macro_rules! assert_steps {
        (
            original=$original:expr;
            $(
                $count:expr => $expected:expr;
            )+
        ) => {{
            let original: Grid = $original;
            // dbg!(&original);
            $(
                let count: usize = $count;
                let expected: Grid = $expected;
                let modified = original.clone().perform_steps(count);
                assert_eq!(modified, expected, "for count {count:?}");
            )+
        }};
    }

    #[test]
    fn small_roll_north() {
        assert_roll! {
            original = grid! {
                "...",
                ".O#",
                "...",
            };
            roll Direction::North;
            expected = grid! {
                ".O.",
                "..#",
                "...",
            };
        };
    }
    #[test]
    fn small_rolls() {
        assert_roll! {
            original = grid! {
                ".#.",
                ".O.",
                "...",
            };
            roll Direction::East;
            expected = grid! {
                ".#.",
                "..O",
                "...",
            };
        };
        assert_roll! {
            original = grid! {
                "...",
                ".O#",
                "...",
            };
            roll Direction::South;
            expected = grid! {
                "...",
                "..#",
                ".O.",
            };
        };
        assert_roll! {
            original = grid! {
                "...",
                ".O#",
                "...",
            };
            roll Direction::West;
            expected = grid! {
                "...",
                "O.#",
                "...",
            };
        };
    }

    #[test]
    fn blocked_roll_north() {
        let unchanged = grid! {
            ".##",
            ".O#",
            "..O",
        };
        assert_roll! {
            original = unchanged.clone();
            roll Direction::North;
            expected = unchanged;
        };
    }
    #[test]
    fn roll_to_stop_north() {
        assert_roll! {
            original = grid! {
                "...#",
                ".#..",
                "..O.",
                "OOOO",
            };
            roll Direction::North;
            expected = grid! {
                "O.O#",
                ".#OO",
                ".O..",
                "....",
            };
        };
    }

    #[test]
    fn counts_round_weight_north() {
        let one = grid! {"O"};
        assert_eq!(one.get_round_weight(), 1);
        let five = grid! {"OOOOO"};
        assert_eq!(five.get_round_weight(), 5);
        let also_five = grid! {
            "O..",
            "...",
            "...",
            "...",
            "...",
        };
        assert_eq!(also_five.get_round_weight(), 5);

        let mut lots = grid! {
            "O...",
            ".O..",
            "..O.",
            "...O",
        };
        assert_eq!(lots.get_round_weight(), 4 + 3 + 2 + 1);
        lots.roll_stones(Direction::North);
        assert_eq!(lots.get_round_weight(), 4 * 4);
    }

    #[test]
    fn sample_input_slowly_north() {
        let grid = assert_roll! {
            original = grid! {
                "O....#....",
                "O.OO#....#",
                ".....##...",
                "OO.#O....O",
                ".O.....O#.",
                "O.#..O.#.#",
                "..O..#O..O",
                ".......O..",
                "#....###..",
                "#OO..#....",
            };
            roll Direction::North;
            expected = grid! {
                "OOOO.#.O..",
                "OO..#....#",
                "OO..O##..O",
                "O..#.OO...",
                "........#.",
                "..#....#.#",
                "..O..#.O.O",
                "..O.......",
                "#....###..",
                "#....#....",
            };
        };
        assert_eq!(grid.get_round_weight(), 136);
    }
    #[test]
    fn sample_input_north() {
        let input = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
";
        let stats = eval_input(input).unwrap();
        assert_eq!(stats.sum_round_weight, 136);
    }
    #[test]
    fn sample_input_one_million() {
        let input = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
";
        let stats = eval_input(input).unwrap();
        assert_eq!(stats.sum_after_one_million, 64);
    }

    #[test]
    fn sample_input_and_another_north() {
        let input = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....

....
.O..
...#
OOOO
";
        let stats = eval_input(input).unwrap();
        // NOTE: 4+4+4+3+1 is from the final config,
        //  OOO.
        //  .O..
        //  ...#
        //  ...O
        assert_eq!(stats.sum_round_weight, 136 + (4 + 4 + 4 + 3 + 1));
    }

    #[test]
    fn sample_input_micromanaging() {
        let original = grid! {
            "O....#....",
            "O.OO#....#",
            ".....##...",
            "OO.#O....O",
            ".O.....O#.",
            "O.#..O.#.#",
            "..O..#O..O",
            ".......O..",
            "#....###..",
            "#OO..#....",
        };
        assert_steps! {
            original = original.clone();
            1 => grid! {
                "OOOO.#.O..",
                "OO..#....#",
                "OO..O##..O",
                "O..#.OO...",
                "........#.",
                "..#....#.#",
                "..O..#.O.O",
                "..O.......",
                "#....###..",
                "#....#...."; Direction::North
            };
            2 => grid! {
                "OOOO.#O...",
                "OO..#....#",
                "OOO..##O..",
                "O..#OO....",
                "........#.",
                "..#....#.#",
                "O....#OO..",
                "O.........",
                "#....###..",
                "#....#...."; Direction::West
            };
            3 => grid! {
                ".....#....",
                "....#.O..#",
                "O..O.##...",
                "O.O#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#....",
                "OO....OO..",
                "#O...###..",
                "#O..O#...."; Direction::South
            };
            4 => grid! {
                ".....#....",
                "....#...O#",
                "...OO##...",
                ".OO#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#....",
                "......OOOO",
                "#...O###..",
                "#..OO#...."; Direction::East
            };
            5 => grid! {
                ".OOO.#.OO.",
                ".O..#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#.O.O#O#",
                ".....#.O.O",
                "..........",
                "#....###..",
                "#....#...."; Direction::North
            };
            6 => grid! {
                "OOO..#OO..",
                "O...#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#OO..#O#",
                ".....#OO..",
                "..........",
                "#....###..",
                "#....#...."; Direction::West
            };
            7 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "..O#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OO..",
                "#O..O###..",
                "#O.OO#..O."; Direction::South
            };
            8 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                ".......OOO",
                "#..OO###..",
                "#.OOO#...O"; Direction::East
            };
            9 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#O...#O#",
                "..O..#.O.O",
                ".........O",
                "#....###.O",
                "#....#...."; Direction::North
            };
            10 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#O...#O#",
                "O....#OO..",
                "O.........",
                "#....###O.",
                "#....#...."; Direction::West
            };
            11 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "O..#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OO..",
                "#O...###O.",
                "#O.OO#..O."; Direction::South
            };
            12 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                ".......OOO",
                "#...O###.O",
                "#.OOO#...O"; Direction::East
            };
            13 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#....#O#",
                "..O..#.O.O",
                ".........O",
                "#....###.O",
                "#....#...O"; Direction::North
            };
            14 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#....#O#",
                "O....#OO..",
                "O.........",
                "#....###O.",
                "#....#O..."; Direction::West
            };
            15 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "O..#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OO..",
                "#O...###O.",
                "#O..O#O.O."; Direction::South
            };
            16 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                ".......OOO",
                "#...O###.O",
                "#..OO#..OO"; Direction::East
            };
            17 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#....#O#",
                ".....#.OOO",
                ".........O",
                "#....###.O",
                "#....#...O"; Direction::North
            };
            18 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#....#O#",
                ".....#OOO.",
                "O.........",
                "#....###O.",
                "#....#O..."; Direction::West
            };
            19 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "...#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OOO.",
                "#O...###O.",
                "#O..O#O.O."; Direction::South
            };
            20 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "...#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                "......OOOO",
                "#...O###.O",
                "#..OO#..OO"; Direction::East
            };
            21 => grid! {
                ".O...#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#...O#O#",
                ".....#.OOO",
                ".........O",
                "#....###.O",
                "#....#...O"; Direction::North
            };
            22 => grid! {
                "O....#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#O...#O#",
                ".....#OOO.",
                "O.........",
                "#....###O.",
                "#....#O..."; Direction::West
            };
            23 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "...#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OOO.",
                "#....###O.",
                "#O.OO#O.O."; Direction::South
            };
            24 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "...#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                "......OOOO",
                "#....###.O",
                "#.OOO#..OO"; Direction::East
            };
            25 => grid! {
                ".O...#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...O..O.#.",
                "..#...O#O#",
                "..O..#.OOO",
                ".........O",
                "#....###.O",
                "#....#...O"; Direction::North
            };
            26 => grid! {
                "O....#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OO......#.",
                "..#O...#O#",
                "O....#OOO.",
                "O.........",
                "#....###O.",
                "#....#O..."; Direction::West
            };
            27 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "O..#......",
                "O......O#.",
                "O.#..O.#.#",
                "O....#O...",
                "O.....OOO.",
                "#....###O.",
                "#O.OO#O.O."; Direction::South
            };
            28 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                "......OO#.",
                ".O#...O#.#",
                "....O#...O",
                "......OOOO",
                "#....###.O",
                "#.OOO#..OO"; Direction::East
            };
            29 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#O.O...",
                "...O..O.#.",
                "..#...O#O#",
                "..O..#.OOO",
                ".........O",
                "#....###.O",
                "#....#...O"; Direction::North
            };
            30 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OO....",
                "OO......#.",
                "..#O...#O#",
                "O....#OOO.",
                "O.........",
                "#....###O.",
                "#....#O..."; Direction::West
            };
            31 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "O..#......",
                "O......O#.",
                "O.#..O.#.#",
                "O....#....",
                "O.....OOO.",
                "#O...###O.",
                "#O.OO#O.O."; Direction::South
            };
            32 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                "......OO#.",
                ".O#...O#.#",
                "....O#....",
                "......OOOO",
                "#...O###.O",
                "#.OOO#..OO"; Direction::East
            };
            33 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#O.O...",
                "...OO.O.#.",
                "..#...O#O#",
                "..O..#.OOO",
                ".........O",
                "#....###.O",
                "#....#...."; Direction::North
            };
            34 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OO....",
                "OOO.....#.",
                "..#O...#O#",
                "O....#OOO.",
                "O.........",
                "#....###O.",
                "#....#...."; Direction::West
            };
            35 => grid! {
                ".....#....",
                "....#.O..#",
                ".....##...",
                "O..#......",
                "O.O....O#.",
                "O.#..O.#.#",
                "O....#....",
                "O.....OOO.",
                "#O...###O.",
                "#O.OO#..O."; Direction::South
            };
            36 => grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "..O#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#....",
                "......OOOO",
                "#...O###.O",
                "#.OOO#...O"; Direction::East
            };
            37 => grid! {
                ".OO..#.OO.",
                "....#....#",
                "....O##...",
                "...#OOO...",
                "...OO.O.#.",
                "..#...O#O#",
                "..O..#.O.O",
                ".........O",
                "#....###.O",
                "#....#...."; Direction::North
            };
            // NOTE on step 38, detects cycle:
            //     count_index = 37
            //     past_count_index = 9  (e.g. same as "10" total in this test)
            //     cycle_length = 28
            //     remaining = 0
            38 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#O...#O#",
                "O....#OO..",
                "O.........",
                "#....###O.",
                "#....#...."; Direction::West
            };
        };
        assert_steps! {
            original = original.clone();
            38 + 35712*28 => grid! {
                "OO...#OO..",
                "....#....#",
                "O....##...",
                "...#OOO...",
                "OOO.....#.",
                "..#O...#O#",
                "O....#OO..",
                "O.........",
                "#....###O.",
                "#....#...."; Direction::West
            };
            4_000_000_000 =>
            /* 24 => */ grid! {
                ".....#....",
                "....#...O#",
                ".....##...",
                "...#......",
                ".....OOO#.",
                ".O#...O#.#",
                "....O#...O",
                "......OOOO",
                "#....###.O",
                "#.OOO#..OO"; Direction::East
            };
            // /* 38 => */ grid! {
            //     "OO...#OO..",
            //     "....#....#",
            //     "O....##...",
            //     "...#OOO...",
            //     "OOO.....#.",
            //     "..#O...#O#",
            //     "O....#OO..",
            //     "O.........",
            //     "#....###O.",
            //     "#....#...."; Direction::West
            // };
            // /* 30 => */ grid! {
            //     "OO...#OO..",
            //     "....#....#",
            //     "O....##...",
            //     "...#OO....",
            //     "OO......#.",
            //     "..#O...#O#",
            //     "O....#OOO.",
            //     "O.........",
            //     "#....###O.",
            //     "#....#O..."; Direction::West
            // };
            // /* 31 => */ grid! {
            //     ".....#....",
            //     "....#.O..#",
            //     ".....##...",
            //     "O..#......",
            //     "O......O#.",
            //     "O.#..O.#.#",
            //     "O....#....",
            //     "O.....OOO.",
            //     "#O...###O.",
            //     "#O.OO#O.O."; Direction::South
            // };
            // /* 32 => */ grid! {
            //     ".....#....",
            //     "....#...O#",
            //     ".....##...",
            //     "..O#......",
            //     "......OO#.",
            //     ".O#...O#.#",
            //     "....O#....",
            //     "......OOOO",
            //     "#...O###.O",
            //     "#.OOO#..OO"; Direction::East
            // };
        }

        assert_eq!(37 - 9, 28);

        // 4_000_000_000 / 28 = 142857141 remainder 14
        assert_eq!(38 + (28 * 142857141) + 14, 4_000_000_000_u64);

        let state10_step9 = grid! {
            "OO...#OO..",
            "....#....#",
            "O....##...",
            "...#OOO...",
            "OOO.....#.",
            "..#O...#O#",
            "O....#OO..",
            "O.........",
            "#....###O.",
            "#....#...."; Direction::West
        };
        let state38_step37 = grid! {
            "OO...#OO..",
            "....#....#",
            "O....##...",
            "...#OOO...",
            "OOO.....#.",
            "..#O...#O#",
            "O....#OO..",
            "O.........",
            "#....###O.",
            "#....#...."; Direction::West
        };
        assert_eq!(state10_step9, state38_step37);

        let state24_step23 = grid! {
            ".....#....",
            "....#...O#",
            ".....##...",
            "...#......",
            ".....OOO#.",
            ".O#...O#.#",
            "....O#...O",
            "......OOOO",
            "#....###.O",
            "#.OOO#..OO"; Direction::East
        };
        // let state32_step31 = grid! {
        //     ".....#....",
        //     "....#...O#",
        //     ".....##...",
        //     "..O#......",
        //     "......OO#.",
        //     ".O#...O#.#",
        //     "....O#....",
        //     "......OOOO",
        //     "#...O###.O",
        //     "#.OOO#..OO"; Direction::East
        // };
        assert_eq!(state24_step23.get_round_weight(), 64);
    }

    #[test]
    fn tiny_cycle_lowlevel() {
        assert_steps! {
            original = grid! {
                "..",
                "O.",
            };
            1 => grid! {
                "O.",
                ".."; Direction::North
            };
            2 => grid! {
                "O.",
                ".."; Direction::West
            };
            3 => grid! {
                "..",
                "O."; Direction::South
            };
            4 => grid! {
                "..",
                ".O"; Direction::East
            };
            5 => grid! {
                ".O",
                ".."; Direction::North
            };
            6 => grid! {
                "O.",
                ".."; Direction::West
            };
        }
    }
    #[test]
    fn tiny_cycle() {
        let input = "..
O.";
        let stats = eval_input(input).unwrap();
        assert_eq!(stats.sum_after_one_million, 1);
    }

    #[test]
    fn medium_cycle() {
        assert_steps! {
            original = grid! {
                "##O.####",
                "######..",
                "..####..",
                ".O#####O",
            };
            1 => grid! {
                "##O.####",
                "######.O",
                ".O####..",
                "..#####."; Direction::North
            };
            2 => grid! {
                "##O.####",
                "######O.",
                "O.####..",
                "..#####."; Direction::West
            };
            3 => grid! {
                "##O.####",
                "######..",
                "..####O.",
                "O.#####."; Direction::South
            };
            4 => grid! {
                "##.O####",
                "######..",
                "..####.O",
                ".O#####."; Direction::East
            };
            8 => grid! {
                "##.O####",
                "######..",
                "..####.O",
                ".O#####."; Direction::East
            };
            8_000_000 => grid! {
                "##.O####",
                "######..",
                "..####.O",
                ".O#####."; Direction::East
            };
        }
    }
    #[test]
    fn random_pattern() {
        assert_roll! {
            original = grid! {
                "##O.####",
                "######..",
                "..####..",
                ".O#####O",
            };
            roll Direction::North;
            expected = grid! {
                "##O.####",
                "######.O",
                ".O####..",
                "..#####.",
            };
        };
    }
    #[test]
    fn dimension_bounds() {
        let mut direction = Direction::North;
        for _ in 0..4 {
            let mut tall = grid! {
                "...",
                "...",
                "...",
                "...",
                ".O.",
                "...",
            };
            tall.roll_stones(direction);

            let mut wide = grid! {
                ".......................",
                ".....................O.",
                ".......................",
            };
            wide.roll_stones(direction);

            // advance
            direction = direction.next();
        }
    }

    #[test]
    fn traveling_to_cycle() {
        let original = grid! {
            "#####################....",
            "#####################.##O",
            "###################....##",
            "###################.#####",
            "###########..........####",
            "###########.#############",
            ".............########.###",
            ".###################O..##",
            ".#..#################OO.#",
            "....#################..##",
        };
        assert_steps! {
            original = original.clone();
            0 => original;
            1 => grid! {
                "#####################...O",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########O###",
                ".###################O.O##",
                ".#..#################...#",
                "....#################..##"; Direction::North
            };
            2 => grid! {
                "#####################O...",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########O###",
                ".###################OO.##",
                ".#..#################...#",
                "....#################..##"; Direction::West
            };
            3 => grid! {
                "#####################....",
                "#####################.##.",
                "###################..O.##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################O..##",
                ".#..#################O..#",
                "....#################O.##"; Direction::South
            };
            4 => grid! {
                "#####################....",
                "#####################.##.",
                "###################...O##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################..O##",
                ".#..#################..O#",
                "....#################.O##"; Direction::East
            };
            // skip 5, North is not interesting
            6 => grid! {
                "#####################....",
                "#####################.##.",
                "###################O...##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################O..##",
                ".#..#################OO.#",
                "....#################..##"; Direction::West
            };
            7 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########........O.####",
                "###########.#############",
                ".............########.###",
                ".###################O..##",
                ".#..#################...#",
                "....#################OO##"; Direction::South
            };
            8 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########.........O####",
                "###########.#############",
                ".............########.###",
                ".###################..O##",
                ".#..#################...#",
                "....#################OO##"; Direction::East
            };
            // skip 11, North is not interesting
            10 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########O.........####",
                "###########.#############",
                ".............########O###",
                ".###################O..##",
                ".#..#################O..#",
                "....#################..##"; Direction::West
            };
            11 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                "...........O.########.###",
                ".###################O..##",
                ".#..#################O..#",
                "....#################O.##"; Direction::South
            };
            12 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                "............O########.###",
                ".###################..O##",
                ".#..#################..O#",
                "....#################.O##"; Direction::East
            };
            // skip 13, North is not interesting
            14 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                "O............########.###",
                ".###################O..##",
                ".#..#################OO.#",
                "....#################..##"; Direction::West
            };
            15 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################O..##",
                ".#..#################...#",
                "O...#################OO##"; Direction::South
            };
            16 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################..O##",
                ".#..#################...#",
                "...O#################OO##"; Direction::East
            };
            16+4 => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################..O##",
                ".#..#################..O#",
                "...O#################.O##"; Direction::East
            };
            16+(10000000*4) => grid! {
                "#####################....",
                "#####################.##.",
                "###################....##",
                "###################.#####",
                "###########..........####",
                "###########.#############",
                ".............########.###",
                ".###################..O##",
                ".#..#################...#",
                "...O#################OO##"; Direction::East
            };
        }
    }
}
