fn main() -> anyhow::Result<()> {
    println!("hello, pipe maze");

    let input = advent_2023::get_input_string()?;

    let grid = Grid::try_new(&input)?;

    let (grid_usage, longest) = find_longest_step(grid)?;
    println!("Longest step: {longest}");

    let area = find_area_enclosed(&grid_usage)?;
    println!("Area enclosed: {area}");

    Ok(())
}

fn find_longest_step(grid: Grid) -> anyhow::Result<(GridUsage, usize)> {
    println!("Grid:\n{grid}");

    let (start, directions) = grid.get_start_directions();
    let mut state: Vec<(Point, Direction)> = directions
        .iter()
        .copied()
        .map(|direction| (start, direction))
        .collect();

    let mut used = vec![None; grid.cells.len()];
    {
        // mark Start as used
        let index = start
            .index_for_width(grid.width)
            .expect("start position valid");
        let used = used.get_mut(index).expect("start position valid");
        *used = Some(Used);
    }

    for step in 1.. {
        // advance state
        for (point, direction) in state.iter_mut() {
            let Some((new_point, new_direction)) = grid.get_next_linked(*point, *direction) else {
                anyhow::bail!("no next found for {point:?} {direction:?}")
            };
            *point = new_point;
            *direction = new_direction;
        }
        // mark cells used
        for (point, _) in &state {
            let index = point
                .index_for_width(grid.width)
                .expect("previous state point is still valid");
            let used = used
                .get_mut(index)
                .expect("previous state point indexes used");
            let _prev_used = std::mem::replace(used, Some(Used));
        }
        // detect collisions
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
            // end
            let longest = step;
            let usage = GridUsage::new(grid, used, (start, directions))?;
            return Ok((usage, longest));
        }
    }
    anyhow::bail!("overflowed step count tracing the paths")
}

fn find_area_enclosed(grid_usage: &GridUsage) -> anyhow::Result<usize> {
    println!("Grid usage:\n{grid_usage}");

    let mut area = 0;
    'outer: for row in 0.. {
        let mut inside_boundary = false;
        let mut vertical_bend_prev = None;
        for col in 0..grid_usage.grid.width {
            let point = Point { row, col };
            let Some((cell, used)) = grid_usage.get_cell_used(point) else {
                break 'outer;
            };

            if let Some(Used) = used {
                // update `inside_boundary`

                let Some(directions) = cell.get_directions() else {
                    anyhow::bail!("missing directions for *used* cell {cell:?} at {point:?}")
                };
                let boundary_change = match directions {
                    (Direction::V(_), Direction::V(_)) => {
                        if let Some(vertical_bend_prev) = vertical_bend_prev {
                            anyhow::bail!("vertical pipe seen at {point:?} when bend {vertical_bend_prev:?} in progress")
                        }
                        true
                    }
                    (Direction::H(_), Direction::H(_)) => {
                        if vertical_bend_prev.is_none() {
                            anyhow::bail!(
                                "horizontal pipe seen at {point:?} when bend *NOT* in progress"
                            )
                        }
                        false
                    }
                    (Direction::V(v), Direction::H(_h)) | (Direction::H(_h), Direction::V(v)) => {
                        if let Some(vertical_bend_prev) = vertical_bend_prev.take() {
                            // TRUE for vertical change between bends
                            v != vertical_bend_prev
                        } else {
                            vertical_bend_prev = Some(v);
                            // wait for bend to complete...
                            false
                        }
                    }
                };

                if boundary_change {
                    inside_boundary = !inside_boundary;
                    let inside = if inside_boundary { "IN" } else { "OUT" };
                    println!("{inside} at Point({row}, {col}) cell {cell:?}");
                }
            } else if inside_boundary {
                println!("\tInside boundary at Point({row}, {col}) cell {cell:?}");
                area += 1;
            }
        }
    }
    Ok(area)
}

