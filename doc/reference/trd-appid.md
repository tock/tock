Application IDs (AppID)
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Johnathan Van Why<br/>
**Draft-Created:** 2021/09/01 <br/>
**Draft-Modified:** 2022/01/27 <br/>
**Draft-Version:** 5 <br/>
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
compliance with [TRD1][TRD1].

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
an application. Multiple binaries can be associated with a single
application. For example, software updates may cause a system to have
more than one version of an application, such that it can roll back to
the old version if there is a problem with the new one. In this case,
there are two different userspace binaries, both associated with the
same application. 

To remain flexible and support many use cases, the Tock kernel makes
minimal assumptions on the structure and form of application
credentials that bind an application identifier to a
binary. Application credentials are arbitrary k-byte sequences that
are stored in an userspace binary's Tock binary format (TBF)
footers. When a Tock board instantiates the kernel, it passes a
reference to an AppID (application identifier) checker, which the
kernel uses to determine the AppIDs of each userspace binary it reads
and decide whether to load the binary into a process.

The Tock kernel ensures that each running process has a unique
application identifier; if two userspace binaries have the same AppID,
the kernel will only permit one of them to run at any time.


Most of the complications in AppIDs stem from the fact that they are a
general mechanism used for many different use cases. Therefore, the
exact structure and semantics of application credentials can vary
widely. Tock's TBF footer formats, kernel interfaces and mechanisms
must accommodate this wide range.

The interfaces and standard implementations for AppIDs and AppID
checkers are in the kernel crate, in the module
`process_checking`. There are three main traits:

  * `kernel::process_checking::AppCredentialsChecker` is responsible
  for defining which types of application identifiers the kernel
  accepts and whether it accepts a particular application identifier
  for a specific application binary. The kernel only loads userspace
  programs that the kernel's `AppCredentialsChecker` accepts.
  
  * `kernel::process_checking::AppIdentification` compares the
  application identifiers of two processes and reports whether they
  differ. The kernel uses this trait to ensure that each running
  process has a unique application identifier.

  * `kernel::process_checking::Compress` compresses application
  identifiers into short, 32-bit identifiers called
  `ShortID`s. `ShortID`s provide a mechanism for fast comparison,
  e.g., for an application identifier against an access control list.


2 Terminology
===============================

This document uses several terms in precise ways. Because these terms
overlap somewhat with general terminology in the Tock kernel, this
section defines them for clarity. The Tock kernel often uses the term
"application" to refer to what this document calls a "process binary."

**Userspace Binary**: a code image compiled to run in a Tock process,
consisting of text, data, read-only data, and other segments.

**TBF Object**: a [Tock binary format][TBF] object stored on a Tock
device, containing TBF headers, a Userspace Binary, and TBF footers.

**Application**: userspace software developed and maintained by an
individual, group, corporation, or organization that meets the
requirements of a Tock device use case. An Application can have
multiple Userspace Binaries, e.g., to support versioning.

**Application Identifier**: a numerical identifier for an application.
Each loaded process has a single Application Identifier. Application
Identifiers are not unique across loaded processes: multiple loaded
processes can share the same application identifier. Application
Identifiers, however, are unique across running processes. If multiple
loaded processes share the same Application Identifier, at most one of
them can be runnable at any time. An Application Identifier can be
persistent across boots or restarts of a userspace binary. The Tock
kernel assigns Application Identifiers to processes using a Process
Checking Policy.

**Application Credentials**: data that binds an Application Identifier
to an loaded process. Application Credentials are usually stored in
[Tock Binary Format][TBF] footers. A TBF object can have multiple
ApplicationCcredentials.

**Credentials Checker**: a component of the Tock kernel which is
responsible for validating Application Credentials and assigning
Application Identifiers based on them.

**Credentials Checking Policy**: the algorithm that a Credentials
Checking uses to assign an Application Identifier to a loaded
process. A Credentials Checking Policy defines an Application Identifier
space. Two Tock kernels, using different Credentials Checking Policies
and loading the same TBF object into one of their processes, can and
often do assign different Application Identifiers to those processes.

