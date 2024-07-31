# Tock Meeting Notes 2024-07-26

## Attendees
- Hudson Ayers
- Pat Pannuto
- Brad Campbell
- Leon Schuermann
- Benjamin Prevor
- Tyler Potyondy
- Alyssa Haroldsen
- Andrew Imwalle
- Alexandru Radovici

## Updates

* Ben: We have a new employee of the Tock foundation – Benjamin Prevor, he is
  working on the treadmill testing infrastructure. Before this he was a full
  time Haskell engineer.
* Pat: We have a libtock-c PR pulling in a Lora example that brings in GPL
  code. It is constrained to an example so hopefully contained and does not
  cause issues, but wanted to flag it so people could check it out.
  https://github.com/tock/libtock-c/pull/456

## Outstanding PR triage

* Hudson: https://github.com/tock/tock/pull/4109
* Brad: Lets mark that as waiting on author.
* Hudson: https://github.com/tock/tock/pull/4110
* Pat: I am still waiting on the high-level descriptive document detailing goals, philosophy etc. before diving into it.
* Leon: That was opened by a labmate of ours, it stems from a project on timing isolation. I will be managing this for the most part. Implementation is driving forward regardless because this is a research project, and also outlines how invasive changes would be. In practice we won't move forward with this before a descriptive document is ready.
* Hudson: https://github.com/tock/tock/pull/4075
* Leon: Should we merge it on the call? It is last call
* Everyone: Yes!
* Brad: as an aside, it would be nice to have a Tock community bot. For example I opened this PR, but then could not approve it even though other people contributed many of the commits.
* Alyssa: Could we allow people with write access to the Tock repo directly merge without approval? For community PR cases like this?
* Brad: I think that would be fine, but the issue I am talking about is sort of the opposite – for example I could not put a "changes requested" review on my own PR if Amit adds commits and then approves. Probably not a thing we need to discuss, just a workflow mismatch.

## Dynamic allocation in libtock-rs

[meta: Hudson was notetaker for this meeting so this initial part is copied from Hudson's
email]

* Hudson: I would like to discuss options for dynamic allocation in libtock-rs.
  There are two basic approaches
* Hudson: First is the approach used in the old libtock-rs: Allow apps to
  specify a heap size at compile time, reserve that much space for the heap
  using brk() while initializing the runtime, and then use an allocator that
  assumes a backing store of a fixed size (such as linked\_list\_allocator).
* Hudson: The second approach is similar to that originally suggested by
  Johnathan when the old libtock-rs allocator was first implemented: a heap
  that only takes memory from the kernel as needed, and return memory to the
  kernel when no longer needed. The idea is generally an allocator that works
  on a backing store it was given, but will grow the size of this contiguous
  backing store as needed. One advantage of this approach is that if an app
  wants to allocate a bunch of memory, then return it before calling any
  syscalls that lead to grant allocation, that app can use more memory than
  would otherwise be possible given the size of a grant allocation. This
  approach can also just be useful toward maximizing the memory available to
  the app -- rather than having to specify a constant heap size that is known
  to be smaller than (app memory - allocated grant size), the app can just use
  everything that is not taken by grants, and then allocations will start to
  fail once that point is reached.
* Hudson: The second approach is more challenging to integrate with existing
  open source embedded Rust allocators, because in practice I have not found
  any that provide APIs for returning memory given to the allocator back to the
  kernel, without having to add an OS port to the allocator itself. IMO this
  approach seems more likely to require us to write our own allocator, or to
  modify an existing open source allocator with a port to Tock specifically.
  Interested to hear what others think.
* Brad: It makes sense that most existing embedded allocators do this, what if
  the memory freed is not at the end of the heap?
* Hudson: Yeah, then it would not be returned to the kernel, but you can have
  heuristics that return memory when there does end up being enough freed near
  the end of the heap.
* Brad: If we used an existing allocator, would your design preclude using one
  that does enable returning memory to the heap later?
* Hudson: No I don't think so
* Hudson: I did kinda get the feeling that Johnathan was expecting us to go
  forward with the latter approach. He had some other comments in 2022 when
  dczself was looking at implementing an allocator that seemed to kind of
  assume, oh, we're not gonna want to use an existing allocator we're gonna
  want to write our own. And he mentioned we're gonna want to use two crates
  for unit testability, which makes sense if we are writing our own allocator
  but less if we are using another crate that already has its own unit tests.
* Hudson: One other observation is that libtock-c uses this kind of Max heap
  approach as well where apps specify the maximum amount of heap memory that
  they'll use. Does the libtock-c allocator reserve all of that memory from the
  kernel as soon as the app is initialized or does it only actually take memory
  from the kernel gradually as needed?
* Brad: Gradually as needed
* Hudson: Does the libtock-c allocator have a mechanism to return memory to the
  kernel via a negative sbrk?
* Brad: I was wondering if you would ask that, I am not sure
* Hudson: OK, I think I will start with an approach that gradually allocates
  memory from the kernel as it goes, but does not return memory.
* Brad: Makes sense to me, I don't really see what the downside would be, the
  memory is there.

## OT WG 

* Hudson: lets defer this discussion to next week with Amit on the call