#[derive(Debug)]
struct GridUsage {
    grid: Grid,
    used: Vec<Option<Used>>,
}
impl GridUsage {
    fn new(
        mut grid: Grid,
        used: Vec<Option<Used>>,
        (start, directions): (Point, Vec<Direction>),
    ) -> anyhow::Result<Self> {
        assert_eq!(grid.cells.len(), used.len());

        // modify START into appropriate 2-direction pipe
        {
            let Some(start_cell) = grid.get_cell_mut(start) else {
                anyhow::bail!("start cell not found at {start:?}")
            };
            let [direction1, direction2]: [Direction; 2] = match directions.try_into() {
                Ok(directions) => directions,
                Err(directions) => {
                    anyhow::bail!("start directions count is not 2: {directions:?}")
                }
            };
            *start_cell = match Cell::try_from((direction1, direction2)) {
                Ok(cell) => cell,
                Err(direction_equal) => {
                    anyhow::bail!(
                        "invalid equal start directions: {direction_equal:?}, {direction_equal:?}"
                    )
                }
            };
        }

        Ok(Self { grid, used })
    }
    fn get_cell_used(&self, point: Point) -> Option<(Cell, Option<Used>)> {
        let index = point.index_for_width(self.grid.width)?;
        if let Some(cell) = self.grid.cells.get(index).copied() {
            let used = self
                .used
                .get(index)
                .copied()
                .expect("index with cell is in bounds for used");
            Some((cell, used))
        } else {
            None
        }
    }
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
                // search for `Start` in current row
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

    fn get_cell(&self, point: Point) -> Option<Cell> {
        let index = point.index_for_width(self.width)?;
        self.cells.get(index).copied()
    }
    fn get_cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        let index = point.index_for_width(self.width)?;
        self.cells.get_mut(index)
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
                let target_cell = self.get_cell(target)?;
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
        let current_cell = self.get_cell(current)?;

        let reverse_entry = -direction;
        let next_direction = current_cell.connects_any(reverse_entry)?;
        Some((current, next_direction))
    }
}

#[derive(Clone, Copy, Debug)]
struct Used;

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
}