**Global Application Identifier**: an application identifier which,
given an expected Credentials Checking Policy, is globally consistent
across all TBF objects for a particular application as well as unique
to that Application. All instances of the Application loaded with this
Credentials Checking Policy have this Application Identifier. No
instances of other Applications loaded with this Credentials Checking
Policy have this Application Identifier. An example of a Global
Application Identifer is a public key used to generating Application
Credentials for every TBF Object of a single Application.

**Local Application Identifier**: an Application Identifier which is
locally unique for the Credentials Checking Policy that assigned
it. The same TBF Object, loaded with the same Credentials Checking
Policy on another Tock node, may have a different Application
Identifier. An example of a Local Application Identifier is an
incrementing counter that the Credentials Checking Policy checks for
uniqueness (skipping values already in use if it loops around).

**Short ID**: a 32-bit compressed representation of an Application
Identifier.

In normal use of Tock, TBF Objects are copied into an application
flash region by a software tool. When the Tock kernel boots, it scans
this application flash region for TBF Objects. After inspecting the
Userspace Binary and TBF headers in a TBF Object, the kernel assigns
it an Application Identifier and decides whether to run it.

3 Application Identifiers and Application Credentials
===============================

There is a relationship between Application Identifiers and
Application Credentials, but they are not the same thing. An
Application Identifier is a numerical representation of the
Application's Identity, while credentials are the data that, combined
with a Credentials Checking Policy, bind an Application Identifier to a
process. 

Suppose there are two versions (v1.1 and v1.2) of the same
Application. They have different Userspace Binaries. Each version has
an Application Credentials consisting of a cryptographic hash of their
TBF headers and Userspace Binary signed by a known public key. In this
use case (supported by a Process Checking Policty), the public key
defines the Application Identifier: all versions of this Application
have Application Credentials signed by this key.  The two versions
have different Application Credentials, because their hashes differ,
but they have the same Application Identifier.

Every running Tock process MUST have an Application Identifier.
Application Identifiers MUST be unique across running processes in a
Tock system.  Global Application Identifiers MUST persist across
process restarts or reloads.

If the Credentials Checking Policy assigns the same Application Identifier
to multiple processes, then the Tock kernel MUST NOT run more than one
of them at any given time. Following the above example, the Tock
kernel can run v1.1 or v1.2 of the Application, but will not run both
simultaneously. A kernel MAY perform this check by comparing Short IDs
generated from application identifiers using the `Compress` trait
(described below). Kernels using Short IDs to test collisions between
application identifiers SHOULD implement `Compress` in a manner that
minimizes cases when two different valid application identifiers
compress to the same Short ID (e.g., taking the low-order bits of a
strong cryptographic hash function, or using a known, deterministic
mapping).

In cases when a TBF Object does not have any application credentials,
the Credentials Checking Policy MAY assign it a global or local
application identifier. If the verifier policy does not assign a TBF
Object an Application Identifier then the kernel MUST NOT run that
process.

Consider these five use cases.

  1. A TBF Object with no application credentials: it only runs on
  kernels that are willing to load TBF Objects without credentials
  (e.g., research systems). The Credentials Checking Policy defines that
  TBF Objects with no credentials have a Global Application Identifier
  of a SHA256 hash of the application binary.

  1. The verifier policy defines that the Global Application
  Identifier of a process is the public key used to generate an
  Application Credentials for the TBF Object.  Before verifiying a
  signature in a TBF footer, the Credentials Checking Policy decides
  whether to it accepts the associated public key. The Process Checking Pol
  assigns a global application identifier as the public key in the TBF
  header.

  1. Multiple separate process binaries that run concurrently need to
  be signed with a single public key. Each process binary is
  identified by a unique identifier I. Application credentials for
  these process binaries consist of a TBF header containing an ECDSA
  public key, the unique identifier I, and a signature over the
  process binary and I signed with the private key corresponding to
  the public key in the header. The kernel decides whether to accept a
  particular public key for verification. The verifier policy assigns
  a global application identifier as the concatenation of the public
  key and I.

  1. A Tock system wants to load the same process binary in 
  two different processes at the same time. It cannot. Every process
  binary has a single application identifier, and Tock will not run
  two processes with the same application identifier.

  1. A Tock system wants to load the same application binary
  in two different processes at the same time. The system administrator
  installs two process binaries on the device, which contain the
  same application binary. The process binaries have no credentials.
  The verifier policy assigns a local application identifier to each
  process binary based on its order in application flash.

