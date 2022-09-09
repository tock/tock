Application IDs (AppID), Credentials, and Process Loading
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Johnathan Van Why<br/>
**Draft-Created:** 2021/09/01 <br/>
**Draft-Modified:** 2022/08/01 <br/>
**Draft-Version:** 7 <br/>
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
Identifiers, however, are unique across running processes. If multiple
loaded processes share the same Application Identifier, at most one of
them can be runnable at any time. An Application Identifier can be
persistent across boots or restarts of a userspace binary. The Tock
kernel assigns Application Identifiers to processes using a
Credentials Checking Policy.

**Application Credentials**: data that binds an Application Identifier
to an loaded process. Application Credentials are usually stored in
[Tock Binary Format][TBF] footers. A TBF object can have multiple
ApplicationCcredentials.

**Process Checker**: a component of the Tock kernel which is
responsible for validating Application Credentials and assigning
Application Identifiers based on them.

**Identifier Policy**: the algorithm that the Process Checker uses
to assign Application Identifiers to loaded processes.  An Identifier
Policy defines an Application Identifier space. An Identifier Policy
can derive Application Identifiers from Application Credentials, other
process state, or local data.

**Credentials Checking Policy**: the algorithm that the Process
Checker uses to decide how Tock responds to particular Application
Credentials. The boot sequence typically passes the Credentials
Checking Policy to the kernel at startup, which the Process
Checker then uses when the kernel loads processes.

**Global Application Identifier**: an application identifier which,
given an expected combination of Credentials Checking Policy and
Identifier Policy, is both globally consistent across all TBF objects
for a particular Application and unique to that Application. All
instances of the Application loaded with this combination of policies
have this Application Identifier. No instances of other Applications
loaded with this Credentials Checking Policy have this Application
Identifier. An example of a Global Application Identifer is a public
key used to generating Application Credentials for every TBF Object of
a single Application.

**Local Application Identifier**: an Application Identifier which is
locally unique for the Credentials Checking Policy that assigned
it. The same TBF Object, loaded with the same Credentials Checking
Policy on another Tock node, may have a different Application
Identifier. An example of a Local Application Identifier is an
incrementing counter that the Credentials Checking Policy checks for
uniqueness (skipping values already in use if it loops around).

**Short ID**: a 32-bit compressed representation of an Application
Identifier.

In normal use of Tock, a software tool running on a host copies TBF
Objects into an application flash region. When the Tock kernel boots,
it scans this application flash region for TBF Objects. After
inspecting the Userspace Binary, TBF headers, and TBF Footers
in a TBF Object, the kernel assigns it an Application Identifier and 
decides whether to run it.

3 Application Identifiers and Application Credentials
===============================

There is a relationship between Application Identifiers and
Application Credentials, but they are not the same thing. An
Application Identifier is a numerical representation of the
Application's identity. Application Credentials are data that,
combined with an Identifier Policy, can cryptographically bind an
Application Identifier to a process.

Suppose there are two versions (v1.1 and v1.2) of the same
Application. They have different Userspace Binaries. Each version has
an Application Credentials consisting of a cryptographic hash of their
TBF headers and Userspace Binary signed by a known public key. In this
use case (supported by a Identifier Policy), the public key defines
the Application Identifier: all versions of this Application have
Application Credentials signed by this key.  The two versions have
different Application Credentials, because their hashes differ, but
they have the same Application Identifier.

3.1 Application Identifiers
-------------------------------

The kernel MUST NOT simultaneously run two processes that have the
same Application Identifier. This restriction is because an
Application Identifier provides an identity for a Userspace Binary.
Two processes with the same Application Identifier are two copies or
versions of the same Application. Application Identifiers are used to
control access to resources such as storage. This restriction ensures
there is at most one process accessing resources or data belonging to
an Application Identifier, which precludes the need for consistency
mechanisms for concurrent access.

Application Identifiers can be used for security policy decisions in
the rest of the kernel. For example, a kernel may allow only
Applications whose Application Credentials use a particular trusted
public key to access restricted functionality, but restrict other
applications to use a subset of available system calls. By defining
the Application Identifier of a process to be the public key, the
system can map this key to a Short IDs (described below) that gives
access to retricted functionality.

