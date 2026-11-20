Code Tiers
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell, Amit Levy<br/>
**Draft-Created:** 2025/12/15 <br/>
**Draft-Modified:** 2026/11/20 <br/>
**Draft-Version:** 3 <br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

As an operating system, Tock has numerous subsytems to support a variety of
hardware platforms, kernel features, and use cases. With a continuously growing
code base, different code modules and files receive a different level of
scrutiny and have different levels of significance for the overall project.
However, this scrutiny and importance is often implicit to code authors and reviewers. Code Tiers
addresses this, by adding annotations in the source directly to denote the
importance and scrutiny applied to each code module. This helps clarify to
contributors the expectations for changes to a particular module and signals to
reviewers the level of scrutiny that should be applied to those changes. This
document is in full compliance with [TRD1][TRD1].

1 Introduction
===============================

Software development is commonly structured based on stability
guarantees where during code review developers ensure that
functionality that was promised to remain unchanged is in fact
unchanged, or, the version number is incremented. Tock does include
certain but limited
stability guarantees, and we find that established software
engineering practices are sufficient to maintain these (e.g., semantic
versioning and unit testing). However, because what is promised to be stable
is relatively narrow, applying only to the low-level interface between
userspace and the kernel, stability guarantees are ineffective to guide code review
on internal Tock interfaces and subsystems, which is the majority of code contributions.

What is less commonly expressed in software projects is the degree to
which particular code has been reviewed, validated, considered, and
audited. Certain interfaces and modules within Tock feature subtle
correctness requirements, have non-obvious yet wide-ranging
implications for the OS, or were notoriously buggy in earlier
implementations. These get extra scrutiny, and developers gain an
intuition over time as to which code has been highly vetted, and the
bar for its modification is very high. However, the trusted and vetted
nature of particular modules is difficult for new developers or
contributors to perceive, which leads to frustration and confusion
when a small or simple seeming change requires extensive discussion.
More importantly, code reviewers must be able to identify when a
change impacts vetted code and review it appropriately. Otherwise,
seemingly innocuous (but incorrect) changes could have significant
impacts to Tock security.

One version of this insight can be extracted from the code base automatically.
Using control-flow analysis techniques, the amount of code affected by any
change can be automatically determined. In addition to quantity, which code
and crates are affected, and how many different subsystems are affected, can
be determined. This automated analysis can help illuminate the extent to
which a change might have effects outside of the immediately changed code,
and this impact may not be obvious when working on a particular subsystem.

We identify, however, that automated analysis is insufficient to fully
understand the implication of an arbitrary change to the code base. Certain
properties of a function or module may not be readily apparent based on
either a code base analysis like a control flow analysis, or from project and
contribution metadata, like commit history. Code Tiers is an approach to
capture those implications that cannot be inferred automatically.

Manually annotating code with "trust tier" context can have major benefits to code
review as well as user trust in Tock's security. For reviewers and
contributors alike, it can help guide and prepare contributors for the
level of scrutiny their contributions might receive. For example, if a
contribution changes code marked "critical," it should be clearer that
such changes will require more scrutiny than code marked
"experimental." Similarly, explicit annotations might deter
contributors from _unnecessarily_ modifying code in high trust tiers
in contributions that are otherwise unrelated. For users, it can help
inform which subsystems are the most well-scrutinized and tested and
which are unwise to rely on without further auditing.

Explicit annotations in the code itself can also enable tools to
enforce related rules. For example, functions in highly scrutinized
code should not call functions in experimental code.

2 Code Tier Methodology
===============================

Exposing code tiers is focused on adding context to the code base that is
necessary for code review and security that cannot be automatically inferred.
That is, this mechanism should try to make expert intuition and general
project understanding explicit in the code base directly. However, it should
not duplicate what can be done with automated tools or the language.

2.1 Automated Analysis
-------------------------------

Tock should use automated analysis to determine the impact and severity of
proposed code changes. This reduces the burden of manually annotating code,
and updates as the code base changes, extends, or is refactored.

Tock has traditionally included some automated analysis for understanding the
impacts of a change. For example:

- Understanding code size effects. An automated tool compares the size of
  built kernels before and after each PR.
- Understanding stack size effects. A tool measures the largest stack frames
  for any given kernel build.

To further this, additional automated analysis should be considered and
implemented. This document sketches out the desired analyses to identify what
is not included in the manual annotations.

- Number of affected functions: For every changed function in a proposed
  patch, use the control flow graph to determine how many other functions are
  affected as those functions call (directly or indirectly) the modified
  function.
- Number of affected unsafe blocks: Use the control flow graph to determine
  how many unsafe blocks contain code that is affected by this change.

2.2 Rust-based Annotations
-------------------------------

Some code context is conveyed through language mechanisms. As Tock is
implemented in Rust, Tock uses Rust's mechanisms to annotated code.

The most prominent example is marking functions and traits as `unsafe` to
convey there are pre-conditions that callers and implementers must discharge
to avoid unsound code. Functions and traits marked `unsafe` are accompanied
by a `/// # Safety` comment explaining the requirements.

