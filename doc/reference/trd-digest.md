Digest HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Alistair Francis, Philip Levis <br/>
**Draft-Created:** June 8, 2022<br/>
**Draft-Modified:** June 8, 2022<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL)
for hash functions. A digest is the output of a hash function. It
describes the Rust traits and other definitions for this service as
well as the reasoning behind them. This document is in full compliance
with [TRD1](./trd1-trds.md). The HIL in this document also adheres to
the rules in the [HIL Design Guide](./trd2-hil-design.md), which
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

This document describes Tock's traits and their semantics for
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

Clients provide input to `DigestData` through the `SubSlice`
and `SubSliceMut` types. These allow a client to ask a
digest engine to compute a digest over a subset of their data, e.g. to
exclude the area where the digest that will be compared against is stored. 
These types have a source slice and maintain an active range over that slice.
The digest will be computed only over the active range, rather than the
entire slice.

```rust
pub trait DigestData<'a, const L: usize> {
    fn set_data_client(&'a self, client: &'a dyn ClientData<'a, L>);
    fn add_data(&self, data: SubSlice<'static, u8>) 
       -> Result<(), (ErrorCode, SubSlice<'static, u8>)>;
    fn add_mut_data(&self, data: SubSliceMut<'static, u8>)
       -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)>;
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
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>);
    fn add_mut_data_done(
        &'a self,
        result: Result<(), ErrorCode>,
        data: SubSliceMut<'static, u8>,
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

Once all of the data has been added as the input to a digest, a client
can either compute the digest or ask the digest engine to compare its
computed digest with a known value (verify). These traits have a
generic parameter `L` which defines the length of the digest in bytes.
A SHA256 digest engine, for example, has an `L` of 32.


```rust
pub trait DigestHash<'a, const L: usize> {
    fn set_hash_client(&'a self, client: &'a dyn ClientHash<'a, L>);
    fn run(&'a self, digest: &'static mut [u8; L])
        -> Result<(), (ErrorCode, &'static mut [u8; L])>;
}

pub trait ClientHash<'a, const L: usize> {
    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]);
}

pub trait DigestVerify<'a, const L: usize> {
    fn set_verify_client(&'a self, client: &'a dyn ClientVerify<'a, L>);
    fn verify(&'a self, compare: &'static mut [u8; L])
	    -> Result<(), (ErrorCode, &'static mut [u8; L])>;
}

pub trait ClientVerify<'a, const L: usize> {
    fn verification_done(&'a self, result: Result<bool, ErrorCode>, compare: &'static mut [u8; L]);
}
```

Calls to `DigestHash::run` and `DigestHash::verify` perform the hash
function on all of the data that has been added with calls to
`add_data` and `add_data_mut`. If there is an outstanding call to
`add_data`, `add_data_mut`, `run`, or `verify` they MUST return
`Err(ErrorCode::BUSY)`.

The `ClientHash::hash_done` callback returns the computed digest
stored in the `digest` slice. If the `result` argument is `Err((...))`,
the `digest` slice may store any values. If the `result` argument
is `Ok(())` the `digest` slice MUST store the computed digest.

The `DigestVerity:verify` takes an existing digest as its `compare`
parameter. It triggers the digest engine to compute the digest, then
compares the computed value with what was passed in `compare`. If the
computed and provided values match, then `ClientVerify` passes
`Ok(true)`; if they do not match then it passes `Ok(false)`.  An `Err`
result indicates that there was an error in computing the digest.

Calling either `DigestHash::run` or `DigestVerify::verify` completes
the digest calculation, returning the digest engine to an idle
state for the next computation.

4 Composite Traits 
===============================

The Digest HIL provides many composite traits, so that structures
which implement multiple traits can be passed around as a single
reference. The `ClientDataHash` trait is for a client that implements
both `ClientData` and `ClientHash`. The `ClientDataVerify` trait is
for a client that implements both `ClientData` and `ClientVerify`.
The `Client` trait is for a client that implements `ClientData`,
`ClientHash`, and `ClientVerify`.

```rust
pub trait ClientDataHash<'a, const L: usize>: ClientData<'a, L> + ClientHash<'a, L> {}
pub trait ClientDataVerify<'a, const L: usize>: ClientData<'a, L> + ClientVerify<'a, L> {}
pub trait Client<'a, const L: usize>:
    ClientData<'a, L> + ClientHash<'a, L> + ClientVerify<'a, L> {}
