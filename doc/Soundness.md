Soundness and Unsafe Issues
---------------------------

An operating system necessarily must use unsafe code. This document explains
the rationale behind some of the key mechanisms in Tock that do use unsafe code
but should still preserve safety in the overall OS.


## `static_init!`

The "type" of `static_init!` is basically:

```
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

```
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


