Licensing and Copyrights
========================================

**TRD:** <br/>
**Working Group:** Core<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Pat Pannuto<br/>
**Draft-Created:** 2022/11/05 <br/>
**Draft-Modified:** 2022/11/05 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** tock-dev@googlegroups.com<br/>

Abstract
--------

This document describes Tock’s policy on licensing and copyright. It explains
the rationale behind the license selection (dual-license, MIT or Apache2) and
copyright policy (optional, and authors may retain copyright if desired). It
further outlines how licensing and copyright shall be handled throughout
projects under the Tock umbrella.

Explicitly not discussed in this TRD are issues of trademark, the Tock brand,
logos, or other non-code assets and artifacts.


1. Introduction
===============

Tock’s goal is to provide a safe, secure, and efficient operating system for
microcontroller-class devices. The Tock project further believes it is
important to enable and support widespread adoption of safe, secure, and
efficient software. Tock also seeks to be an open and inclusive project, and
Tock welcomes contributions from any individuals or entities wishing to
improve the safety, security, reliability, efficiency, or usability of Tock
and the Tock ecosystem.

The intent of these policies is to best satisfy the needs of all stakeholders
in the Tock ecosystem.


2. Licensing
============

Tock’s primary licensing goal is to ensure ease of use for Tock project elements.
For this reason, Tock offers its resources under the minimally encumbered MIT
license and under the Apache2 license for those entities concerned about possible
litigation issues.

All software artifacts under the umbrella of the Tock project are
dual-licensed as Apache License, Version 2.0 (LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0) or MIT license (LICENSE-MIT or
http://opensource.org/licenses/MIT).

All contributions to Tock projects MUST be licensed under these terms.


3. Copyright
============

Entities contributing resources to open-source projects often require attribution
to recognize their efforts. Copyright is a common means to provide this. For
downstream users, the license terms of Tock ensures unencumbered use.

Copyrights in Tock projects are retained by their contributors. No
copyright assignment is required to contribute to Tock projects.

Artifacts in the Tock project MAY include explicit copyright notices.
Substantial updates to an artifact MAY add additional copyright notices
to an artifact. Copyright notices MUST NOT be removed by any party except
the original copyright holder, and removal is done solely at their discretion.

For full authorship information, see the version control history.


4. Implementation
=================

All code artifacts in Tock projects MUST include a license notice.

Code artifacts MAY include copyright notice(s). Copyright notices MUST
include a year. Newer copyright notices MUST be placed after existing
copyright notices. If non-trivial updates are performed by an original
copyright author, they MAY amend the year(s) indicated on their existing
copyright statement or MAY add an additional copyright line, at their
discretion.


4.1 Format
----------

License and copyright information MUST appear near the top of a file.
Generally, license and copyright information SHOULD be the first lines
in a file, however exceptions may be made where technically necessary
(e.g. to accommodate the shebang in a shell script).

License and copyright information MUST have exactly one (1) blank line
separating it from any other content in the file.

License information MUST come before copyright information.
There MUST NOT be any whitespace between lines in the license/copyright block.

Text described in this section MAY be pre-fixed or post-fixed with
technically necessary characters (i.e. to mark as a comment in source
code) as appropriate.

The first line of license text MUST appear exactly as-follows:

> Licensed under the Apache License, Version 2.0 or the MIT License.

The second line of license MUST adhere to the [SPDX](https//spdx.dev)
specification for license description. As of this writing, it SHOULD
appear as-follows:

> SPDX-License-Identifier: Apache-2.0 OR MIT

The [current (v2.3) normative rules](https://spdx.github.io/spdx-spec/v2.3/SPDX-license-expressions/)
permit case-insensitive matches of the license identifier but require
case-sensitive matching of the disjunction operator. To simplify
enforcement of licensing and documentation rules, Tock dictates that
license information MUST preserve case as-shown in the SPDX license
list (i.e. as-presented above).

Copyright lines MUST follow this pattern:

> Copyright {entity} {year}([,year],[-year]).

The `{entity}` field is REQUIRED and should reflect the entity wishing
to claim copyright. The `{year}` field is REQUIRED and should reflect
when the copyright is first established. Substantial updates in the
future MAY indicate renewed copyright, via additional comma-separated
years or via range syntax, at the copyright holder’s discretion. The
initial year SHALL NOT be removed unless it is the express intent of
the copyright holder to relinquish the initial copyright.


### 4.1.1 Examples

The common-case format is the top of a file with only license information:

```rust
// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Module-level documentation...
```

A file with a long history and multiple copyrights may look as follows:

```bash
#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Pat Pannuto 2014,2016-2018,2021.
# Copyright Tock Project Developers 2015.
# Copyright Amit Levy 2016-2019.
# Copyright Bradford James Campbell 2022.

set -e
...
```

Many additional examples are available throughout the Tock repositories.


4.2 Enforcement
---------------

To ensure coverage and compliance with these policies, the Core Team
SHALL author and maintain tooling which enforces the presence and correct
format of license and copyright information. This SHOULD be automated and
integrated with continuous integration systems. Contributions which do
not satisfy these license and copyright rules MUST NOT be accepted.

In exceptional situations, consensus from the Core Team MAY circumvent
this policy. Such situations MUST include public explanation and public
record of non-anonymized vote results. This is not expected to ever occur.


6 Author’s Address
==================

    Pat Pannuto
    3202 EBU3, Mail Code #0404
    9500 Gilman Dr
    La Jolla, CA 92093, USA
    ppannuto@ucsd.edu