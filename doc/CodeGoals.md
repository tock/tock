Code Goals
==========

Over more than a decade of development, Tock has established certain conventions
for its development that are implicitly fairly well understood but not
explicitly documented. This guide attempts to document and explain those
conventions.

## Support Varied Use Cases, Known and Unknown

As an operating system, we anticipate Tock will be used on a variety of hardware
platforms, under a variety of requirements, and supporting a variety of
applications. Some of these are well-known and public, some are on the
periphery, and others may be secret. Upstream development tries, when
reasonable, to consider these use cases and balance benefits for one with
drawbacks for another.

This means certain contributions may require additional scrutiny or review, even
when they clearly benefit one use case, to understand how they may affect other
use cases. Our goal is not to necessarily reject any contributions that are not
broadly applicable, but instead to understand the effects more comprehensively
and use that understanding when making a decision.

## Prioritize Maintainability and Long-Term Code

Building an OS doesn't happen quickly, and we believe building a trusted OS
takes time and careful consideration. For many changes, we prefer to act
deliberately rather than quickly, even if that means a contribution may take
months to merge. We believe a value proposition of Tock is its careful design.

Accordingly, many substantial changes to Tock are accompanied by a Tock
Reference Document (TRD) to not only capture the proposed design, but also the
intuition behind the design, pros and cons, and alternate designs not used.

We also aim to have Tock be used for years to come, and we aim to have
maintainable code that helps future developers. This sometimes means that even
simple fixes or contributions may require changes to promote the long-term
development of Tock.

## Embrace the Safety Ethos

Tock has used Rust since its inception, and we strongly believe in the benefits
of leveraging memory-safety checks and the other safety benefits of the Rust
compiler. In some cases we find that Rust is even too lenient, and we would
prefer stronger guarantees. This attention to safety guides many Tock design
decisions, and sometimes means that even valid Rust code will require changes
before it is merged upstream.

We believe this supports one of Tock's value propositions around providing a
safe OS. However, it sometimes means that contributions require more scrutiny
around more low-level components of Tock to ensure we are confident in their
implications for overall safety.
