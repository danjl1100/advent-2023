use anyhow::Context;
use map::Map;
use std::collections::HashMap;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    println!("hello farmer");

    let input = advent_2023::get_input_string()?;

    let closest_location = get_closest_location(&input)?;
    println!("closest location: {closest_location}");

    Ok(())
}
fn get_closest_location(input: &str) -> anyhow::Result<u64> {
    const START_UNIT_SEED: &str = "seed";
    const END_UNIT_LOCATION: &str = "location";

    let mut input_lines = input.lines();

    let seeds_line = input_lines.next().expect("seeds line");
    let blank_line = input_lines.next().expect("at least one map");
    assert!(blank_line.is_empty());

    let seeds_entries =
        parse_seeds(seeds_line).with_context(|| format!("seeds line {seeds_line:?}"))?;
    let maps = parse_maps(input_lines)?;
    println!("Loaded maps. Seed entries {seeds_entries:?}");

    let seed_ranges: Vec<_> = seeds_entries
        .chunks(2)
        .map(|entries| {
            let [start, len] = entries.try_into().expect("chunks of 2");
            start..(start + len)
        })
        .collect();

    assert!(seed_ranges.iter().any(|r| !r.is_empty()));

    println!("There are {} ranges defining seeds.", seed_ranges.len());

    let combined_map = MapUnit::from_map_set(maps.clone(), (START_UNIT_SEED, END_UNIT_LOCATION));
    println!("simplified the map: {combined_map:#?}");

    let reversed_map = combined_map.map.clone().reverse();
    println!("reversed the map: {reversed_map:#?}");

    let mut all_locations = reversed_map
        .ranges()
        .iter()
        .flat_map(|range| range.sources.clone());
    let closest_location = loop {
        let location = all_locations
            .next()
            .expect("none of the locations in the reversed map route back to the input seeds");
        if let Ok(source_seed) = reversed_map.lookup_value(location) {
            if seed_ranges
                .iter()
                .any(|seed_range| seed_range.contains(&source_seed))
            {
                break location;
            }
        }
    };

    // let closest_location = seeds
    //     .iter()
    //     .copied()
    //     .map(|seed| {
    //         let mut current_value = seed;
    //         let mut current_type = START_UNIT_SEED;
    //         while current_type != END_UNIT_LOCATION {
    //             let Some(map) = maps.get(current_type) else {
    //                 panic!("map not found for {current_type:?}");
    //             };
    //             let (new_type, new_value) = map.lookup_value(current_value).expect("lookup failed");
    //             current_value = new_value;
    //             current_type = new_type;
    //         }
    //         current_value
    //     })
    //     .min()
    //     .expect("at least one seed provided");

    // let closest_location = locations
    //     .iter()
    //     .copied()
    //     .min()
    //     .expect("at least one seed provided");

    Ok(closest_location)
}

