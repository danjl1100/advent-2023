use advent_2023::{
    dimension::Dimension,
    direction::{Direction, DirectionH, DirectionV, EAST, NORTH, SOUTH, WEST},
    point::Point,
};

fn main() -> anyhow::Result<()> {
    println!("hello, mirror beams!");

    let input = advent_2023::get_input_string()?;

    let Stats {
        default_energy_sum,
        highest_energy_sum,
    } = eval_input(&input)?;

    println!("Sum of energy (default entry): {default_energy_sum}");
    println!("Highest energy sum: {highest_energy_sum}");

    Ok(())
}

struct Stats {
    default_energy_sum: usize,
    highest_energy_sum: usize,
}

fn eval_input(input: &str) -> anyhow::Result<Stats> {
    const DEFAULT_ENTRY: (Point, Direction) = (Point { row: 0, col: 0 }, EAST);

    let Some(grid) = Grid::new(input)? else {
        anyhow::bail!("no grid in input")
    };
    println!("{grid:?}");

    let default_energy = EnergizedGrid::new(&grid, DEFAULT_ENTRY);
    println!("{default_energy:?}");
    let default_energy_sum = default_energy.sum();

    let row_first = 0;
    let row_last = grid.max_row;
    let col_first = 0;
    let col_last = grid.width - 1;

    let row_entries = (row_first..=row_last).flat_map(|row| {
        vec![
            (
                Point {
                    row,
                    col: col_first,
                },
                EAST,
            ),
            (
                Point {
                    row,
                    col: col_last, //
                },
                WEST,
            ),
        ]
    });
    let col_entries = (col_first..=col_last).flat_map(|col| {
        vec![
            (
                Point {
                    row: row_first,
                    col,
                },
                SOUTH,
            ),
            (
                Point {
                    row: row_last, //
                    col,
                },
                NORTH,
            ),
        ]
    });

    let highest_energy_sum = row_entries
        .chain(col_entries)
        .map(|entry| EnergizedGrid::new(&grid, entry).sum())
        .max()
        .expect("nonempty entrypoints");

    Ok(Stats {
        default_energy_sum,
        highest_energy_sum,
    })
}

#[derive(Clone)]
struct Grid<T> {
    cells: Vec<T>,
    width: usize,
    max_row: usize,
}
impl Grid<Cell> {
    fn new(input: &str) -> anyhow::Result<Option<Self>> {
        let mut cells = vec![];
        let mut width = None;
        let mut max_row = 0;
        for line in input.lines() {
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
        }))
    }
}
impl<T> Grid<T> {
    fn empty_with<U>(&self, generator_fn: impl Fn() -> U) -> Grid<U> {
        self.map(|_, _| generator_fn())
    }
    fn map<U>(&self, map_fn: impl Fn(Point, &T) -> U) -> Grid<U> {
        let Self {
            ref cells,
            width,
            max_row,
        } = *self;
        let new_cells = cells
            .iter()
            .enumerate()
            .map(|(index, old)| {
                let point = Point::from_index_width(index, width);
                map_fn(point, old)
            })
            .collect();
        Grid {
            cells: new_cells,
            width,
            max_row,
        }
    }
    fn row_range(&self, row: usize) -> std::ops::Range<usize> {
        assert!(row <= self.max_row, "row {row}");
        let start = Point { row, col: 0 }
            .index_for_width(self.width)
            .expect("col 0 within nonzero width");
        let end = Point {
            row: row + 1,
            col: 0,
        }
        .index_for_width(self.width)
        .expect("col 0 within nonzero width");
        start..end
    }
    fn row(&self, row: usize) -> Option<&[T]> {
        self.cells.get(self.row_range(row))
    }
    // fn size(&self, dimension: Dimension) -> usize {
    //     match dimension {
    //         Dimension::Row => self.max_row,
    //         Dimension::Col => self.width,
    //     }
    // }
    fn get(&self, point: Point) -> Option<&T> {
        let index = point.index_for_width(self.width)?;
        self.cells.get(index)
    }
    fn get_mut(&mut self, point: Point) -> Option<&mut T> {
        let index = point.index_for_width(self.width)?;
        self.cells.get_mut(index)
    }
}

// #[derive(Clone)]
// struct ColIter<'a> {
//     cells: &'a [Cell],
//     width: usize,
//     col: usize,
//     next_row: Option<usize>,
// }
// impl Iterator for ColIter<'_> {
//     type Item = Cell;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let row = self.next_row?;
//         let elem = {
//             let point = Point { row, col: self.col };
//             let index = point.index_for(self.width);
//             self.cells.get(index).copied()
//         };
//
//         self.next_row = elem.is_some().then_some(row + 1);
//         elem
//     }
// }

