# Mutable References in Tock - Memory Containers

- [Brief Overview of Borrowing in Rust](#borrowing_overview)
- [Issues with borrowing in event-driven code without a heap](#issues)
- [The TakeCell abstraction](#takecell)
  - [Description of the struct](#structure_of_takecell)
  - [How it solves the problem](#takecell_solution)
- [Example](#example)

Borrows are critical to Rust's design, however it raises issues in event-driven code without 
a heap (can't dynamically allocate objects). We subvert Rust's systems by using memory 
containers such as the TakeCell abstraction.

## <a href="#borrowing_overview"></a> Brief Overview of Borrowing in Rust 
Ownership and Borrowing are two of the most fundamental design choices in Rust as it,by nature, 
prevents race conditions and makes it impossible to write code that'll produce the dangaling pointer
problem. 

Borrowing is Rust's mechanism to allow references to a part of memory. So similar to C++, and other
languages, it makes it possible to pass large structures simply by using a reference to that structure.
However, Rust's compiler limits your borrows so that we it doesn't run into the reader-writer problem,
meaning you can either have one mutable reference to part of memory or an "infinite" amount of 
read-only references. Given that read and write are mutally exclusive for borrows, it's impossible
to run into a race condition using safe rust.

But, what does this mean for Tock? It's a single-threaded enviroment right now, so clearly we won't 
have race-conditions anyway. A problem arises when the borrowing system of Rust clashes with
event-driven code without a Heap. 

## <a href="#issues"></a> Issues with Borrowing in Event-Driven code without a Heap 
Embedded Devices are often low-resource device -- they don't have resources, such as space to waste.
Thus it is extremely common to share buffers between drivers, and applications using those drivers
normally with a pointer to the memory. But, with pointers how can we 'revoke' access when the driver
is done sharing its buffer? How can we prevent one process from mangaling the data of another?

We solve this issue of uniquely sharing memory with the memory container abstraction, TakeCell which
will be explained below.

Besides the difficulty of sharing memory safey, it's difficult to have an overall dynamic memory 
manager for these devices, the differences between the needs of the memory manager would make it 
such that no one is happy. Furthermore, what would happen if we try to dynamically allocate memory
in the Kernal or Drivers and memory exhausted... crashing seems like a bad idea.

For this reason, in Tock both the Capsules and the Kernal don't have a heap. If we did, we could run into 
the problem of the Kernal/Capsules leaking memory, exhausting memory, and crashing. Thus, everything is 
statically allocated for the two.

But, we might run into the issue of a Capsule Driver (code written in Rust, with the Driver trait implemented)
needing more space, perhaps, because it is handling more clients. A janky solution to this would be always 
reserving space for all of the potential clients, but that's a huge waste of space that we don't have. We
solve this conundrum with the `allow()` system call which you can read about [INSERT_LINK](#).

## <a href="#takecell"></a> The TakeCell abstraction
As described above, we run into several problems on microcontrollers regarding sharing buffers.
Another problem we encounter with the Rust's type system is borrowing in regards to mutability. 
We might need multiple references because of interactions with multiple clients. Thus, we want to avoid making
everything mutable in Tock, because if we did make everything mutable, how could we pass out references? 
We'd only be able to have one mutable reference. We can solve this issue by having variables declared immutable,
but we might still want to modify those "immutable" variable. Thus the question becomes, how can we subvert the
type system of rust, by having both multiple read-only borrows, while also making it mutable?
We do this with the TakeCell, a critical component to shared buffers.

### <a href="#structure_of_takecell"></a> TakeCell structure
From tock/kernal/src/common/take_cell.rs:
> A `TakeCell` is a potential reference to mutable memory. Borrow rules are
> enforced by forcing clients to either move the memory out of the cell or
> operate on a borrow within a closure.


### <a href="#takecell_solution"></a> TakeCell solution
Essentially, you can either use `TakeCell.map()` which would take wrap some clousure given between a
`TakeCell.take()` and `TakeCell.replace()`. When `TakeCell.take()` is called, ownership of a location in memory 
moves out of the cell. It can then be freely used by whoever took it (as they own it) and then put back with
`TakeCell.put()` or `TakeCell.replace()`.

Thus we can share a driver's buffer, among multiple clients one at a time ensuring no one can mutate another
clients information as they're no pointers involved. 

Transferring ownership with TakeCell is safer than just giving out a reference, since it makes use 
of Rust’s memory safety guarantees. For example, if it gave out references to the same object, then 
two processes could clobber each other’s data. With TakeCell there's only one borrower,so, this is impossible. 

## <a href="#example"></a>Example

TakeCells are also critically important for accessing larger pieces of data as multiple functions can be 
performed with a single TakeCell. For example, when writing a buffer to UART, a TakeCell would allow a 
process to temporarily transfer ownership of a buffer while writing a byte before retaking ownership. 
It is more space efficient, since all processes can share the same buffer, instead of all having individual ones. 
