External Dependencies
=====================

<!-- npm i -g markdown-toc; markdown-toc -i ExternalDependencies.md -->

<!-- toc -->

- [External Dependency Design](#external-dependency-design)
  * [Rationale](#rationale)
  * [Specific Approach Motivation](#specific-approach-motivation)
- [External Dependency Selection](#external-dependency-selection)
  * [Core External Dependencies](#core-external-dependencies)
    + [Provide Important Functionality](#provide-important-functionality)
    + [Project Maturity](#project-maturity)
    + [Limited Sub-dependencies](#limited-sub-dependencies)
  * [Board-Specific External Dependencies](#board-specific-external-dependencies)
- [Including the Dependency](#including-the-dependency)
  * [Including Core External Dependencies](#including-core-external-dependencies)
  * [Including Board-Specific External Dependencies](#including-board-specific-external-dependencies)
  * [Documenting the Dependency and its Tree](#documenting-the-dependency-and-its-tree)

<!-- tocstop -->

Tock's general policy is the kernel does not include external dependencies (i.e.
rust crates outside of the `tock/tock` repository) that are not part of the Rust
standard library. However, on a limited, case-by-case basis with appropriate
safeguards, external dependencies can be used in the Tock kernel. The rationale
and policy for this is described in this document.



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
including one external crate likely pulls in several external crate. As of Nov
2022, cargo provides no mechanism for auditing and prohibiting `unsafe` in a
dependency hierarchy. Also, the dependency chain for an external crate is
largely hidden from developers using the external crate. Lacking automated
tools, managing dependencies is a manual process, and to limit overhead Tock
generally avoids external dependencies.

### Specific Approach Motivation

The mechanism for including external dependencies is designed to satisfy the
following goals. These goals were converged upon over multiple discussions of
the Tock developers.

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

## External Dependency Selection

External dependencies can be added to Tock on a case-by-case basis. Each
dependency will be reviewed for inclusion, according to the criteria in this
section. The requirements are intentionally strict.

There are two general methods to for including an external dependency in the
Tock kernel: core external dependencies, and board-specific external
dependencies. Core external dependencies may be used in "core" Tock crates, such
as the `kernel`, `chips`, and `capsules` crates. Board-specific external
dependencies may _only_ be used by crates in the `board/` folder. The processes
for inclusion between these two methods are different.

### Core External Dependencies

There are well-specified requirements for including a core external dependency.

#### Provide Important Functionality

The external crate must provide important functionality that couldn't
easily or realistically be provided by the Tock developers.

The list of currently accepted important functionality:

* Cryptography libraries. Writing cryptographically secure code that is both
  correct and resistant to attacks is challenging. Leveraging validated,
  high-quality cryptographic libraries instead of Tock-specific cryptographic
  code increases the security of the Tock kernel.

#### Project Maturity

The external crate being added must be a mature project, with a high quality
of code. The project must be well regarded in the Rust community.

The top-level external crate must belong to one of the following set of
repository organizations:

* [RustCrypto](https://github.com/RustCrypto)

#### Limited Sub-dependencies

The external crate should have a limited sub-dependency tree. The fewer
dependencies the crate introduces the more likely it is to be accepted. There is
no set threshold, instead this is evaluated on a case-by-case basis.


### Board-Specific External Dependencies

As board crates (i.e. crates in the `boards/` directory) are generally regarded
as use-case specific, managed by specific board maintainers, and audited by the
specific board maintainers, Tock is more flexible with including external
dependencies in board crates.

Examples of when a board may want to use an external library:

* Wireless protocols.
  * Wireless implementations are difficult to get the correct timing.
  * Wireless protocols are also very expensive to certify.

Note, however, that _only_ the crate in `boards/` may include an external
dependency in its `Cargo.toml` file. Other crates in the kernel must not include
the dependency, specifically a chip crate or the capsules crate. Therefore, a
wrapper must be provided to interface Tock kernel code with the external
dependency. This prevents external APIs from leaking directly into the Tock
kernel.


## Including the Dependency

To help ensure maintainability and to promote transparency with including
external dependencies, Tock follows a specific policy for their inclusion.

### Including Core External Dependencies

The only crates that can contain external dependencies must be inside the
`capsules/` directory. For each new dependency a new crate should be added
within the `capsules/` directory. Only that crate can have the external
dependency.

The only crates that can use the newly created external dependency crates are
board crates in the `boards/` directory. This ensures that boards which do not
want to include the external dependency can avoid including the crate with the
external dependency.

### Including Board-Specific External Dependencies

Boards may include external dependencies directly in their board's `Cargo.toml`
file and use them directly.

### Documenting the Dependency and its Tree

Each crate that includes an external dependency in its `Cargo.toml` file must
include a section titled "External Dependencies" in its README. Each external
dependency must be listed along with its dependency tree. This documentation
must be included in the PR that adds the external dependency.

The Tock dependency tree can be generated by running `cargo tree`. The tree
should be updated whenever a dependency change is made.
