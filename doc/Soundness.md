Soundness and Unsafe Issues
---------------------------

An operating system necessarily must use unsafe code. This document explains
the rationale behind some of the key mechanisms in Tock that do use unsafe code
but should still preserve safety in the overall OS.

<!-- npm i -g markdown-toc; markdown-toc -i Soundness.md -->

<!-- toc -->

- [`static_init!`](#static_init)
  * [Use](#use)
  * [Soundness](#soundness)
  * [Alternatives](#alternatives)
- [Capabilities: Restricting Access to Certain Functions and Operations](#capabilities-restricting-access-to-certain-functions-and-operations)
  * [Capability Examples](#capability-examples)

<!-- tocstop -->

## `static_init!`

The "type" of `static_init!` is basically:

```rust
T => (fn() -> T) -> &'static mut T
```

Meaning that given a function that returns something of type `T`, `static_init!`
returns a mutable reference to `T` with static lifetime.

This is effectively meant to be equivalent to declaring a mutable static
variable:

```rust
static mut MY_VAR: SomeT = SomeT::const_constructor();
```

Then creating a reference to it:

```rust
let my_ref: &'static mut = &mut MY_VAR;
```

However, the rvalue in static declarations must be `const` (because Rust doesn't
have pre-initialization sections). So `static_init!` basically allows static
variables that have non-const initializers.

Note that in both of these cases, the caller must wrap the calls in `unsafe`
since references a mutable variable is unsafe (due to aliasing rules).

### Use

`static_init!` is used in Tock to initialize capsules, which will eventually
reference each other. In all cases, these references are immutable. It is
important for these to be statically allocated for two reasons. First, it helps
surface memory pressure issues at link time (if they are allocated on the stack,
they won't trivially show up as out-of-memory link errors if the stack isn't
sized properly). Second, the lifetimes of mutually-dependent capsules needs to
be equal, and `'static` is a convenient way of achieving this.

However, in a few cases, it is useful to start with a mutable reference in order
to enforce _who_ can make certain calls. For example, setting up buffers in the
SPI driver is, for practical reasons, deferred until after construction but we
would like to enforce that it can only be called by the platform initialization
function (before `main` starts). This is enforced because all references after
the platform is setup are immutable, and the `config_buffers` method takes an
`&mut self` (_Note: it looks like this is not strictly necessary, so maybe not a
big deal if we can't do this_).

### Soundness

The thing that would make the use of `static_init!` unsafe is if it was used to
create aliases to mutable references. The fact that it returns an `&'static mut`
is a red flag, so it bears explanation why I think this is OK.

Just as with any `&mut`, as soon as it is reborrowed it can no longer be used.
What we do in Tock, specifically, is use it mutably in some cases immediately
after calling `static_init!`, then reborrow it immutably to pass into capsules.
If a particular capsule happened to treat accept an `&mut`, the compiler would
try to move the reference and it would either fail that call (if it's already
reborrowed immutably elsewhere) or disallow further reborrows. Note that this is
fine if it is indeed not used as a shared reference (although I don't think we
have examples of that use).

It is important, though, that the same code calling `static_init!` is not
executed twice. This creates two major issues. First, it could technically
result in multiple mutable references. Second, it would run the constructor
twice, which may create other soundness or functional issues with existing
references to the same memory. I believe this is not different that code that
takes a mutable reference to a static variable. Again, importantly both cases
must be wrapped with `unsafe`.

### Alternatives

It seems technically possible to return an immutable static reference from
`static_init!` instead. It would require a bit of code changes, and wouldn't
allow us to restrict certain capsule methods to initialization, but may not be a
particularly big deal.

Also, something something static variables of type `Option` everywhere (ugh...
but maybe reasonable).


## Capabilities: Restricting Access to Certain Functions and Operations

Certain operations and functions, particularly those in the kernel crate, are
not "unsafe" from a language perspective, but are unsafe from an isolation and
system operation perspective. For example, restarting a process, conceptually,
does not violate type or memory safety (even though the specific implementation
in Tock does), but it would violate overall system safety if any code in the
kernel could restart any arbitrary process. Therefore, Tock must be careful with
how it provides a function like `restart_process()`, and, in particular, must
not allow capsules, which are untrusted code that must be sandboxed by Rust, to
have access to the `restart_process()` function.

Luckily, Rust provides a primitive for doing this restriction: use of the
`unsafe` keyword. Any function marked as `unsafe` can only be called from a
different `unsafe` function or from an `unsafe` block. Therefore, by removing
the ability to define an `unsafe` block, using the `#![forbid(unsafe_code)]`
attribute in a crate, all modules in that crate cannot call any functions marked
with `unsafe`. In the case of Tock, the capsules crate is marked with this
attribute, and therefore all capsules cannot use `unsafe` functions. While this
approach is effective, it is very coarse-grained: it provides either access to
all `unsafe` functions or none. To provide more nuanced control, Tock includes
a mechanism called Capabilities.

Capabilities are essentially zero-memory objects that are required to call
certain functions. Abstractly, restricted functions, like `restart_process()`,
would require that the caller has a certain capability:

    restart_process(process_id: usize, capability: ProcessRestartCapability) {}

Any attempt to call that function without possessing that capability would
result in code that does not compile. To prevent unauthorized uses of
capabilities, capabilities can only be created by trusted code. In Tock, this is
implemented by defining capabilities as unsafe traits, which can only be
implemented for an object by code capable of calling `unsafe`. Therefore, code
in the untrusted capsules crate cannot generate a capability on its own, and
instead must be passed the capability by module in a different crate.

Capabilities can be defined for very broad purposes or very narrowly, and code
can "request" multiple capabilities. Multiple capabilities in Tock can be passed
by implementing multiple capability traits for a single object.

### Capability Examples

1. One example of how capabilities are useful in Tock is with loading processes.
   Loading processes is left as a responsibility of the board, since a board may
   choose to handle its processes in a certain way, or not support userland
   processes at all. However, the kernel crate provides a helpful function
   called `load_processes()` that provides the Tock standard method for finding
   and loading processes. This function is defined in the kernel crate so that
   all Tock boards can share it, which necessitates that the function be made
   public. This has the effect that _all_ modules with access to the kernel
   crate can call `load_processes()`, even though calling it twice would lead to
   unwanted behavior. One approach is to mark the function as `unsafe`, so only
   trusted code can call it. This is effective, but not explicit, and conflates
   language-level safety with system operation-level safety. By instead
   requiring that the caller of `load_processes()` has a certain capability, the
   expectations of the caller are more explicit, and the unsafe function does
   not have to be repurposed.

2. A similar example is a function like `restart_all_processes()` which causes
   all processes on the board to enter a fault state and restart from their
   original `_start` point with all grants removed. Again, this is a function
   that could violate the system-level goals, but could be very useful in
   certain situations or for debugging grant cleanup when apps fail. Unlike
   `load_processes()`, however, it might make sense for a capsule to be able to
   call `restart_all_processes()`, in response to a certain event or to act as a
   watchdog. In that case, restricting access by marking it as `unsafe` will not
   work: capsules cannot call unsafe code. By using capabilities, only a caller
   with the correct capability can call `restart_all_processes()`, and
   individual boards can be very explicit about which capsules they grant which
   capabilities.
