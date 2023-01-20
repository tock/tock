// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2022.

//! A partial parser that parses source code files *just well enough* to find
//! license headers.
//!
//! `Parser` needs some slow-to-initialize resources, so each `Parser` must be
//! initialized with a reference to a `Cache` instance.

// It is not obvious how we should handle having multiple comments on one line,
// e.g.:
//
//     /* Comment A */ /* Comment B */ // Comment C
//
// To avoid answering that question, this parser only recognizes line comments.
// That effectively requires license headers to be before any block comments. If
// that is an issue, we can adapt this parser to read block comments as well.

use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::Path;
use std::str::FromStr;
use syntect::easy::ScopeRangeIterator;
use syntect::highlighting::ScopeSelector;
use syntect::parsing::{ParseState, ParsingError, ScopeError, ScopeStack, SyntaxSet};

pub struct Cache {
    is_comment: ScopeSelector,
    is_punctuation: ScopeSelector,
    syntax_set: SyntaxSet,
}

impl Default for Cache {
    fn default() -> Self {
        const COMMENT_SELECTOR: &str = "comment.line - comment.line.documentation";
        Self {
            is_comment: ScopeSelector::from_str(COMMENT_SELECTOR).unwrap(),
            is_punctuation: ScopeSelector::from_str("punctuation").unwrap(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Returned when we discover the file is binary (not valid UTF-8)
    #[error("binary file")]
    Binary,

    /// Returned at the end of the file.
    #[error("end-of-file")]
    Eof,

    #[error("bad byte span")]
    BadSpan,

    #[error("io error {0}")]
    IoError(#[from] io::Error),

    #[error("parse error {0}")]
    ParsingError(#[from] ParsingError),

    #[error("scope error {0}")]
    ScopeError(#[from] ScopeError),
}

impl PartialEq for ParseError {
    fn eq(&self, rhs: &Self) -> bool {
        use ParseError::*;
        match (self, rhs) {
            (Binary, Binary) => true,
            (Eof, Eof) => true,
            (BadSpan, BadSpan) => true,
            _ => false,
        }
    }
}

/// Indicates what a particular line of source code contains.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineContents<'l> {
    /// This line of code consists only of whitespace and comments. The
    /// contained string slice points to the contents of the comment with
    /// whitespace trimmed away.
    Comment(&'l str),

    /// This line only contains whitespace.
    Whitespace,

    /// This line contains something that is not a comment or whitespace.
    Other,
}

pub struct Parser<'cache> {
    cache: &'cache Cache,
    line: String,
    parse_state: Option<ParseState>,
    reader: BufReader<File>,
    scopes: ScopeStack,
}

impl<'cache> Parser<'cache> {
    /// Creates a new `Parser` that reads the specified file. Will return None
    /// if the file is binary.
    pub fn new(cache: &'cache Cache, path: &Path) -> Result<Self, ParseError> {
        let syntax = match cache.syntax_set.find_syntax_for_file(path) {
            Err(error) if error.kind() == ErrorKind::InvalidData => return Err(ParseError::Binary),
            Err(error) => return Err(error.into()),
            Ok(syntax) => syntax,
        };

        Ok(Self {
            cache,
            line: String::new(),
            parse_state: syntax.map(ParseState::new),
            reader: BufReader::new(File::open(path)?),
            scopes: ScopeStack::new(),
        })
    }

    /// Parses the next line and returns its contents. Returns None at EOF.
    pub fn next(&mut self) -> Result<LineContents, ParseError> {
        self.line.clear();
        let bytes_read = match self.reader.read_line(&mut self.line) {
            Err(error) if error.kind() == ErrorKind::InvalidData => return Err(ParseError::Binary),
            Err(error) => return Err(error.into()),
            Ok(bytes) => bytes,
        };
        if bytes_read == 0 {
            // End of file.
            return Err(ParseError::Eof);
        }

        // Manual comment extraction for file types syntect doesn't recognize.
        let Some(ref mut parse_state) = self.parse_state else {
            return Ok(parse_unknown(&self.line));
        };

        let cache = self.cache;
        let ops = parse_state.parse_line(&self.line, &cache.syntax_set)?;
        let mut contents = None;
        let mut has_comment = false;

        for (byte_span, op) in ScopeRangeIterator::new(&ops, &self.line) {
            self.scopes.apply(op)?;
            // Shortcut execution when we've already classified this line.
            if contents.is_some() {
                continue;
            }

            // Syntaxes sometimes push and pop spurious scopes; don't bother
            // checking the scopes until we have a non-empty span.
            if byte_span.is_empty() {
                continue;
            }

            let scopes = self.scopes.as_slice();
            let span = self.line.get(byte_span).ok_or(ParseError::BadSpan)?;
            if cache.is_comment.does_match(scopes).is_none() {
                if !span.chars().all(char::is_whitespace) {
                    contents = Some(LineContents::Other);
                }
                continue;
            }
            has_comment = true;
            // Skip the comment's punctuation (e.g. "//")
            if cache.is_punctuation.does_match(scopes).is_some() {
                continue;
            }

            contents = Some(LineContents::Comment(span.trim()));
        }

        Ok(match (contents, has_comment) {
            (None, false) => LineContents::Whitespace,
            (None, true) => LineContents::Comment(""),
            (Some(contents), _) => contents,
        })
    }
}

// Backup parser for file types that syntect doesn't recognize. Strips "# " and
// "// " comment prefixes and assumes every no-whitespace line is a comment.
fn parse_unknown(line: &str) -> LineContents {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineContents::Whitespace;
    }
    LineContents::Comment(if let Some(comment) = trimmed.strip_prefix("# ") {
        comment
    } else if let Some(comment) = trimmed.strip_prefix("// ") {
        comment
    } else {
        trimmed
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use LineContents::*;

    // Test function that confirms the parser produces a particular sequence of
    // LineContents.
    #[track_caller]
    fn assert_produces(parser: Result<Parser, ParseError>, sequence: &[LineContents]) {
        let mut parser = parser.unwrap();
        for &expected in sequence {
            assert_eq!(parser.next(), Ok(expected));
        }
        assert_eq!(parser.next(), Err(ParseError::Eof));
    }

    // Test with a file that causes SyntaxSet::find_syntax_for_file to return an
    // invalid data error (which binary files can cause).
    #[test]
    fn binary() {
        use ParseError::Binary;
        let binary = Path::new("testdata/binary");
        let cache = &Cache::default();
        assert!(matches!(Parser::new(cache, binary), Err(Binary)));
    }

    // Confirm Parser correctly processes files identified by their shebang
    // lines if their extension is unknown.
    #[test]
    fn by_first_line() {
        const EXPECTED: &[LineContents] = &[
            Comment("!/bin/bash"),
            Whitespace,
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2022."),
            Comment("Copyright Google LLC 2022."),
            Whitespace,
            Other,
            Other,
        ];
        let by_first_line = Path::new("testdata/by_first_line");
        assert_produces(Parser::new(&Cache::default(), by_first_line), EXPECTED);
    }

    #[test]
    fn fallback_parser() {
        assert_eq!(parse_unknown(" \t "), Whitespace);
        assert_eq!(parse_unknown("# Hash comment \n"), Comment("Hash comment"));
        assert_eq!(
            parse_unknown("// Slashes comment \n"),
            Comment("Slashes comment")
        );
        assert_eq!(parse_unknown("Plain text \n"), Comment("Plain text"));
    }

    // Test with a file type that syntect doesn't recognize.
    #[test]
    fn unknown_file_type() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2022."),
            Comment("Copyright Google LLC 2022."),
            Whitespace,
            Comment("Syntect should not be able to recognize this file's type."),
            Comment("Parser should adapt and automatically strip // and # comment prefixes."),
        ];
        let path = Path::new("testdata/source.unknown_file_type");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    // Test Parser with a variety of line types. This also confirms that syntect
    // can identify a file type by extension.
    #[test]
    fn various_line_types() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2022."),
            Comment("Copyright Google LLC 2022."),
            Whitespace,
            Comment("Syntect should recognize this file's type by extension."),
            Whitespace,
            Other,
            Comment(""),
            Whitespace,
            Other,
            Other,
            Comment(""),
            Whitespace,
            Other,
            Other,
            Whitespace,
            Whitespace,
            Other,
        ];
        let path = Path::new("testdata/variety.rs");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }
}