fn parse_seeds(line: &str) -> anyhow::Result<Vec<u64>> {
    const SEEDS_COLON: &str = "seeds: ";
    let Some((empty, numbers_str)) = line.split_once(SEEDS_COLON) else {
        anyhow::bail!("not found: {SEEDS_COLON:?}");
    };
    anyhow::ensure!(empty.is_empty(), "extra text before seeds label");
    Ok(numbers_str
        .split_whitespace()
        .map(u64::from_str)
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_maps(mut input_lines: std::str::Lines<'_>) -> anyhow::Result<HashMap<String, MapUnit>> {
    let mut maps = HashMap::new();
    while let Some(title_line) = input_lines.next() {
        let (title_from, title_to) =
            parse_title_line(title_line).with_context(|| format!("title line {title_line:?}"))?;

        let mut ranges = vec![];
        for line in input_lines.by_ref() {
            if line.is_empty() {
                break;
            }
            let range = parse_range(line).with_context(|| format!("range line {line:?}"))?;
            ranges.push(range);
        }
        let map = MapUnit {
            map: Map::new(ranges),
            output_kind: title_to,
        };

        maps.insert(title_from, map);
    }
    Ok(maps)
}
fn parse_title_line(title_line: &str) -> anyhow::Result<(String, String)> {
    const MAP_COLON: &str = "map:";
    let Some((title_map_str, map_colon)) = title_line.split_once(' ') else {
        anyhow::bail!("expected map declaration to have space");
    };
    if map_colon != MAP_COLON {
        anyhow::bail!("expected map declaration to end with {MAP_COLON:?}, found {map_colon:?}");
    }
    let mut title_parts = title_map_str.split('-');
    let Some(a) = title_parts.next() else {
        anyhow::bail!("missing map part A");
    };
    match title_parts.next() {
        Some("to") => {}
        Some(unexpected) => anyhow::bail!("unexpected token in map title: {unexpected:?}"),
        None => anyhow::bail!("incomplete map title"),
    }
    let Some(b) = title_parts.next() else {
        anyhow::bail!("missing map part B");
    };
    Ok((a.to_string(), b.to_string()))
}

fn parse_range(line: &str) -> anyhow::Result<Range> {
    let numbers = line
        .split_whitespace()
        .map(u64::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    anyhow::ensure!(numbers.len() == 3, "expected 3 numbers to define range");
    let dest_start = numbers[0];
    let source_start = numbers[1];
    let len = numbers[2];

    let offset = {
        let Ok(dest_start) = i64::try_from(dest_start) else {
            anyhow::bail!("destination start {dest_start} too large");
        };
        let Ok(source_start) = i64::try_from(source_start) else {
            anyhow::bail!("source start {source_start} too large");
        };

        dest_start - source_start
    };

    let sources = source_start..(source_start + len);

    Ok(Range { sources, offset })
}

#[derive(Clone, Debug)]
pub struct MapUnit {
    map: Map,
    output_kind: String,
}
#[allow(unused)] // for tests
impl MapUnit {
    fn new(ranges: Vec<Range>, output_kind: String) -> Self {
        Self {
            map: Map::new(ranges),
            output_kind,
        }
    }
    fn into_parts(self) -> (Vec<Range>, String) {
        let Self { map, output_kind } = self;
        let ranges = map.into_inner();
        (ranges, output_kind)
    }
}

mod map {
    //! Privacy boundary to ensure `Map::ranges()` is always sorted
    use crate::Range;

    #[derive(Clone, Debug)]
    pub struct Map {
        ranges: Vec<Range>,
    }
    impl Map {
        pub fn new(mut ranges: Vec<Range>) -> Self {
            ranges.sort_by_key(|range| range.sources.start);
            Map { ranges }
        }
        pub fn ranges(&self) -> &[Range] {
            &self.ranges
        }
        pub fn into_inner(self) -> Vec<Range> {
            let Map { ranges } = self;
            ranges
        }
    }
}
impl Map {
    pub fn lookup_value(&self, value: u64) -> anyhow::Result<u64> {
        // TODO use binary search instead
        for range in self.ranges() {
            if range.sources.contains(&value) {
                let Ok(value) = i64::try_from(value) else {
                    anyhow::bail!("value {value} exceeds i64")
                };
                let with_offset = value + range.offset;
                let Ok(with_offset) = u64::try_from(with_offset) else {
                    anyhow::bail!("offset value {with_offset} exceeds u64")
                };
                return Ok(with_offset);
            }
        }
        anyhow::bail!("no matching range for value {value}")
    }

    pub(crate) fn reverse(self) -> Self {
        let ranges = self.into_inner();
        let new_ranges = ranges
            .into_iter()
            .map(|range| {
                let Range { sources, offset } = range;
                let new_start = apply_offset(sources.start, offset).expect("offset out of bounds");
                let new_end = apply_offset(sources.end, offset).expect("offset out of bounds");
                Range {
                    sources: new_start..new_end,
                    offset: -offset,
                }
            })
            .collect();
        Self::new(new_ranges)
    }
}

fn apply_offset(bound: u64, offset: i64) -> Result<u64, std::num::TryFromIntError> {
    u64::try_from(i64::try_from(bound).expect("range input outside i64 bounds") + offset)
}

impl std::ops::Add for MapUnit {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let MapUnit {
            map: lhs_map,
            output_kind: _,
        } = self;
        let lhs_ranges = lhs_map.into_inner();

        let MapUnit {
            map: rhs_map,
            output_kind: rhs_output_kind,
        } = rhs;
        let rhs_ranges = rhs_map.into_inner();

        let mut ranges = vec![];
        for lhs_range in lhs_ranges {
            let lhs_range_start = lhs_range.sources.start;
            let lhs_range_end = lhs_range.sources.end;
            let lhs_offset = lhs_range.offset;

            let lhs_range_start_translated =
                apply_offset(lhs_range_start, lhs_offset).expect("offset range out of bounds");

            let rhs_index = rhs_ranges
                .binary_search_by_key(&lhs_range_start_translated, |r| r.sources.start)
                .unwrap_or_else(|insert_key| insert_key.saturating_sub(1));
            let mut prev_range_end = lhs_range_start;
            for rhs_range in rhs_ranges.iter().skip(rhs_index) {
                let rhs_range_end = apply_offset(rhs_range.sources.end, -lhs_offset)
                    .expect("offset range out of bounds");

                if prev_range_end >= lhs_range_end {
                    // accumulated ranges has filled the entire "lhs_range" input as desired
                    break;
                }
                if prev_range_end >= rhs_range_end {
                    // this "rhs_range" is too short (empty?) for the next input range
                    continue;
                }
                let new_range_end = lhs_range_end.min(rhs_range_end);

                let new_range = prev_range_end..new_range_end;
                let new_range = Range {
                    sources: new_range,
                    offset: rhs_range.offset + lhs_range.offset,
                };

                if !new_range.sources.is_empty() {
                    ranges.push(new_range);
                }

                prev_range_end = new_range_end;
            }
        }
        Self {
            map: Map::new(ranges),
            output_kind: rhs_output_kind,
        }
    }
}

impl MapUnit {
    fn from_map_set(
        mut maps: HashMap<String, MapUnit>,
        (start_unit, end_unit): (&'static str, &'static str),
    ) -> Self {
        let start_map = maps.remove(start_unit).expect("start unit present in map");
        let mut result_map = start_map;
        while result_map.output_kind != end_unit {
            let next_map = maps
                .remove(&result_map.output_kind)
                .expect("break in output kind linkage");
            result_map = result_map + next_map;
        }
        result_map
    }
}

// impl std::ops::Add for Map {
//     type Output = Map;
//     fn add(self, rhs: Self) -> Self::Output {
//         let ranges = self.ranges().into_iter().flat_map(|range| {
//             let range_sources = range.sources;
//
//             let rhs_ranges = rhs.ranges();
//             let match_index =
//                 rhs_ranges.binary_search_by_key(&range.sources.start, |r| r.sources.start);
//             let start_index = match_index.unwrap_or_else(|insert_index| {
//                 insert_index
//                     .checked_sub(1)
//                     .expect("rhs map ranges too high for for lhs range")
//             });
//             let mut rhs_matching_ranges = vec![];
//             loop {
//                 let rhs_range = rhs_ranges[start_index].sources;
//                 if !rhs_range.contains(&range_sources.start)
//                     && !rhs_range.contains(&range_sources.end)
//                 {
//                     break;
//                 }
//                 rhs_matching_ranges.push(rhs_range);
//             }
//             (range, rhs_ranges)
//         });
//         todo!()
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq)]
struct Range {
    sources: std::ops::Range<u64>,
    offset: i64,
}
// impl std::ops::Add for Range {
//     type Output = Vec<Range>;
//
//     fn add(self, rhs: Self) -> Self::Output {
//         // NOTE: Range has two serial steps:
//         //  1. Match input to `sources`
//         //  2. Apply the offset
//         //
//         // Two `Range`s applied serially execute steps 1.2. then 1.2.
//         //
//         // Combining two `Range`s then involves:
//         //  A. move rhs.sources to the left,
//         //  B. move self.offset to the right
//         let Self {
//             sources: self_sources,
//             offset: self_offset,
//         } = self;
//         let Self {
//             sources: other_sources,
//             offset: other_offset,
//         } = rhs;
//         assert!(!self_sources.is_empty());
//         assert!(!other_sources.is_empty());
//         // Move rhs.sources to the left
//         let other_sources = {
//             let len = other_sources.end - other_sources.start;
//             let old_start = i64::try_from(other_sources.start).unwrap();
//             let new_start = old_start - self_offset;
//             let new_start = u64::try_from(new_start).unwrap();
//             new_start..(new_start + len)
//         };
//         // Move self.offset to the right, depending on range intersections
//         let IntersectedRanges {
//             both,
//             a_only: self_only,
//             b_only: other_only,
//         } = intersect_ranges(self_sources, other_sources);
//
//         let self_only = self_only.into_iter().map(|sources| Self {
//             sources,
//             offset: self_offset,
//         });
//         let both = both.map(|sources| Self {
//             sources,
//             offset: self_offset + other_offset,
//         });
//         let other_only = other_only.into_iter().map(|sources| Self {
//             sources,
//             offset: other_offset,
//         });
//         let output_ranges: Vec<_> = self_only
//             .into_iter()
//             .chain(both)
//             .chain(other_only)
//             .collect();
//         assert!(!output_ranges.is_empty(), "at least one resulting range");
//         output_ranges
//     }
// }

