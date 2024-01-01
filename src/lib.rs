use clap::Parser;
use std::{collections::VecDeque, io::Read, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    filename: Option<PathBuf>,
}

/// Returns the input string read from the cli argument file, or stdin
pub fn get_input_string() -> anyhow::Result<String> {
    let args = Args::parse();

    let input = if let Some(filename) = args.filename {
        std::fs::read_to_string(filename)?
    } else {
        println!("Awaiting instructions from stdin:");
        let mut input_buf = String::new();
        std::io::stdin().read_to_string(&mut input_buf)?;
        input_buf
    };
    Ok(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharIndex {
    /// How many bytes you need to skip to find the start of the char
    byte_index: usize,
    /// How many chars proceed this char
    char_sequence: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharEndSequence {
    /// How many chars proceed this char
    char_sequence: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharIndexEnd {
    /// End occurs within middle of the string
    Position(CharIndex),
    /// End is the end of the string (no index available for the next char, after the end)
    End(CharEndSequence),
}
impl CharIndex {
    pub fn slice_string(self, limit: CharIndexEnd, input: &str) -> Option<&str> {
        match limit {
            CharIndexEnd::Position(CharIndex {
                byte_index: end_index,
                ..
            }) => input.get(self.byte_index..end_index),
            CharIndexEnd::End(_) => input.get(self.byte_index..),
        }
    }
    pub fn is_zero(self) -> bool {
        self.byte_index == 0 && self.char_sequence == 0
    }
    pub fn sequence(self) -> usize {
        self.char_sequence
    }
}
impl CharEndSequence {
    pub fn sequence(self) -> usize {
        self.char_sequence
    }
}
impl CharIndexEnd {
    pub fn sequence(self) -> usize {
        match self {
            CharIndexEnd::Position(index) => index.sequence(),
            CharIndexEnd::End(index) => index.sequence(),
        }
    }
    pub fn is_end(self) -> bool {
        matches!(self, Self::End(_))
    }
}

pub struct CharIndices<'a> {
    iter: std::iter::Peekable<std::iter::Enumerate<std::str::CharIndices<'a>>>,
    last_char_sequence: Option<usize>,
}
impl<'a> CharIndices<'a> {
    pub fn new(input: &'a str) -> Self {
        CharIndices {
            iter: input.char_indices().enumerate().peekable(),
            last_char_sequence: None,
        }
    }
    pub fn take_end_char_sequence(&mut self) -> Option<CharEndSequence> {
        let char_sequence = self.last_char_sequence.take()? + 1;
        Some(CharEndSequence { char_sequence })
    }
    pub fn peek(&mut self) -> Option<(CharIndex, char)> {
        let (char_sequence, (byte_index, char_value)) = self.iter.peek().copied()?;
        Some((
            CharIndex {
                byte_index,
                char_sequence,
            },
            char_value,
        ))
    }
}
impl Iterator for CharIndices<'_> {
    type Item = (CharIndex, char);
    fn next(&mut self) -> Option<Self::Item> {
        let (char_sequence, (byte_index, char_value)) = self.iter.next()?;
        self.last_char_sequence = Some(char_sequence);
        Some((
            CharIndex {
                byte_index,
                char_sequence,
            },
            char_value,
        ))
    }
}

pub struct CharScanner<'a, T> {
    line: &'a str,
    char_indices: CharIndices<'a>,
    last_indices: VecDeque<CharIndex>,
    lookback_range_indices: Option<(usize, usize)>,
    pending_results: VecDeque<T>,
}

impl<'a, T> CharScanner<'a, T>
where
    T: std::fmt::Debug,
{
    /// Creates a scanner with the specified lookback range
    ///
    /// `lookback_range` - if specified, tuple: (min_length, max_length) where [`Self::find_next`]
    ///                    will call `f_lookback_str` for string lengths `min_length..=max_length`
    pub fn new(line: &'a str, lookback_range_len: Option<(usize, usize)>) -> Self {
        let char_indices = CharIndices::new(line);

        let lookback_capacity = lookback_range_len.map(|(_min, len)| len + 1).unwrap_or(0);

        let lookback_range_indices =
            lookback_range_len.map(|(min_len, max_len)| ((min_len - 1), max_len));

        Self {
            line,
            char_indices,
            last_indices: VecDeque::with_capacity(lookback_capacity),
            lookback_range_indices,
            pending_results: VecDeque::with_capacity(lookback_capacity),
        }
    }
    /// Returns the next element matched by a matching function
    ///
    /// `f_single_char` - Function accepting single character (and char index within the line)
    /// `f_lookback_str` - Function accepting a lookback string, with char indices `(start, end)` corresponding to `&line[start..end]`
    pub fn find_next(
        &mut self,
        f_single_char: Option<impl Fn(char, CharIndex) -> Option<T>>,
        f_lookback_str: Option<impl Fn(&str, (CharIndex, CharIndexEnd)) -> Option<T>>,
    ) -> Option<T> {
        if f_lookback_str.is_some() {
            assert!(
                self.lookback_range_indices.is_some(),
                "f_lookback_str required lookback_range to be specified in constructor"
            );
        }

        if let Some(result) = self.pending_results.pop_front() {
            return Some(result);
        }

        while let Some((current_index, current_char)) = self.char_indices.next() {
            let next_index = self.char_indices.peek().map_or_else(
                || {
                    CharIndexEnd::End(
                        self.char_indices
                            .take_end_char_sequence()
                            .expect("iterator gave next, should have end"),
                    )
                },
                |(index, _char)| CharIndexEnd::Position(index),
            );
            let lookback_truncate = self
                .lookback_range_indices
                .map(|(_min, len)| len - 1)
                .unwrap_or(0);
            self.last_indices.truncate(lookback_truncate);
            self.last_indices.push_front(current_index);

            let mut clear_last_indices = false;
            if let Some(f_single_char) = &f_single_char {
                if let Some(result) = f_single_char(current_char, current_index) {
                    clear_last_indices = true;
                    self.pending_results.push_back(result);
                }
            }

            for lookback in self
                .lookback_range_indices
                .map_or(0..0, |(min, len)| min..len)
            {
                if let Some(&start_index) = self.last_indices.get(lookback) {
                    let end_index = next_index;
                    let last_part = start_index
                        .slice_string(end_index, self.line)
                        .expect("slicing on char boundaries");

                    if let Some(f_lookback_str) = &f_lookback_str {
                        if let Some(lookback_result) =
                            f_lookback_str(last_part, (start_index, end_index))
                        {
                            self.pending_results.push_back(lookback_result);
                        }
                    }
                } else {
                    break;
                }
            }

            if clear_last_indices {
                self.last_indices.clear();
            }

            if let Some(result) = self.pending_results.pop_front() {
                return Some(result);
            }
        }
        // exhausted char_indices
        None
    }
}

pub mod math {
    use std::num::NonZeroUsize;

    /// Greatest Common Divisor
    ///
    /// ```
    /// use advent_2023::math::gcd;
    ///
    /// // trivial cases
    /// assert_eq!(gcd(0, 3).get(), 3);
    /// assert_eq!(gcd(2, 0).get(), 2);
    /// assert_eq!(gcd(0, 0).get(), 1);
    ///
    /// // nontrivial
    /// assert_eq!(gcd(2, 3).get(), 1, "2, 3");
    /// assert_eq!(gcd(42, 6).get(), 6, "42, 6");
    /// assert_eq!(gcd(6, 42).get(), 6, "6, 42");
    /// assert_eq!(gcd(48, 18).get(), 6, "48, 18");
    ///
    /// ```
    pub fn gcd(a: usize, b: usize) -> NonZeroUsize {
        let one = NonZeroUsize::new(1).expect("nonzero");

        let a = NonZeroUsize::try_from(a).ok();
        let b = NonZeroUsize::try_from(b).ok();

        match (a, b) {
            (None, None) => one,
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (Some(a), Some(b)) => {
                let a = a.get();
                let b = b.get();

                let (smaller, larger) = if a < b { (a, b) } else { (b, a) };
                let larger = larger.rem_euclid(smaller);

                gcd(smaller, larger)
            }
        }
    }

    /// Least Common Multiple
    ///
    /// ```
    /// use advent_2023::math::lcm;
    ///
    /// assert_eq!(lcm(2, 4), 4);
    /// assert_eq!(lcm(10, 7), 70);
    /// assert_eq!(lcm(5, 7), 35);
    /// ```
    pub fn lcm(a: usize, b: usize) -> usize {
        let divisor = gcd(a, b);
        (a * b) / divisor
    }
}

pub mod nonempty {
    /// `Vec` guaranteed to be non-empty
    ///
    /// # Usage:
    ///
    /// Mutable slice operations are OK
    ///
    /// ```
    /// use advent_2023::{nonempty::NonEmptyVec, vec_nonempty};
    /// let mut compile_error = NonEmptyVec::new(vec![1, 2, 3]).unwrap();
    ///
    /// // slice mutation - allowed
    /// compile_error[0] = 5;
    ///
    /// assert_eq!(compile_error, vec_nonempty![5, 2, 3]);
    /// ```
    ///
    /// Using mutable `Vec` methods is forbidden (slice mutation shown above is fine)
    /// ```compile_fail
    /// # use advent_2023::{nonempty::NonEmptyVec, vec_nonempty};
    /// # let mut compile_error = NonEmptyVec::new(vec![1, 2, 3]).unwrap();
    /// #
    /// # // slice mutation - allowed
    /// # compile_error[0] = 5;
    /// #
    /// # assert_eq!(compile_error, vec_nonempty![5, 2, 3]);
    /// // Vec mutation - disallowed
    /// compile_error.remove(0);
    /// ```
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonEmptyVec<T>(Vec<T>);

    impl<T> NonEmptyVec<T> {
        pub fn new(inner: Vec<T>) -> Option<Self> {
            if inner.is_empty() {
                None
            } else {
                Some(Self(inner))
            }
        }

        pub fn first(&self) -> &T {
            self.0.first().expect("nonempty")
        }

        pub fn split_first(&self) -> (&T, &[T]) {
            self.0.split_first().expect("nonempty")
        }

        pub fn last(&self) -> &T {
            self.0.last().expect("nonempty")
        }

        pub fn into_split_last(mut self) -> (Vec<T>, T) {
            let last = self.0.pop().expect("nonempty");
            (self.0, last)
        }

        pub fn map<U>(self, map_fn: impl Fn(T) -> U) -> NonEmptyVec<U> {
            let vec = self.0.into_iter().map(map_fn).collect();
            NonEmptyVec::new(vec).expect("nonempty after map")
        }
    }
    // NOTE: Mutable access ONLY allowed as a slice
    impl<T> std::ops::Deref for NonEmptyVec<T> {
        type Target = [T]; // explicitly *NOT* type Target = Vec<T>;
        fn deref(&self) -> &Self::Target {
            &self.0[..]
        }
    }
    impl<T> std::ops::DerefMut for NonEmptyVec<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0[..]
        }
    }
    impl<T> std::iter::IntoIterator for NonEmptyVec<T> {
        type Item = T;
        type IntoIter = <Vec<T> as std::iter::IntoIterator>::IntoIter;
        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for NonEmptyVec<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Vec<T> as std::fmt::Debug>::fmt(&self.0, f)
        }
    }

    #[macro_export]
    macro_rules! vec_nonempty {
        ($elem:expr; $n:expr) => {{
            let _compile_time_assert = match $n {
                0 => [][0],
                _ => {}
            };
            let v = vec![$elem; $n];
            $crate::nonempty::NonEmptyVec::new(v).expect("nonempty via macro")
        }};
        ($($x:expr),+ $(,)?) => {{
            let v = vec![$($x),+];
            $crate::nonempty::NonEmptyVec::new(v).expect("nonempty via macro")
        }};
    }
}