3.2 Application Credentials
-------------------------------

Application Credentials are information stored in TBF Footers. The exact
format and information of Application Credentials are described in the next
section. They typically store cryptographic information that establishes
the Application a Userspace Binary belongs to as well as provide integrity.

Application Identifiers can, but do not have to be, be derived from
Application Credentials. For example, a Tock system with a permissive
Credentials Checking Policy may allow processes with no Application
Credentials to run, and have an Identifier Policy that defines
Application Identifiers to be the ASCII name stored in a TBF header.
Global Application Identifiers MUST persist across process restarts or
reloads.

In cases when a TBF Object does not have any Application Credentials,
the Identifier Policy MAY assign it a Global Application Identifier or
a Local Application Identifier.

Each Tock process has a unique process identifier assigned by the
kernel, which can be re-used over time (like POSIX process
identifiers).  Application Identifiers are distinct from these process
identifiers. An Application Identifier is intended to identify an
Application, while a process identifier identifies a particular
execution of that binary. For example, if a Userspace Binary exits and
runs a second time, there is no binding between the process
identifiers of those two invocations, while the Application Identifier
will be the same.

3.3 Example Use Cases 
-------------------------------

The following five use cases demonstrate different ways in which
Application Policies can assign Application Identifiers, some of which
use Application Credentials:

  1. A TBF Object with no Application Credentials: it only runs on
  kernels that are willing to load TBF Objects without credentials
  (e.g., research systems). The Identifier Policy defines
  that TBF Objects with no credentials have a Global Application
  Identifier of a SHA256 hash of the Application Binary. This
  means the system does not support versioning of Userspace Binaries:
  two different versions of the same Application have different
  Application Identifiers.

  1. The Credentials Checking Policy only accepts TBF Objects with an
  Application Credentials containing an RSA signature. The Identifier
  Policies defines that the Global Application Identifier of a process
  is the public key used to generate the accepted Application
  Credentials for the TBF Object.  Before verifiying a signature in a
  TBF footer, the Process Checker decides whether to it accepts the
  associated public key using the Credentials Checking Policy. The
  Identifier Policy assigns a Global Application Identifier as the
  public key in the TBF footer.

  1. Multiple separate Applications that run concurrently need to be
  signed with a single public key. Each Application is identified by
  an identifier I. The Application Credentials for these Userspace
  Binaries consist of a TBF footer containing an ECDSA public key, the
  identifier I, and a signature over the TBF Headers, Userspace
  Binary, I signed with the private key corresponding to the public
  key in the footer. The Credentials Checking Policy decides whether
  to accept a particular public key for verification. The Identifier
  Policy assigns a Global Application Identifier as the concatenation
  of the public key and I.

  1. A Tock system wants to load the same Userspace Binary in two
  different processes at the same time. The Userspace Binary is stored
  in a single TBF Object O that the Credentials Checking Policy
  assigns a Global Application Identifier to. The system cannot run O
  in two processes at the same time because two processes with the
  same Application Identifier cannot run simultaneously.
  
  1. A Tock system wants to load the same Userspace Binary in two
  different processes at the same time. The Credentials Checking
  Policy assigns a unique Local Application Identifier to each
  Userspace Binary based on its position in flash.

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
the loading stage: Unloaded, Unchecked, Failed, Unstarted and
Running. Processes start in the Unloaded state.

First, when it boots, the Tock kernel scans the TBF Objects stored in
its application flash region. It checks that they are valid and
can run on the system. It loads them in order from lowest to highest
address. Each successfully loaded TBF Object is loaded into one of
the process slots, that process is moved into the Unchecked state.

Once the application flash has been scanned or the last process slot
has been filled, the kernel checks the credentials of the processes.
Using the provided Credentials Checking Policy, it decides whether
each process can be run. Processes whose TBF Objects are allowed to
run are moved into the Unstarted state. Processes whose TBF Objects
are not allowed to run are moved into the Failed state.

The kernel traverses the process one final time, checking whether the
Application Identifier assigned to a process collides with a running
process. If the Application Identifier collides, the kernel places the
process into the Unstarted state. If the Application Identifier does
not collide, the kernel places the process into the Running state, at
which point that Application Identifier is in use by a Running
process.