const NORTH: Direction = Direction::V(DirectionV::North);
const SOUTH: Direction = Direction::V(DirectionV::South);
const EAST: Direction = Direction::H(DirectionH::East);
const WEST: Direction = Direction::H(DirectionH::West);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    V(DirectionV),
    H(DirectionH),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirectionV {
    North,
    South,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirectionH {
    East,
    West,
}
impl Direction {
    const ALL: &'static [Self] = &[NORTH, SOUTH, EAST, WEST];
    fn of(self, src: Point) -> Option<Point> {
        let Point { row, col } = src;

        let row = match self {
            Self::V(DirectionV::North) => row.checked_sub(1),
            Self::V(DirectionV::South) => Some(row + 1),
            Self::H(_) => Some(row),
        };
        let col = match self {
            Self::H(DirectionH::West) => col.checked_sub(1),
            Self::H(DirectionH::East) => Some(col + 1),
            Self::V(_) => Some(col),
        };

        row.zip(col).map(|(row, col)| Point { row, col })
    }
}
impl std::ops::Neg for Direction {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::V(inner) => Self::V(-inner),
            Self::H(inner) => Self::H(-inner),
        }
    }
}
impl std::ops::Neg for DirectionV {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
        }
    }
}
impl std::ops::Neg for DirectionH {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
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
    fn get_directions(self) -> Option<(Direction, Direction)> {
        match self {
            Cell::PipeVertical => Some((NORTH, SOUTH)),
            Cell::PipeHorizontal => Some((WEST, EAST)),
            Cell::BendNE => Some((NORTH, EAST)),
            Cell::BendNW => Some((NORTH, WEST)),
            Cell::BendSE => Some((SOUTH, EAST)),
            Cell::BendSW => Some((SOUTH, WEST)),
            Cell::Start | Cell::Empty => None,
        }
    }
    fn connects_any(&self, from: Direction) -> Option<Direction> {
        let Some((connection1, connection2)) = self.get_directions() else {
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
impl TryFrom<(Direction, Direction)> for Cell {
    type Error = Direction;
    fn try_from(directions: (Direction, Direction)) -> Result<Self, Self::Error> {
        match directions {
            (Direction::V(v1), Direction::V(v2)) if v1 != v2 => Ok(Cell::PipeVertical),
            (Direction::H(h1), Direction::H(h2)) if h1 != h2 => Ok(Cell::PipeHorizontal),
            (Direction::V(v), Direction::H(h)) | (Direction::H(h), Direction::V(v)) => {
                Ok(Cell::from((v, h)))
            }
            (inner_equal @ Direction::V(_), Direction::V(_))
            | (inner_equal @ Direction::H(_), Direction::H(_)) => Err(inner_equal),
        }
    }
}
impl From<(DirectionV, DirectionH)> for Cell {
    fn from(value: (DirectionV, DirectionH)) -> Self {
        match value {
            (DirectionV::North, DirectionH::East) => Self::BendNE,
            (DirectionV::North, DirectionH::West) => Self::BendNW,
            (DirectionV::South, DirectionH::East) => Self::BendSE,
            (DirectionV::South, DirectionH::West) => Self::BendSW,
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
impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Cell::Empty => '.',
            Cell::PipeVertical => '|',
            Cell::PipeHorizontal => '-',
            Cell::BendNE => 'L',
            Cell::BendNW => 'J',
            Cell::BendSE => '7',
            Cell::BendSW => 'F',
            Cell::Start => 'S',
        };
        write!(f, "{code}")
    }
}

fn fmt_grid<T>(
    f: &mut std::fmt::Formatter<'_>,
    width: usize,
    get_cell: impl Fn(Point) -> Option<T>,
    fmt_cell: impl Fn(&mut std::fmt::Formatter<'_>, T) -> std::fmt::Result,
) -> std::fmt::Result {
    // column header
    write!(f, "  ")?;
    for col in 0..width {
        write!(f, "{}", col % 10)?;
    }
    writeln!(f)?;

    for row in 0.. {
        for col in 0..width {
            let point = Point { row, col };
            let Some(cell) = get_cell(point) else {
                return Ok(());
            };
            if col == 0 {
                // row header
                write!(f, "{} ", row % 10)?;
            }
            fmt_cell(f, cell)?;
        }
        writeln!(f)?;
    }
    unreachable!()
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.width;
        let get_cell = |point| self.get_cell(point);
        let fmt_cell = |f: &mut std::fmt::Formatter<'_>, cell| write!(f, "{cell}");
        fmt_grid(f, width, get_cell, fmt_cell)
    }
}
impl std::fmt::Display for GridUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.grid.width;
        let get_cell = |point| self.get_cell_used(point);
        let fmt_cell = |f: &mut std::fmt::Formatter, (cell, used)| {
            if let Some(Used) = used {
                write!(f, "{cell}")
            } else {
                write!(f, " ")
            }
        };
        fmt_grid(f, width, get_cell, fmt_cell)
    }
}

#[cfg(test)]
mod tests {
    use crate::{find_area_enclosed, find_longest_step, Grid};

    #[test]
    fn sample_input() {
        let input = ".....
.S-7.
.|.|.
.L-J.
.....";

        let grid = Grid::try_new(input).unwrap();
        dbg!(&grid);

        let (_, longest) = find_longest_step(grid).unwrap();
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

        let (_, longest) = find_longest_step(grid).unwrap();
        assert_eq!(longest, 8);
    }

    #[test]
    fn sample_enclosed() {
        let input = "...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........";
        let grid = Grid::try_new(input).unwrap();

        let (grid_usage, _longest) = find_longest_step(grid).unwrap();
        let area = find_area_enclosed(&grid_usage).unwrap();
        assert_eq!(area, 4);
    }
    #[test]
    fn sample_enclosed_2() {
        let input = "..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
..........";
        let grid = Grid::try_new(input).unwrap();

        let (grid_usage, _longest) = find_longest_step(grid).unwrap();
        let area = find_area_enclosed(&grid_usage).unwrap();
        assert_eq!(area, 4);
    }

    #[test]
    fn sample_enclosed_larger() {
        let input = ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";
        let grid = Grid::try_new(input).unwrap();

        let (grid_usage, _longest) = find_longest_step(grid).unwrap();
        let area = find_area_enclosed(&grid_usage).unwrap();
        assert_eq!(area, 8);
    }

    #[test]
    fn sample_enclosed_junk_pipe() {
        let input = "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";
        let grid = Grid::try_new(input).unwrap();

        let (grid_usage, _longest) = find_longest_step(grid).unwrap();
        let area = find_area_enclosed(&grid_usage).unwrap();
        assert_eq!(area, 10);
    }
}
