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

pub struct CharScanner<'a> {
    line: &'a str,
    char_indices: std::iter::Peekable<std::str::CharIndices<'a>>,
    last_indices: VecDeque<usize>,
    lookback_range: Option<(usize, usize)>,
}

impl<'a> CharScanner<'a> {
    /// Creates a scanner with the specified lookback range
    ///
    /// `lookback_range` - if specified, tuple: (min_length, max_length) where [`Self::find_next`]
    ///                    will call `f_lookback_str` for string lengths `min_length..=max_length`
    pub fn new(line: &'a str, lookback_range: Option<(usize, usize)>) -> Self {
        let char_indices = line.char_indices().peekable();

        let lookback_capacity = lookback_range.map(|(_min, len)| len + 1).unwrap_or(0);

        Self {
            line,
            char_indices,
            last_indices: VecDeque::with_capacity(lookback_capacity),
            lookback_range,
        }
    }
    /// Returns the next element matched by a matching function
    ///
    /// `f_single_char` - Function accepting single character (and char index within the line)
    /// `f_lookback_str` - Function accepting a lookback string, with char indices `(start, end)` corresponding to `&line[start..end]`
    pub fn find_next<T>(
        &mut self,
        f_single_char: Option<impl Fn(char, usize) -> Option<T>>,
        f_lookback_str: Option<impl Fn(&str, (usize, usize)) -> Option<T>>,
    ) -> Option<T> {
        let lookback_indices = &self.lookback_range.and_then(|(min_len, max_len)| {
            min_len.checked_sub(1).map(|min_index| min_index..max_len)
        });
        if f_lookback_str.is_some() {
            assert!(
                lookback_indices.is_some(),
                "f_lookback_str required lookback_range to be specified in constructor"
            );
        }

        while let Some((current_index, current_char)) = self.char_indices.next() {
            let next_index = self.char_indices.peek().map(|(index, _char)| index);
            let lookback_truncate = self.lookback_range.map(|(_min, len)| len - 1).unwrap_or(0);
            self.last_indices.truncate(lookback_truncate);
            self.last_indices.push_front(current_index);

            if let Some(f_single_char) = &f_single_char {
                if let Some(result) = f_single_char(current_char, current_index) {
                    self.last_indices.clear();
                    return Some(result);
                }
            }

            for lookback in lookback_indices.clone().unwrap_or(0..0) {
                if let Some(&start_index) = self.last_indices.get(lookback) {
                    let end_index = next_index.map_or(self.line.len(), |&idx| idx);
                    let last_part = self
                        .line
                        .get(start_index..end_index)
                        .expect("slicing on char boundaries");

                    if let Some(f_lookback_str) = &f_lookback_str {
                        if let Some(result) = f_lookback_str(last_part, (start_index, end_index)) {
                            return Some(result);
                        }
                    }
                } else {
                    break;
                }
            }
        }
        // exhausted char_indices
        None
    }
}
