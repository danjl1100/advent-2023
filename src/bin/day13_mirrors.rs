fn main() -> anyhow::Result<()> {
    println!("hello, where are the mirrors?");

    let input = advent_2023::get_input_string()?;

    let Stats { summary_sum } = eval_input(&input, Some(1))?;

    println!("Sum of reflection notes: {summary_sum}");

    Ok(())
}

struct Stats {
    summary_sum: usize,
}

fn eval_input(input: &str, goal_error_count: Option<usize>) -> anyhow::Result<Stats> {
    let mut summary_sum = 0;
    let mut lines = input.lines();
    loop {
        let Some(mut grid) = Grid::new(lines.by_ref())? else {
            break;
        };
        if let Some(goal_error_count) = goal_error_count {
            grid.goal_error_count = goal_error_count;
        }
        summary_sum += grid.summarize_reflection();
    }
    Ok(Stats { summary_sum })
}

struct Grid {
    cells: Vec<Cell>,
    width: usize,
    max_row: usize,
    goal_error_count: usize,
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
            goal_error_count: 0,
        }))
    }
    fn row(&self, row: usize) -> Option<&[Cell]> {
        let start = Point { row, col: 0 }.index_for(self.width);
        let end = Point {
            row: row + 1,
            col: 0,
        }
        .index_for(self.width);
        self.cells.get(start..end)
    }
    fn col(&self, col: usize) -> impl Iterator<Item = Cell> + Clone + '_ {
        if col >= self.width {
            Either::A(std::iter::empty())
        } else {
            Either::B(ColIter {
                cells: &self.cells,
                width: self.width,
                col,
                next_row: Some(0),
            })
        }
    }
    fn series(&self, dimension: Dimension, n: usize) -> impl Iterator<Item = Cell> + Clone + '_ {
        match dimension {
            Dimension::Row => {
                let row = self.row(n).unwrap_or(&[]).iter().copied();
                Either::A(row)
            }
            Dimension::Col => Either::B(self.col(n)),
        }
    }
    fn size(&self, dimension: Dimension) -> usize {
        match dimension {
            Dimension::Row => self.max_row,
            Dimension::Col => self.width,
        }
    }
    /// Returns `Some(true)` if all cases matched, `Some(false)` if a contradiction is found,
    /// or `None` if no comparison could be performed
    fn get_error_count(&self, dimension: Dimension, (a, b): (usize, usize)) -> Option<usize> {
        const DEBUG: bool = false;

        let mut a_iter = self.series(dimension, a);
        let mut b_iter = self.series(dimension, b);
        let (a_debug, b_debug) = if DEBUG {
            let a_debug = a_iter.clone().collect::<Vec<_>>();
            let b_debug = b_iter.clone().collect::<Vec<_>>();
            (a_debug, b_debug)
        } else {
            (vec![], vec![])
        };
        let mut error_count = None;
        loop {
            match (a_iter.next(), b_iter.next()) {
                (None, None) => {
                    break;
                }
                (Some(a), Some(b)) if a != b => {
                    error_count = Some(error_count.unwrap_or_default() + 1);
                }
                (Some(_), Some(_)) => {
                    error_count = Some(error_count.unwrap_or(0));
                }
                (None, Some(_)) | (Some(_), None) => {}
            }
        }
        if DEBUG {
            println!("\terrors={error_count:?} {dimension:?} ({a},{b}): {a_debug:?}; {b_debug:?}");
        }
        error_count
    }
    fn is_reflection(&self, dimension: Dimension, pivot: usize) -> Option<bool> {
        let mut can_start = Some(());
        let mut matched = None;
        for add in 0.. {
            let first = pivot.checked_sub(add);
            let second = pivot + add + 1;
            match first.and_then(|first| self.get_error_count(dimension, (first, second))) {
                None => {
                    break;
                }
                Some(error_count) => {
                    let is_new = error_count == self.goal_error_count;
                    let is_allowed = error_count <= self.goal_error_count;
                    matched = if is_new && can_start.take().is_some() {
                        Some(true)
                    } else if !is_allowed {
                        Some(false)
                    } else {
                        matched
                    };
                    if !is_allowed {
                        break;
                    }
                }
            }
        }
        println!("reflection? {dimension:?} pivot {pivot}: {matched:?}");
        matched
    }
    fn summarize_reflection(&self) -> usize {
        assert!(self.max_row % 2 == 1, "odd rows {}", self.max_row);
        assert!(self.width % 2 == 1, "odd cols {}", self.width);

        // analyze ROWS
        for dimension in [Dimension::Row, Dimension::Col] {
            for pivot in 0..self.size(dimension) {
                if let Some(true) = self.is_reflection(dimension, pivot) {
                    let code = match dimension {
                        Dimension::Row => (pivot + 1) * 100,
                        Dimension::Col => pivot + 1,
                    };
                    return code;
                }
            }
        }
        0
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Ash,
    Rock,
}
impl TryFrom<char> for Cell {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Ash),
            '#' => Ok(Self::Rock),
            extra => Err(extra),
        }
    }
}
impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Ash => '.',
            Cell::Rock => '#',
        };
        write!(f, "{c}")
    }
}

#[derive(Clone, Copy, Debug)]
enum Dimension {
    Row,
    Col,
}

#[derive(Clone)]
enum Either<A, B> {
    A(A),
    B(B),
}
impl<A, B, T> Iterator for Either<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::A(inner) => inner.next(),
            Either::B(inner) => inner.next(),
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
    // TODO delete, unused
    // fn from_index_width(index: usize, width: usize) -> Self {
    //     assert!(width > 0);
    //     Self {
    //         row: index / width,
    //         col: index % width,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::{eval_input, Cell, Dimension, Grid};

    #[test]
    fn finds_cols() {
        let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.";
        let grid = Grid::new_from_str(input)
            .expect("valid text")
            .expect("nonempty");
        let col_8 = grid.col(8).collect::<Vec<_>>();
        assert_eq!(
            col_8,
            vec![
                Cell::Ash,
                Cell::Ash,
                Cell::Rock,
                Cell::Rock,
                Cell::Ash,
                Cell::Ash,
                Cell::Ash,
            ]
        );

        assert_eq!(grid.is_reflection(Dimension::Col, 4), Some(true));
    }

    #[test]
    fn sample_input() {
        let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";
        let stats = eval_input(input, None).unwrap();
        assert_eq!(stats.summary_sum, 405);
    }

    #[test]
    fn sample_input_with_one_error() {
        let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";
        let stats = eval_input(input, Some(1)).unwrap();
        assert_eq!(stats.summary_sum, 400);
    }
}
