// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2022.

//! License-header checking tool for the Tock project.
//!
//! # Description
//! This tool recursively traverses through the current working directory,
//! verifying that every source code file inside has a Tock project license
//! header.
//!
//! # Ignore files
//! This tool respects gitignore files with the following names (ordered from
//! highest-precedence to lowest-precedence):
//! 1. .lcignore
//! 2. .ignore
//! 3. .gitignore

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::process::exit;

mod parser;
use parser::{Cache, LineContents, ParseError, Parser};

const LICENSED_LINE: &str = "Licensed under the Apache License, Version 2.0 or the MIT License.";
const SPDX_LINE: &str = "SPDX-License-Identifier: Apache-2.0 OR MIT";

fn is_first(comment: &str) -> bool {
    comment.starts_with("Licensed under ")
}
fn is_spdx(comment: &str) -> bool {
    comment.starts_with("SPDX-License-Identifier:")
}
fn is_copyright(comment: &str) -> bool {
    comment.starts_with("Copyright ")
}

#[derive(clap::Parser)]
struct Args {
    /// Enable verbose debugging output
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Debug, thiserror::Error, PartialEq)]
enum LicenseError {
    #[error("license header missing")]
    Missing,

    #[error("missing blank line after header")]
    MissingBlank,

    #[error("missing copyright line")]
    MissingCopyright,

    #[error("missing SPDX line")]
    MissingSpdx,

    #[error("incorrect first line")]
    WrongFirst,

    #[error("wrong SPDX line")]
    WrongSpdx,
}

#[derive(Debug, PartialEq)]
struct ErrorInfo {
    file: PathBuf,
    line_num: u64,
    error: LicenseError,
}

#[derive(PartialEq)]
enum State {
    /// We need a blank line before the header can start. This state is entered
    /// if there is a non-blank, non-license-header line before the license
    /// header.
    NeedBlank,

    /// We have not yet found a header, and are ready for one. This is the
    /// starting (top-of-file) state, and is re-entered after a blank line if a
    /// header has not been found.
    ReadyForHeader,

    /// We have found the first line of the header and the next must be the SPDX
    /// line.
    NeedSpdx,

    /// We have found the first and SPDX line and now need a copyright line.
    NeedCopyright,

    /// We have found at least one copyright and are now waiting for the header
    /// to end.
    WaitForEnd,

    /// The complete header (with or without errors) has been found, and we do
    /// not need to continue processing this file.
    Done,
}

fn check_file(cache: &Cache, path: &Path) -> Vec<ErrorInfo> {
    use LicenseError::*;
    use LineContents::*;
    use State::*;

    let mut license_errors = vec![];
    let mut parser = match Parser::new(cache, path) {
        Err(ParseError::Binary) => return vec![],
        Err(error) => panic!("{}: {}", path.display(), error),
        Ok(parser) => parser,
    };
    let mut line_num = 0;
    let mut state = ReadyForHeader;
    while state != Done {
        line_num += 1;
        let line_contents = match parser.next() {
            Err(ParseError::Binary) => return vec![],
            // Coerce end-of-file into Other, as they are treated identically.
            Err(ParseError::Eof) => Other,
            Err(error) => panic!("Parse error at {}:{}: {}", path.display(), line_num, error),
            Ok(contents) => contents,
        };
        let (new_state, error) = match (state, line_contents) {
            (NeedBlank, Comment(_)) => (NeedBlank, None),
            (NeedBlank, Whitespace) => (ReadyForHeader, None),
            (NeedBlank, Other) => (Done, Some(Missing)),
            (ReadyForHeader, Comment(comment)) if !is_first(comment) => (NeedBlank, None),
            (ReadyForHeader, Comment(comment)) if comment == LICENSED_LINE => (NeedSpdx, None),
            (ReadyForHeader, Comment(_)) => (NeedSpdx, Some(WrongFirst)),
            (ReadyForHeader, Whitespace) => (ReadyForHeader, None),
            (ReadyForHeader, Other) => (Done, Some(Missing)),
            (NeedSpdx, Comment(comment)) if comment == SPDX_LINE => (NeedCopyright, None),
            (NeedSpdx, Comment(comment)) if is_spdx(comment) => (NeedCopyright, Some(WrongSpdx)),
            (NeedSpdx, _) => (Done, Some(MissingSpdx)),
            (NeedCopyright, Comment(comment)) if is_copyright(comment) => (WaitForEnd, None),
            (NeedCopyright, _) => (Done, Some(MissingCopyright)),
            (WaitForEnd, Comment(comment)) if is_copyright(comment) => (WaitForEnd, None),
            (WaitForEnd, Comment("")) => (Done, None),
            (WaitForEnd, Whitespace) => (Done, None),
            (WaitForEnd, _) => (Done, Some(MissingBlank)),
            (Done, _) => unreachable!("Loop didn't end at EOF"),
        };
        state = new_state;
        if let Some(error) = error {
            license_errors.push(ErrorInfo {
                file: path.to_owned(),
                line_num,
                error,
            });
        }
    }
    license_errors
}

fn main() {
    use clap::Parser as _;
    let args = Args::parse();
    let cache = &Cache::default();
    let fs_walk = WalkBuilder::new("./")
        .add_custom_ignore_filename(".lcignore")
        .git_exclude(false)
        .git_global(false)
        .hidden(false)
        .require_git(false)
        .build();

    let mut failed = false;
    for result in fs_walk {
        let dir_entry = result.expect("Directory walk failed");
        let file_type = dir_entry.file_type().expect("File type read failed");
        if !file_type.is_file() {
            continue;
        }
        if args.verbose {
            println!("Checking {}", dir_entry.path().display());
        }
        for error_info in check_file(cache, dir_entry.path()) {
            failed = true;
            eprintln!(
                "{}:{}: {}",
                error_info.file.display(),
                error_info.line_num,
                error_info.error
            );
        }
    }

    if !failed {
        println!("License check passed.");
        return;
    }
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trailing_comment() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/blank_is_comment.rs")),
            []
        );
    }

    #[test]
    fn many_errors() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/many_errors.rs")),
            [
                ErrorInfo {
                    file: "testdata/many_errors.rs".into(),
                    line_num: 1,
                    error: LicenseError::WrongFirst,
                },
                ErrorInfo {
                    file: "testdata/many_errors.rs".into(),
                    line_num: 2,
                    error: LicenseError::WrongSpdx,
                },
                ErrorInfo {
                    file: "testdata/many_errors.rs".into(),
                    line_num: 5,
                    error: LicenseError::MissingBlank,
                },
            ]
        );
    }

    #[test]
    fn missing() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/error_missing.rs")),
            [ErrorInfo {
                file: "testdata/error_missing.rs".into(),
                line_num: 1,
                error: LicenseError::Missing
            }]
        );
    }

    #[test]
    fn missing_copyright() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/no_copyright.rs")),
            [ErrorInfo {
                file: "testdata/no_copyright.rs".into(),
                line_num: 3,
                error: LicenseError::MissingCopyright
            }]
        );
    }

    #[test]
    fn missing_spdx() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/no_spdx.rs")),
            [ErrorInfo {
                file: "testdata/no_spdx.rs".into(),
                line_num: 2,
                error: LicenseError::MissingSpdx
            }]
        );
    }

    /// Run check_file on a file that should have a valid header. Note this file
    /// has a shebang line, so it will have to search past the first line to
    /// find the header.
    #[test]
    fn successful() {
        assert_eq!(
            check_file(&Cache::default(), Path::new("testdata/by_first_line")),
            []
        );
    }
}
