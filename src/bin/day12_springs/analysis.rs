//! Segment-specific analysis functions/types

use crate::{DebugParts, Part, Segment};
use advent_2023::nonempty::NonEmptyVec;
use std::num::NonZeroUsize;

impl Segment {
    pub(crate) fn count_possibilities(
        &self,
        expected_counts: &[NonZeroUsize],
        debug_indent: usize,
    ) -> usize {
        print!("{:width$}", "", width = debug_indent);
        println!("[Segment::count_possibilities]");
        let Some(counts_split) = expected_counts.split_first_copy() else {
            // empty counts, nonempty parts (impossible)
            return 0;
        };

        let parts_split = self.0.split_first_copy();
        SegmentAnalysis::new(parts_split, counts_split, debug_indent).count()
    }
}

struct SegmentAnalysis<'a> {
    part_first: Part,
    parts_rest: &'a [Part],
    count_first: NonZeroUsize,
    counts_rest: &'a [NonZeroUsize],
    force_left_align: Option<ForceLeftAlign>,
    debug_indent: usize,
}
impl<'a> SegmentAnalysis<'a> {
    fn new(
        parts_split: (Part, &'a [Part]),
        counts_split: (NonZeroUsize, &'a [NonZeroUsize]),
        debug_indent: usize,
    ) -> Self {
        let (part_first, parts_rest) = parts_split;
        let (count_first, counts_rest) = counts_split;
        Self {
            part_first,
            parts_rest,
            count_first,
            counts_rest,
            force_left_align: None,
            debug_indent,
        }
    }
    fn recurse(
        &self,
        debug_msg: &'static str,
        (part_first, parts_rest): (Part, &'a [Part]),
        (count_first, counts_rest): (NonZeroUsize, &'a [NonZeroUsize]),
        force_left_align: Option<ForceLeftAlign>,
    ) -> Self {
        println!("{:width$}-> {debug_msg} --v", "", width = self.debug_indent);
        Self {
            part_first,
            parts_rest,
            count_first,
            counts_rest,
            force_left_align,
            debug_indent: self.debug_indent + 4,
        }
    }
    /// Counts the number of possibilities for the Segment covering ALL counts
    ///
    /// `Segment` = `Vec<Part>`, and multiple parts can cover one count
    ///
    /// |-------------SEGMENT--------------|
    /// |-PART-|-PART-|-PART-|-PART-|-PART-|
    /// ====================================
    /// |--COUNT--| |--COUNT--|  |--COUNT--|
    /// ====================================
    ///            ^-----------^^----Unknowns chosen to be NONE
    ///
    fn count(self) -> usize {
        let Self {
            part_first,
            parts_rest,
            count_first,
            counts_rest,
            force_left_align,
            debug_indent,
        } = self;

        print!("{:width$}", "", width = debug_indent);
        print!("* count");
        print!("(parts: ({part_first:?}, {parts_rest:?})");
        print!(", counts: ({count_first:?}, {counts_rest:?})");
        print!(
            ", {}",
            force_left_align.as_ref().map_or("none", |_| "Align")
        );
        let debug_parts = DebugParts(unsplit_to_vec(part_first, parts_rest));
        let debug_counts = unsplit_to_vec(count_first, counts_rest);
        println!(")\tpartial-record {debug_parts} {debug_counts:?}",);

        let (branch, (result, reason)): (_, (usize, &'static str)) =
            if parts_rest.is_empty() && counts_rest.is_empty() {
                (
                    "case_singleton",
                    Self::case_singleton(part_first, count_first, force_left_align),
                )
            } else {
                // Non-singleton case
                match part_first {
                    Part::Absolute(count_part_absolute) => {
                        ("case_absolute", self.case_absolute(count_part_absolute))
                    }
                    Part::Unknown(count_part_unknown) => {
                        ("case_unknown", self.case_unknown(count_part_unknown))
                    }
                }
            };
        println!(
            "{:width$}{result} <- {reason} ({branch})",
            "",
            width = debug_indent
        );
        result
    }
    fn case_singleton(
        part_first: Part,
        count_first: NonZeroUsize,
        force_left_align: Option<ForceLeftAlign>,
    ) -> (usize, &'static str) {
        // ONE part in segment, and ONE count
        let expected = count_first;
        match part_first {
            // Absoulute - must match completely
            Part::Absolute(count_part) => {
                if count_part == expected {
                    (1, "absolute perfect match")
                } else {
                    (0, "absolute NOT perfect match")
                }
            }
            // Unknown - possible to slide around
            Part::Unknown(count_part) => {
                if let Some(blank_area) = count_part.get().checked_sub(expected.get()) {
                    if force_left_align.is_some() {
                        (1, "unknowns left-aligned covers remaining expected")
                    } else {
                        // blanked area can slide, with one side containing: 0..=blank_area
                        (blank_area + 1, "unknowns slide")
                    }
                } else {
                    // expecting more than possible
                    (0, "unknowns too short")
                }
            }
        }
    }
    fn case_absolute(&self, count_part_absolute: NonZeroUsize) -> (usize, &'static str) {
        // anchored by first
        //
        // For a match, the absolute part MUST fully contain `count_first`
        let Some(count_remaining_opt) = self
            .count_first
            .get()
            .checked_sub(count_part_absolute.get())
            .map(NonZeroUsize::new)
        else {
            // REJECT, Part::Absolute larger than count_first
            return (0, "absolute too long for current count");
        };

        let (counts_next, new_force_left_align) = if let Some(count_remaining) = count_remaining_opt
        {
            // Absolute matches all, force align for remaining
            ((count_remaining, self.counts_rest), Some(ForceLeftAlign))
        } else {
            // Pop count
            let Some(counts_next) = self.counts_rest.split_first_copy() else {
                // verify remainder is all Unknowns
                let possible = self.parts_rest.iter().copied().all(Part::is_nullable);
                return if possible {
                    (1, "absolute exhausted count, remainder is nullable")
                } else {
                    (0, "absolute exhausted count, but remainder is not nullable")
                };
            };
            (counts_next, None)
        };

        let Some((part_next, parts_rest)) = self.parts_rest.split_first_copy() else {
            return (0, "absolute exhausted parts, but counts remain");
        };
        let parts_next: (Part, &[Part]) = if count_remaining_opt.is_some() {
            (part_next, parts_rest)
        } else {
            // perfect match, need to nullify immediate next
            match part_next {
                Part::Unknown(part_count) => {
                    if let Some(part_count_reduced) =
                        part_count.get().checked_sub(1).and_then(NonZeroUsize::new)
                    {
                        (Part::Unknown(part_count_reduced), parts_rest)
                    } else {
                        let Some(parts_next_split2) = parts_rest.split_first_copy() else {
                            return (0, "absolute perfect match, but after consuming separator no parts remain to satify next counts");
                        };
                        parts_next_split2
                    }
                }
                Part::Absolute(_) => {
                    return (
                        0,
                        "absolute perfect match, but need separator and immediate next is absolute",
                    );
                }
            }
        };

        let result = self
            .recurse(
                "part pop_front, count reduced",
                parts_next,
                counts_next,
                new_force_left_align,
            )
            .count();
        (result, "recurse")
    }
    fn case_unknown(&self, count_part_unknown: NonZeroUsize) -> (usize, &'static str) {
        if self.force_left_align.is_some()
            && self.counts_rest.is_empty()
            && count_part_unknown >= self.count_first
            && !self.parts_rest.iter().copied().all(Part::is_nullable)
        {
            return (
                0,
                "unknowns longer than final count, but remainder not nullable and force_left_align",
            );
        }

        // FIXME avoid N-based recurse for Unknowns...
        // For now, input is not excessively long so recurse for each question-mark choice

        let mut drain_unknown = DrainUnknown::start_after(count_part_unknown);

        let part_reduced_opt = drain_unknown.next();
        let parts_reduced = match part_reduced_opt {
            Some(part_reduced) => {
                // if let Some(part_reduced) = count_part_unknown.checked_sub(1)
                let part_reduced = Part::Unknown(part_reduced);
                // -1 to first
                (part_reduced, self.parts_rest)
            }
            None => {
                // if count_part_unknown == ONE
                // else

                // remove empty first
                let Some(parts_next) = self.parts_rest.split_first_copy() else {
                    return (0, "unknown exhausted parts, but counts remain");
                };
                parts_next
            }
        };
        let parts_reduced_2 = part_reduced_opt.and_then(|_| {
            let part_reduced_2 = drain_unknown.next();

            match part_reduced_2 {
                Some(part_reduced_2) => {
                    let part_reduced_2 = Part::Unknown(part_reduced_2);
                    // -2 (total) to first
                    Some((part_reduced_2, self.parts_rest))
                }
                None => {
                    // remove empty first
                    self.parts_rest.split_first_copy()
                }
            }
        });
        drop(drain_unknown); // dangerous to re-use, thinking it's from the beginning

        // split into two possibilities (ON, OFF)
        let unknown_on = {
            let count_reduced_opt = self.count_first.get().checked_sub(1).map(NonZeroUsize::new);
            match count_reduced_opt {
                Some(None) => {
                    // if self.count_first.get() == 1
                    if let Some(counts_next) = self.counts_rest.split_first_copy() {
                        if let Some(parts_reduced_2) = parts_reduced_2 {
                            // use `part_reduced_2` to add pseudo-separator of Unknown OFF, after the assumed ON
                            self.recurse(
                                "assume Unknown = ON, pop_front count, parts reduce x2",
                                parts_reduced_2,
                                counts_next,
                                None,
                            )
                            .count()
                        } else {
                            // the assumed "Unknown ON" completes the count,
                            // but there are remaining counts with no remaining part
                            0
                        }
                    } else {
                        // verify remainder is all Unknowns
                        let possible = self.parts_rest.iter().copied().all(Part::is_nullable);
                        if possible {
                            // unknown exhasted count, remainder is nullable
                            1
                        } else {
                            // unknown exhasted count, but remainder is not nullable
                            0
                        }
                    }
                }
                Some(Some(count_reduced)) => {
                    // else if let Some(count_reduced) = self.count_first.checked_sub(1)

                    let (_, new_parts_rest) = parts_reduced;
                    // let (new_part_first_orig, count_reduced_orig) = {
                    //     let mut drain_unknown = DrainUnknown::start_after(count_part_unknown);
                    //     let mut new_part_first = parts_reduced.0;
                    //     let mut count_reduced = count_reduced;
                    //     drain_unknown.next();
                    //     while let Some(reduced_unknown_count) = drain_unknown.next() {
                    //         if let Some(new_count_reduced) =
                    //             NonZeroUsize::new(count_reduced.get() - 1)
                    //         {
                    //             new_part_first = Part::Unknown(reduced_unknown_count);
                    //             count_reduced = new_count_reduced;
                    //         } else {
                    //             break;
                    //         }
                    //     }
                    //     (new_part_first, count_reduced)
                    // };
                    let (new_part_first, count_reduced) = {
                        if count_part_unknown.get() >= 2 {
                            let (new_part_first, new_count_reduced) =
                                DrainUnknown::last_pair(count_part_unknown, self.count_first);
                            let new_part_first = Part::Unknown(new_part_first);
                            (new_part_first, new_count_reduced)
                        } else {
                            (parts_reduced.0, count_reduced)
                        }
                    };
                    // assert_eq!(new_part_first, new_part_first_orig);
                    // assert_eq!(count_reduced, count_reduced_orig);

                    self.recurse(
                        "assume Unknown = ON x-many, reduce count x-many",
                        (new_part_first, new_parts_rest),
                        (count_reduced, self.counts_rest),
                        Some(ForceLeftAlign),
                    )
                    .count()
                }
                None => {
                    //else
                    // cannot reduce count, impossible for ON
                    dbg!(0)
                }
            }
        };
        let unknown_off = if self.force_left_align.is_some() {
            // cannot assume off, forced left align
            0
        } else {
            // let counts_next = counts_rest.split_first_copy().unwrap_or((0, &[]));
            self.recurse(
                "assume Unknown = OFF, count pop_front",
                parts_reduced,
                (self.count_first, self.counts_rest),
                self.force_left_align,
            )
            .count()
        };
        let sum = unknown_on + unknown_off;
        (sum, "sum of ON and OFF options")
    }
}

#[derive(Clone, Copy, Debug)]
struct ForceLeftAlign;

// NOT copy
struct DrainUnknown(usize);
impl DrainUnknown {
    fn start_after(value: NonZeroUsize) -> Self {
        Self(value.get() - 1)
    }
    fn last_pair(lhs: NonZeroUsize, rhs: NonZeroUsize) -> (NonZeroUsize, NonZeroUsize) {
        Self(lhs.get())
            .last_with(Self(rhs.get()))
            .expect("nonzero's have a last")
    }
    fn last_with(self, other: Self) -> Option<(NonZeroUsize, NonZeroUsize)> {
        let steps = self.0.min(other.0).checked_sub(1)?;
        let last_self =
            NonZeroUsize::new(self.0 - steps).expect("subtracting size less 1 yields nonzero");
        let last_other =
            NonZeroUsize::new(other.0 - steps).expect("subtracting size less 1 yields nonzero");
        Some((last_self, last_other))
    }
}
impl Iterator for DrainUnknown {
    type Item = NonZeroUsize;

    fn next(&mut self) -> Option<Self::Item> {
        let count = NonZeroUsize::new(self.0)?;
        self.0 = count.get() - 1;
        Some(count)
    }
}

pub fn unsplit_to_vec<T: Copy>(first: T, rest: &[T]) -> Vec<T> {
    std::iter::once(first).chain(rest.iter().copied()).collect()
}

// Convenience functions (as traits, for foreign types)
trait SplitFirstCopyOption<T: Copy> {
    fn split_first_copy(&self) -> Option<(T, &[T])>;
}
impl<'a, T: Copy> SplitFirstCopyOption<T> for &'a [T] {
    fn split_first_copy(&self) -> Option<(T, &[T])> {
        self.split_first().map(|(&first, rest)| (first, rest))
    }
}
trait SplitFirstCopy<T: Copy> {
    fn split_first_copy(&self) -> (T, &[T]);
}
impl<T: Copy> SplitFirstCopy<T> for NonEmptyVec<T> {
    fn split_first_copy(&self) -> (T, &[T]) {
        let (&first, rest) = self.split_first();
        (first, rest)
    }
}
