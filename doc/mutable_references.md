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

In Tock, both the Capsules and the Kernal don't have a heap because we don't want to allow dynamic memory allocation. If we did,
we could run into the problem of the Kernal/Capsules leaking memory, exhausting memory, and crashing. For this reason, everything
is statically allocated for the two.

But what if a Capsule needs more memory because it's handling more clients? A janky solution would be always reserving 
space for all of the potential clients, but that's a huge waste of space that we don't have on microcontrollers. 

## <a href="#takecell"></a> The TakeCell abstraction 

We want to avoid making everything mutable in Tock, because if we did make everything mutable, how could we pass out references? We'd only be able to have one mutable reference. We can solve this issue by having variables declared immutable, but we might still want to modify those "immutable" variable. Thus the question becomes, how can we subvert the type system of rust, by having both multiple read-only borrows, while also making it mutable? We do this with the TakeCell.

Once a TakeCell has been taken to perform a particular function (or, when TakeCell.take has been called), the TakeCell no longer owns a location in memory as ownership has been transferred. Using the map function and a closure, an operation may be performed on a TakeCell that takes and returns a block of memory. 

TakeCells are also critically important for accessing larger pieces of data as multiple functions can be performed with a single TakeCell. For example, when writing a buffer to UART, a TakeCell would allow a process to temporarily transfer ownership of a buffer while writing a byte before retaking ownership. It is more space efficient, since all processes can share the same buffer, instead of all having individual ones. 

Transferring ownership with TakeCell is safer than just giving out a reference, since it makes use of Rust’s memory safety guarantees. For example, if it gave out references to the same object, then two processes could clobber each other’s data. Since with TakeCell only one thing can have access at a time, this is impossible. 




However, a down side to this could be what if two processes, A and B, want to use the UART to print to serial, but we only allocated
room for one buffer in the UART Driver ( as of now drivers < capsules )? 
One solution for this could be giving whoever comes first exclusive access to the UART, and blocking the second process. 
A drawback to this would be starving out the other process entirely, making it not function properly. We solve this issue of needing
more space to save state for drivers with the `allow()` system call. This way when the process needs more
