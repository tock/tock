Code Tiers
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell, Amit Levy<br/>
**Draft-Created:** 2025/12/15 <br/>
**Draft-Modified:** 2025/12/15 <br/>
**Draft-Version:** 2 <br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

As an operating system, Tock has numerous subsytems to support a variety of
hardware platforms, kernel features, and use cases. With a continuously growing
code base, different code modules and files receive a different level of
scrutiny and have different levels of significance for the overall project.
However, this scrutiny is implicit to code authors and reviewers. Code Tiers
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
stability guarantees, and we find that established software
engineering practices are sufficient to maintain these (e.g., semantic
versioning and unit testing).

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

Marking code's "trust tier" explicitly can have major benefits to code
review as well as user trust in Tock's security. For reviewers and
contributors alike, it can help guide and prepare contributors for the
level of scrutiny their contributions might receive. For example, if a
contribution changes code marked "Critical," it should be clearer that
such changes will require more scrutiny than code marked
"Experimental." Similarly, explicit annotations might deter
contributors from _unnecessarily_ modifying code in higher trust tiers
in contributions that are otherwise unrelated. For users, it can help
inform which subsystems are the most well-scrutinized and tested and
which are unwise to rely on without further auditing.

Explicit annotations in the code itself can also enable tools to
enforce related rules. For example, functions in highly scrutinized
code should not call functions in experimental code.

2 Tiers
===============================

Tock code is tiered based on two dimensions:

1. **Assurance**: How proven, tested, or verified the code is.
2. **Importance**: How critical the code is to the Tock project or its threat
   model.

These dimensions capture the underlying significance of any piece of Tock code,
revealing its relevance to the project and expectations for correctness.

2.1 Assurance
-------------------------------

The assurance tier describes the status of how the code has been shown to be
correct. This indicates to what extant the code should be considered working,
and what the expectation is for edits to the code.

| Tier # | Tier                | Description                                        |
|--------|---------------------|----------------------------------------------------|
| 1      | Formally Verified   | Code is checked by formal verification tools       |
| 2      | Extensively Tested  | Code is tested in CI and extensively used          |
| 3      | Functionally Tested | Code is tested in CI or other automated checks     |
| 4      | Normal              | Code is tested for correctness when it was written |

2.1.1 Assurance Tier Descriptions
-------------------------------

- **Formally Verified**: This code is checked by a static verification tool
  with the necessary annotations and proofs included in the Tock source
  code. This is likely accompanied by extensive test cases, including
  on-hardware tests. Any changes must include updated checks, such as the
  necessary annotations and proofs or new tests.

- **Extensively Tested**: This code is tested with a combination of Rust unit
  tests, integration tests, and automatic CI tests. The code is also widely
  used over a significant time (i.e., multiple years) on multiple hardware
  platforms by multiple users. This code is highly scrutinized during any
  proposed changes. Extensive testing may be required to validate changes.

- **Functionally Tested**: This code is tested with some combination of Rust
  unit tests, integration tests, and automatic CI tests. Changes must pass
  or update existing tests, and new tests may be required for any new
  code.

- **Normal**: This is the default tier for all Tock code not otherwise
  classified into another tier. The code was tested by running it in an
  appropriate context and verifying expected behavior.

2.2 Importance
-------------------------------

The importance tier describes how important the code is within the Tock
project and especially its threat model.

| Tier # | Tier         | Description                                        |
|--------|--------------|----------------------------------------------------|
| 1      | Critical     | Code is central to Tock and its threat model       |
| 2      | Widely Used  | Code is widely used in most Tock instances         |
| 3      | Normal       | Code is a normal part of the Tock project          |
| 4      | Experimental | Code is new, unfinished, and/or being designed     |

2.2.1 Importance Tier Descriptions
-------------------------------

- **Critical**: This code is directly related to Tock's system-level security
  guarantees. Correctness is critical for Tock to uphold its threat model.
  This code is highly scrutinized during any proposed changes. Changes
  often require extensive discussions and careful audits.

- **Widely Used**: This code is not necessarily related to Tock's security
  model, but is widely used and tested, is necessary for correct operation
  of many platforms, and has been carefully implemented. Changes must be
  extensively tested.

- **Normal**: This is the default tier for all Tock code not otherwise
  classified into another tier.

- **Experimental**: This code is new, a work-in-progress, or otherwise an
  experimental subsystem or module within Tock. It likely will change
  substantially and may be not fully implemented or working. New
  contributions will not receive significant scrutiny to support rapid
  development.

3 Annotation Mechanism
===============================

Code is assigned tiers in Tock using comments on the source code, leveraging
the same infrastructure as doc comments and safety comments.

Any code in Tock that can be commented with a doc comment can be annotated
with tiers.

The code's tier is specified with a top-level header `Code Tier`. Each tier is
specified as a list with the tier name. The format is like this:

```
/// # Code Tier
///
/// - Assurance: <Assurance tier>
/// - Importance: <Importance tier>
///
/// <Optional description or comments>
```

For example:

```rust
/// Create a new Tock process.
///
/// # Code Tier
///
/// - Assurance: Critical
/// - Importance: Extensively Tested
pub fn process_standard_create() -> ProcessStandard;
```

3.1 Default Tier
-------------------------------

If there is no annotation present code is, by default, in the Normal assurance
tier and the Normal importance tier.

4 Using the Code Tier Annotations
===============================

Annotating code tiers enables some automated processes to aid Tock development.
This list is not comprehensive, but outlines some anticipated benefits of
annotating code tiers.

1. Detecting code which relies on a lower tier. Code in the highest assurance
   tiers(i.e., Formally Verified and Extensively Tested) may have an implicit
   assumption that it only uses or calls code in the same or higher tier.
   Similarly, code in a high importance tier may not expect to depend on code in
   a low importance tier. As an extreme example, code in the Critical importance
   tier should not rely on code in the Experimental importance tier. With
   annotated code, static analysis can determine if there are any possible
   execution paths where any code relies on lower-tier code.

   With Tock's modularity, it is possible that certain kernel configurations
   would unexpectedly cause high tier code to rely on low tier code. This
   analysis would help Tock developers and users detect these scenarios.

2. Code review assistance. Reviewing Tock PRs does not require the same scrutiny
   across all code. However, the required scrutiny is typically determined
   based on experience and intuition. With labeled code tiers, the expectation
   for code review is made explicit.

3. Avoiding undetected changes to trusted code. A contribution may change code
   that is in the normal tier, but that code is used by code in a higher tier.
   Control flow analysis from code in higher tiers would reveal the potential
   impact of this change and mark the contribution for increased scrutiny.

5 Open Questions
===============================

1. Is higher tier code relying on lower tier code an error or a warning? If a
   warning, how do we specify the cases where we want it to be an error? For
   example, we may mark a function as a high tier, and then examine all code it
   depends on to ensure it is at the same tier or higher. We then want to be
   able to record that, so any future change that makes that no longer hold is
   flagged. If an error, how do we specify the cases that we don't want to cause
   CI to fail?


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