An application identifier provides an identity for an application
binary. It allows the Tock kernel to know about the provenance and
origin of the binary and make access control or security decisions
based on this information. For example, a kernel may allow only
applications whose credentials use a particular trusted public key to
access restricted functionality, but restrict other applications to
use a subset of available system calls.

Application identifiers are distinct from process
identifiers. An application identifier is per-application (persists
across restarts of a process binary, for example), while a process
identifier identifies a particular execution of that binary. At any
time on a Tock device, each process has a unique process identifier,
but they can be re-used over time (like POSIX process identifiers).

As the above examples illustrate, application credentials can vary in
size and content. The credentials that a kernel's verifier policy will
accept depends on its use case. Certain devices will only accept
credentials which include a particular public key, while others will
accept many. Furthermore, the internal format of these credentials can
vary.  Finally, the cryptography used in credentials can vary, either
due to security policies or certification requirements.

Because the verifier policy is responsible for assigning application
identifiers to process binaries, it is possible for the same process
binary to have different application identifiers on different Tock
systems.  For example, suppose a process binary has two application
credential TBF headers: one signs with a key A, and the other with key
B. Tock systems using a verifier policy that accepts key A may assign
A as the global application identifier, while Tock systems using a
different verifier policy that accepts key B may assign B as the
global application identifier.

4 Credentials in Tock Binary Format Objects
===============================

Application credentials are usually stored in a [Tock Binary
Format][TBF] object, along with the process binary they are associated
with. They are usually stored as footers (after the TBF header and
application binary) to simplify computing integrity values such as
checksums or hashes. This requires have a TBF header that specifies
where the application binary ends and the footers begin, information
which the previous `TbfHeaderV2Main` header (the Main Header) does not
include.  Including application credentials in a process binary
therefore requires using an alternative `TbfHeaderV2Program` header
(the Program Header), which specifics where footers begin. This
section describes the format and semantics of Program Headers and
Credentials Footers.


4.1 Program Header
-------------------------------

The Program Header is similar to the Main Header, in that it specifies the
offset of the entry function of the executable and memory parameters. It
adds one field, `binary_end_offset`, which indicates the offset at which
the application binary ends within the TBF object. The space between
this offset and the end of the TBF object is reserved for footers.