This final traversal occurs in a specific order. The kernel MUST check
processes in order of decreasing version numbers (the highest version
number is checked first).  If more than one process has the same
version number, they MUST be checked from lowest to highest
address. For example, if two TBF Objects both have version number 0,
one at address 0x20000 and one at address 0x21000, the one with
address 0x20000 will be checked first and the one with address 0x21000
will be checked second. The Version field of a Program Header of a
process's TBF Object specifies the version number of the process. The
kernel MUST give a version number of 0 to processes whose TBF Object
does not have a TBF Program Header.

The Unloaded, Unchecked, Failed, Unstarted and Running states describe
the conceptual state machine of a process at loading; the values or
names of state variables in the kernel implemementation might be
different. The exact Tock process state machine and names of its
states is outside the scope of this document.


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

The `Reserved` type has a variable length. This credentials type is
used to reserve space for future credentials (e.g., that will be added
by another party in the deployment process). Because the `total_size`
field of a TBF Base Header is covered by integrity, once the total
size of the TBF Object has been decided it cannot be changed: if
credentials need to be added later, space must be reserved for them.

The `CleartextID` type has a data length of 8 bytes. It contains a
64-bit number in big-endian format representing an application
identifier.

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

The `Rsa3072KeyWithID` type has a data of length of 768 bytes. It
contains a public 3072-bit RSA key (384 bytes), followed by a 384-byte
signature that contains a 32-bit identifier. **Exact signature
algorithm/approach to be determined.**

The `Rsa4096KeyWithID` type has a data of length of 1024 bytes. It
contains a public 4096-bit RSA key (512 bytes), followed by a 512-byte
signature that contains a 32-bit identifier. **Exact signature
algorithm/approach to be determined.**

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

When the kernel successfully parses and loads a Userspace Binary into
a `Process` structure, it places it into the Unchecked state,
indicating that it has been loaded but that it may not be allowed or
safe to run. If the kernel is provided an `AppCredentialsChecker`, it
uses it to check whether each of the loaded proceses is safe to run.
If no `AppCredentialsChecker` is provided, the kernel skips this check
and transitions the process into the Unstarted state.

To check the integrity of a process, the kernel scans the footers
in order, starting at the beginning of that process's footer
region. At each `TbfFooterV2Credentials` footer it encounters, the
kernel calls `check_credentials` on the provided
`AppCredentialsChecker`. If `check_credentials` returns `Accept`, the
kernel stops processing credentials and calls `mark_credentials_pass`
on the process, which transitions it to the Unstarted state. If the
`Verifier` returns `Reject`, the kernel stops processing credentials
and calls `mark_credentials_fail` on the process, which transitions it
to the Failed state.

If the `AppCredentialsChecker` returns `Pass`, the kernel tries the
next `TbfFooterV2Credentials`, if there is one. If the kernel reaches
the end of the TBF Footers (or if there is a Main Header and so no
Footers) without encountering a `Reject` or `Accept` result, it calls
`require_credentials` to ask the `AppCredentialsChecker` what the
default behavior is.  If `require_credentials` returns `true`, the
kernel calls `mark_credentials_fail` on the process, transitioning it
into the Failed state. If `require_credentials` returns `false`, the
kernel calls `mark_credentials_pass` on the process, transitioning it
to the Unstarted state. If a process binary has no
`TbfFooterV2Credentials` footers then there will be no `Accept` or
`Reject` results and `require_credentials` defines whether the
Userspace Binary is runnable.

The `binary` argument to `check_credentials` is a reference to slice
covering the process binary, from the end of the TBF Header to the
location indicated by the `binary_end_offset` field in the Program
Header. The size of this slice is therefore equal to
`binary_end_offset`.

7 Identifier Policy: the `ApplicationIdentification` trait
==============================

The `ApplicationIdentification` trait defines the API the Process
Checker provides to decide whether two processes have the same
Application Identifier.  An implementer of `ApplicationIdentification`
implements the `different_identifier` method, which performs a
pairwise comparison of two processes. There is also a
`has_unique_identifier` method, which compares a process against all
of the processes in a process array. The trait has a default
implementation of this method, but implementations may override it.

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

