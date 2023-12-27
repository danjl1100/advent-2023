use std::collections::{BTreeMap, BTreeSet};

fn main() -> anyhow::Result<()> {
    println!("hello, cosmic...?");

    let input = advent_2023::get_input_string()?;

    let original = Galaxies::new(&input)?;
    let expanded = original.expand();

    let distances_sum = expanded.get_distances_sum();
    println!("Sum of distances in expanded: {distances_sum}");

    Ok(())
}

struct Galaxies {
    points: BTreeSet<Point>,
    // ONLY for formatting
    max_col: usize,
}
impl Galaxies {
    fn new(input: &str) -> anyhow::Result<Self> {
        let mut points = BTreeSet::new();
        for (row, line) in input.lines().enumerate() {
            for (col, value) in line.chars().enumerate() {
                match value {
                    '.' => {}
                    '#' => {
                        let point = Point { row, col };
                        points.insert(point);
                    }
                    extra => {
                        anyhow::bail!("unrecognized symbol {extra:?} at row {row}, col {col}")
                    }
                }
            }
        }
        if points.is_empty() {
            anyhow::bail!("no galaxies in input")
        }

        let max_col = points.iter().map(|p| p.col).max().expect("nonempty");

        let this = Self { points, max_col };
        println!("Original galaxies:\n{this}");
        Ok(this)
    }
    fn max_row(&self) -> usize {
        self.points.last().expect("nonempty").row
    }
    fn expand(self) -> Self {
        let mut this = self;
        for &dimension in Dimension::ALL {
            this = this.expand_dimension(dimension);
        }
        println!("Expanded galaxies:\n{this}");
        this
    }
    fn expand_dimension(self, dimension: Dimension) -> Self {
        println!("------ EXPAND DIMENSION: {dimension:?} ------");
        let Self {
            mut points,
            max_col,
        } = self;

        let mut points_by_dim: BTreeMap<usize, Vec<Point>> = BTreeMap::new();
        for point in points.iter().copied() {
            let points_list = points_by_dim.entry(dimension.of(point)).or_default();
            points_list.push(point);
        }

        let amount_to_add: BTreeMap<usize, usize> = {
            println!("Keys: {:?}", points_by_dim.keys().collect::<Vec<_>>());
            let distances = std::iter::once(None)
                .chain(points_by_dim.keys().copied().map(Some))
                .collect::<Vec<_>>();
            println!("Distances: {distances:?}");
            let diffs = distances
                .windows(2)
                // .zip(points_by_dim.keys().copied())
                // .map(|(values, key)| {
                .map(|values| {
                    let [prev, next] = values.try_into().expect("windows of 2");
                    let next = next.expect("later elements are all Some");
                    if let Some(prev) = prev {
                        next - prev - 1
                    } else {
                        next
                    }
                })
                .collect::<Vec<_>>();
            println!("Diffs: {diffs:?}");
            let cumulative = (0..diffs.len())
                .map(|index| {
                    // sum all items *before and including* current
                    diffs.iter().take(index + 1).sum()
                })
                .collect::<Vec<_>>();
            println!("cumulative: {cumulative:?}");
            let cumulative_map = points_by_dim.keys().copied().zip(cumulative).collect();
            cumulative_map
        };
        println!("{dimension:?}: amounts to add {amount_to_add:#?}");

        let max_col = if dimension == Dimension::Col {
            let (_, &last_added) = amount_to_add.last_key_value().expect("nonempty");
            max_col + last_added
        } else {
            max_col
        };

        // expand columns
        points = points_by_dim
            .into_iter()
            .flat_map(|(key, points_list)| {
                let offset = amount_to_add
                    .get(&key)
                    .copied()
                    .expect("amount_to_add map covers all");
                points_list
                    .into_iter()
                    .map(move |point| point + dimension.new_point(offset))
            })
            .collect();

        Self { points, max_col }
    }

    fn get_distances_sum(&self) -> usize {
        self.points
            .iter()
            .copied()
            .enumerate()
            .map(|(index_outer, point_a)| {
                // sum pair-wise (start iteration *after* point_a)
                self.points
                    .iter()
                    .copied()
                    .skip(index_outer + 1)
                    .map(|point_b| point_a.get_distance(point_b))
                    .sum::<usize>()
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dimension {
    Row,
    Col,
}
impl Dimension {
    const ALL: &'static [Self] = &[Self::Row, Self::Col];
    fn of(self, target: Point) -> usize {
        match self {
            Dimension::Row => target.row,
            Dimension::Col => target.col,
        }
    }
    fn new_point(self, amount: usize) -> Point {
        match self {
            Dimension::Row => Point {
                row: amount,
                col: 0,
            },
            Dimension::Col => Point {
                row: 0,
                col: amount,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    row: usize,
    col: usize,
}
impl Point {
    /// <https://en.wikipedia.org/wiki/Taxicab_geometry>
    fn get_distance(self, other: Self) -> usize {
        let Self { row, col } = self;
        let Self {
            row: other_row,
            col: other_col,
        } = other;
        let row_delta = {
            let max = row.max(other_row);
            let min = row.min(other_row);
            max - min
        };
        let col_delta = {
            let max = col.max(other_col);
            let min = col.min(other_col);
            max - min
        };
        row_delta + col_delta
    }
}
impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

impl std::fmt::Display for Galaxies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { points, max_col } = self;
        let max_row = self.max_row();

        let mut points = points.iter().copied().peekable();
        for row in 0..(max_row + 1) {
            for col in 0..(max_col + 1) {
                let present = if let Some(next) = points.peek() {
                    if row == next.row && col == next.col {
                        let _ = points.next();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                let value = if present { "#" } else { "." };
                write!(f, "{value}")?;
            }
            writeln!(f)?;
            if points.peek().is_none() {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Galaxies, Point};

    #[test]
    fn simple_distance() {
        let p1 = Point { row: 2, col: 7 };
        let p2 = Point { row: 10, col: 1 };
        assert_eq!(p1.get_distance(p2), 6 + 8);

        // identity distance is 0
        assert_eq!(p1.get_distance(p1), 0);
        assert_eq!(p2.get_distance(p2), 0);
    }

    #[test]
    fn sample_input() {
        let input = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";
        let original = Galaxies::new(input).unwrap();
        let expanded = original.expand();

        let distances_sum = expanded.get_distances_sum();
        assert_eq!(distances_sum, 374);
    }
}
