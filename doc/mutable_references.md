# Mutable References in Tock

Ownership and Borrowing are two of the most fundamental design choices in Rust as it,by nature, prevents race
conditions and makes prevents the dangaling pointer problem. To read more about these concepts, I'd recommend 
[this](http://words.steveklabnik.com/a-30-minute-introduction-to-rust).

In Rust, you can either have one mutable reference to a place in memory or an "infinite" amount of read-only references.
It's also used to simply reference large segments of memory (that you wouldn't want to move or copy often). 
Given that read and write are mutally exclusive for borrows, it's simple impossible to run into a race condition using safe rust.

But, what does this mean for Tock? It's a single-threaded enviroment right now, so clearly we won't have race-conditions
anyway. The problem with the Tock and the borrowing system arises when we consider memory use in Tock, and event driven code
without a heap...

In Tock, both the Capsules and the Kernal don't have a heap because we don't want to allow dynamic memory allocation. If we did,
we could run into the problem of the Kernal/Capsules leaking memory, exhausting memory, and crashing. For this reason, everything
is statically allocated for the two.

But what if a Capsule needs more memory because it's handling more clients? A janky solution would be always reserving 
space for all of the potential clients, but that's a huge waste of space that we don't have on microcontrollers. 

## The TakeCell abstraction

We want to avoid making everything mutable in Tock, because if we did make everything mutable, how could we pass out references? We'd only be able to have one mutable reference. We can solve this issue by having variables declared immutable, but we might still want to modify those "immutable" variable. Thus the question becomes, how can we subvert the type system of rust, by having both multiple read-only borrows, while also making it mutable? We do this with the TakeCell.

Once a TakeCell has been taken to perform a particular function (or, when TakeCell.take has been called), the TakeCell no longer owns a location in memory as ownership has been transferred. Using the map function and a closure, an operation may be performed on a TakeCell that takes and returns a block of memory. 

TakeCells are also critically important for accessing larger pieces of data as multiple functions can be performed with a single TakeCell. For example, when writing a buffer to UART, a TakeCell would allow a process to temporarily transfer ownership of a buffer while writing a byte before retaking ownership. It is more space efficient, since all processes can share the same buffer, instead of all having individual ones. 

Transferring ownership with TakeCell is safer than just giving out a reference, since it makes use of Rust’s memory safety guarantees. For example, if it gave out references to the same object, then two processes could clobber each other’s data. Since with TakeCell only one thing can have access at a time, this is impossible. 




However, a down side to this could be what if two processes, A and B, want to use the UART to print to serial, but we only allocated
room for one buffer in the UART Driver ( as of now drivers < capsules )? 
One solution for this could be giving whoever comes first exclusive access to the UART, and blocking the second process. 
A drawback to this would be starving out the other process entirely, making it not function properly. We solve this issue of needing
more space to save state for drivers with the `allow()` system call. This way when the process needs more


3) Mutable references in Tock. Since borrows are so critical in Rust, and they raise issues in 
event-driven code without a heap, we need to explain how Tock structures its code (memory containers).
