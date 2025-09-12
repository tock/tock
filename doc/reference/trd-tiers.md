Code Tiers
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell<br/>
**Draft-Created:** 2025/09/12 <br/>
**Draft-Modified:** 2025/09/12 <br/>
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



2 Tiers
===============================

Tock code is grouped into five tiers, with higher numbered tiers denoting more
important code.

| Tier # | Tier         | Default | Description                                                     |
|--------|--------------|---------|-----------------------------------------------------------------|
| 5      | Verified     | No      | Formally verified code with necessary proof                     |
| 4      | Critical     | No      | High-priority code directly relevant to the Tock security model |
| 3      | Priority     | No      | Important, long-standing code with significant scrutiny         |
| 2      | Normal       | Yes     | Typical Tock code                                               |
| 1      | Experimental | No      | Explicitly experimental code likely to change                   |

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

If there is not annotation present the code is in the Normal tier.




Author Addresses
=================================

```
Brad Campbell
Computer Science
241 Olsson Hall
P.O. Box 400336
Charlottesville, Virginia 22904

email: Brad Campbell <bradjc@virginia.edu>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
