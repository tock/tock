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

A client adds data to a hash function's input with the `DigestData` trait
and receives callbacks with the `ClientData` trait.

These traits support both mutable and immutable data. Most HIL traits
in Tock support only mutable data, because it is assumed the data is 
in RAM and passing it without the `mut` qualifier in a split-phase
operation can discard its mutability 
(see Rule 5 in [TRD2](./trd2-hil-design.md)). Digest supports immutable
data because many services need to compute digests over large, read-only data
in flash. One example of this is the kernel's process loader, which needs
to check that process images are not corrupted. Because digests are 
computationally inexpensive, copying the data from flash to RAM in order
to compute a digest is a large overhead. Furthermore, the data input can
be large (tens or hundreds of kilobytes). Therefore `DigestData` and 
`ClientData` support both mutable and immutable inputs.

Clients provide input to `DigestData` through the `LeasableBuffer`
and `LeasableMutableBuffer` types. These allow a client to ask a
digest engine to compute a digest over a subset of their data, e.g. to
exclude the area where the digest that will be compared against is stored. 
These types have a source slice and maintain an active range over that slice.
The digest will be computed only over the active range, rather than the
entire slice.

```rust
pub trait DigestData<'a, const L: usize> {
    fn set_data_client(&'a self, client: &'a dyn ClientData<'a, L>) {}
    fn add_data(&self, data: LeasableBuffer<'static, u8>) 
       -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)>;
    fn add_mut_data(&self, data: LeasableMutableBuffer<'static, u8>)
       -> Result<(), (ErrorCode, LeasableMutableBuffer<'static, u8>)>;
    fn clear_data(&self);
}
```

A successful call to `add_data` or `add_mut_data` will add all of the
data in the active range of the leasable buffer as input to the hash
function. A successful call is one which returns `Ok(())` and whose
completion event passes `Ok(())`. If a client needs to compute a hash over several non-contiguous
regions of a slice, or multiple slices, it can call these methods multiple
times. 

There may only be one outstanding `add_data`  or `add_mut_data` operation at
any time. If either `add_data` or `add_mut_data` returns `Ok(())`, then all
subsequent calls to `add_data` or `add_mut_data` MUST return `Err((ErrorCode::BUSY, ...))`
until a completion callback delivered through `ClientData`.

```rust
pub trait ClientData<'a, const L: usize> {
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: LeasableBuffer<'static, u8>);
    fn add_mut_data_done(
        &'a self,
        result: Result<(), ErrorCode>,
        data: LeasableMutableBuffer<'static, u8>,
    );
}
```

The `data` parameters of `add_data_done` and `add_mut_data_done` indicate what
data was added and what remains to be added to the digest. If either callback
has a `result` value of `Ok(())`, then the active region of `data` MUST be zero
length and all of the data in the active region passed through the corresponding
call MUST have been added to the digest. 

A call to `DigestData::clear_data()` terminates the current digest computation and
clears out all internal state to start a new one.
If there is an outstanding `add_data` or `add_data_mut` when `clear_data()` is called,
the digest engine MUST issue a corresponding callback with an `Err(ErrorCode::CANCEL)`.

A digest engine MUST accept multiple calls to `add_data` and `add_mut_data`. Each 
call appends to the data over which the digest is computed. 

3 Computing and Verification: `DigestHash`, `DigestVerify`, `ClientHash`, and `ClientVerify`
===============================

Once all of the data has been added as the input to a digest, a client can either
compute the digest or ask the digest engine to compare its computed digest with
a known value (verify).




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