struct EnergizedGrid<'a> {
    cells: &'a Grid<Cell>,
    energized: Grid<Energized>,
    traveled: Grid<DirectionMap<()>>,
}
impl<'a> EnergizedGrid<'a> {
    fn new(cells: &'a Grid<Cell>, (entry_point, entry_direction): (Point, Direction)) -> Self {
        let energized = cells.empty_with(Energized::default);
        let traveled = cells.empty_with(DirectionMap::default);
        let mut this = Self {
            cells,
            energized,
            traveled,
        };
        this.activate(entry_point, entry_direction);
        this
    }
    fn activate_next(&mut self, current: Point, next_direction: Direction) {
        if let Some(dest) = next_direction.of(current) {
            self.activate(dest, next_direction);
        }
    }
    fn activate(&mut self, point: Point, direction: Direction) {
        let Some(((cell, dest_energy), traveled)) = self
            .cells
            .get(point)
            .zip(self.energized.get_mut(point))
            .zip(self.traveled.get_mut(point))
        else {
            // point out of bounds
            return;
        };

        if traveled.insert(direction, ()).is_some() {
            // already calculated travel this direction for this cell
            return;
        }

        // activate current cell
        *dest_energy = Energized::True;

        // travel to next cell(s)
        match cell {
            Cell::Empty => {
                self.activate_next(point, direction);
            }
            Cell::Mirror(mirror) => {
                use Direction::{H, V};
                let next = match (mirror, direction) {
                    (MirrorKind::NwToSe, V(DirectionV::North)) => WEST,
                    (MirrorKind::NwToSe, V(DirectionV::South)) => EAST,
                    (MirrorKind::NwToSe, H(DirectionH::West)) => NORTH,
                    (MirrorKind::NwToSe, H(DirectionH::East)) => SOUTH,
                    (MirrorKind::SwToNe, V(DirectionV::North)) => EAST,
                    (MirrorKind::SwToNe, V(DirectionV::South)) => WEST,
                    (MirrorKind::SwToNe, H(DirectionH::West)) => SOUTH,
                    (MirrorKind::SwToNe, H(DirectionH::East)) => NORTH,
                };
                self.activate_next(point, next);
            }
            Cell::SplitAlongThis(along) => {
                let (next1, next2) = match along {
                    Dimension::Row => (WEST, EAST),
                    Dimension::Col => (NORTH, SOUTH),
                };
                self.activate_next(point, next1);
                self.activate_next(point, next2);
            }
        }
    }
    fn sum(&self) -> usize {
        self.energized
            .cells
            .iter()
            .map(|&elem| match elem {
                Energized::True => 1,
                Energized::False => 0,
            })
            .sum()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Energized {
    True,
    #[default]
    False,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Mirror(MirrorKind),
    SplitAlongThis(Dimension),
}
impl TryFrom<char> for Cell {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            '\\' => Ok(Self::Mirror(MirrorKind::NwToSe)),
            '/' => Ok(Self::Mirror(MirrorKind::SwToNe)),
            '-' => Ok(Self::SplitAlongThis(Dimension::Row)),
            '|' => Ok(Self::SplitAlongThis(Dimension::Col)),
            extra => Err(extra),
        }
    }
}
impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Empty => '.',
            Cell::Mirror(MirrorKind::NwToSe) => '\\',
            Cell::Mirror(MirrorKind::SwToNe) => '/',
            Cell::SplitAlongThis(Dimension::Row) => '-',
            Cell::SplitAlongThis(Dimension::Col) => '|',
        };
        write!(f, "{c}")
    }
}

impl<T> std::fmt::Debug for Grid<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  ")?;
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
impl std::fmt::Debug for Energized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Energized::True => '#',
            Energized::False => '.',
        };
        write!(f, "{c}")
    }
}
impl std::fmt::Debug for EnergizedGrid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            cells,
            energized,
            traveled: _, // TODO
        } = self;
        writeln!(f, "Grid:")?;
        writeln!(f, "{cells:?}")?;
        writeln!(f, "Energized?")?;
        writeln!(f, "{energized:?}")
    }
}

struct DirectionMap<T>([Option<T>; 4]);
impl<T> DirectionMap<T> {
    fn insert(&mut self, direction: Direction, value: T) -> Option<T> {
        let index = Self::index(direction);
        std::mem::replace(&mut self.0[index], Some(value))
    }
    // TODO
    // fn get(&mut self, direction: Direction) -> Option<&T> {
    //     let index = Self::index(direction);
    //     self.0[index].as_ref()
    // }
    fn index(direction: Direction) -> usize {
        use Direction::{H, V};
        match direction {
            V(DirectionV::North) => 0,
            V(DirectionV::South) => 1,
            H(DirectionH::East) => 2,
            H(DirectionH::West) => 3,
        }
    }
}
impl<T> Default for DirectionMap<T> {
    fn default() -> Self {
        Self([None, None, None, None])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MirrorKind {
    /// `\`
    NwToSe,
    /// `/`
    SwToNe,
}

#[cfg(test)]
mod tests {
    use crate::eval_input;

    #[test]
    fn sample_input() {
        let input = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."#;
        let stats = eval_input(input).unwrap();
        assert_eq!(stats.default_energy_sum, 46);
        assert_eq!(stats.highest_energy_sum, 51);
    }
}