2.3 Manual Annotations
-------------------------------

For context and impact that cannot be quantified automatically, or where the
automatic analysis tools generate misleading outputs, manual annotations
provide explicit context to code in the code base.

Because these annotations are intentionally subjective, based on experience,
and reflect intended designs not captured by the semantics of the programming
language, the annotations should not be focused on objective rankings or
scorings. That is, the manual annotations should _not_ be based on an
explicit hierarchy or try to provide code with an absolute score. While
objective metrics like a hierarchy or score have appeal, they ultimately are
too rigid to capture the subtleties in real code bases, lead to unproductive
discussions about the "correct" classification, and hinder adoption by making
annotating difficult.

Instead, the annotations should only strive to provide missing context
necessary for code review and capturing security requirements. These
annotations need not represent a single dimension of classification, be
comprehensive along a single dimension, or be mutually exclusive. They should
be flexible enough to capture implicit context while remaining intuitive to
be effectively communicative.

3 Code Tier Tags
===============================

Code is annotated with a list of tags. The tags convey something related to
the context of the code, its importance, its assurance, or another property
that is not conveyed by the programming language or computable by an
automated analysis.

The tags are well defined with clear meanings for the annotated code. By
having a fixed set of tags that are consistent throughout the code base,
different modules and crates can be compared and analyzed based on the tags.
Compared to an unconstrained written description, this promotes some
consistency in the code base.

Notably, the tags are not ranked or scored, and there is no ordering function
between tags. This makes them flexible, and introduces enough subjectivity
that they can be used across a code base and by multiple developers. Whereas
any numeric-based approach inevitably requires normalization against a common
baseline, which can be difficult to establish across an entire code base,
tags can naturally be understood in the context of a subsystem or crate.

3.1 Defined Tags
-------------------------------

| Tag                   | Description                                    |
|-----------------------|------------------------------------------------|
| threat-model-critical | Code directly impacts the Tock threat model    |
| experimental          | Code is new, unfinished, and/or being designed |
| formally-verified     | Code is checked by formal verification tools   |

3.1.1 Tag Descriptions
-------------------------------

- **threat-model-critical**: This code is directly related to Tock's
  system-level security guarantees. Correctness is critical for Tock to
  uphold its threat model. This code is highly scrutinized during any
  proposed changes. Changes often require extensive discussions and careful
  audits.

- **experimental**: This code is new, a work-in-progress, or otherwise an
  experimental subsystem or module within Tock. It likely will change
  substantially and may be not fully implemented or working. New
  contributions will not receive significant scrutiny to support rapid
  development.

- **formally-verified**: This code is checked by a static verification tool
  with the necessary annotations and proofs included in the Tock source
  code. This is likely accompanied by extensive test cases, including
  on-hardware tests. Any changes must include updated checks, such as the
  necessary annotations and proofs or new tests.

4 Annotation Mechanism
===============================

Code is annotated in Tock using comments on the source code, leveraging
the same infrastructure as doc comments and safety comments.

Any code in Tock that can be commented with a doc comment can be annotated
with tags.

The code's tier is specified with a top-level header `Code Tier`. Then the
tags are listed. Any additional context needed to explain the tags can then
be included.

```
/// # Code Tier
///
/// Tags: <tag 1>, <tag 2>, <tag N>
///
/// <Optional description or comments>
```

For example:

```rust
/// Create a new Tock process.
///
/// # Code Tier
///
/// Tags: threat-model-critical
/// 
/// A correctly initialized process is crucial to uphold Tock's threat model
/// guarantees, particularly concerning isolation between processes.
pub fn process_standard_create() -> ProcessStandard;
```

5 Using the Code Tier Annotations
===============================

Annotating code tiers enables some automated processes to aid Tock development.
This list is not comprehensive, but outlines some anticipated benefits of
annotating code tiers.

1. Detecting code with likely mismatches in expectations. Code with certain
   tags, such as threat-model-critical, likely should not rely on code tagged
   experimental. With annotated code, static analysis can determine if there are
   any possible execution paths where any code with a certain tag relies on code
   with other, potentially incompatible tags.

   With Tock's modularity, it is possible that certain kernel configurations
   would unexpectedly cause this type of mismatch. This analysis would help
   Tock developers and users detect these scenarios.

2. Code review assistance. Reviewing Tock PRs does not require the same scrutiny
   across all code. However, the required scrutiny is typically determined
   based on experience and intuition. With labeled code tiers, the expectation
   for code review is made explicit.

3. Avoiding undetected changes to trusted code. A contribution may change code
   that is not tagged and is not flagged by automated analysis, but that code is
   used by code with certain tags. Control flow analysis from code with specific
   tags would reveal the potential impact of this change and mark the
   contribution for increased scrutiny.

6 Open Questions
===============================




Author Addresses
=================================

```
Brad Campbell
Computer Science
241 Olsson Hall
P.O. Box 400336
Charlottesville, Virginia 22904

email: Brad Campbell <bradjc@virginia.edu>

Amit Levy
email: Amit Levy <amit@betterbytes.org>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