8 Short IDs and the `Compress` trait
===============================

While `TbfFooterV2Credentials` typically define the identity and
credentials of an application, they are typically large data
structures that are too large to store in RAM. When parts of the
kernel wish to apply security or access policies based on Application
Identifiers, they need a concise way to represent these
policies. Requiring policies to be encoded in terms of Application
Identifiers is extremely costly: a table, for example, that says that
only Applications signed with a particular 4096-bit RSA key can access
certain system calls requires storing the whole 4096-bit key. If there
are multiple such security policies through the kernel, they must each
store this information.

The `Compress` trait provides a mechanism to map an Application
Identifier to a small (32-bit) integer called a Short ID. Short IDs
can be used throughout the kernel as an identifier for security
policies. 

For example, suppose that a device wants to grant access to
all Userspace Binaries signed by a certain 3072-bit RSA key K and has
no other security policies. The Credentials Checking Policy only
accepts `Rsa3072KeyWithID` credentials with key K. The `Compress`
trait implementation assigns a Short ID as the ID stored in the
credentials. Access control systems within the kernel can define their
policies in terms of these identifiers, such that they can check
access by comparing 32-bit integers rather than 384-byte keys.

The 32-bit value MUST be non-zero. `ShortID` uses `core::num::NonZeroU32`
so that an `Option<ShortID>` can be 32 bits in size, with 0 reserved
for `None`.

```rust
#[derive(Clone, Copy, Eq)]
struct ShortID {
    id: core::num::NonZeroU32
}

pub trait Compress {
    fn to_short_id(process: &dyn Process) -> Option<ShortID>;
}
```

The `to_short_id` method returns an `Option` so that it has a clear
default action if the Process Checker does not wish to assign it any
meaningful identifier that might allow additional access.  A return
value of `None` semantically means that the process's Application
Credentials or other state do not map to any known security group or
set of privileges. A `Some` result means the credentials map to a known
security group or set of privileges.

Generally, the Process Checker that implements `AppCredentialsChecker`
and `AppIdentification` also implements `Compress`. This allows it to
share copies of public keys or other credentials that it uses to make
decisions, reducing flash space dedicated to these constants. Doing so
also makes it less likely that the two are inconsistent.
 
`ShortID` values MUST be locally unique among running processes.  The
mapping between Global Application Identifiers and `ShortID` values
MUST be deterministic.  Kernels SHOULD implement `Compress` in a
manner that minimizes the chance that two different Application
Identifiers compress to the same Short ID (e.g., taking the low-order
bits of a strong cryptographic hash function, or using a known,
deterministic mapping).

Short IDs are locally unique for three reasons. First, it simplifies
process management and naming: a particular application identifier
uniquely identifies a running process. Second, it ensures that
resources bound to an application identifier (such as non-volatile
storage) do not have to handle concurrent accesses from multiple
processes. Finally, generally one does not want two copies of the same
Application running: they can create conflicting responses and
behaviors.

For example, suppose there is a system that wants to grant extra
permissions to a particular Application. TBF Objects for this
Application have a `TbfFooterV2Credentials` of `Rsa4096Key` with a
certain public key, and the Identifier Policy uses this public key as
a Global Application Identifier: only one Userspace Binary signed with
that key can run at any time. In this example, the Process Checker
implementing `AppCredentialsChecker` and `Compress` stores a copy of
this key. It returns `Accept` to calls to `check_credentials` with
valid `TbfFooterV2Credentials` using this key. Calls to `Compress`
return `None` for all credentials except a `Rsa4096Key` with this key,
for which it returns `ShortID {id: 1}`. The Process Checker also has a
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

`ShortID` values MAY persist across boots and restarts of a process
binary. If `ShortID` is derived from a Global Application Identifier,
then it is by definition persistent, since it is a determinstic
mapping from the identifier. `ShortID` values derived from local
application identifiers, however, MAY be transient and not persist.

9 Capsules
===============================

10 Implementation Considerations
===============================

11 Authors' Addresses
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
