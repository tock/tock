Digest HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Alistair Sinclair, Philip Levis <br/>
**Draft-Created:** June 8, 2022<br/>
**Draft-Modified:** June 8, 2022<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL)
for hash functions. A digest is the output of a hash function. It
describes the Rust traits and other definitions for this service as
well as the reasoning behind them. This document is in full compliance
with [TRD1](./trd1-trds.md). The HIL in this document also adheres to
the rules in the [HIL Design Guide](./trd-hil-design.md), which
requires all callbacks to be asynchronous -- even if they could be
synchronous.


1 Introduction
===============================

A hash function takes a potentially large input and transforms it into
a fixed-length value. Hash functions have many uses and so there are
many types of hash functions with different properties (computational
speed, memory requirements, output distributions). A *digest* is the
output of a hash function. Generally, hash functions seek to produce
digest values that are uniformly distributed over their space of
possible values, "hashing" the input and mixing it up such that the
distance between the digests from two similar input values seem
randomly distributed.

*Cryptographic hash functions* are a class of hash functions which
have two properties that make them useful for checking the integrity
of data. First, they have *collision resistance*: it is difficult to find
two messages, m1 and m2, such that hash(m1) = hash(m2). Second, they
have *pre-image resistance*, such that given a digest d, it is difficult
to find a message m such that hash(m) = d. SHA256 and SHA3 are
example cryptographic hash functions that are commonly used today
and believed to provide both collision resistance and pre-image
resistance.[Boneh and Shoup](https://toc.cryptobook.us/).

*Message authentication codes (MACs)* are a method for providing
integrity when both the generator and checker share a secret. MACs are
a distinct integrity mechanism than digests. They provide both
integrity and authenticity that the message came from a certain sender
(who holds the secret). Some MACs, such as
[HMAC](https://en.wikipedia.org/wiki/HMAC), are built on top of hash
functions.

This document describes Tock's traits for com and their semantics for
computing digests in the Tock operating system. These traits can also be
used for generating HMACs.


2 Adding Data to a Digest: `DigestData` and `ClientData`
===============================


3 Computing and Verification: `DigestHash`, `DigestVerify`, `ClientHash`, and `ClientVerify`
===============================

4 Composite Traits and Configuration
===============================

5 Capsules
===============================

The Tock kernel provides several capsules for digests:


6 Authors' Address
=================================

    Alistair Sinclair
    alistair.francis@wdc.com

    Philip Levis
    409 Gates Hall
    Stanford University
    Stanford, CA 94305
    USA
    pal@cs.stanford.edu
