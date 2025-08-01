// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2022.

//! A partial parser that parses source code files *just well enough* to find
//! license headers. `Parser` needs some slow-to-initialize resources, so each
//! `Parser` must be initialized with a reference to a `Cache` instance.
//!
//! It is built on top of the [`syntect`](https://crates.io/crates/syntect)
//! crate. `syntect` is designed to perform syntax highlighting for text
//! editors, and can therefore parse a variety of common languages. However, it
//! cannot parse every language present in the Tock project. For languages that
//! `syntect` does not have a definition for, we use a fallback syntax, defined
//! in `fallback_syntax.yaml`.

use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use syntect::easy::ScopeRangeIterator;
use syntect::highlighting::ScopeSelector;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::parsing::{ParseState, ParsingError, ScopeError, ScopeStack, SyntaxSet};

const FALLBACK_NAME: &str = "fallback";

pub struct Cache {
    is_comment: ScopeSelector,
    is_punctuation: ScopeSelector,
    // Syntect's plain text syntax does not mark anything as comments, which
    // causes the license check to fail. Instead, plain text files should use
    // the fallback parser. This stores the name of the plain text syntax, so
    // that Parser can detect when syntect has returned the plain text syntax
    // for a file.
    plain_text_name: String,
    syntax_set: SyntaxSet,
}

