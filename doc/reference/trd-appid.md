Application IDs (AppID), Credentials, and Process Loading
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Johnathan Van Why<br/>
**Draft-Created:** 2021/09/01 <br/>
**Draft-Modified:** 2022/10/14 <br/>
**Draft-Version:** 10 <br/>
**Draft-Discuss:** tock-dev@googlegroups.com<br/>

Abstract
-------------------------------

This document describes the design and implementation of application
identifiers (AppIDs) in the Tock operating system. AppIDs provide a
mechanism to identify the application contained in a userspace binary
that is distinct from a process identifier.  AppIDs allow the kernel
to apply security policies to applications as their code evolves and
their binaries change. A board defines how the kernel verifies AppIDs
and which AppIDs the kernel will load. This document describes the
Rust traits and software architecture for AppIDs as well as the
reasoning behind them. This document is in full compliance with
[TRD1][TRD1].

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
`process_checker`. There are three main traits:

  * `kernel::process_checker::AppCredentialsChecker` is responsible
  for defining which types of application identifiers the kernel
  accepts and whether it accepts a particular application identifier
  for a specific application binary. The kernel only loads userspace
  programs that the kernel's `AppCredentialsChecker` accepts.

  * `kernel::process_checker::AppUniqueness` compares the application
  identifiers of two processes and reports whether they differ. The
  kernel uses this trait to ensure that each running process has a
  unique application identifier.

  * `kernel::process_checker::Compress` compresses application
  identifiers into short, 32-bit identifiers called
  `ShortID`s. `ShortID`s provide a mechanism for fast comparison,
  e.g., for an application identifier against an access control list.

Example implementations can be found in
`kernel::process_checker::basic`.


2 Terminology
===============================

This document uses several terms in precise ways. Because these terms
overlap somewhat with general terminology in the Tock kernel, this
section defines them for clarity. The Tock kernel often uses the term
"application" to refer to what this document calls an "Application
Binary."

**Userspace Binary**: a code image compiled to run in a Tock process,
consisting of text, data, read-only data, and other segments.

