Application IDs (AppID)
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Johnathan van Why<br/>
**Draft-Created:** 2021/09/01 <br/>
**Draft-Modified:** 2021/09/01 <br/>
**Draft-Version:** 1 <br/>
**Draft-Discuss:** tock-dev@googlegroups.com<br/>

Abstract
-------------------------------

This document describes the design and implementation of application
identifiers (AppIDs) in the Tock operating system. AppIDs provide a
mechanism to identify the application contained in a userspace binary.
AppIDs allow the kernel to apply security policies to applications as
their code evolves and their binaries change. A board defines how the
kernel verifies AppIDs and which AppIDs the kernel will load. This
document describes the Rust traits and software architecture for
AppIDs as well as the reasoning behind them. This document is in full
compliance with [TRD1](./trd1-trds.md).

1 Introduction
===============================

The Tock kernel needs to be able to manage and restrict what userspace
applications can do. Examples include:
  - making sure other applications cannot access an application's sensitive
  data stored in non-volatile memory,
  - restricting certain system calls to be used only by trusted applications,
  - run and load only applications that a trusted third party has signed.

In order to accomplish this, the kernel needs a way to identify an
application and know whether a particular userspace binary belongs to
an application. 

The mapping between binaries and applications can be 
many-to-many. Multiple binaries can be associated with a single application
when there are software updates/versions or when an application needs
to run in multiple processes. A program that migrates data from one
application to another (e.g., transitions keys from an old U2F
application to a new one) needs to be associated with both the source
and destination applications.

To remain flexible and support many use cases, the Tock kernel makes 
minimal assumptions on the structure and form of
application credentials that bind an application identifier to a
binary. Application credentials are arbitrary k-byte sequences that
are stored in an userspace binary's Tock binary format (TBF)
headers. When a Tock board instantiates the kernel, it passes a
reference to an AppID (application identifier) verifier, which the
kernel uses to determine the AppIDs of each userspace binary it reads
and decide whether to load the binary into a process.

Most of the complications in AppIDs stem from the fact that they are a
general mechanism used for many different use cases. Therefore, the
exact structure and semantics of application credentials can vary
widely. Tock's TBF header formats, kernel interfaces and mechanisms
must accomodate this wide range.

The interfaces and standard implemenentations for AppIDs and AppID
verifiers are in the kernel crate, in module `appid`. There are two
main traits:

  * `kernel::appid::Verifier` is responsible for defining which types
  of application identifier it can accepts and whether it accepts a
  particular application identifier for a specific application
  binary. The kernel only loads application binaries the `Verifier`
  accepts.
  
  * `kernel::appid::Compress` is repsonsible for compressing
  application identifiers into short, 32-bit identifiers called
  `ShortID`s. `ShortID`s provide a mechanism for fast comparison,
  e.g., for an application identifier against an access control list.
  

2 Terminology
===============================

This document uses several terms in precise ways. Because these terms
overlap somewhat with general terminology in the Tock kernel, this
section defines them for clarity. The Tock kernel often uses the term
"application" to refer to what this document calls a "Tock binary."

**Application**: userspace software developed and maintained by an
individual, group, corporation, or organization that meets the
requirements of a particular Tock device use case. An application can
consist of a single userspace binary, multiple userspace binaries that
run concurrently (an application split into several processes), or
multiple userspace binaries only one of which runs at a time (an
application with multiple versions).

**Tock binary**: a Tock binary format (TBF)[TBF] object stored on a
Tock device, containing TBF headers and an application binary.

**Application binary**: a code image compiled to run in a Tock
process.

**Application identifier**: a numerical identifier for an application.

**Global application identifier**: an application identifier which is
globally unique. All instances of the application have this identifier
and no instances of other applications have this identifier.

**Local application identifer**: an application identifier which is
locally unique. No instances of other applications on the same
Tock device have this identifier.

**Application credentials**: data that binds an application identifier
to an application binary. Application credentials are usually stored
in Tock binary format[TBF] headers.

In normal use of Tock, Tock binaries are copied into an application
flash region by a software tool. When the Tock kernel boots, it scans
this application flash region for Tock binaries. After inspecting the
application binary and TBF headers in a Tock binary, the kernel
decides whether to load it into a process and run it.

3 Application identifiers and application credentials
===============================

