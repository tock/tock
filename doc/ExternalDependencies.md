External Dependencies
=====================

<!-- npm i -g markdown-toc; markdown-toc -i ExternalDependencies.md -->

<!-- toc -->

- [External Dependency Design](#external-dependency-design)
  * [Rationale](#rationale)
  * [Dependency Structure of Tock-Internal Crates](#dependency-structure-of-tock-internal-crates)
- [External Dependency Selection](#external-dependency-selection)
  * [General Guidelines for Dependency Selection](#general-guidelines-for-dependency-selection)
    + [Provide Important Functionality](#provide-important-functionality)
    + [Project Understandability](#project-understandability)
    + [Limited Sub-dependencies](#limited-sub-dependencies)
  * [Board-Specific External Dependencies](#board-specific-external-dependencies)
  * [Capsule Crate-Specific External Dependencies](#capsule-crate-specific-external-dependencies)
  * [Core Kernel External Dependencies](#core-kernel-external-dependencies)
    + [Approved Exceptions for Core Kernel External Dependencies](#approved-exceptions-for-core-kernel-external-dependencies)
      - [**tock-registers**](#tock-registers)
      - [**flux-rs**](#flux-rs)
- [Including the Dependency](#including-the-dependency)
  * [Including Capsule Crate-Specific External Dependencies](#including-capsule-crate-specific-external-dependencies)
  * [Including Board-Specific External Dependencies](#including-board-specific-external-dependencies)
  * [Documenting the Dependency and its Tree](#documenting-the-dependency-and-its-tree)
- [Opening a Pull Request with an External Dependency](#opening-a-pull-request-with-an-external-dependency)
- [Design Goals and Alternative Approaches](#design-goals-and-alternative-approaches)

<!-- tocstop -->

Tock's general policy is the kernel does not include external dependencies (i.e.
rust crates outside of the `tock/tock` repository) that are not part of the Rust
standard library. However, on a limited, case-by-case basis with appropriate
safeguards, external dependencies can be used in the Tock kernel. The rationale
and policy for this is described in this document. This document only applies to
the Tock kernel binary itself, not userspace or other tools or binaries within
the Tock project.


## External Dependency Design

This document describes both Tock's external dependency policy and mechanism, as
well as the rationale behind the approach.


### Rationale

Tock limits its use of external libraries for all crates in the kernel. This is
done to promote safety, as auditing the Tock code only requires inspecting the
code in the Tock repository. Tock tries to be very specific with its use of
`unsafe`, and tries to ensure that when it is used it is clear as to why. With
external dependencies, verifying uses of `unsafe` are valid is more challenging
to, particularly as external libraries evolve.

External dependencies also typically themselves rely on dependencies, so
including one external crate likely pulls in several external crates. As of May
2023, cargo does not provide a robust way to audit and prohibit `unsafe` within
a dependency hierarchy. Also, the dependency chain for an external crate is
largely hidden from developers using the external crate. Lacking automated
tools, managing dependencies is a manual process, and to limit overhead Tock
generally avoids external dependencies.



### Dependency Structure of Tock-Internal Crates

Following from the above, an external dependency added to a crate which is
depended on internally within Tock (e.g. the `kernel` crate) will have a higher
impact than a dependency added to a crate with no reverse dependencies (e.g. a
board crate). Thus, this policy is increasingly liberal with crate-types that
have fewer reverse dependencies.

This document considers Tock's crate structure by referring to the following
types of crates internal to Tock:

- the kernel crate: `kernel/`
- arch crates: crates in the `arch/` directory
- chip crates: crates in the `chips/` directory
- board crates: crates in the `boards/` directory
- capsule crates: crates in the `capsules/` directory

Furthermore, this policy assumes the following rules regarding crate
dependencies internal to Tock:

- a _board crate_ is not a dependency of any other Tock-internal crate
- a _chip crate_ is only a dependency of _board crates_ or other _chip crates_
- a _capsule crate_ is only a dependency of other _capsule crates_ or _board
  crates_
- an _arch crate_ may only depend on the _kernel crate_ and other _arch crates_
- the _kernel crate_ does not depend on _arch_, _chip_, _board_, or _capsule
  crates_

## External Dependency Selection

External dependencies can be added to Tock on a case-by-case basis. Each
dependency will be reviewed for inclusion, according to the criteria in this
section. The requirements are intentionally strict.

There are two general methods to for including an external dependency in the
Tock kernel: capsule-specific or board-specific external dependencies.

### General Guidelines for Dependency Selection

In general, the following guidelines can provide an indication whether an
external dependency is suitable for inclusion in Tock.

#### Provide Important Functionality

The external crate provides important functionality that could not easily or
realistically be provided by the Tock developers.

Such functionality includes:

* Cryptography libraries. Writing cryptographically secure code that is both
  correct and resistant to attacks is challenging. Leveraging validated,
  high-quality cryptographic libraries instead of Tock-specific cryptographic
  code increases the security of the Tock kernel.

#### Project Understandability

The external crate should be focused to a particular, understandable operation
or feature set. The code should be high quality and straightforward to
encourage confident auditing. The crate should only use standard and commonly
used Rust and Rust ecosystem mechanisms.

#### Limited Sub-dependencies

The external crate should have a limited sub-dependency tree. The fewer
dependencies the crate introduces the more likely it is to be accepted. There is
no set threshold, instead this is evaluated on a case-by-case basis.

### Board-Specific External Dependencies

As board crates are generally regarded as use-case specific, managed by specific
chip and board maintainers, and audited by those maintainers, Tock is more
flexible with including external dependencies in those crates.

Examples of when a board may want to use an external library:

* Wireless protocols.
  * Wireless implementations are difficult to get the correct timing.
  * Wireless protocols are also very expensive to certify.

Note, however, that _only_ the board crate itself may include such an external
dependency in its `Cargo.toml` file.

A possible way to have other crates indirectly use such a dependency is through
a wrapper-trait. Such traits abstract the external dependency in a way that
allows other crates to still be built without the dependency included. While
using a wrapper-trait is not required, in certain scenarios wrapper-traits may
be useful or desirable.

### Capsule Crate-Specific External Dependencies

Capsules are a mechanism to provide semi-trusted infrastructure to a Tock board,
for instance non chip-specific peripheral drivers (see
[Design](https://book.tockos.org/doc/design)). As such, external dependencies
may be useful to implement complex subsystems. Examples for this are wireless or
networking protocols such as Bluetooth Low Energy or TCP.

To support such use-cases without forcing all boards to include external
dependencies, capsules are split into multiple crates:

- The `capsules/core` crate contains drivers and abstractions deemed essential
  to most boards' operation, in addition to commonly used infrastructure and
  _virtualizers_. It must not have any external dependencies.

- The `capsules/extra` crate contains miscellaneous drivers and abstractions
  which do not fit into other capsule crates. It must not have any external
  dependencies.

Capsule crates other than `core` and `extra` _may_ include external
dependencies. The granularity of such crates may range from implementing an
entire subsystem (e.g. a TCP/IP stack) to a single module providing some
isolated functionality.  Whether an external dependency may be added to a given
crate, and the granularity of said crate, is evaluated on a case-by-case
basis. Concerns to take into account could be the utility, complexity and
quality of the external dependency, and whether the capsule would provide value
without this dependency.

Newly contributed code or code from `capsules/extra` can be moved to a new
capsule crate when deemed necessary; this is evaluated on a case-by-case basis.

### Core Kernel External Dependencies

Tock does not permit any external dependencies for core kernel crates, with one
exception. As these Tock crates are not optional, Tock users cannot opt-out of
the external dependencies as they can with board-level and capsule-level
external dependencies. In general, external dependencies for `arch/`, `chips/`,
and `kernel/` crates must be vendored into the Tock repository.

The lone exception is for significant crates enumerated here that are maintained
by the Tock project itself. These can only be crates that have significant
value to the greater Rust and/or embedded development community, whose
development in a standalone repository is necessary for the continued
development, improvement, and impact of the crate.

All crates permitted by this exception MUST NOT have any external dependencies
themselves.

Any crates included in this exception list, with their associated justification,
do not imply any predisposition to allowing an exception for any future
crates. All exceptions will be considered independently.

#### Approved Exceptions for Core Kernel External Dependencies

##### **tock-registers**
This crate provides an interface for using MMIO registers which are extensively
used in `chips/` crates.

- **Repository:** https://github.com/tock/tock-registers
- **Justification:** This crate has moved to an external dependency to
  encourage its development and use in projects beyond Tock. Specifically, the
  benefits of development in a separate repository include:

  1. Encouraging more rigorous backwards compatibility considerations for
     external users. The close interplay of Tock and tock-registers means
     breaking changes for external users can be hidden in Tock-specific pull
     requests. Having a separate development area limits the risk of inadvertent
     breaking changes.
  2. Encouraging regular releases. The close interplay of Tock and
     tock-registers minimizes the benefit of formal tock-register releases.
     However, the broader community would benefit from more frequent releases,
     and a separate development space with separate release tracking (i.e., in
     Github) helps facilitate that.
  3. Easing engagement with external users. Having a dedicated development site
     makes it more obvious where to post issues and propose changes.
  4. Encouraging external contributions. Having tock-registers in its own
     repository makes it clear it is a standalone project and can be developed
     independently of Tock. This should help contributions that benefit
     tock-registers but not necessarily Tock as well.

##### **flux-rs**
This crate provides support for formal verification of Rust code via
_refinement types_. This is an __optional dependency__ and is __used only
during testing__.  `flux` does not affect built artifacts.

- **Repository:** https://github.com/flux-rs/flux
- **Justification:** Tock is interested in maximizing safety and correctness,
  and formal verification is a key facet for going beyond the capabilities
  afforded by out of the box Rust. Most formal verification tools are also
  highly experimental, however. To allow for experimentation with emerging
  formal verification tools, Tock allows for inclusion of verification
  infrastructure in the primary repository, so long as such infrastructure is
  fully optional and is not required to produce a kernel image.
- **Implementation Nuances:** Conceptually, Tock would like to use `flux` as a
  [development dependency][dev-dep], i.e., it is a dependency that is only
  used during testing and is not a dependency that is used when building
  actual code artifacts. Currently, `cargo` hard-codes a one-to-one
  relationship between `dev-dependencies` and the `cfg(test)` feature flag.
  As `flux` aims to verify the code in the final built artifact (e.g., it
  verifies arch-specific code such as context switch logic) it cannot run under
  the `cfg(test)` context, and thus it cannot be listed in `dev-dependencies`.
  For this reason, it is listed as an __optional__ dependency for the primary
  workspace.
  - _Impact on `Cargo.lock`:_ While `flux` is an optional dependency,
    [cargo limitations](https://github.com/rust-lang/cargo/issues/10801) are
    such that `flux` and its dependencies will be included in the lockfile
    even if they are unused. `flux` adds the following packages to the
    lockfile: `flux-attrs`, `flux-attrs-impl`, `flux-rs`, `proc-macro2`,
    `quote`, `syn`, and `unicode-ident`.
- **Scope and Usage:** `flux` refinements in Tock aim to parallel established
  conventions around unit testing. I.e.,
   - Refinements are located at the end of the file in a dedicated module
     (after `mod test` if-present).
   - Inclusion of the `flux` module must be conditioned on the `flux` feature
     flag, like this:

         #[cfg(feature = "flux")]
         mod flux_specs {
          ...
         }

   - The `flux` feature flag must not be used anywhere else, nor may any
     `flux`-related concepts be used outside of a `mod flux_specs` block.


## Including the Dependency

To help ensure maintainability and to promote transparency with including
external dependencies, Tock follows a specific policy for their inclusion.

### Including Capsule Crate-Specific External Dependencies

Capsules other than `capsules/core` and `capsules/extra` may include external
dependencies directly in their `Cargo.toml` file and use them directly.

### Including Board-Specific External Dependencies

Board crates may include external dependencies directly in their `Cargo.toml`
file and use them directly.

### Documenting the Dependency and its Tree

Each crate that includes an external dependency in its `Cargo.toml` file must
include a section titled "External Dependencies" in its README. Each external
dependency must be listed along with its dependency tree. This documentation
must be included in the PR that adds the external dependency.

The Tock dependency tree can be generated by running `cargo tree`. The tree
should be updated whenever a dependency change is made.

## Opening a Pull Request with an External Dependency

When opening a pull request to contribute code that includes an external
dependency, please copy the following template into the PR description. For each
item, please provide a brief (i.e., one sentence) description of how the
proposed use case for the external dependency is consistent (or not) with the
guidelines in this document.

```text
#### External Dependency Requirements

1. **Provides Important Functionality**:
2. **Project Maturity**:
3. **Limited Sub-dependencies**:
4. **Included as a board or optional capsule**:
```

## Design Goals and Alternative Approaches

While exploring a policy for including external dependencies, the Tock project
considered many options. This resulted in establishing a list of goals for an
external dependency approach. These goals were converged upon over multiple
discussions of the Tock developers.

Goals:

- Boards which do not need or want the functionality provided by the external
  dependency can ensure the dependency is not included in the kernel build.
- Boards which do not use the dependency do not have to compile the dependency.
- Boards should have discretion on which code to include in their build.
- All uses of the external dependency in the Tock code base are explicit and
  obvious.
- The location within the Tock code tree for external dependencies is clear and
  consistent, and there is a consistent format to document the dependency.
- There is not undue overhead or boilerplate required to add an external
  dependency.

These goals necessitate a few design decisions. For example, as crates are the
smallest unit of compilation in Rust, external dependencies must be included
through new crates added to the Tock source tree so they can be individually
included or excluded in specific builds. Also, crates provide a namespace to use
to identify when external dependencies are being incorporated.

Additionally, we avoid using traits or HIL-like interfaces for dependencies
(i.e. core Tock capsules/modules would use a Tock-defined trait much like
capsules use HILs, and a wrapper would use the external dependency to implement
the trait) to avoid the overhead of implementing and maintaining a wrapper to
implement the trait. While architecturally this has advantages, the overhead was
deemed too burdensome for the expected benefit.

We explicitly document the goals to help motivate the specific design in the
remainder of this document. Also, this policy may change in the future, but
these goals should be considered in any future updates.


[dev-dep]: https://doc.rust-lang.org/rust-by-example/testing/dev_dependencies.html
