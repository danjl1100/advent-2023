use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
};

use advent_2023::{dimension::Dimension, point::Point};

const FACTOR_MILLION: NonZeroUsize = const_factor(1_000_000);

fn main() -> anyhow::Result<()> {
    println!("hello, cosmic...?");

    let input = advent_2023::get_input_string()?;

    let original = Galaxies::new(&input)?;
    let expanded = original.expand(FACTOR_MILLION);

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
    fn expand(self, factor: NonZeroUsize) -> Self {
        let additive_factor = factor
            .get()
            .checked_sub(1)
            .expect("subtract one from nonzero");

        let mut this = self;
        for &dimension in Dimension::ALL {
            this = this.expand_dimension(dimension, additive_factor);
        }
        println!("Expanded galaxies:\n{this}");
        this
    }
    fn expand_dimension(self, dimension: Dimension, additive_factor: usize) -> Self {
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
                .map(|values| {
                    let [prev, next] = values.try_into().expect("windows of 2");
                    let next = next.expect("later elements are all Some");
                    let gap_count = if let Some(prev) = prev {
                        next - prev - 1
                    } else {
                        next
                    };
                    // NOTE: existing empty column counts as 1
                    gap_count * additive_factor
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

trait TaxicabDistance {
    fn get_distance(self, other: Self) -> usize;
}
impl TaxicabDistance for Point {
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

const fn const_factor(factor: usize) -> NonZeroUsize {
    match NonZeroUsize::new(factor) {
        Some(v) => v,
        None =>
        {
            #[allow(unconditional_panic)]
            [][0]
        }
    }
}

impl std::fmt::Display for Galaxies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            ref points,
            max_col,
        } = *self;
        let max_row = self.max_row();

        if max_col > 1_000 || max_row > 1_000 {
            return writeln!(f, "[Galaxies display disabled for dimensions > 1000]");
        }

        // column header
        write!(f, "  ")?;
        for col in 0..=max_col {
            write!(f, "{}", (col / 10) % 10)?;
        }
        writeln!(f)?;

        let mut points = points.iter().copied().peekable();
        for row in 0..=max_row {
            write!(f, "{} ", (row / 10) % 10)?;
            for col in 0..=max_col {
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
    use std::num::NonZeroUsize;

    use crate::{const_factor, Galaxies, Point, TaxicabDistance};

    const FACTOR_TWO: NonZeroUsize = const_factor(2);
    const FACTOR_TEN: NonZeroUsize = const_factor(10);
    const FACTOR_HUNDRED: NonZeroUsize = const_factor(100);

    #[test]
    fn simple_distance() {
        let p1 = Point { row: 2, col: 7 };
        let p2 = Point { row: 10, col: 1 };
        assert_eq!(p1.get_distance(p2), 6 + 8);

        // identity distance is 0
        assert_eq!(p1.get_distance(p1), 0);
        assert_eq!(p2.get_distance(p2), 0);
    }

    fn test_sample_input(factor: NonZeroUsize, expected: usize) {
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
        let expanded = original.expand(factor);

        let distances_sum = expanded.get_distances_sum();
        assert_eq!(distances_sum, expected, "factor {factor}");
    }

    #[test]
    fn sample_input() {
        test_sample_input(FACTOR_TWO, 374);
    }
    #[test]
    fn sample_input_10() {
        test_sample_input(FACTOR_TEN, 1030);
    }
    #[test]
    fn sample_input_100() {
        test_sample_input(FACTOR_HUNDRED, 8410);
    }
}
