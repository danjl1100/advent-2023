use std::num::NonZeroUsize;

use advent_2023::nonempty::NonEmptyVec;

use crate::ONE;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Segment(pub NonEmptyVec<Part>);

type SegmentAndMeta = (Option<LeadingSep>, Segment, Option<TrailingSep>);

impl Segment {
    #[allow(unused)] // for tests
    pub fn new_from_str(symbols: &str) -> anyhow::Result<Option<SegmentAndMeta>> {
        let symbols = &mut symbols.chars().peekable();
        Self::new(symbols)
    }
    pub fn new(
        symbols: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    ) -> anyhow::Result<Option<SegmentAndMeta>> {
        let mut leading = None;
        let mut trailing = None;

        // ignore duplicate separators
        while let Some('.') = symbols.peek() {
            let _ = symbols.next();
            leading = Some(LeadingSep);
        }

        let mut builder = SegmentBuilder::default();

        for symbol in symbols {
            let new = match symbol {
                '#' => Part::Absolute(ONE),
                '?' => Part::Unknown(ONE),
                '.' => {
                    trailing = Some(TrailingSep);
                    break;
                }
                extra => {
                    anyhow::bail!("unknown character {extra:?}")
                }
            };
            builder.push(new);
        }
        Ok(builder.finish().map(|this| (leading, this, trailing)))
    }
    pub fn is_nullable(&self) -> bool {
        self.iter().all(Part::is_nullable)
    }
    /// Largest single count this segment *must* cover (if all ?=0)
    pub fn get_largest_min_run(&self) -> usize {
        self.iter().map(Part::min_run).max().expect("nonempty")
    }
    /// Largest total amount this segment *could* cover (if all ?=1)
    pub fn get_maximum_count(&self) -> usize {
        self.iter().map(Part::len).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = Part> + '_ {
        self.0.iter().copied()
    }

    pub fn into_builder(self) -> SegmentBuilder {
        self.into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LeadingSep;

#[derive(Clone, Copy, Debug)]
pub struct TrailingSep;

#[derive(Default)]
pub struct SegmentBuilder {
    parts: Vec<Part>,
    prev: Option<Part>,
}
impl SegmentBuilder {
    pub fn push(&mut self, new: Part) {
        self.prev = match self.prev.take() {
            None => Some(new),
            Some(prev) => {
                let (finished, combined) = prev + new;
                if let Some(finished) = finished {
                    self.parts.push(finished);
                }
                Some(combined)
            }
        };
    }
    pub fn finish(self) -> Option<Segment> {
        let Self { mut parts, prev } = self;

        // append unfinished part
        if let Some(prev) = prev {
            parts.push(prev);
        }

        NonEmptyVec::new(parts).map(Segment)
    }
}
impl From<Segment> for SegmentBuilder {
    fn from(value: Segment) -> Self {
        let Segment(parts) = value;
        let (parts, last) = parts.into_split_last();
        Self {
            parts,
            prev: Some(last),
        }
    }
}
impl FromIterator<Part> for SegmentBuilder {
    fn from_iter<T: IntoIterator<Item = Part>>(iter: T) -> Self {
        let mut builder = SegmentBuilder::default();
        for part in iter {
            builder.push(part);
        }
        builder
    }
}
impl IntoIterator for Segment {
    type Item = Part;
    type IntoIter = <Vec<Part> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    Absolute(NonZeroUsize),
    Unknown(NonZeroUsize),
}
impl Part {
    #[allow(clippy::len_without_is_empty)] // empty has a different meaning for Part(NonZeroUsize)
    pub fn len(self) -> usize {
        match self {
            Part::Absolute(count) | Part::Unknown(count) => count.get(),
        }
    }
    pub fn min_run(self) -> usize {
        match self {
            Part::Absolute(count) => count.get(),
            Part::Unknown(_) => 0,
        }
    }
    pub fn is_nullable(self) -> bool {
        match self {
            Part::Absolute(_) => false,
            Part::Unknown(_) => true,
        }
    }
}
impl std::ops::Add for Part {
    type Output = (Option<Part>, Part);
    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Absolute(count_old), Self::Absolute(count_new)) => {
                let sum =
                    NonZeroUsize::new(count_old.get() + count_new.get()).expect("nonzerousize add");
                (None, Self::Absolute(sum))
            }
            (Self::Unknown(count_old), Self::Unknown(count_new)) => {
                let sum =
                    NonZeroUsize::new(count_old.get() + count_new.get()).expect("nonzerousize add");
                (None, Self::Unknown(sum))
            }
            (Self::Absolute(_), Self::Unknown(_)) | (Self::Unknown(_), Self::Absolute(_)) => {
                (Some(self), other)
            }
        }
    }
}
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Part::Absolute(count) => {
                if count.get() == 1 {
                    write!(f, "#")
                } else {
                    write!(f, "#x{count}")
                }
            }
            Part::Unknown(count) => {
                if count.get() == 1 {
                    write!(f, "?")
                } else {
                    write!(f, "?x{count}")
                }
            }
        }
    }
}
impl std::fmt::Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <NonEmptyVec<Part> as std::fmt::Debug>::fmt(&self.0, f)
    }
}

pub struct DebugParts(pub Vec<Part>);
impl std::fmt::Display for DebugParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = Some(());
        for &part in &self.0 {
            if is_first.take().is_none() {
                write!(f, " ")?;
            }
            let (symbol, count) = match part {
                Part::Absolute(count) => ('#', count),
                Part::Unknown(count) => ('?', count),
            };
            for _ in 0..count.get() {
                write!(f, "{symbol}")?;
            }
        }
        write!(f, "}}")
    }
}
