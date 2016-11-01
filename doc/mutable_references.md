# Mutable References in Tock - Memory Containers

- [Brief Overview of Borrowing in Rust](#borrowing_overview)
- [Issues with borrowing in event-driven code without a heap](#issues)
- [The TakeCell abstraction](#takecell)
  - [Description of the struct](#structure_of_takecell)
  - [How it solves the problem](#takecell_solution)
- [Example](#example)

Borrows are a critical part of the Rust language that help provide its
safety guarantees. However, they can complicate event-driven code without  
a heap (no dynamic allocation objects). Tock uses memory containers
such as the TakeCell abstraction to allow simple code to keep
the safety properties Rust provides.

## <a href="#borrowing_overview"></a> Brief Overview of Borrowing in Rust 
Ownership and Borrowing are two design features in Rust which 
prevent race conditions and make it impossible to write code that produces
dangling pointers.

Borrowing is the Rust mechanism to allow references to memory. Similarly references in
C++ and other
languages, borrows makes it possible to efficiently pass large structures by passing pointer
rather than copying the entire structure.
The Rust compiler, however, limits borrows so that they cannot create race conditions,
which are caused
by concurrent writes or concurrent reads and writes to memory. Rust limits code
to either a single mutable (writeable) reference or an "infinite" number of 
read-only references. 

But what does this mean for Tock? As the Tock kernel is single threaded, it doesn't have
race conditions. However, Rust doesn't know this so its rules still hold. In practice,
Rust's rules cause problems in event-driven code when there isn't a heap.

<a href="#issues"></a> Issues with Borrowing in Event-Driven code without a Heap 
Embedded Devices are often low-resource devices -- they don't have RAM to waste.
Thus it is extremely common to share buffers between drivers, and applications using those drivers
normally do so with a pointer to the memory that holds the buffer. However, this raises several potential 
problems. With pointers, how can we "revoke" access when the driver is done sharing its buffer? How can we prevent one 
process from mangling the data of another?

We solve this issue of uniquely sharing memory with the memory container abstraction, TakeCell, which
will be explained below.

Besides the difficulty of sharing memory safely, it's also difficult to have an overall dynamic memory 
manager for these devices, as the differences between the needs of the memory manager would make it 
such that no one is happy. Furthermore, what would happen if we tried to dynamically allocate memory
in the Kernel or Drivers and memory exhausted... crashing seems like a bad idea.

For this reason, in Tock both the Capsules and the Kernel don't have a heap. If we did, we could run into 
the problem of the Kernel or Capsules leaking memory, exhausting memory, and crashing. Thus, everything is 
statically allocated for both the Kernel and Capsules.

However, we might run into the issue of a Capsule Driver (code written in Rust, with the Driver trait implemented)
needing more space, perhaps, because it is handling more clients. A janky solution to this would be always 
reserving space for all of the potential clients, but that would waste space that we don't have. We
solve this conundrum with the `allow()` system call which you can read about [INSERT_LINK](#).

## <a href="#takecell"></a> The TakeCell abstraction
As described above, we run into several problems on microcontrollers regarding sharing buffers.
Another problem we encounter with the Rust's type system is borrowing in regards to mutability: 
with multiple clients, each of which needs access to the features of a capsule, we might need multiple references. 
Thus, we must avoid making everything mutable in Tock, because if we did make everything mutable, we 
would be unable pass out references (recall that Rust allows at most a single mutable reference).
We could solve this issue by having variables declared immutable, but we might still want to modify 
those "immutable" variables. Thus the question becomes one of how to subvert the
type system of Rust so that we can have both multiple read-only borrows as well as making it mutable.
We do this with the TakeCell, a critical component to shared buffers.

### <a href="#structure_of_takecell"></a> TakeCell structure
From tock/kernel/src/common/take_cell.rs:
> A `TakeCell` is a potential reference to mutable memory. Borrow rules are
> enforced by forcing clients to either move the memory out of the cell or
> operate on a borrow within a closure.


### <a href="#takecell_solution"></a> TakeCell solution
Although it can also be done directly, Tock typically uses `TakeCell.map()`, which wraps the provided closure 
between a `TakeCell.take()` and `TakeCell.replace()`. When `TakeCell.take()` is called, ownership of a location 
in memory moves out of the cell. It can then be freely used by whoever took it (as they own it) and then put 
back with `TakeCell.put()` or `TakeCell.replace()`.

Thus we can share a driver's buffer among multiple clients one at a time while still ensuring no client can mutate
another's information (as there are no pointers involved). 

Transferring ownership with TakeCell is safer than just giving out a reference, since it makes use 
of Rust’s memory safety guarantees. For example, if it gave out references to the same object, then 
two processes could clobber each other’s data. With TakeCell there's only one borrower, so this is impossible. 

## <a href="#example"></a>Example

TakeCells are also critically important for accessing larger pieces of data, as multiple functions can be 
performed with a single TakeCell. For example, when writing a buffer to UART, a TakeCell would allow a 
process to temporarily transfer ownership of a buffer so that writing a byte before retaking ownership. 
It is more space efficient, since all processes can share the same buffer, instead of all having individual ones. 
