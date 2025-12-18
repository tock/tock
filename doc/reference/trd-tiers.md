Code Tiers
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell, Amit Levy<br/>
**Draft-Created:** 2025/12/15 <br/>
**Draft-Modified:** 2025/12/15 <br/>
**Draft-Version:** 1 <br/>
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
document is in full compliance with[TRD1][TRD1].

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

Tock code is grouped into five tiers, with higher numbered tiers denoting more
important code.

| Tier # | Tier         | Description                                                     |
|--------|--------------|-----------------------------------------------------------------|
| 5      | Verified     | Formally verified code with necessary proof                     |
| 4      | Critical     | High-priority code directly relevant to the Tock security model |
| 3      | Priority     | Important, long-standing code with significant scrutiny         |
| 2      | Normal       | Typical Tock code                                               |
| 1      | Experimental | Explicitly experimental code likely to change                   |

By default, all Tock code not otherwise categorized is considered "Normal".

2.1 Tier Descriptions
-------------------------------

- **Verified**: This code has been checked by a static verification tool. The
  necessary annotations and proofs are included in the Tock source code. The
  verification is checked by CI. Any changes must include updated annotations
  and proofs.

- **Critical**: This code is directly related to Tock's system-level security
  guarantees. Correctness is critical for Tock to uphold its threat model.
  This code is highly scrutinized during any proposed changes. Changes often
  require extensive discussions and careful audits.

- **Priority**: This code is not necessarily related to Tock's security model,
  but is widely used and tested, is necessary for correct operation of many
  platforms, and has been carefully implemented. Changes must be extensively
  tested.

- **Normal**: This is the default tier for all Tock code not otherwise
  classified into another tier.

- **Experimental**: This code is new, a work-in-progress, or otherwise an
  experimental subsystem or module within Tock. It likely will change
  substantially and may be not fully implemented or working. New
  contributions will not receive significant scrutiny to support rapid
  development.


3 Annotation Mechanism
===============================

Code is assigned a tier in Tock using...

3.1 Default Tier
-------------------------------

If there is no annotation present code is, by default, in the Normal
tier.

4 Using the Code Tier Annotations
===============================

Annotating code tiers enables some automated processes to aid Tock development.
This list is not comprehensive, but outlines some anticipated benefits of
annotating code tiers.

1. Detecting code which relies on a lower tier. Code in the highest tiers
   (i.e., critical and verified) may have an implicit assumption that it only
   uses or calls code in the same or higher tier. As an extreme example,
   critical code should not rely on experimental code for normal operation. With
   annotated code, static analysis can determine if there are any possible
   execution paths where any code relies on lower-tier code.

   With Tock's modularity, it is possible that certain kernel configurations
   would unexpectedly cause high tier code to rely on low tier code. This
   analysis would help Tock developers and users detect these scenarios.

2. Code review assistance. Reviewing Tock PRs does not require the same scrutiny
   across all code. However, the required scrutiny is typically determined
   based on experience and intuition. With labeled code tiers, the expectation
   for code review is made explicit.



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