```


The `DigestDataHash` trait is for a structure that implements both
`DigestData` and `DataHash`. The `DigestDataVerify` trait is for a
client that implements both `DigestData` and `DigestVerify`.  The
`Digest` trait is for a client that implements `DigestData`,
`DigestHash`, and `DigestVerify`. These each add an additional
method, `set_client`, which allows it to store the corresponding
client as a single reference and use it for all of the relevant
client callbacks (e.g., `add_data`, `add_mut_data`, `hash_done`, and
`verification_done`). A digest implementation that implements
`set_client` MAY choose to not implement the individual client set
methods for the different traits (e.g., `DigestData::set_client`); if
it does so, each of these client set methods MUST be marked
`unimplemented!()`.


```rust
pub trait DigestDataHash<'a, const L: usize>: DigestData<'a, L> + DigestHash<'a, L> {
    /// Set the client instance which will receive `hash_done()` and
    /// `add_data_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn ClientDataHash<L>);
}

pub trait DigestDataVerify<'a, const L: usize>: DigestData<'a, L> + DigestVerify<'a, L> {
    /// Set the client instance which will receive `verify_done()` and
    /// `add_data_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn ClientDataVerify<L>);
}

pub trait Digest<'a, const L: usize>:
    DigestData<'a, L> + DigestHash<'a, L> + DigestVerify<'a, L>
{
    /// Set the client instance which will receive `hash_done()`,
    /// `add_data_done()` and `verification_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn Client<'a, L>);
}
```


5 Configuration
===============================

Digest engines can often operate in multiple modes, supporting several
different hash algorithms and digest sizes. Configuring a digest
engine occurs out-of-band from adding data and computing digests,
through separate traits. Each digest algorithm is described by
a separate trait. This allows compile-time checking that a given
digest engine supports the required algorithm. For example,
a digest engine that can compute a SHA512 digest implements
the `Sha512` trait:

```rust
pub trait Sha512 {
    /// Call before Digest::run() to perform Sha512
    fn set_mode_sha512(&self) -> Result<(), ErrorCode>;
}
``` 

 
The Digest HIL defines seven standard Digest traits:
  - `Sha224`
  - `Sha256`
  - `Sha384`
  - `Sha512`
  - `HmacSha256`
  - `HmacSha384`
  - `HmacSha512`
  
The HMAC configuration methods take a secret key, which is
used in the HMAC algorithm. For example,

```rust
pub trait HmacSha384 {
    /// Call before `Digest::run()` to perform HMACSha384
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode>;
}
```
Configuration methods MUST be called before the first call to `add_data`
or `add_data_mut`. 


6 Capsules
===============================

There are 5 standard Tock capsules for digests:

  1. `capsules::hmac` provides a system call interface to a digest engine that
  supports `Digest`, `HmacSha256`, `HmacSha384`, and `HmacSha512`.
  2. `capsules::sha` provides a system call interface to a digest engine that
  supports `Digest`, `Sha256`, `Sha384`, and `Sha512`.
  3. `capsules::virtual_hmac` virtualizes an HMAC engine, allowing multiple clients
  to share it through queueing. It requires a digest engine that
  supports `Digest`, `HmacSha256`, `HmacSha384`, and `HmacSha512`.
  4. `capsules::virtual_sha` virtualizes a SHA engine, allowing multiple clients
  to share it through queueing. It requires a digest engine that
  supports `Digest`, `Sha256`, `Sha384`, and `Sha512`.
  5. `capsules::virtual_digest` virtualizes a SHA/HMAC engine,
  allowing multiple clients to share it through queueing. It requires
  a digest engine that supports `Digest`, `HmacSha256`, `HmacSha384`,
  and `HmacSha512`, `Sha256`, `Sha384`, and `Sha512` and supports
  all of these operations.

6 Authors' Address
=================================

    Alistair Francis
    alistair.francis@wdc.com

    Philip Levis
    409 Gates Hall
    Stanford University
    Stanford, CA 94305
    USA
    pal@cs.stanford.edu