This is the format of a Program Header:

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (9)    | Length (16) | init_fn_offset            |
+-------------+-------------+---------------------------+
| protected_size            | min_ram_size              |
+---------------------------+---------------------------+
| binary_end_offset         |
+---------------------------+
```

It is represented in the Tock kernel with this Rust structure:

```rust
pub struct TbfHeaderV2Program {
    init_fn_offset: u32,
    protected_size: u32,
    minimum_ram_size: u32,
    binary_end_offset: u32,
}
```

A TBF object MUST NOT have both a Program Header and a Main Header and
MUST NOT have more than one Program Header.

4.2 Credentials Footer
-------------------------------

To support credentials in Tock binaries, the Tock Binary Format has a
`TbfFooterV2Credentials` TLV. This TLV is variable length and has two
fields, a 32-bit value specifying the `format` of the credentials and
a variable length `data` field. The `format` field defines the format
and size of the `data` field. Each value of the `format` field except
`Padding` MUST have a fixed data size and format. This is the format
of a Credentials Footer:

```
0             2             4                           8
+-------------+-------------+---------------------------+
| Type (128)  | Length      | format                    |
+-------------+-------------+---------------------------+
| data                      |
+-------------+--------...--+
```

It is represented in the Tock kernel with this structure:

```rust
pub struct TbfFooterV2Credentials {
    format: TbfFooterV2CredentialsType,
    data: &[u8],
}
```

Currently supported values of `format` are:

```rust
pub enum TbfFooterV2CredentialsType {
    Padding = 0,
    CleartextID = 1,
    Rsa3072Key = 2,
    Rsa4096Key = 3,
    Rsa3072KeyWithID = 4,
    Rsa4096KeyWithID = 5,
	SHA256 = 6,
	SHA384 = 7,
	SHA512 = 8,
}
```

The `Padding` type has a variable length. This credentials type is
used to reserve space for future credentials or pad their placement.

The `CleartextID` type has a data length of 8 bytes. It contains a
64-bit number in big-endian format representing an application
identifier.

The `Rsa3072Key` type has a data of length of 768 bytes. It contains
a public 3072-bit RSA key (384 bytes), followed by a 384-byte
ciphertext block, consisting of the SHA512 hash of the application
binary in this process binary, signed by the private key of the public
key in the TLV.

The `Rsa4096Key` type has a data of length of 1024 bytes. It contains
a public 4096-bit RSA key (512 bytes), followed by a 512-byte
ciphertext block, consisting of the SHA512 hash of the application
binary in this process binary, encrypted by the private key of the
public key in the TLV.

The `Rsa3072KeyWithID` type has a data of length of 768 bytes. It
contains a public 3072-bit RSA key (384 bytes), followed by a 384-byte
ciphertext block, consisting of the SHA512 hash of the application
binary in this process binary followed by a 32-bit application ID and
padded with zeroes, encrypted by the private key of the public key in
the TLV.

The `Rsa4096KeyWithID` type has a data of length of 1024 bytes. It
contains a public 4096-bit RSA key (512 bytes), followed by a 512-byte
ciphertext block, consisting of the SHA512 hash of the application
binary in this process binary followed by a 32-bit application ID and
padded with zeroes, encrypted by the private key of the public key in
the TLV.

The `SHA256` type has a data length of 32 bytes. It contains a 256-bit
(32 byte) SHA256 hash of the application binary.

The `SHA384` type has a data length of 48 bytes. It contains a 384-bit
(48 byte) SHA384 hash of the application binary.

The `SHA512` type has a data length of 64 bytes. It contains a 512-bit
(64 byte) SHA512 hash of the application binary.

`TbfFooterV2Credentials` type follow the compiled app binary in a
TBF object.  If a `TbfFooterV2Credentials` footer includes a
cryptographic hash, signature, or other value to check the integrity
of a process binary, the computation of this value MUST include the
complete TBF Header and the compiled app binary.

Integrity values MUST be computed over the TBF Header and compiled
application binary, i.e., from the start of the TBF object until
`binary_end_offset`. Computing an integrity value in a Credentials
Footer MUST NOT include the contents of Footers. If new metadata associated
with an application binary needs to be covered by integrity, it MUST
be a Header. If new metadata associated with an application binary needs to
not be covered by integrity, it MUST be a Foorter.

5 `AppCredentialsChecker` trait
===============================

The `AppCredentialsChecker` trait defines an interface to a module
that accepts, passes on, or rejects application credentials. When a
Tock board asks the kernel to load processes, it passes a reference to
a `AppCredentialsChecker`, which the kernel uses to check credentials.
An implementer of `AppCredentialsChecker` sets the security policy of
process binary loading by deciding which types of credentials, and
which credentials, are acceptable and which are rejected.


```rust
pub enum CheckResult {
    Accept,
    Pass,
    Reject
}

pub trait Client<'a> {
    fn check_done(&self,
                  result: Result<CheckResult, ErrorCode>,
                  credentials: TbfFooterV2Credentials,
                  binary: &'a [u8]);
}