impl Default for Cache {
    fn default() -> Self {
        const COMMENT_SELECTOR: &str =
            "comment - comment.block.documentation - comment.line.documentation";
        const FALLBACK_SYNTAX: &str = include_str!("fallback_syntax.yaml");
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        builder.add(
            SyntaxDefinition::load_from_str(FALLBACK_SYNTAX, true, Some(FALLBACK_NAME))
                .expect("Failed to parse fallback syntax"),
        );
        let syntax_set = builder.build();
        Self {
            is_comment: ScopeSelector::from_str(COMMENT_SELECTOR).unwrap(),
            is_punctuation: ScopeSelector::from_str("punctuation.definition.comment").unwrap(),
            plain_text_name: syntax_set.find_syntax_plain_text().name.clone(),
            syntax_set,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[rustfmt::skip]
pub enum ParseError {
    #[error("binary file")]     Binary,
    #[error("end-of-file")]     Eof,
    #[error("bad byte span")]   BadSpan,
    #[error("io error {0}")]    IoError(#[from] io::Error),
    #[error("parse error {0}")] ParsingError(#[from] ParsingError),
    #[error("scope error {0}")] ScopeError(#[from] ScopeError),
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
    /// This line of code consists only of whitespace and one comment. The
    /// contained string slice points to the contents of the comment with
    /// whitespace trimmed away.
    Comment(&'l str),

    /// This line is not a comment and contains only whitespace.
    Whitespace,

    /// This line contains more than one comment (e.g. /* A */ /* B */) or text
    /// that is neither whitespace nor part of a comment.
    Other,
}

pub struct Parser<'cache> {
    cache: &'cache Cache,
    line: String,
    parse_state: ParseState,
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
            Ok(Some(syntax)) if syntax.name != cache.plain_text_name => syntax,
            Ok(_) => cache.syntax_set.find_syntax_by_name(FALLBACK_NAME).unwrap(),
        };

        Ok(Self {
            cache,
            line: String::new(),
            parse_state: ParseState::new(syntax),
            reader: BufReader::new(File::open(path)?),
            scopes: ScopeStack::new(),
        })
    }

    /// Parses the next line and returns its contents. Returns
    /// Err(ParseError::Eof) at EOF.
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

        // Comments are assumed to take the following overall form, using SGML
        // as a pathological example:
        //
        //     <!-- -- -- -- ------ Hello, world! -------- -- -- -- -- -->
        // |--Opening punctuation--|--Body text--|--Closing punctuation--|
        //
        // The opening punctuation and closing punctuation are both optional --
        // e.g. line comments do not have closing punctuation. The opening and
        // closing punctuation consist of a mix of whitespace and non-whitespace
        // punctuation characters.
        //
        // Yes, this means that the boundary between opening punctuation and the
        // body text is vague if that boundary consists of whitespace. That is
        // fine because the comment body will be trimmed, so the final return is
        // deterministic.
        enum State {
            // All text processed so far has been whitespace and not part of a
            // comment.
            WhitespaceOnly,
            // The loop has processed comment-begin punctuation (#, //, /*)
            // and/or comment whitespace but no comment text.
            CommentStart,
            // The loop has processed comment body text. The range is the byte
            // span of the comment body, possibly with some whitespace trimmed
            // off.
            CommentBody(Range<usize>),
            // The loop has processed comment-end punctuation (*/). The range is
            // the same as for CommentBody.
            CommentEnd(Range<usize>),
            // This line fits the criteria of LineContents::Other.
            Other,
        }

        let cache = self.cache;
        let ops = self.parse_state.parse_line(&self.line, &cache.syntax_set)?;
        let mut state = match cache.is_comment.does_match(self.scopes.as_slice()) {
            None => State::WhitespaceOnly,
            Some(_) => State::CommentStart,
        };

        for (span, op) in ScopeRangeIterator::new(&ops, &self.line) {
            self.scopes.apply(op)?;
            // Syntaxes sometimes push and pop spurious scopes; don't bother
            // checking the scopes until we have a non-empty span.
            if span.is_empty() {
                continue;
            }

            // Classification applied to each span based on what scopes apply.
            enum FragmentType {
                Whitespace, // Whitespace OUTSIDE a comment.
                CommentPunctuation,
                CommentText,
                Other,
            }
            let scopes = self.scopes.as_slice();
            let span_str = self.line.get(span.clone()).ok_or(ParseError::BadSpan)?;
            let fragment_type = match cache.is_comment.does_match(scopes) {
                None => match span_str.chars().all(char::is_whitespace) {
                    true => FragmentType::Whitespace,
                    false => FragmentType::Other,
                },
                Some(_) => match cache.is_punctuation.does_match(scopes) {
                    Some(_) => FragmentType::CommentPunctuation,
                    None => FragmentType::CommentText,
                },
            };
            state = match (state, fragment_type) {
                (_, FragmentType::Other) => State::Other,
                (State::WhitespaceOnly, FragmentType::Whitespace) => State::WhitespaceOnly,
                (State::WhitespaceOnly, FragmentType::CommentPunctuation) => State::CommentStart,
                (State::WhitespaceOnly, FragmentType::CommentText) => State::CommentBody(span),
                (State::CommentStart, FragmentType::CommentText) => State::CommentBody(span),
                (State::CommentStart, _) => State::CommentStart,
                (State::CommentBody(old_span), FragmentType::CommentPunctuation) => {
                    State::CommentEnd(old_span)
                }
                (State::CommentBody(old_span), _) if span.start == old_span.end => {
                    State::CommentBody(old_span.start..span.end)
                }
                (State::CommentBody(_), _) => State::Other, // Disjointed comments
                (State::CommentEnd(_), FragmentType::CommentText) => State::Other,
                (State::CommentEnd(old_span), _) => State::CommentEnd(old_span),
                (State::Other, _) => State::Other,
            };
        }

        Ok(match state {
            State::WhitespaceOnly => LineContents::Whitespace,
            State::CommentStart => LineContents::Comment(""),
            State::CommentBody(span) => LineContents::Comment(self.line[span].trim()),
            State::CommentEnd(span) => LineContents::Comment(self.line[span].trim()),
            State::Other => LineContents::Other,
        })
    }
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
    fn fallback_block_comments() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Whitespace,
            Comment(""),
            Comment("This file contains two styles of block comment."),
            Comment(""),
        ];
        let path = Path::new("testdata/block_comments.ld");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    #[test]
    fn fallback_number_signs() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Whitespace,
            Other,
            Other,
            Other,
            Comment("Lines starting with a '#' without a space are comments too."),
        ];
        let path = Path::new("testdata/number_signs.fallback");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    #[test]
    fn fallback_shebang() {
        const EXPECTED: &[LineContents] = &[
            Comment("Shebang line here"),
            Whitespace,
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Whitespace,
            Other,
            Other,
            Other,
        ];
        let path = Path::new("testdata/shebang.fallback");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    #[test]
    fn fallback_slashes() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Whitespace,
            Other,
            Other,
            Other,
        ];
        let path = Path::new("testdata/slashes.fallback");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    #[test]
    fn plain_text_no_extension() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Comment(""),
            Comment("This is a plain text file; the license checker should recognize that it does"),
            Comment("# not use a common comment syntax."),
        ];
        let path = Path::new("testdata/plain_text_no_extension");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }

    #[test]
    fn plain_text_txt() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2024."),
            Comment("Copyright Google LLC 2024."),
            Comment(""),
            Comment("This is a plain text file with the .txt extension. The license checker"),
            Comment("should use the fallback parser, even though syntect may recognize it as a"),
            Comment("plain text file."),
        ];
        let path = Path::new("testdata/plain_text.txt");
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
            Comment("Multi-line comment. The next comment contains only whitespace."),
            Comment(""),
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

    #[test]
    fn xml_block_comments() {
        const EXPECTED: &[LineContents] = &[
            Comment("Licensed under the Apache License, Version 2.0 or the MIT License."),
            Comment("SPDX-License-Identifier: Apache-2.0 OR MIT"),
            Comment("Copyright Tock Contributors 2023."),
            Comment("Copyright Google LLC 2023."),
            Whitespace,
            Comment(""),
            Comment("This file contains two styles of block comment."),
            Comment(""),
        ];
        let path = Path::new("testdata/block_comments.xml");
        assert_produces(Parser::new(&Cache::default(), path), EXPECTED);
    }
}
