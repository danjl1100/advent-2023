use anyhow::Context;
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
    println!("simplified the map:{combined_map}");

    let reversed_map = combined_map.map.clone().reverse();
    println!("reversed the map:{reversed_map}");

    let mut all_locations = 0..;
    // let mut all_locations = reversed_map
    //     .ranges()
    //     .iter()
    //     .flat_map(|range| range.sources.clone());
    let closest_location = loop {
        let location = all_locations
            .next()
            .expect("no locations route back to the input seeds");
        if let Ok(Some(source_seed)) = reversed_map.lookup_value(location) {
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
    let mut map_units = HashMap::new();
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

        let map_unit = MapUnit {
            map: Map::new(ranges),
            output_kind: title_to,
        };

        map_units.insert(title_from, map_unit);
    }
    Ok(map_units)
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
    let [dest_start, source_start, len] = numbers.try_into().expect("length 3");

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

type Range = RangeGeneric<i64>;
type ReverseRange = RangeGeneric<Option<i64>>;

type Map = map::MapGeneric<i64>;
type ReverseMap = map::MapGeneric<Option<i64>>;
mod map {
    //! Privacy boundary to ensure `Map::ranges()` is always sorted
    use crate::RangeGeneric;

    #[derive(Clone, Debug)]
    pub struct MapGeneric<T>
    where
        T: std::fmt::Debug,
    {
        // NOTE: empty means the identity map
        ranges: Vec<RangeGeneric<T>>,
    }
    impl<T> MapGeneric<T>
    where
        T: std::fmt::Debug,
    {
        pub fn new(mut ranges: Vec<RangeGeneric<T>>) -> Self {
            ranges.sort_by_key(|range| range.sources.start);

            print!("Map::new(");
            for range in &ranges {
                print!("{:?}, ", range.sources);
            }
            println!(")");

            for range_window in ranges.windows(2) {
                let [prev, next]: &[RangeGeneric<T>; 2] =
                    range_window.try_into().expect("windows of 2");

                // well-defined ranges
                assert!(prev.sources.start <= prev.sources.end);
                assert!(next.sources.start <= next.sources.end);

                // no overlap
                assert!(prev.sources.end <= next.sources.start);
            }
            MapGeneric { ranges }
        }
        pub fn ranges(&self) -> &[RangeGeneric<T>] {
            &self.ranges
        }
        pub fn into_inner(self) -> Vec<RangeGeneric<T>> {
            let MapGeneric { ranges } = self;
            ranges
        }
    }
}
impl<T> map::MapGeneric<T>
where
    T: std::fmt::Debug,
{
    pub fn iter(&self) -> impl Iterator<Item = &RangeGeneric<T>> {
        self.ranges().iter()
    }
    pub fn find_start_index(&self, start: u64) -> usize {
        self.ranges()
            .binary_search_by_key(&start, |r| r.sources.start)
            .unwrap_or_else(|insert_key| insert_key.saturating_sub(1))
    }
}
impl ReverseMap {
    pub fn lookup_value(&self, value: u64) -> anyhow::Result<Option<u64>> {
        let index = self.find_start_index(value);
        let range = &self.ranges()[index];
        if range.sources.contains(&value) {
            let Ok(value) = i64::try_from(value) else {
                anyhow::bail!("value {value} exceeds i64")
            };
            if let Some(offset) = range.offset {
                let with_offset = value + offset;
                let Ok(with_offset) = u64::try_from(with_offset) else {
                    anyhow::bail!("offset value {with_offset} exceeds u64")
                };
                Ok(Some(with_offset))
            } else {
                Ok(None)
            }
        } else {
            // non-mapped values are understood as Identity
            Ok(Some(value))
        }
    }
}
impl Map {
    #[allow(unused)]
    pub fn lookup_value(&self, value: u64) -> anyhow::Result<u64> {
        let index = self.find_start_index(value);
        let range = &self.ranges()[index];
        if range.sources.contains(&value) {
            let Ok(value) = i64::try_from(value) else {
                anyhow::bail!("value {value} exceeds i64")
            };
            let with_offset = value + range.offset;
            let Ok(with_offset) = u64::try_from(with_offset) else {
                anyhow::bail!("offset value {with_offset} exceeds u64")
            };
            Ok(with_offset)
        } else {
            // non-mapped values are understood as Identity
            Ok(value)
        }
    }

    pub(crate) fn reverse(self) -> ReverseMap {
        let ranges = self.into_inner();

        let mut reverse_range_candidates = vec![];
        let mut new_ranges: Vec<_> = ranges
            .into_iter()
            .map(|range| {
                // NOTE: If forward maps A..B -> C..D,
                // then the reverse maps C..D -> A..B, as well as the remainder going to null
                let Range {
                    sources: sources_input,
                    offset,
                } = range;

                let sources_output = {
                    let new_start =
                        apply_offset(sources_input.start, offset).expect("offset out of bounds");
                    let new_end =
                        apply_offset(sources_input.end, offset).expect("offset out of bounds");
                    new_start..new_end
                };

                let reverse = ReverseRange {
                    sources: sources_output.clone(),
                    offset: Some(-offset),
                };
                let reverse_null = {
                    let IntersectedRanges {
                        both: _,
                        a_only: input_only,
                        b_only: _,
                    } = intersect_ranges(sources_input, sources_output.clone());
                    let [input_only] = input_only
                        .try_into()
                        .expect("input to output lengths identical (no double a_only)");
                    ReverseRange {
                        sources: input_only,
                        offset: None,
                    }
                };
                reverse_range_candidates.push(reverse_null);
                // std::iter::once(reverse).chain(std::iter::once(reverse_null))
                reverse
            })
            .collect();

        let start_key = |range: &ReverseRange| range.sources.start;
        new_ranges.sort_by_key(start_key);

        let mut candidates_changed = true;
        while candidates_changed {
            candidates_changed = false;
            reverse_range_candidates = reverse_range_candidates
                .into_iter()
                .flat_map(|candidate| {
                    let wrap_in_range = move |sources| RangeGeneric {
                        sources,
                        offset: candidate.offset,
                    };
                    for new_range in new_ranges.iter().map(|r| &r.sources) {
                        let IntersectedRanges {
                            both: _,
                            a_only: shortened,
                            b_only: _,
                        } = intersect_ranges(candidate.sources.clone(), new_range.clone());
                        let shortened: Result<[std::ops::Range<u64>; 1], _> = shortened.try_into();
                        match shortened {
                            Ok([original]) if original == candidate.sources => continue,
                            Ok([different]) => {
                                candidates_changed = true;
                                return vec![different].into_iter().map(wrap_in_range);
                            }
                            Err(shortened) => {
                                candidates_changed = true;
                                return shortened.into_iter().map(wrap_in_range);
                            }
                        }
                    }
                    vec![candidate.sources].into_iter().map(wrap_in_range)
                })
                .collect();
        }

        new_ranges.extend(reverse_range_candidates);

        ReverseMap::new(new_ranges)
    }
}
impl<T> IntoIterator for map::MapGeneric<T>
where
    T: std::fmt::Debug,
{
    type Item = RangeGeneric<T>;
    type IntoIter = <Vec<RangeGeneric<T>> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

fn apply_offset(bound: u64, offset: i64) -> Result<u64, std::num::TryFromIntError> {
    u64::try_from(i64::try_from(bound).expect("range input outside i64 bounds") + offset)
}
// NOTE: Non-lossy version is not used
// fn apply_offset_to_range(
//     src: std::ops::Range<u64>,
//     offset: i64,
// ) -> Result<std::ops::Range<u64>, std::num::TryFromIntError> {
//     let start = apply_offset(src.start, offset)?;
//     let end = apply_offset(src.end, offset)?;
//     Ok(start..end)
// }
fn apply_offset_to_range_saturating(
    src: std::ops::Range<u64>,
    offset: i64,
) -> std::ops::Range<u64> {
    let self_offset_neg = offset < 0;
    let self_offset_mag = u64::try_from(offset.abs()).expect("i64 magnitude fits in u64");
    let add_sub_fn = if self_offset_neg {
        u64::saturating_sub
    } else {
        u64::saturating_add
    };
    let new_start = add_sub_fn(src.start, self_offset_mag);
    let new_end = add_sub_fn(src.end, self_offset_mag);
    new_start..new_end
}

impl std::ops::Add for MapUnit {
    type Output = Self;
    fn add(self, b: Self) -> Self::Output {
        let MapUnit {
            map: map_a,
            output_kind: _,
        } = self;

        let MapUnit {
            map: map_b,
            output_kind: second_output_kind,
        } = b;

        println!("--- BEGIN ADD ---");
        println!("map_a = {map_a}");
        println!("map_b = {map_b}");

        let mut ranges = vec![];

        for range_a in map_a {
            let Range {
                sources: ref sources_a,
                offset: offset_a,
            } = range_a;

            let mut ranges_this_range = vec![];

            // NOTE quadratic time
            for range_b in map_b.iter() {
                let Range {
                    sources: ref sources_b_unoffset,
                    offset: offset_b,
                } = *range_b;
                let sources_b =
                    apply_offset_to_range_saturating(sources_b_unoffset.clone(), -offset_a);

                if sources_b.is_empty() {
                    continue;
                }

                let IntersectedRanges {
                    both,
                    a_only: _,
                    b_only: _,
                } = intersect_ranges(sources_a.clone(), sources_b.clone());
                ranges_this_range.extend(both.map(|sources| Range {
                    sources,
                    offset: offset_a + offset_b,
                }));
            }

            println!("A-range {range_a}, intersections {ranges_this_range:?}");
            ranges_this_range.sort_by_key(|r| r.sources.start);
            let mut start = sources_a.start;
            for range in ranges_this_range {
                if start < range.sources.start {
                    let fill = Range {
                        sources: start..range.sources.start,
                        offset: offset_a,
                    };
                    println!("\tfill in empty range {fill:?}");
                    ranges.push(fill);
                }
                start = range.sources.end;

                println!("\tadd range {range:?}");
                ranges.push(range);
            }

            if start < sources_a.end {
                let fill = Range {
                    sources: start..sources_a.end,
                    offset: offset_a,
                };
                println!("\tfill in empty range {fill:?} (tail end)");
                ranges.push(fill);
            }
        }

        Self {
            map: Map::new(ranges),
            output_kind: second_output_kind,
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

impl std::fmt::Display for MapUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { map, output_kind } = self;
        write!(f, "{map} -> output {output_kind:?}")
    }
}
impl<T> std::fmt::Display for map::MapGeneric<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for range in self.ranges() {
            write!(f, "\n\t{range},")?;
        }
        Ok(())
    }
}
impl<T> std::fmt::Display for RangeGeneric<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let RangeGeneric { sources, offset } = self;
        write!(f, "{sources:?} offset={offset:?}")
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
pub struct RangeGeneric<T> {
    pub sources: std::ops::Range<u64>,
    pub offset: T,
}
impl From<Range> for ReverseRange {
    fn from(value: Range) -> Self {
        let Range { sources, offset } = value;
        ReverseRange {
            sources,
            offset: Some(offset),
        }
    }
}

// NOTE This does not take into account "a" filtering "b", so not useful
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
//         let other_sources = apply_offset_to_range_saturating(other_sources, -self_offset);
//         // {
//         //     let self_offset_neg = self_offset < 0;
//         //     let self_offset_mag =
//         //         u64::try_from(self_offset.abs()).expect("i64 magnitude fits in u64");
//         //     // NOTE: Goal is to *SUBTRACT* self_offset.
//         //     //  If negative, then add the magnitude
//         //     //  If positive, then subtract the magnitude
//         //     let add_sub_fn = if self_offset_neg {
//         //         u64::saturating_add
//         //     } else {
//         //         u64::saturating_sub
//         //     };
//         //     let new_start = add_sub_fn(other_sources.start, self_offset_mag);
//         //     let new_end = add_sub_fn(other_sources.end, self_offset_mag);
//         //     new_start..new_end
//         // };
//         // Move self.offset to the right, depending on range intersections
//         let IntersectedRanges {
//             both,
//             a_only: self_only,
//             b_only: _other_only,
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
//         // NOTE: DO NOT include other_only, since `self` is the filter
//         // let other_only = other_only.into_iter().map(|sources| Self {
//         //     sources,
//         //     offset: other_offset,
//         // });
//         let output_ranges: Vec<_> = self_only
//             .into_iter()
//             .chain(both)
//             // .chain(other_only)
//             .collect();
//         assert!(!output_ranges.is_empty(), "at least one resulting range");
//         output_ranges
//     }
// }

use crate::arithmetic::{intersect_ranges, IntersectedRanges};
mod arithmetic {
    type StdRange = std::ops::Range<u64>;
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct IntersectedRanges {
        pub both: Option<StdRange>,
        pub a_only: Vec<StdRange>,
        pub b_only: Vec<StdRange>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Either {
        A,
        B,
    }
    impl Either {
        fn choose<T>(self, a: T, b: T) -> T {
            match self {
                Either::A => a,
                Either::B => b,
            }
        }
    }
    /// Returns the intersection of ranges: (A only, Both, B only)
    pub fn intersect_ranges(a: StdRange, b: StdRange) -> IntersectedRanges {
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        let a_debug = a.clone();
        let b_debug = b.clone();
        // Ignoring outside the range, there are 3 possibilities:
        //
        // 1.    |---AB---------|  Intersect completely (1 region)
        // 2.    |--A--|  |--B--|  Disjoint (2 regions)
        // 3.i   |--A-|-AB-|-A--|  Contained (3 regions)
        // 3.ii  |--B-|-AB-|-B--|  Contained (3 regions)
        // 3.iii |--A-|-AB-|-B--|  Intersect partially (3 regions)
        // 3.iv  |--B-|-AB-|-A--|  Intersect partially (3 regions)
        //
        let (mut a_only, both, mut b_only) = if a.start == b.start && a.end == b.end {
            // 1. Intersect completely (1 region)
            let equal = a;
            (vec![], Some(equal), vec![])
        } else {
            let a_start_in_b = b.contains(&a.start);
            let a_end_in_b = b.contains(&a.end);
            let b_start_in_a = a.contains(&b.start);
            let b_end_in_a = a.contains(&b.end);
            if !a_start_in_b && !a_end_in_b && !b_start_in_a && !b_end_in_a {
                // 2. Disjoint (2 regions)
                (vec![a], None, vec![b])
            } else {
                // 3. Contained or intersect partially (3 regions)

                // TODO identify the middle 2 of the 4 endpoints
                let endpoints_sorted = {
                    let mut endpoints = [
                        (Either::A, a.start),
                        (Either::A, a.end),
                        (Either::B, b.start),
                        (Either::B, b.end),
                    ];
                    endpoints.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));
                    endpoints
                };
                let both = {
                    let [_, (_, both_start), (_, both_end), _] = endpoints_sorted;
                    both_start..both_end
                };

                let (a_ranges, b_ranges) = {
                    let (left_ty, left) = {
                        let [(ty, start), (_, end), _, _] = endpoints_sorted;
                        (ty, start..end)
                    };
                    let (right_ty, right) = {
                        let [_, _, (_, start), (ty, end)] = endpoints_sorted;
                        (ty, start..end)
                    };
                    let mut a_ranges = Vec::with_capacity(2);
                    let mut b_ranges = Vec::with_capacity(2);

                    debug_assert_eq!(a_ranges.capacity(), 2);
                    debug_assert_eq!(b_ranges.capacity(), 2);

                    left_ty.choose(&mut a_ranges, &mut b_ranges).push(left);
                    right_ty.choose(&mut a_ranges, &mut b_ranges).push(right);

                    debug_assert_eq!(a_ranges.capacity(), 2);
                    debug_assert_eq!(b_ranges.capacity(), 2);

                    (a_ranges, b_ranges)
                };

                (a_ranges, Some(both), b_ranges)
            }
        };
        let both = both.and_then(|range| (!range.is_empty()).then_some(range));
        a_only.retain(|range| !range.is_empty());
        b_only.retain(|range| !range.is_empty());
        println!("Intersection ({a_debug:?}, {b_debug:?}) -> (both: {both:?}, a_only: {a_only:?}, b_only: {b_only:?})");
        IntersectedRanges {
            both,
            a_only,
            b_only,
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::arithmetic::{intersect_ranges, IntersectedRanges};

        fn test_symmetric(
            (a, b): (std::ops::Range<u64>, std::ops::Range<u64>),
            expected: IntersectedRanges,
        ) {
            let test_forward = intersect_ranges(a.clone(), b.clone());
            assert_eq!(test_forward, expected);

            let IntersectedRanges {
                both,
                a_only,
                b_only,
            } = expected;
            let expected_reverse = IntersectedRanges {
                both,
                a_only: b_only,
                b_only: a_only,
            };

            let test_reverse = intersect_ranges(b, a);
            assert_eq!(test_reverse, expected_reverse);
        }

        // 1.    |---AB---------|  Intersect completely (1 region)
        #[test]
        fn identical() {
            test_symmetric(
                (2..5, 2..5),
                IntersectedRanges {
                    both: Some(2..5),
                    a_only: vec![],
                    b_only: vec![],
                },
            );
        }
        // 2.    |--A--|  |--B--|  Disjoint (2 regions)
        #[test]
        fn disjoint() {
            test_symmetric(
                (2..5, 7..9),
                IntersectedRanges {
                    both: None,
                    a_only: vec![2..5],
                    b_only: vec![7..9],
                },
            );
        }
        // 3.i   |--A-|-AB-|-A--|  Contained (3 regions)
        #[test]
        fn contained() {
            test_symmetric(
                (19..93, 25..30),
                IntersectedRanges {
                    both: Some(25..30),
                    a_only: vec![19..25, 30..93],
                    b_only: vec![],
                },
            );
        }
        #[test]
        fn contained_aa_end_empty() {
            test_symmetric(
                (19..30, 25..30),
                IntersectedRanges {
                    both: Some(25..30),
                    a_only: vec![19..25],
                    b_only: vec![],
                },
            );
        }
        #[test]
        fn contained_aa_start_empty() {
            test_symmetric(
                (25..93, 25..30),
                IntersectedRanges {
                    both: Some(25..30),
                    a_only: vec![30..93],
                    b_only: vec![],
                },
            );
        }
        // 3.iii |--A-|-AB-|-B--|  Intersect partially (3 regions)
        #[test]
        fn intersect_ab() {
            test_symmetric(
                (5..30, 10..90),
                IntersectedRanges {
                    both: Some(10..30),
                    a_only: vec![5..10],
                    b_only: vec![30..90],
                },
            )
        }
        #[test]
        fn intersect_ab_end_empty() {
            test_symmetric(
                (5..30, 10..30),
                IntersectedRanges {
                    both: Some(10..30),
                    a_only: vec![5..10],
                    b_only: vec![],
                },
            )
        }
        #[test]
        fn intersect_ab_start_empty() {
            test_symmetric(
                (10..30, 10..90),
                IntersectedRanges {
                    both: Some(10..30),
                    a_only: vec![],
                    b_only: vec![30..90],
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{get_closest_location, MapUnit, Range};

    #[test]
    fn sample_input() {
        // NOTE: format is [DEST] [SOURCE] [LEN]
        let input = "seeds: 79 14 55 13

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
        let closest = get_closest_location(input).unwrap();
        assert_eq!(closest, 46);
    }

    //     #[test]
    //     fn simpler_input() {
    //         // NOTE: format is [DEST] [SOURCE] [LEN]
    //         let input = "seeds: 79 9999999
    //
    // seed-to-soil map:
    // 50 98 2
    // 52 50 48
    //
    // soil-to-fertilizer map:
    // 0 15 37
    // 37 52 2
    // 39 0 15
    //
    // fertilizer-to-location map:
    // 60 56 37
    // 56 93 4";
    //         let closest = get_closest_location(input).unwrap();
    //         assert_eq!(closest, 35); // TODO why is this 39 outside both the input and output ranges?
    //     }

    #[test]
    fn single_layer() {
        // NOTE: format is [DEST] [SOURCE] [LEN]
        let input = "seeds: 25 50

seed-to-location map:
30 20 15";
        // Maps 20..35 -> 30..45
        let closest = get_closest_location(input).unwrap();
        assert_eq!(closest, 25);
    }

    fn map_unit(ranges: Vec<Range>, output_kind: &'static str) -> MapUnit {
        MapUnit::new(ranges, output_kind.to_string())
    }

    #[test]
    fn adding_maps_one_to_one() {
        let map1 = map_unit(
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
            "dummy",
        );
        let map2 = map_unit(
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
            "final output kind",
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
        let map1 = map_unit(
            vec![Range {
                sources: 10..30,
                offset: 20,
            }],
            "dummy",
        );
        let map2 = map_unit(
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
            "final output kind",
        );
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(
            ranges,
            vec![
                // Range {
                //     sources: 5..10,
                //     offset: 2, // only map2
                // },
                Range {
                    sources: 10..20,
                    offset: 22, // map1 & map2
                },
                Range {
                    sources: 20..25,
                    offset: 20, // only map1
                },
                Range {
                    sources: 25..30,
                    offset: 120, // map1 & map2
                },
                // Range {
                //     sources: 30..40,
                //     offset: 2, // only map2
                // },
                // ---
                // nothing 40..45
                // Range {
                //     sources: 45..70,
                //     offset: 100, // only map2
                // },
            ]
        );
        assert_eq!(&output_kind, "final output kind");
    }

    #[test]
    fn adding_maps_joins() {
        let map1 = map_unit(
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
            "dummy",
        );
        let map2 = map_unit(
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
            "final output kind",
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
    fn not_really_a_dead_end() {
        let map1 = map_unit(
            vec![Range {
                sources: 10..50,
                offset: 10,
            }],
            "dummy",
        );
        let map2 = map_unit(
            vec![Range {
                sources: 0..5,
                offset: 20,
            }],
            "end",
        );
        let result = map1 + map2;
        let (ranges, output_kind) = result.into_parts();
        assert_eq!(
            ranges,
            vec![Range {
                sources: 10..50,
                offset: 10,
            }]
        );
        assert_eq!(output_kind, "end");
    }
}