To explain the distinction between application identifiers and credentials,
consider these four use cases.
  
  1. The application has a single process. It has no application
  credentials: it only runs on kernels that are willing to load
  applications without credentials (e.g., research systems). The
  application has only a local application identifier.

  1. The application has a single process and there is a one-to-one
  mapping between public keys and applications. The application is
  identified by a public key (e.g., an RSA public key) that is also a
  global application identifier. This application's application
  credentials consist of a TBF header containing this key as well as a
  signed SHA512 hash of the application binary, signed with the
  private key corresponding to the public key in the header. The
  kernel decides whether to accept a particular public key for
  verification.

  1. The application has a single process and the same key is used for
  multiple applications. The application is identified by a unique
  identifier I. This application's application credentials consist of
  a TBF header containing an ECDSA public key, the unique identifier
  I, and a signed SHA512 hash of the application binary and unique
  identifier I, signed with the private key corresponding to the
  public key in the header. The kernel decides whether to accept a
  particular public key for verification. The application has a global
  application identifier is the concatenation of its public key and I.

  1. The application has multiple processes. The application is
  identified by a public key that is also its global application
  identifier. The application credentials of each Tock binary of the
  process consist a TBF header containing the public key and a
  signature of the SHA512 of the application binary made with the
  corresponding private key.

An application identifier provides an identity for an application
binary. It allows the Tock kernel to know about the provenance and
origin of the binary and make access control or security decisions
based on this information. For example, a kernel may allow only applications whose
credentials use a particular trusted public key to access restricted
functionality, but restrict other applications to use a subset of
available system calls.

Application identifiers are distinct from process
identifiers; an application identifier is per-application (persists
across restarts of a Tock binary, for example), while a process
identifier identifies a particular execution of that binary. At any
time on a Tock device, each process has a unique process identifier,
but they can be re-used over time (like POSIX process identifiers).

As the above examples illustrate, application credentials can vary in
size and content. The credentials that the kernel will accept depends
on its use case: certain devices will only accept credentials which
include a particular public key, while others will accept
many. Furthermore, the internal format of these credentials can vary.
Finally, the cryptography used in credentials can vary, either due to
security policies or certification requirements.

4. Credentials in Tock Binary Format Headers
===============================

To support credentials in Tock binaries, the Tock Binary Format has
a `TbfHeaderV2Credentials` header. This header is variable length
and has two fields:

```rust
pub struct TbfHeaderV2Credentials {
    format: TbfHeaderV2CredentialsType,
    data: &[u8],
}  
```

The `TbfHeaderV2CredentialsType` defines the format and size of `data`
field. A `TbfHeaderV2CredentialsType` value MUST have a fixed
data size and format. Currently supported values are:

```rust
pub enum TbfHeaderV2CredentialsType {
    CleartextID = 0,
    Rsa3072Key = 1,
    Rsa4096Key = 2,
    Rsa3072KeyWithID = 3,
    Rsa4096KeyWithID = 4,
}
```

**These are not intended to be final or prescriptive. They are merely some examples
of what kind of information we might put here. Among other things, the exact format 
of the data blocks needs to be more precise. -pal**

The `CleartextID` value has a data length of 8 bytes. It contains a 64-bit number in
big-endian format representing an application identifier.

The `Rsa3072Key` value has a data of length of 768 bytes. It contains a public 3072-bit
RSA key (384 bytes), followed by a 384-byte ciphertext block, consisting of the SHA512 
hash of the application binary in this Tock binary, encrypted by the private key 
of the public key in the header.

The `Rsa4096Key` value has a data of length of 1024 bytes. It contains a public 4096-bit
RSA key (512 bytes), followed by a 512-byte ciphertext block, consisting of the SHA512 
hash of the application binary in this Tock binary, encrypted by the private key 
of the public key in the header.

The `Rsa3072KeyWithID` value has a data of length of 768 bytes. It contains a public 3072-bit
RSA key (384 bytes), followed by a 384-byte ciphertext block, consisting of the SHA512 
hash of the application binary in this Tock binary followed by a 32-bit application
ID, encrypted by the private key of the public key in the header.

The `Rsa4096KeyWithID` value has a data of length of 1024 bytes. It contains a public 4096-bit
RSA key (512 bytes), followed by a 512-byte ciphertext block, consisting of the SHA512 
hash of the application binary in this Tock binary followed by a 32-bit application
ID, encrypted by the private key of the public key in the header.


4 `Verifier` trait
===============================

The `Verifier` trait defines an interface to a module that accepts,
passes on, or rejects application credentials. When a Tock board
asks the kernel to load processes, it passes a reference to a 
`Verifier`, which the kernel uses to check credentials.


```rust
pub enum VerificationResult {
  Accept,
  Pass,
  Reject
}

pub trait Verifier {
  fn require_credential(&self) -> bool;
  fn check_credentials(&self, 
                       credentials: &TbfHeaderV2Credentials, 
                       binary: &mut [u8]) -> VerificationResult;
}
```

