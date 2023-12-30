use std::num::NonZeroUsize;

use advent_2023::nonempty::NonEmptyVec;

use crate::ONE;

#[derive(Clone, PartialEq, Eq)]
pub struct Segment(pub NonEmptyVec<Part>);

impl Segment {
    #[allow(unused)] // for tests
    pub fn new_from_str(symbols: &str) -> anyhow::Result<Option<Self>> {
        let symbols = &mut symbols.chars().peekable();
        Self::new(symbols)
    }
    pub fn new(
        symbols: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    ) -> anyhow::Result<Option<Self>> {
        // ignore duplicate separators
        while let Some('.') = symbols.peek() {
            let _ = symbols.next();
        }

        let mut builder = SegmentBuilder::default();

        for symbol in symbols {
            let new = match symbol {
                '#' => Part::Absolute(ONE),
                '?' => Part::Unknown(ONE),
                '.' => {
                    break;
                }
                extra => {
                    anyhow::bail!("unknown character {extra:?}")
                }
            };
            builder.push(new);
        }
        Ok(builder.finish())
    }
    pub fn is_nullable(&self) -> bool {
        self.0.iter().copied().all(Part::is_nullable)
    }

    pub fn into_builder(self) -> SegmentBuilder {
        self.into()
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Absolute(NonZeroUsize),
    Unknown(NonZeroUsize),
}
impl Part {
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
