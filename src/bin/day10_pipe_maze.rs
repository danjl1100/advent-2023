fn main() -> anyhow::Result<()> {
    println!("hello, pipe maze");

    let input = advent_2023::get_input_string()?;

    let grid = Grid::try_new(&input)?;

    let longest = find_longest_step(&grid)?;
    println!("Longest step: {longest}");

    Ok(())
}

fn find_longest_step(grid: &Grid) -> anyhow::Result<usize> {
    let (start, directions) = grid.get_start_directions();
    let mut state: Vec<(Point, Direction)> = directions
        .into_iter()
        .map(|direction| (start, direction))
        .collect();
    for step in 1.. {
        for (point, direction) in state.iter_mut() {
            let Some((new_point, new_direction)) = grid.get_next_linked(*point, *direction) else {
                anyhow::bail!("no next found for {point:?} {direction:?}")
            };
            *point = new_point;
            *direction = new_direction;
        }
        let ended: Vec<bool> = state
            .iter()
            .enumerate()
            .map(|(index, (point, _))| {
                // check if two different points are equal
                state
                    .iter()
                    .enumerate()
                    .any(|(i, (p, _))| index != i && point == p)
            })
            .collect();
        for (ended_index, _) in ended
            .into_iter()
            .enumerate()
            .filter(|&(_, ended)| ended)
            .rev()
        {
            state.remove(ended_index);
        }
        if state.is_empty() {
            return Ok(step);
        }
    }
    anyhow::bail!("overflowed step count tracing the paths")
}

#[derive(Debug)]
struct Grid {
    width: usize,
    cells: Vec<Cell>,
    start: Point,
}
impl Grid {
    fn try_new(input: &str) -> anyhow::Result<Self> {
        let mut cells = vec![];
        let mut width: Option<usize> = None;
        let mut start = None;
        for (row, line) in input.lines().enumerate() {
            let row_cells = line
                .chars()
                .map(Cell::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|c| anyhow::anyhow!("unrecognized char {c}"))?;
            let new_width = row_cells.len();
            match &mut width {
                &mut Some(width) if width == new_width => {}
                Some(old_width) => {
                    anyhow::bail!("width mismatch {old_width} <---> {new_width}")
                }
                width @ None => {
                    *width = Some(new_width);
                }
            }
            if start.is_none() {
                // TODO
                start = row_cells
                    .iter()
                    .copied()
                    .enumerate()
                    .find_map(|(col, cell)| (cell == Cell::Start).then_some(Point { row, col }))
            }
            cells.extend(row_cells);
        }
        let Some(width) = width else {
            anyhow::bail!("empty input")
        };
        let Some(start) = start else {
            anyhow::bail!("no start location given")
        };
        Ok(Self {
            width,
            cells,
            start,
        })
    }
    fn lookup_cell(&self, point: Point) -> Option<Cell> {
        let index = point.index_for_width(self.width)?;
        self.cells.get(index).copied()
    }

    /// Returns the start location, and valid outward travel directions
    fn get_start_directions(&self) -> (Point, Vec<Direction>) {
        let Self { start, .. } = *self;
        let directions = Direction::ALL
            .iter()
            .copied()
            .filter_map(|direction| {
                // START --direction-> TARGET
                let target = direction.of(start)?;
                let target_cell = self.lookup_cell(target)?;
                // if TARGET connects any of reverse direction
                target_cell
                    .connects_any(-direction)
                    .is_some()
                    .then_some(direction)
            })
            .collect();
        (start, directions)
    }
    /// Returns the next location/direction given the previous location/direction
    fn get_next_linked(&self, prev: Point, direction: Direction) -> Option<(Point, Direction)> {
        let current = direction.of(prev)?;
        let current_cell = self.lookup_cell(current)?;

        let reverse_entry = -direction;
        let next_direction = current_cell.connects_any(reverse_entry)?;
        Some((current, next_direction))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    row: usize,
    col: usize,
}
impl Point {
    fn index_for_width(self, width: usize) -> Option<usize> {
        let Self { row, col } = self;
        (col < width).then_some(row * width + col)
    }
    // /// Returns the 4 adjacent neighbors of the point (if any)
    // fn neighbors(self) -> impl Iterator<Item = Self> {
    //     let Self {
    //         row: row_center,
    //         col: col_center,
    //     } = self;
    //     let row_min = row_center.saturating_sub(1);
    //     let col_min = col_center.saturating_sub(1);
    //     (row_min..(row_center + 2)).flat_map(move |row| {
    //         (col_min..(col_center + 2)).filter_map(move |col| {
    //             (row == row_center || col == col_center)
    //                 .not()
    //                 .then_some(Self { row, col })
    //         })
    //     })
    // }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}
impl Direction {
    const ALL: &'static [Self] = &[Self::North, Self::South, Self::East, Self::West];
    fn of(self, src: Point) -> Option<Point> {
        let Point { row, col } = src;

        let row = match self {
            Self::North => row.checked_sub(1),
            Self::South => Some(row + 1),
            Self::West | Self::East => Some(row),
        };
        let col = match self {
            Self::West => col.checked_sub(1),
            Self::East => Some(col + 1),
            Self::North | Self::South => Some(col),
        };

        row.zip(col).map(|(row, col)| Point { row, col })
    }
}
impl std::ops::Neg for Direction {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Empty,
    PipeVertical,
    PipeHorizontal,
    BendNE,
    BendNW,
    BendSE,
    BendSW,
    Start,
}
impl Cell {
    fn connections(self) -> Option<(Direction, Direction)> {
        use Direction::{East, North, South, West};
        match self {
            Cell::PipeVertical => Some((North, South)),
            Cell::PipeHorizontal => Some((West, East)),
            Cell::BendNE => Some((North, East)),
            Cell::BendNW => Some((North, West)),
            Cell::BendSE => Some((South, East)),
            Cell::BendSW => Some((South, West)),
            Cell::Start | Cell::Empty => None,
        }
    }
    // fn connects(self, from: Direction, to: Direction) -> bool {
    //     let Some(connections) = self.connections() else {
    //         return false;
    //     };
    //     (from, to) == connections || (to, from) == connections
    // }
    fn connects_any(&self, from: Direction) -> Option<Direction> {
        let Some((connection1, connection2)) = self.connections() else {
            return None;
        };
        if from == connection1 {
            Some(connection2)
        } else if from == connection2 {
            Some(connection1)
        } else {
            None
        }
    }
}
impl TryFrom<char> for Cell {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let cell = match value {
            '|' => Self::PipeVertical,
            '-' => Self::PipeHorizontal,
            'L' => Self::BendNE,
            'J' => Self::BendNW,
            '7' => Self::BendSW,
            'F' => Self::BendSE,
            '.' => Self::Empty,
            'S' => Self::Start,
            other => {
                return Err(other);
            }
        };
        Ok(cell)
    }
}

#[cfg(test)]
mod tests {
    use crate::{find_longest_step, Grid};

    #[test]
    fn sample_input() {
        let input = ".....
.S-7.
.|.|.
.L-J.
.....";

        let grid = Grid::try_new(input).unwrap();
        dbg!(&grid);

        let longest = find_longest_step(&grid).unwrap();
        assert_eq!(longest, 4);
    }

    #[test]
    fn sample_complex_input() {
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";

        let grid = Grid::try_new(input).unwrap();
        dbg!(&grid);

        let longest = find_longest_step(&grid).unwrap();
        assert_eq!(longest, 8);
    }
}