The kernel, when it loads a Tock binary, scans its headers in order
from the beginning of the Tock binary. At each
`TbfHeaderV2Credentials` header it encounters, it calls
`check_credentials` on the provided `Verifier`. If the `Verifier`
returns `Accept`, the kernel stops processing credentials and
continues loading the Tock binary. If the `Verifier` returns `Reject`,
the kernel stops processing credentials and terminates loading the
Tock binary. If the `Verifier` returns `Pass`, the kernel tries the
next `TbfHeaderV2Credentials`, if there is one. 

If the kernel reaches the end of the TBF headers without encountering
a `Reject` or `Accept` result, it calls `require_credentials` to ask
the `Verifier` what the default behavior is.  If `require_credentials`
returns `true`, the kernel rejects the Tock binary and terminates
loading it. If `require_credentials` returns `false`, the kernel
accepts the Tock binary and continues loading it. If a Tock binary has
no `TbfHeaderV2Credentials` headers then there will be no `Accept` or
`Reject` results and `require_credentials` defines whether to load
such a binary.

An implementer of `Verifier` sets the security policy of Tock binary
loading by deciding which types of credentials, and which credentials,
are acceptable and which are rejected.

If `check_credentials` returns `Accept` for a
`TbfHeaderV2Credentials`, the kernel stores a reference to this
`TbfHeaderV2Credentials` in the process structure. This data
represents the acting credentials of the process.


5 Short IDs and the `Compress` trait
===============================

While `TbfHeaderV2Credentials` define the identity and credentials of
an application, they are typically large data structures that are too
large to store in RAM. When parts of the kernel wish to apply
application-based security or access policies, they need a concise way
to represent these policies. Requiring policies to be encoded in terms
of application credentials is extremely costly: a table, for example,
that says that only applications signed with a particular 4096-bit RSA
key can access certain system calls requires storing the whole
4096-bit key. If there are multiple such security policies through the
kernel, they must each store this information. 

The `Compress` trait provides a mechanism to map credentials to a
small (32-bit) integer, which can then be used throughout the kernel
as an identifier for security policies. For example, suppose that a
device wants to grant access to all application binaries signed by a
certain 3072-bit RSA key. The `Compress` trait can map all such
`TbfHeaderV2Credentials` to a known identifier. This identifier is
stored in the process structure. Access control systems within the
kernel can define their policies in terms of these identifiers, such
that they can check access by comparing 32-bit integers rather than
512-byte keys.

```rust
#[derive(Clone, Copy, Eq)]
struct ShortID {
  id: u32
}

pub trait Compress {
    fn to_short_id(credentials: &TbfHeaderV2Credentials) -> Option<ShortID>;
}
```

Generally, the same structure that implements `Verifier` also
implements `Compress`. This allows it to share copies of public keys
or other credentials that it uses to make decisions. Doing so also
makes it less likely that the two are inconsistent, e.g., credentials
are correctly mapped to security policies via `Compress`.

The mechanism by which kernel modules gain access to
`TbfHeaderV2Credentials` with which to construct `ShortID`s for access
tables is outside of scope for this document and are system-specific.
The structure implementing `Verifier` and `Compress` typically has
additional traits or methods that expose these. 

For example, suppose there is a system that wants to grant extra
permissions to Tock binaries with a `TbfHeaderV2Credentials` of
`Rsa4096Key` with the public key of a certain university researcher. A
structure implementing `Verifier` and `Compress` stores a copy of this
key, and returns `Accept` to calls to `check_credentials` with valid
`TbfHeaderV2Credentials` with this key. Calls to `Compress` return
`ShortID {id: 0}` for all credentials except `Rsa4096Key` with this
key, for which it returns `ShortID {id: 1}`. The structure also has a
method `owner_id`, which returns `ShortID {id: 1}`.

Kernel modules which want to give these processes extra permissions
can check whether the `ShortID` associated with a process matches the
`ShortID` returned from `owner_id`. Alternatively, when they are
initialized, they can be passed a slice or array of `ShortID`s which
are allowed; system initialization generates this set once and passes
it into the module so it does not need to maintain a reference to the
structure implementing `Verifier` and `Compress`.

6 Capsules 
===============================

7 Implementation Considerations
===============================

8 Authors' Addresses
===============================
```
Philip Levis
414 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu

Johnathan Van Why <jrvanwhy@google.com>
```

9 Citations
===============================

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
[TBF]: ../TockBinaryFormat.md "Tock Binary Format"