pub trait AppCredentialsChecker<'a> {
    fn set_client(&self, client: &'a dyn Client<'a>);
    fn require_credentials(&self) -> bool;
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'a [u8])  ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}
```

When the kernel successfully parses and loads a process binary into a
`Process` structure, it places it into a state indicating that its
integrity has not been checked. If the process loading function is
provided an instance of `AppCredentialsChecker`, it uses this
instances to check whether each of the loaded proceses is safe to run.
If no `AppCredentialsChecker` is provided it skips this check and runs
all successfully loaded processes.

To check the integrity of processes, the kernel scans the footers in
in each process binary in order, from the beginning of that process's
footer region. At each `TbfFooterV2Credentials` footer it encounters,
it calls `check_credentials` on the provided
`AppCredentialsChecker`. If the `AppCredentialsChecker` returns
`Accept`, the kernel stops processing credentials and calls
`mark_credentials_pass` on the process, which makes it runnable. If
the `Verifier` returns `Reject`, the kernel stops processing
credentials and calls `mark_credentials_fail` on the process, which
makes it unrunnable. 

If the `AppCredentialsChecker` returns `Pass`, the kernel tries the
next `TbfFooterV2Credentials`, if there is one. If the kernel reaches
the end of the TBF Footers (or if there is a Main Header and so no
Footers) without encountering a `Reject` or `Accept` result, it calls
`require_credentials` to ask the `AppCredentialsChecker` what the
default behavior is.  If `require_credentials` returns `true`, the
kernel calls `mark_credentials_fail` on the process, which makes it
unrunnable. If `require_credentials` returns `false`, the kernel calls
`mark_credentials_pass` on the process, which makes it runnable.  If a
process binary has no `TbfFooterV2Credentials` footers then there will
be no `Accept` or `Reject` results and `require_credentials` defines
whether to load such a binary.

The `binary` argument to `check_credentials` is a reference to slice
covering the process binary, from the end of the TBF Header to the
location indicated by the `binary_end_offset` field in the Program
Header. The size of this slice is therefore equal to
`binary_end_offset`.

6 Application Identifiers and the `ApplicationIdentification` trait
==============================

The `ApplicationIdentification` trait defines an API for a module that
decides whether two processes have the same application identifier.
An implementer of `ApplicationIdentification` implements the
`different_identifier` method, which performs a pairwise comparison of
two processes. There is also a `has_unique_identifier` method, which
compares a process against all of the processes in a process
array. The trait has a default implementation of this method, but
implementations may override it.

```rust
trait AppIdentification {
    // Returns true if the two processes have different application
	// identifiers.
	fn different_identifier(&self, 
	                        processA: &dyn Process,
				  		    processB: &dyn Process) -> bool;
							
