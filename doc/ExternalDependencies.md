External Dependencies
=====================

<!-- npm i -g markdown-toc; markdown-toc -i ExternalDependencies.md -->

<!-- toc -->

- [Limited External Dependency Rationale](#limited-external-dependency-rationale)
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



## Limited External Dependency Rationale

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

The only crate that may specify an external dependency in its `Cargo.toml` file
is the `tockextern` crate. This crate solely exists to house external
dependencies, and only re-exports the dependencies within the `tockextern`
namespace.

Other crates in the Tock kernel then include the `tockextern` crate, and use the
dependencies through that namespace.

This removes any ambiguity about when an external dependency is used throughout
the Tock kernel, and makes such usages easier to search for. As Rust allows
external dependencies to be used in a source file with the same syntax as other
modules in the same crate, determining which code is external and which is part
of the Tock kernel is difficult without deeper inspection. By including our own
namespace we force the source file to include `use tockextern::` when using an
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