// use crate::arithmetic::{intersect_ranges, IntersectedRanges};
// mod arithmetic {
//     type StdRange = std::ops::Range<u64>;
//     #[derive(Clone, Debug, PartialEq, Eq)]
//     pub struct IntersectedRanges {
//         pub both: Option<StdRange>,
//         pub a_only: Vec<StdRange>,
//         pub b_only: Vec<StdRange>,
//     }
//     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//     enum Either {
//         A,
//         B,
//     }
//     impl Either {
//         fn choose<T>(self, a: T, b: T) -> T {
//             match self {
//                 Either::A => a,
//                 Either::B => b,
//             }
//         }
//     }
//     /// Returns the intersection of ranges: (A only, Both, B only)
//     pub fn intersect_ranges(a: StdRange, b: StdRange) -> IntersectedRanges {
//         assert!(!a.is_empty());
//         assert!(!b.is_empty());
//         // Ignoring outside the range, there are 3 possibilities:
//         //
//         // 1.    |---AB---------|  Intersect completely (1 region)
//         // 2.    |--A--|  |--B--|  Disjoint (2 regions)
//         // 3.i   |--A-|-AB-|-A--|  Contained (3 regions)
//         // 3.ii  |--B-|-AB-|-B--|  Contained (3 regions)
//         // 3.iii |--A-|-AB-|-B--|  Intersect partially (3 regions)
//         // 3.iv  |--B-|-AB-|-A--|  Intersect partially (3 regions)
//         //
//         let (mut a_only, both, mut b_only) = if a.start == b.start && a.end == b.end {
//             // 1. Intersect completely (1 region)
//             let equal = a;
//             (vec![], Some(equal), vec![])
//         } else {
//             let a_start_in_b = b.contains(&a.start);
//             let a_end_in_b = b.contains(&a.end);
//             let b_start_in_a = a.contains(&b.start);
//             let b_end_in_a = a.contains(&b.end);
//             if !a_start_in_b && !a_end_in_b && !b_start_in_a && !b_end_in_a {
//                 // 2. Disjoint (2 regions)
//                 (vec![a], None, vec![b])
//             } else {
//                 // 3. Contained or intersect partially (3 regions)
//
//                 // TODO identify the middle 2 of the 4 endpoints
//                 let endpoints_sorted = {
//                     let mut endpoints = [
//                         (Either::A, a.start),
//                         (Either::A, a.end),
//                         (Either::B, b.start),
//                         (Either::B, b.end),
//                     ];
//                     endpoints.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));
//                     endpoints
//                 };
//                 let both = {
//                     let [_, (_, both_start), (_, both_end), _] = endpoints_sorted;
//                     both_start..both_end
//                 };
//
//                 let (a_ranges, b_ranges) = {
//                     let (left_ty, left) = {
//                         let [(ty, start), (_, end), _, _] = endpoints_sorted;
//                         (ty, start..end)
//                     };
//                     let (right_ty, right) = {
//                         let [_, _, (_, start), (ty, end)] = endpoints_sorted;
//                         (ty, start..end)
//                     };
//                     let mut a_ranges = vec![];
//                     let mut b_ranges = vec![];
//
//                     left_ty.choose(&mut a_ranges, &mut b_ranges).push(left);
//                     right_ty.choose(&mut a_ranges, &mut b_ranges).push(right);
//                     (a_ranges, b_ranges)
//                 };
//
//                 (a_ranges, Some(both), b_ranges)
//             }
//         };
//         let both = both.and_then(|range| (!range.is_empty()).then_some(range));
//         a_only.retain(|range| !range.is_empty());
//         b_only.retain(|range| !range.is_empty());
//         IntersectedRanges {
//             both,
//             a_only,
//             b_only,
//         }
//     }
//
//     #[cfg(test)]
//     mod tests {
//         use crate::arithmetic::{intersect_ranges, IntersectedRanges};
//
//         fn test_symmetric(
//             (a, b): (std::ops::Range<u64>, std::ops::Range<u64>),
//             expected: IntersectedRanges,
//         ) {
//             let test_forward = intersect_ranges(a.clone(), b.clone());
//             assert_eq!(test_forward, expected);
//
//             let IntersectedRanges {
//                 both,
//                 a_only,
//                 b_only,
//             } = expected;
//             let expected_reverse = IntersectedRanges {
//                 both,
//                 a_only: b_only,
//                 b_only: a_only,
//             };
//
//             let test_reverse = intersect_ranges(b, a);
//             assert_eq!(test_reverse, expected_reverse);
//         }
//
//         // 1.    |---AB---------|  Intersect completely (1 region)
//         #[test]
//         fn identical() {
//             test_symmetric(
//                 (2..5, 2..5),
//                 IntersectedRanges {
//                     both: Some(2..5),
//                     a_only: vec![],
//                     b_only: vec![],
//                 },
//             );
//         }
//         // 2.    |--A--|  |--B--|  Disjoint (2 regions)
//         #[test]
//         fn disjoint() {
//             test_symmetric(
//                 (2..5, 7..9),
//                 IntersectedRanges {
//                     both: None,
//                     a_only: vec![2..5],
//                     b_only: vec![7..9],
//                 },
//             );
//         }
//         // 3.i   |--A-|-AB-|-A--|  Contained (3 regions)
//         #[test]
//         fn contained() {
//             test_symmetric(
//                 (19..93, 25..30),
//                 IntersectedRanges {
//                     both: Some(25..30),
//                     a_only: vec![19..25, 30..93],
//                     b_only: vec![],
//                 },
//             );
//         }
//         #[test]
//         fn contained_aa_end_empty() {
//             test_symmetric(
//                 (19..30, 25..30),
//                 IntersectedRanges {
//                     both: Some(25..30),
//                     a_only: vec![19..25],
//                     b_only: vec![],
//                 },
//             );
//         }
//         #[test]
//         fn contained_aa_start_empty() {
//             test_symmetric(
//                 (25..93, 25..30),
//                 IntersectedRanges {
//                     both: Some(25..30),
//                     a_only: vec![30..93],
//                     b_only: vec![],
//                 },
//             );
//         }
//         // 3.iii |--A-|-AB-|-B--|  Intersect partially (3 regions)
//         #[test]
//         fn intersect_ab() {
//             test_symmetric(
//                 (5..30, 10..90),
//                 IntersectedRanges {
//                     both: Some(10..30),
//                     a_only: vec![5..10],
//                     b_only: vec![30..90],
//                 },
//             )
//         }
//         #[test]
//         fn intersect_ab_end_empty() {
//             test_symmetric(
//                 (5..30, 10..30),
//                 IntersectedRanges {
//                     both: Some(10..30),
//                     a_only: vec![5..10],
//                     b_only: vec![],
//                 },
//             )
//         }
//         #[test]
//         fn intersect_ab_start_empty() {
//             test_symmetric(
//                 (10..30, 10..90),
//                 IntersectedRanges {
//                     both: Some(10..30),
//                     a_only: vec![],
//                     b_only: vec![30..90],
//                 },
//             )
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{get_closest_location, MapUnit, Range};

    // NOTE: format is [DEST] [SOURCE] [LEN]
    const SAMPLE_INPUT_STR: &str = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    #[test]
    fn sample_input() {
        let input = SAMPLE_INPUT_STR;
        let closest = get_closest_location(input).unwrap();
        assert_eq!(closest, 46);
    }

    #[test]
    fn adding_maps_one_to_one() {
        let map1 = MapUnit::new(
            vec![
                Range {
                    sources: 5..10,
                    offset: 2,
                },
                Range {
                    sources: 10..20,
                    offset: 50,
                },
                Range {
                    sources: 30..40,
                    offset: -10,
                },
            ],
            "dummy".to_string(),
        );
        let map2 = MapUnit::new(
            vec![
                Range {
                    sources: 7..12,
                    offset: 10,
                },
                Range {
                    sources: 60..70,
                    offset: 100,
                },
                Range {
                    sources: 20..30,
                    offset: 9000,
                },
            ],
            "final output kind".to_string(),
        );
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(
            ranges,
            vec![
                Range {
                    sources: 5..10,
                    offset: 12,
                },
                Range {
                    sources: 10..20,
                    offset: 150,
                },
                Range {
                    sources: 30..40,
                    offset: 8990,
                },
            ]
        );
        assert_eq!(&output_kind, "final output kind");
    }

    #[test]
    fn adding_maps_splits() {
        let map1 = MapUnit::new(
            vec![Range {
                sources: 10..30,
                offset: 20,
            }],
            "dummy".to_string(),
        );
        let map2 = MapUnit::new(
            vec![
                Range {
                    sources: 5..40, // NOTE: maps to -15..20
                    offset: 2,
                },
                Range {
                    sources: 45..70, // NOTE: maps to 25..50
                    offset: 100,
                },
            ],
            "final output kind".to_string(),
        );
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(
            ranges,
            vec![
                Range {
                    sources: 10..20,
                    offset: 22,
                },
                Range {
                    sources: 20..30,
                    offset: 120,
                },
            ]
        );
        assert_eq!(&output_kind, "final output kind");
    }

    #[test]
    fn adding_maps_joins() {
        let map1 = MapUnit::new(
            vec![
                Range {
                    sources: 10..30,
                    offset: 5,
                },
                Range {
                    sources: 30..40,
                    offset: 100,
                },
            ],
            "dummy".to_string(),
        );
        let map2 = MapUnit::new(
            vec![
                Range {
                    sources: 0..135, // NOTE: maps to 5..95 and also 130..135 and onward
                    offset: 50,
                },
                Range {
                    sources: 135..200, // NOTE: maps to 135..140 and onward
                    offset: 20,
                },
            ],
            "final output kind".to_string(),
        );
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(
            ranges,
            vec![
                Range {
                    sources: 10..30,
                    offset: 55,
                },
                Range {
                    sources: 30..35,
                    offset: 150,
                },
                Range {
                    sources: 35..40,
                    offset: 120,
                },
            ]
        );
        assert_eq!(&output_kind, "final output kind");
    }

    #[test]
    fn dead_end() {
        let map1 = MapUnit::new(
            vec![Range {
                sources: 10..50,
                offset: 10,
            }],
            "dummy".to_string(),
        );
        let map2 = MapUnit::new(vec![], "end".to_string());
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(output_kind, "end");
        assert_eq!(ranges, vec![]);
    }
}