	// Return whether `process` has a unique application identifier (whether 
	// it does not collide with the application identifier of any `Process`
	// in `processes`.
    fn has_unique_identifier(&self,
                             process: &dyn Process,
                             processes: &[Option<&dyn Process>]) -> bool {
        let len = processes.len();
        if process.get_state() != State::Unstarted && 
		   process.get_state() != State::Terminated {
            return false;
        }

        // Note that this causes `process` to compare against itself;
        // however, since `process` should not be running, it will
        // not check the identifiers and say they are different. This means
        // this method returns false if the process is running.
        for i in 0..len {
            let checked_process = processes[i];
            let diff = checked_process
                .map_or(true, |other| {
                    !other.is_running() ||
                        self.different_identifier(process, other)
                });
            if !diff {
                return false;
            }
        }
        true
}
```

This interface encapsulates the method by which a module assigns or
calculates application identifers. The kernel uses this interfaces to
determine if a process submitted to run will collide with a running
process's application identifier.

7 Short IDs and the `Compress` trait
===============================

While `TbfFooterV2Credentials` define the identity and credentials of
an application, they are typically large data structures that are too
large to store in RAM. When parts of the kernel wish to apply
application-based security or access policies, they need a concise way
to represent these policies. Requiring policies to be encoded in terms
of application credentials (or application identifiers) is extremely
costly: a table, for example, that says that only applications signed
with a particular 4096-bit RSA key can access certain system calls
requires storing the whole 4096-bit key. If there are multiple such
security policies through the kernel, they must each store this
information.

The `Compress` trait provides a mechanism to map the application
identifier defined by application credentials to a small (32-bit)
integer, which can be used throughout the kernel as an identifier
for security policies. For example, suppose that a device wants to
grant access to all application binaries signed by a certain 3072-bit
RSA key. The `Compress` trait can map all such
`TbfFooterV2Credentials` to a known identifier. This identifier is
stored in the process structure. Access control systems within the
kernel can define their policies in terms of these identifiers, such
that they can check access by comparing 32-bit integers rather than
384-byte keys.

```rust
#[derive(Clone, Copy, Eq)]
struct ShortID {
    id: u32
}

pub trait Compress {
    fn to_short_id(credentials: &TbfFooterV2Credentials) -> Option<ShortID>;
}
```

The `to_short_id` method returns an `Option` so that it has a clear
default action if it does not recognize or give any special meaning to
the credentials passed. A return value of `None` semantically means
that these credentials do not map to any known security group or set
of privileges, while a `Some` result denotes the credentials map to
a known security group or set of privileges.

Generally, the same structure that implements `Verifier` also
implements `Compress`. This allows it to share copies of public keys
or other credentials that it uses to make decisions, reducing flash
space dedicated to these constants. Doing so also makes it less likely
that the two are inconsistent, e.g., that credentials are correctly
mapped to security policies via `Compress`.

The mechanism by which kernel modules gain access to
`TbfFooterV2Credentials` with which to construct `ShortID`s for access
tables is outside of scope for this document and is system-specific.
The structure implementing `Verifier` and `Compress` typically has
additional traits or methods that expose these.

For example, suppose there is a system that wants to grant extra
permissions to Tock binaries with a `TbfFooterV2Credentials` of
`Rsa4096Key` with a certain public key. The public key is the global
application identifier of the process binary. Note this means only one
process signed with that key can run at any time. 

A structure implementing `Verifier` and `Compress` stores a copy of
this key, and returns `Accept` to calls to `check_credentials` with
valid `TbfFooterV2Credentials` using this key. Calls to `Compress`
return `None` for all credentials except a `Rsa4096Key` with this key,
for which it returns `ShortID {id: 1}`. The structure also has a
method `privileged_id`, which returns `ShortID {id: 1}`.

Kernel modules which want to give these processes extra permissions
can check whether the `ShortID` associated with a process matches the
`ShortID` returned from `privileged_id`. Alternatively, when they are
initialized, they can be passed a slice or array of `ShortID`s which
are allowed; system initialization generates this set once and passes
it into the module so it does not need to maintain a reference to the
structure implementing `Verifier` and `Compress`.

It is RECOMMENDED that the `id` field of `ShortID` be completely
hidden and unknown to modules that use `ShortID` to manage security
policies. They should depend on obtaining `ShortID` values based on
known names or methods, as in the `privileged_id` example above. The
exact `id` values used is an internal implementation decision for the
implementer of `Compress`. Doing so more cleanly decouples modules
through APIs and does not leak internal state.

`ShortID` values MUST be locally unique among running processes.  The
mapping between global application identifiers and `ShortID` values
MUST be deterministic. 

`ShortID` values MAY persist across boots and restarts of a process
binary. If `ShortID` is derived from a global application identifier,
then it is by definition persistent, since it is a determinstic
mapping from the identifier. `ShortID` values derived from local
application identifiers, however, MAY be transient and not persist.


8 Capsules
===============================

9 Implementation Considerations
===============================

10 Authors' Addresses
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

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
[TBF]: ../TockBinaryFormat.md "Tock Binary Format"
