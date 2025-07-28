Tock Style
==========

This document overviews some stylistic conventions that Tock generally adheres
to.

<!-- npm i -g markdown-toc; markdown-toc -i Style.md -->

<!-- toc -->

- [Code Style](#code-style)
  * [Assembly Code Style](#assembly-code-style)
- [Commenting](#commenting)
  * [Example: mycapsule.rs](#example-mycapsulers)
- [Using Descriptive Names](#using-descriptive-names)

<!-- tocstop -->

## Code Style

Tock uses [rustfmt](https://github.com/rust-lang/rustfmt) for source code style
and formatting. In general, all of Tock's code is formatted according to the
`rustfmt` defaults. There are a few exceptions, but these should generally be
avoided.

### Assembly Code Style

As of July 7, 2025, [rustfmt](https://github.com/rust-lang/rustfmt) does not
rigidly format assembly code blocks and assembly macros. To compensate, Tock
defines its own format. If, in the future, rustfmt does rigidly format assembly
code block and macros we will switch to that format.

Tock's format is:

```text
{
    naked_asm!(         // macro indented with the relevant code block as usual
        "               // opening quote on next line, indented
    inst r1, r2         // assembly instructions always indented exactly four spaces
    inst r3, r4
    // Comment.         // comments always use "//" style
    inst r5, r6

label:                  // labels always indented exactly zero spaces
    inst r1, r2
        ",              // closing quote always aligned with opening quote
        out("r10") _,   // additional arguments aligned with quotes
        out("r11") _,   // exactly one argument per line
    );                  // closing brace aligned with macro
}
```

Notes:

1. One-line assembly blocks may be formatted on a single line.
2. Assembly instructions are always indented four spaces, even if the macro is
   indented more due to nesting. For example, an `asm!()` macro doubly nested:
    ```
    {
        {
            asm!(
                "
        inst r1, r2
        inst r2, r3
                "
            );
        }
    }
    ```
3. If the assembly macro only has one argument (i.e., the instructions), there
   is no comma after the closing quote.


## Commenting

Rust includes three types of comments: `//`, `///`, and `//!`. Tock uses all
three in line with their usage in Rust code more generally.

- `//`: Two slashes are used for "internal" comments that document specific
  details about certain code, leave notes for other developers, or specify
  internal metadata like the primary author of a file. These comments are only
  visible in the current file and are not used for documentation generation.

- `///`: Three slashes are used to specify public documentation about data
  structures and functions. These comments generally describe what a certain
  element does and how to use it. All `///` comments are used to automatically
  generate API documentation and will be shared outside of the file they are
  written in. In general, every public function or object should have a `///`
  comment.

- `//!`: Two slashes and a bang are used for document-level comments. These
  comments are only used at the top of a file to provide an overview of all of
  the code contained in the file. Typically these comments also include a
  general usage example. These comments will also be used for automatic
  documentation generation.

  The first line of a `//!` comment will be used as a descriptive tagline, and
  as such should be short and provide essentially a subtitle for the code file
  (where the file name acts as the title). Generally the first line should be
  no more than 80 characters. To identify the tagline, the second line of the
  comment should just be `//!` with no other text.

Both `///` and `//!` comments support Markdown.

### Example: mycapsule.rs

```rust
//! Prints "hello" on boot.
//!
//! This simple capsule implements hello world by printing a message when it
//! is initialized.
//!
//! Usage
//! -----
//!
//! ```
//! let helloworld = mycapsule::HelloWorld::new();
//! helloworld.initialize();
//! ```

/// This struct contains the resources necessary for the Hello World example
/// module. Boards should create this to run the hello world example.
struct HelloWorld {
    ...
}

impl HelloWorld {
    /// Start the hello world example and print out "Hello World".
    ///
    /// This should only be called after the debugging module is setup.
    // Someday we should use a UART directly, but that can be implemented later.
    fn initialize () {
        debug!("Hello World");
    }
}
```

## Using Descriptive Names

Tock generally tries to avoid abbreviations in variable and object names, and
instead use descriptive and clear names. This helps new readers of the code
understand what different elements are doing. Plus, `rustfmt` helps with
formatting the code when using the longer names, and Github does not charge us
by the character.

- `ArrayIdx` ⇨ `ArrayIndex`
- `BtnInterrupt` ⇨ `ButtonInterrupt`
- `RegVoltOut` ⇨ `RegulatedVoltageOutput`
- `GPIO.low_power()` ⇨ `GPIO.deactivate_and_make_low_power()`
