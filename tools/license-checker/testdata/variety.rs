// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Google LLC 2022.

// Syntect should recognize this file's type by extension.

/// Single-line doc comment. The next line is a comment with no contents.
//

/* Multi-line comment. The next comment contains only whitespace.
 */
//     

/** Multi-line doc comment. The line after this comment contains whitespace.
  */
    

#![rustfmt::skip]  // This line should be considered "other" (i.e. code)