**TBF Object**: a [Tock binary format][TBF] object stored on a Tock
device, containing TBF headers, a Userspace Binary, and TBF footers.
TBF Objects are typically generated from ELF files using the
[`elf2tab`](https://github.com/tock/elf2tab) tool and are the
standard binary format for Tock userspace processes.

**Application**: userspace software developed and maintained by an
individual, group, corporation, or organization that meets the
requirements of a Tock device use case. An Application can have
multiple Userspace Binaries, e.g., to support versioning.

**Application Identifier**: a numerical identifier for an application.
Each loaded process has a single Application Identifier. Application
Identifiers are not unique across loaded processes: multiple loaded
processes can share the same application identifier. Application
Identifiers, however, are unique across *running* processes. If
multiple loaded processes share the same Application Identifier, at
most one of them can run at any time. An Application Identifier can be
persistent across boots or restarts of a userspace binary. The Tock
kernel assigns Application Identifiers to processes using a
Identifier Policy.

**Application Credentials**: metadata that establish integrity of a
Userspace Binary and potentially bind an Application Identifier to an
loaded process. Application Credentials are usually stored in [Tock
Binary Format][TBF] footers. A TBF object can have multiple
Application Credentials.

**Process Checker**: the component of the Tock kernel which is
responsible for validating Application Credentials and determining
which Application Credential (if any) the kernel should apply to a
process.

**Identifier Policy**: the algorithm that the Process Checker uses to
assign Application Identifiers to processes.  An Identifier Policy
defines an Application Identifier space. An Identifier Policy can
derive Application Identifiers from Application Credentials or other
process state.

**Credentials Checking Policy**: the algorithm that the Process
Checker uses to decide how Tock responds to particular Application
Credentials. The boot sequence typically passes the Credentials
Checking Policy to the kernel at startup, which the Process
Checker then uses when the kernel loads processes.

**Global Application Identifier**: an Application Identifier which,
given an expected combination of Credentials Checking Policy and
Identifier Policy, is both globally consistent across all TBF objects
for a particular Application and unique to that Application. All
instances of the Application loaded with this combination of policies
have this Application Identifier. No instances of other Applications
loaded with this Credentials Checking Policy have this Application
Identifier. One example of a Global Application Identifer is a public
key used to verify the digital signature of every TBF Object of a
single Application. Another example of a Global Application Identifier
is a string name stored in a TBF Object header; in this case the party
installing TBF Objects needs to make sure there are no unintended
collisions between these string names.

**Locally Unique Application Identifier**: a special kind of
Application Identifier that is by definition unique from all other
Application Identifiers. Locally Unique Application Indentifiers do
not have a concrete value that can be examined or stored. All tests
for equality with a Locally Unique Application Identifier returns
false. Locally Unique Application Identifiers exist in part to be an
easy way to indicate that a process has no special privileges and its
identity is irrelevant from a security standpoint.

**Short ID**: a 32-bit compressed representation of an Application
Identifier. Application Identifiers can be large (e.g., an RSA key) or
expensive to compare (a string name); Short IDs exist as a way for an
Identifier Policy to map Application Identifiers to a small identifier
space in order to improve both the space and time costs of checking
identity.

In normal use of Tock, a software tool running on a host copies TBF
Objects into an application flash region. When the Tock kernel boots,
it scans this application flash region for TBF Objects. After
inspecting the Userspace Binary, TBF headers, and TBF Footers
in a TBF Object, the kernel assigns it an Application Identifier and
decides whether to run it.

3 Application Identifiers and Application Credentials
===============================
 
Application Identifiers and Application Credentials are related but
they are not the same thing. An Application Identifier is a numerical
representation of the Application's identity. Application Credentials
are data that, combined with an Identifier Policy, can
cryptographically bind an Application Identifier to a process.

For example, suppose there are two versions (v1.1 and v1.2) of the
same Application. They have different Userspace Binaries. Each version
has an Application Credentials consisting of a signature over the TBF
headers and Userspace Binary, signed by a known public key. The
Identifier Policy is that the public key defines the Application
Identifier: all versions of this Application have Application
Credentials signed by this key.  The two versions have different
Application Credentials, because their hashes differ, but they have
the same Application Identifier.

3.1 Application Identifiers
-------------------------------

The key restriction Application Identifiers impose is that the kernel
MUST NOT simultaneously run two processes that have the same
Application Identifier. This restriction is because an Application
Identifier provides an identity for a Userspace Binary.  Two processes
with the same Application Identifier are two copies or versions of the
same Application. As Application Identifiers are used to control
access to resources such as storage, this restriction ensures there is
at most one process accessing resources or data belonging to an
Application Identifier, which precludes the need for consistency
mechanisms for concurrent access.

Application Identifiers can be used for security policy decisions in
the rest of the kernel. For example, a kernel may allow only
Applications whose Application Credentials use a particular trusted
public key to access restricted functionality, but restrict other
applications to use a subset of available system calls. By defining
the Application Identifier of a process to be the public key, the
system can map this key to a Short IDs (described below) that gives
access to retricted functionality.

3.1.1 Global Application Identifiers
-------------------------------

Global Application Identifiers are a class of Application Identifiers
that have properties which make them useful for security policies.
For Applications that use Global Application Identifiers, the
combination of the Application Credentials put in TBF Objects,
Credentials Checking Policy, and Identifier Policy establish a
one-to-one mapping between Applications and Global Application
Identifiers. If an Application has a Global Application Identifier,
then every process running that Application has that Global
Application Identifier. Conversely, that Global Application Identifier
is unique to that Application; two Applications do not share a Global
Application Identifier.

One important implication of this mapping is that Global Application
Identifiers MUST persist across process restarts or reloads.

Poor management of Global Application Identifiers can lead to
unintended collisions. For example, an Identifier Policy might define
the Global Application Identifier of processes to be the public key of
a key pair to sign an Application Credential. If a developer
accidentally uses the wrong key to sign a Userspace Binary, the Tock
kernel will think that Userspace Binary is a different Application.
Similarly, if the Identifier Policy uses a string name in a TBF Object
header as the Global Application Identifier, then incorrectly giving
two different programs the same name could lead them to sharing data.

3.1.2 The "Locally Unique" Identifier
-------------------------------

Some Tock use cases do not require a real notion of Application
identity.  In many research or prototype systems, for example, every
Userspace Binary has complete access to the system and there is no
need for persistent storage or identity. Running processes need an
Application Identifier, but in these cases it is not necessary for a
Tock kernel and Application build system to manage Global Application
Identifiers.

In such use cases, the Identifier Policy can assign a special
Application Identifier called the "Locally Unique Identifier".  This
identifier does not have a concrete value: it is simply a value that
is by definition different from all other Application
Identifiers. Because it does not have a concrete value, one cannot
test for equality with Locally Unique Application Identifier. All
comparisons with a Locally Unique Application Identifier return false.

3.2 Application Credentials
-------------------------------

Application Credentials are information stored in TBF Footers. The
exact format and information of Application Credentials are described
in the next section. They typically store cryptographic information
that establishes the Application a Userspace Binary belongs to as well
as provide integrity.

Application Identifiers can, but do not have to be, be derived from
Application Credentials. For example, a Tock system with a permissive
Credentials Checking Policy may allow processes with no Application
Credentials to run, and have an Identifier Policy that defines
Application Identifiers to be the ASCII name stored in a TBF header.

In cases when a TBF Object does not have any Application Credentials,
the Identifier Policy MAY assign it a Global Application Identifier.
This identifier must follow all of the requirements in Section 3.1.1.

The Tock kernel assigns each Tock process a unique process identifier,
which can be re-used over time (like POSIX process identifiers). These
process identifiers are separate from and unrelated to Application
Identifiers.  An Application Identifier identifies an Application,
while a process identifier identifies a particular execution of a
binary. For example, if a Userspace Binary exits and runs a second
time, the second execution will have the same Application Identifier
but may have a different process identifier.

3.3 Example Use Cases
-------------------------------

The following five use cases demonstrate different ways in which
Application Policies can assign Application Identifiers, some of which
use Application Credentials:

  1. **A research system that (memory permitting) runs every Userspace
  Binary loaded on it.** The Identifier Policy assigns every Userspace
  Binary a Locally Unique Application Identifier and the Credentials
  Checking Policy approves TBF Objects independently of their
  credentials.

  1. **A research system which requires no Application Credentials, but
  will not run more than one copy of the same Userspace Binary**.  The
  Identifier Policy defines that TBF Objects with no credentials have
  a Global Application Identifier of a SHA256 hash of the Application
  Binary. The system does not support versioning of Userspace
  Binaries: two different versions of the same Application have
  different Application Identifiers.

  1. **A system which runs only a small number of pre-defined
  Applications and an Application is defined by a particular public
  RSA key.** The Credentials Checking Policy only accepts TBF Objects
  with an Application Credentials containing an RSA signature from a
  small number of pre-approved keys. The Identifier Policies defines
  that the Global Application Identifier of a process is the public
  key used to generate the accepted Application Credentials for the
  TBF Object.  Before verifiying a signature in a TBF footer, the
  Process Checker decides whether to it accepts the associated public
  key using the Credentials Checking Policy. The Identifier Policy
  assigns a Global Application Identifier as the public key in the TBF
  footer.

  1. **A system which runs any number of Applications but all
  Applications must be signed by a particular RSA key.** The
  Credentials Checking Policy only accepts TBF Objects with a
  Credentials of an RSA signature from the approved key. The
  Identifier Policy defines the Application Identifier as the UTF-8
  encoded package name stored in the TBF Header (or "" if none is
  stored). Two Userspace Binaries with the same package name will not
  run concurrently.

  1. **A system that loads the same Userspace Binary in multiple
  different processes at the same time.** The Identifier Policy
  assigns a Userspace Binary a Locally Unique Identifier. If the
  Userspace Binary needs integrity or authenticity then the
  Credentials Checking Policy can require signatures. This differs
  from the first example in that a single Userspace Binary can be
  loaded into multiple processes, instead of loading each Userspace
  Binary once. The use cases are different but can (in terms of
  identifiers and credentials) implemented the same way.

As the above examples illustrate, Application Credentials can vary in
size and content. The credentials that a kernel's Credentials Checking
Policy will accept depends on its use case. Certain devices might only
accept Application Credentials which include a particular public key,
while others will accept many. Furthermore, the internal format of
these credentials can vary.  Finally, the cryptography used in
credentials can vary, either due to security policies or certification
requirements.

Because the Identifier Policy is responsible for assigning
Application Identifiers to processes, it is possible for the same
Userspace Binary to have different Application Identifiers on
different Tock systems.  For example, suppose a TBF Object has two
Application Credentials TBF footers: one signs with a key A, and the
other with key B. Tock systems using a Credentials Checking Policy
that accepts key A may use A as the Global Application Identifier,
while Tock systems using a different policy that accepts key B may
use B as the Global Application Identifier.

4 Process Loading
===============================

Tock defines its process loading algorithm in order to provide
deterministic behavior in the presence of colliding Application
Identifiers.  This algorithm is designed to protect against downgrade
attacks and misconfiguration. Processes have five possible states in
the loading stage: Unloaded, CredentialsUnchecked, CredentialsFailed,
CredentialsApproved and Yielded/Running. Processes start in the Unloaded
state.

First, when it boots, the Tock kernel scans the TBF Objects stored in
its application flash region. It checks that the TBF Objects are valid
and can run on the system (e.g., do not require a newer kernel
version).  It loads them in order from lowest to highest address. Each
successfully loaded TBF Object is loaded into one of the process
slots, that process is moved into the `CredentialsUnchecked` state.

Once the application flash has been scanned or the last process slot
has been filled, the kernel checks the credentials of the processes.
Using the provided Credentials Checking Policy (described in Section
6), it decides whether each process has permission to run. It moves
processes whose TBF Objects are not allowed to run into the
`CredentialsFailed` state. It moves processes whose TBF Objects are
allowed to run into the `CredentialsApproved` state. Moving a process
into the `CredentialsApproved` state triggers the main kernel loop to
examine the process.

The `CredentialsApproved` state indicates that the process is runnable
in terms of its credentials. However, at any given time it might not
be allowed to run because its Application Identifier or Short ID
conflicts with another processs. When the main kernel loop examines a
process in the `CredentialsApproved` state, it determines whether to
run the process based on its Application Identifier, Short ID, and the
Application Binary version number (stored in the Program Header,
described in Section 5.1). At boot, the kernel starts a process if
either of:

  - The process has a unique Application Identifier and Short ID,
  - The process has a higher Application Binary version number than
    all processes it shares its Application Identifier or Short ID with,

If two processes which share a Short ID or Application ID have the
same version number, the kernel starts one of them. The one which
starts is determined by the scheduling policy of the kernel (whichever
one is run first).

The Unloaded, CredentialsUnchecked, CredentialsFailed,
CredentialsApproved and Running states describe the conceptual state
machine of a process at loading; the values or names of state
variables in the kernel implemementation might be different. The exact
Tock process state machine and names of its states is outside the
scope of this document.

Once a Tock system is running, management interfaces may change the set
of running processes from those which the boot sequence
selected. E.g., the process console might terminate a process so that
it can run a different process with the same Short ID and a lower
Userspace Binary version number (rollback). The kernel maintains that
a running process has a unique Application Identifier and a unique
Short ID among running processes.

5 Credentials and Version in Tock Binary Format Objects
===============================

This section describes the format and semantics of Program Headers and
Credentials Footers.

Application Credentials are usually stored in a TBF Object, along with
the Userspace Binary they are associated with. They are usually stored
as footers (after the TBF header and Userspace Binary) to simplify
computing integrity values such as checksums or hashes. This requires
have a TBF header that specifies where the application binary ends and
the footers begin, information which the `TbfHeaderV2Main` header (the
Main Header) does not include.  Including Application Credentials in a
TBF Object therefore requires using an alternative
`TbfHeaderV2Program` header (the Program Header), which specifics
where footers begin.

The Tock process loading algorithm version numbers when deciding the
order to load processes in. Version numbers are stored in a TBF Object
in the Version field of a TBF Program Header.


5.1 Program Header
-------------------------------

The Program Header is similar to the Main Header, in that it specifies
the offset of the entry function of the executable and memory
parameters. It adds one field, `binary_end_offset`, which indicates
the offset at which the Userspace Binary ends within the TBF
object. The space between this offset and the end of the TBF object is
reserved for footers.

This is the format of a Program Header:

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (9)    | Length (16) | init_fn_offset            |
+-------------+-------------+---------------------------+
| protected_size            | min_ram_size              |
+---------------------------+---------------------------+
| binary_end_offset         | version                   |
+---------------------------+---------------------------+
```

It is represented in the Tock kernel with this Rust structure:

```rust
pub struct TbfHeaderV2Program {
    init_fn_offset: u32,
    protected_size: u32,
    minimum_ram_size: u32,
    binary_end_offset: u32,
    version: u32,
}
```

A TBF object MUST NOT have more than one Program Header. If a TBF
Object has both a Program Header and a Main Header, the kernel's
policy decides which is used. For example, older kernels that do not
understand a Program Header may use the Main Header, while newer
kernels may choose the Program Header.

5.2 Credentials Footer
-------------------------------

To support credentials, the Tock Binary Format has a
`TbfFooterV2Credentials` TLV. This TLV is variable length and has two
fields, a 32-bit value specifying the `format` of the credentials and
a variable length `data` field. The `format` field defines the format
and size of the `data` field. Each value of the `format` field except
`Reserved` MUST have a fixed data size and format. This is the format
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
    Reserved = 0,
    Rsa3072Key = 1,
    Rsa4096Key = 2,
	SHA256 = 3,
	SHA384 = 4,
	SHA512 = 5,
}
```

The `Reserved` type has a variable length. This credentials type is
used to reserve space for future credentials (e.g., that will be added
by another party in the deployment process). Because the `total_size`
field of a TBF Base Header is covered by integrity, once the total
size of the TBF Object has been decided it cannot be changed: if
credentials need to be added later, space must be reserved for them.

The `Rsa3072Key` type has a data of length of 768 bytes. It contains a
public 3072-bit RSA key (384 bytes), followed by a 384-byte PKCS#1
v1.5 signature using SHA512 (`CKM_SHA512_RSA_PKCS`). It does not
contain a public exponent: the Process Checker is responsible for
storing the public exponent for any key it recognizes.

The `Rsa4096Key` type has a data of length of 1024 bytes. It contains
a public 4096-bit RSA key (512 bytes), followed by a 512-byte PKCS#1
v1.5 signature using SHA512 (`CKM_SHA512_RSA_PKCS`). It does not
contain a public exponent: the Process Checker is responsible for
storing the public exponent for any key it recognizes.

The `SHA256` type has a data length of 32 bytes. It contains a 256-bit
(32 byte) SHA256 hash of the application binary.

The `SHA384` type has a data length of 48 bytes. It contains a 384-bit
(48 byte) SHA384 hash of the application binary.

The `SHA512` type has a data length of 64 bytes. It contains a 512-bit
(64 byte) SHA512 hash of the application binary.

`TbfFooterV2Credentials` follow the compiled app binary in a TBF
object.  If a `TbfFooterV2Credentials` footer includes a cryptographic
hash, signature, or other value to check the integrity of a process
binary, this vlaue MUST be computed over the TBF Header and Userspace
Binary, from the start of the TBF object until
`binary_end_offset`. Computing an integrity value in a Credentials
Footer MUST NOT include the contents of Footers. If new metadata
associated with an application binary needs to be covered by
integrity, it MUST be a Header. If new metadata associated with an
application binary needs to not be covered by integrity, it MUST be a
Footer.

Which types of credentials a Credentials Checking Policy supports are
kernel-specific. For example, an application that only accepts TBF
Objects signed with a particular 4096-bit RSA key can support only
`Rsa4096Key` credentials, while an open research system might support
no credentials. Because the `length` field specifies the length of a
given credentials, not understanding a particular credentials type
does not prevent parsing others.


6 Credentials Checking Policy: the `AppCredentialsChecker` trait
===============================

The `AppCredentialsChecker` trait defines the interface that implements
the Credentials Checking Policy of the Process Checker: it accepts,
passes on, or rejects Application Credentials. When a Tock board asks
the kernel to load processes, it passes a reference to a
`AppCredentialsChecker`, which the kernel uses to check credentials.
An implementer of `AppCredentialsChecker` sets the security policy of
Userspace Binary loading by deciding which types of credentials, and
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

If the kernel has been instructed to check credentials of Userspace
Binaries, after it successfully parses and loads a Userspace Binary
into a `Process` structure, it places the process into the Unchecked
state. The Unchecked state indicates that it has been loaded but that
it may not be allowed or safe to run. 

To check the integrity of a process, the kernel scans the footers in
order, starting at the beginning of that process's footer region. At
each `TbfFooterV2Credentials` footer it encounters, the kernel calls
`check_credentials` on the provided `AppCredentialsChecker`. If
`check_credentials` returns `Accept`, the kernel stops processing
credentials and calls `mark_credentials_pass` on the process, which
transitions it to the `CredentialsApproved` state. If the
`AppCredentialsChecker` returns `Reject`, the kernel stops processing
credentials and calls `mark_credentials_fail` on the process, which
transitions it to the `CredentialsFailed` state.

If the `AppCredentialsChecker` returns `Pass`, the kernel tries the
next `TbfFooterV2Credentials`, if there is one. If the kernel reaches
the end of the TBF Footers (or if there is a Main Header and so no
Footers) without encountering a `Reject` or `Accept` result, it calls
`require_credentials` to ask the `AppCredentialsChecker` what the
default behavior is.  If `require_credentials` returns `true`, the
kernel calls `mark_credentials_fail` on the process, transitioning it
into the `CredentialsFailed` state. If `require_credentials` returns
`false`, the kernel calls `mark_credentials_pass` on the process,
transitioning it to the `CredentialsApproved` state. If a process binary
has no `TbfFooterV2Credentials` footers then there will be no `Accept`
or `Reject` results and `require_credentials` defines whether the
Userspace Binary is runnable.

The `binary` argument to `check_credentials` is a reference to slice
covering the process binary, from the end of the TBF Header to the
location indicated by the `binary_end_offset` field in the Program
Header. The size of this slice is therefore equal to
`binary_end_offset`.

7 Identifier Policy: the `AppUniqueness` trait
==============================

The `AppUniqueness` trait defines the API the Process Checker provides
to decide whether two processes have the same Application Identifier
or Short ID.  An implementer of `AppUniqueness` implements the
`different_identifier` method, which performs a pairwise comparison of
two processes. 

The `process_checker` module implements a `has_unique_identifiers`
function, which compares a process against all of the processes in a
process array, checking that both its Application Identifer and its
Short ID are unique. 

```rust
trait AppUniqueness {
    // Returns true if the two processes have different application
	// identifiers.
	fn different_identifier(&self,
	                        processA: &dyn Process,
                            processB: &dyn Process) -> bool;

}

/// Return whether there is a currently running process that has
/// the same application identifier as `process` OR the same short
/// ID as `process`. This means that if `process` is currently
/// running, `has_unique_identifiers` returns false.
pub fn has_unique_identifiers<AU: AppUniqueness>(&self,
    process: &dyn Process,
    processes: &[Option<&dyn Process>],
    id_differ: &AU,
) -> bool {
    let len = processes.len();
    // If the process is running or not runnable it does not have
    // a unique identifier; these two states describe a process
    // that is potentially runnable, dependent on checking for
    // identifier uniqueness at runtime.
    if process.get_state() != State::CredentialsApproved
        && process.get_state() != State::Terminated
    {
        return false;
    }
    
    // Note that this causes `process` to compare against itself;
    // however, since `process` should not be running, it will
    // not check the identifiers and say they are different. This means
    // this method returns false if the process is running.
    for i in 0..len {
        let checked_process = processes[i];
        let diff = checked_process.map_or(true, |other| {
            !other.is_running()
                || (id_differ.different_identifier(process, other)
                    && other.short_app_id() != process.short_app_id())
        });
        if !diff {
            return false;
        }
    }
    true
}
```

These interfaces encapsulate the methods by which a module assigns or
calculates application identifers. The kernel uses
`has_unique_identifiers` to determine if a process submitted to run
will collide with a running process's application identifier,
which in turn uses `AppUniqueness::different_identifier`.

8 Short IDs and the `Compress` trait
===============================

While `TbfFooterV2Credentials` often define the identity and
credentials of an application, they are large data structures that are
too large to store in RAM. When parts of the kernel wish to apply
security or access policies based on Application Identifiers, they
need a concise way to represent these identifiers. Requiring policies
to be encoded in terms of raw Application Identifiers can be extremely
costly: a table, for example, that says that only Applications signed
with a particular 4096-bit RSA key can access certain system calls
requires storing the whole 4096-bit key. If there are multiple such
security policies through the kernel, they must each store this
information.

The `Compress` trait provides a mechanism to map an Application
Identifier to a small (32-bit) integer called a Short ID. Short IDs
can be used throughout the kernel as an identifier of an Application.

For example, suppose that a device wants to grant access to all
Userspace Binaries signed by a certain 3072-bit RSA key K and has no
other security policies. The Credentials Checking Policy only accepts
`Rsa3072Key` credentials with key K. The `Compress` trait
implementation assigns a Short ID based on a string match with the
process package name, with certain names receiving particular Short
IDs.  Access control systems within the kernel can define their
policies in terms of these identifiers, such that they can check
access by comparing 32-bit integers rather than 384-byte keys.

Short IDs support the concept of a "Locally Unique" identifier by
having a special `LocallyUnique` value. All tests for equality with
`ShortID::LocallyUnique` return false.

8.1 Short ID Properties and Examples
-------------------------------

Given a particular combination of deterministic Identifier Policy and
Credentials Checking Policy, Short IDs have two requirements. They

  1. MUST be unique across running processes,
  1. MUST be consistent across all running instances of an Application
     on Tock systems.

Short IDs are locally unique for three reasons. First, it simplifies
process management and naming: a particular Short ID uniquely
identifies a running process. Second, it ensures that resources bound
to an application identifier (such as non-volatile storage) do not
have to handle concurrent accesses from multiple processes. Finally,
generally one does not want two copies of the same Application
running: they can create conflicting responses and behaviors.

These two requirements restrict the set of possible combinations of
Credentials Checking Policy and Identifier Policy. For example, a
Short ID cannot be an incrementing counter; it must be
deterministically derived from the Application Identifier.

A basic challenge that arises with Short IDs is that they are a form
of compression.  In the ideal case, Short IDs would have two
additional properties:
  - Different Application Identifiers map to different Short IDs, and
  - All Application Identifiers have a concrete Short ID that identifies the Application.
  
Unfortunately, it is not possible to satisfy both of these properties
simultaneously. This is because Short IDs potentially compress
Application Identifiers. Consider, for example, a system where the
Application Identifier is the public key in an Rsa4096Key
credential. Short IDs are 32 bits, but there are more than 2^32
4096-bit RSA keys. If every RSA key receives a different Short ID, and
that Short ID is always the same, after 2^32 keys the Short ID space
is exhausted.

Every algorithm to map Application Identifiers to Short IDs therefore
sacrificies one of these two properties:
  - **Different Application Identifiers can map to the same Short
  ID:** An Identifier Policy with this property is one that uses
  string names as Global Application Identifiers and calculates the
  Short ID of process to be the checksum (or hash) of the string name.
  Two different names can checksum or hash to the same value. These
  collisions, however, can be acceptable if a developer is willing to
  pick string names that do not collide or change them when they do.
  A research or prototyping system might use this Identifier Policy.
  - **Some Application Identifiers do not receive concrete Short
  IDs:** An Identifier Policy with this property is one that uses
  public keys in signature credentials as Application Identifiers and
  has a set of public keys it knows and trust. It maps these known
  keys to a small set of Short IDs (e.g., 1 through N). The system may
  run Userspace Binaries signed by other keys, but assigns them a
  Locally Unique Application identifier, which results in a
  Lcocally Unique Short ID.


8.2 Example Short ID use cases
-------------------------------

Here are three example use cases of Short IDs. 

8.2.1 Use Case 1: Anonymous Applications
-------------------------------

There are many Tock systems that do not particularly care about the
identity of Applications. They do not have security policies, or track
Application Identifiers. A prototyping system whose Credentials
Checking Policy accepts all TBF Objects regardless of Application
Credentials is an example of such a system. At boot, it scans the set
of TBF Objects in application flash, trying to load and run each one
until it runs out of resources (RAM, process slots). Applications
cannot store data they expect to persist across reboots. Because the
Tock kernel does not care about the identity of Applications, it has
no security policies for limiting access to functionality or resources
(e.g., system call filters).

In this use case, the Credentials Checking Policy accepts all
correctly formatted TBF Objects and the Identifier Policy assigns
every process a Locally Unique Identifier and a Locally Unique Short
ID.

8.2.2 Use Case 2: U2F Application
-------------------------------

In this use case, Tock needs to run a Universal 2nd Factor
Authentication (U2F) application. This Application needs to store a
private key in flash. No other Application should be able to access
this key. The Tock kernel also restricts certain system calls to only
the U2F Application, such as invoking cryptographic accelerators.
Finally, the U2F Application needs a consistent identity over reboots
of its Userspace Binary, the kernel, and upgrades of the Application
with new versions (and Userspace Binaries).

In this use case, the Application Identifier is a Global Identifier.
To establish the authenticity and integrity of the U2F Application,
the Credential Checking Policy requires that an Application has a
valid Rsa4096Key credential. The system assumes that each Application
has its own public-private key pair. While the system will load and
run any process whose Userspace Binary has a valid Rsa4096Key
credential, it only gives special permissions and access to the U2F
Application.

The Identifier Policy defines the Application Identifier of a process
to depend on the public key of its Rsa4096Key credential. If it is the
key known to belong to the U2F Application, the Application Identifier
is the key. If the key is not recognized, the Application Identifier
is a Locally Unique Identifier. The Short ID of the U2F Application is
1 and the Short ID of all other Applications is Locally Unique.

8.2.3 Use Case 3: Application Isolation
-------------------------------

In this use case, Tock needs to support multiple Applications that can
read and write local flash. Each Application has its own flash
storage, and Tock isolates their flash storage from one another. An
Application cannot access the flash of another Application. However,
this is a development system or a system which does not require
confidentiality. While there is storage isolation between
Applications, this is for debuggability, easy of composition, and
simplicity and not to meet security requirements. The Credentials
Checking Policy is permissive and tries to run every properly
formatted TBF Objects.

In this use case, the Application Identifier is a Global Identifier.
It is the string name of the TBF Object as encoded in a TBF Header.
The Short ID is a one's complement checksum of the string name.

If a developer installs two TBF Objects with the same string name, the
Tock kernel thinks they are the same Application and only runs one of
them. If a developer accidentally uses two different string names that
have the same checksum (e.g. both "dog" and "mal" checksum to 0x13a),
the Tock kernel also only runs one of them. Some local modifications
to `tockloader` check for these collisions and prevent the developer
from accidentally installing colliding Applications.

Note that in this case it is possible that the "mal" application could
read data stored by the "dog" application.


8.3 Short ID Format
-------------------------------

The 32-bit value MUST be non-zero. `ShortID` uses `core::num::NonZeroU32`
so that an `ShortID` can be 32 bits in size, with 0 reserved
for `LocallyUnique`.

```rust
#[derive(Clone, Copy)]
enum ShortID {
    LocallyUnique,
    Fixed(core::num::NonZeroU32),
}

impl PartialEq for ShortID {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
           (ShortID::Fixed(a), ShortID::Fixed(b)) => a == b,
           _ => false
        }
    }
}
impl Eq for ShortID {}

pub trait Compress {
    fn to_short_id(process: &dyn Process) -> ShortID;
}
```

Generally, the Process Checker that implements `AppCredentialsChecker`
and `AppUniqueness` also implements `Compress`. This allows it to
share copies of public keys or other credentials that it uses to make
decisions, reducing flash space dedicated to these constants. Doing so
also makes it less likely that the two are inconsistent.

8.4 Short ID Considerations
-------------------------------

It is RECOMMENDED that the `id` field of `ShortID` be completely
hidden and unknown to modules that use `ShortID` to manage security
policies. They should depend on obtaining `ShortID` values based on
known names or methods. For example, the implementation of an
Identifier Policy can define a method, `privileged_id`, which returns
the Short ID associated with special privileges. Kernel modules which
want to give these processes extra permissions can check whether the
`ShortID` associated with a process matches the `ShortID` returned
from `privileged_id`. Alternatively, when they are initialized, they
can be passed a slice or array of `ShortID`s which are allowed; system
initialization generates this set once and passes it into the module
so it does not need to maintain a reference to the structure
implementing `AppCredentialsChecker` and `Compress`.


The exact `id` values used is an internal implementation decision for
the implementer of `Compress` and the Identifier Poloicy. Doing so
cleanly decouples modules through APIs and does not leak internal
state.

9 The `CredentialsCheckingPolicy` Trait
===============================

The `CredentialsCheckingPolicy` trait is a composite trait that
combines `AppCredentialsChecker`, `AppUniqueness`, and `Compress`
into a single trait so it can be passed as a single reference.

```rust
pub trait CredentialsCheckingPolicy<'a>: AppCredentialsChecker<'a> + AppUniqueness + Compress {}
impl<'a, T: AppCredentialsChecker<'a> + AppUniqueness + Compress> CredentialsCheckingPolicy<'a> for T {}
```

10 Capsules
===============================

Include a sample capsule that uses ShortIDs.

11 Implementation Considerations
===============================

Notes about requirements for application identifier generation/calculation (must be synchronous).


12 Authors' Addresses
===============================
```
Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu

Johnathan Van Why <jrvanwhy@google.com>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
[TBF]: ../TockBinaryFormat.md "Tock Binary Format"
