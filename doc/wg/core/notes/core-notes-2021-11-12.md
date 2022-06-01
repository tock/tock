# Tock Core Notes 2021-11-12

## Attending

- Hudson Ayers
- Brad Campbell
- Brian Granaghan
- Philip Levis
- Amit Levy
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

- Leon: had a meeting with Jett. Talked about the [ProcessBuffer
  issue](https://github.com/tock/tock/issues/2882) as well as the Tock
  git development workflows and git history. Might have found a
  non-invasive method to solve the issue around force pushes on PR and
  loosing the context on pull request reviews. If it turns out to be
  viable, will send around a proposal.

- Phil: implemented the header approach that Brad suggested. A process
  binary can have both a main and program header, where a program
  header is the same as the main header, but also has an end of
  program field for a footer. Up to the kernel as to which to use. Old
  kernels will skip over the program header, new kernels will use the
  program header.

- Hudson: fixed a bug in the documentation of the console system call
  interface, suggested by Jett.

## Proposal for a flash HIL

- Hudson: Brian has sent around a proposal for a flash HIL.

[... people reading the proposal]

- Brian: What are peoples thoughts?

- Phil: why are reads and writes single-page rather than across
  pages?

- Brian: we don't have a use for multi-page flash operations
  currently, most of our operations are restricted to a single page
  (e.g. in the file system implementation).

- Phil: for writes it makes sense, because of erase cycles. Let's say
  I want to read a particular address. Needs to be translated to a
  page and the offset computed.

- Brian: in our context, reads are done on a by-page basis. Extending
  reads to be multi-page would be trivial, when the flash is assumed
  to be memory-mapped.

- Phil: how to I know which page an address is at?

- Brian: in our file system, we have file headers which we cache,
  where we map objects to pages directly. Thus reading files accesses
  specific pages.

- Leon: it appears to make sense to have a flash HIL operating on
  pages, but to be useful in other context we're going to want to have
  a method which calculates from address and length to page and
  offset.

- Brian: this can be implemented either on this HIL, or on the page
  struct. The idea of the page struct is to be able to take this
  abstraction and turn it into an address. We could add the inverse
  function.

- Hudson: in upstream, there's `NonvolatileToPages`, which translates
  address-based reads and writes to page-level reads and writes.

- Phil: believe this is an asynchronous implementation? For writes
  this makes sense because of erase-cycles, whereas for reads it's
  just reading flash in a loop. It does not make sense to translate
  this to individual page reads.

- Brian: biggest concern with expanding reads over a page: we need to
  impose some limit on the read size to prevent operations from taking
  too long. How would we restrict that?

- Phil: reading flash memory off of the SoC is going to be at cycle
  speed, right?

- Pat: this is the question I've written down. The read HIL should not
  cause too much kernel latency -- what is the bound of latency we
  want to put on there? For the record, the Cortex-M3 does have a
  3-word instruction cache to allow for some amount of flash latency
  (presumably because they have a shared instruction/data memory).

- Leon: wondering if it makes sense to have an address-based read
  function in a HIL, when writes are always going to be limited to a
  page granularity. What are the use cases for being able to read
  arbitrary data but only write pages? If that's really required, a
  wrapper performing the loops shouldn't be too complex / have high
  overhead.

- Brian: in this case we actually support writes smaller then one
  page.

- Phil: how does that work? On flash you usually cannot do a bunch of
  byte-level writes. There is a limited write cycle.

- Brian: don't enforce that. Within the file system, we often zero
  stuff out. It does not protect from doing a double write. It's
  important to support partial page writes.

- Phil: chips typically support multiple writes to a page, but only
  flipping ones to zeroes, and there is a limit on the number of times
  this can be done.

- Brian: this is focusing on NOR flashes, where typically one does
  not have to write an entire page at once.

- Phil: it seems like there is an implied state machine of erased to
  written to zeroed. One can go from erased to written or from erased
  to zeroed, but not from zeroed to written or from written to
  written.

- Leon: key point is that this is an implicit state machine. There are
  no guards on the order in which you do these instructions and thus
  the HIL may silently provide entirely unexpected results. It sounds
  like this is very hard to use correctly.

- Phil: not if the interface would provide an error on invalid
  transitions.

- Leon: did not get the impression this happens. Also, might be
  incompatible with the interruptible nature of writes.

- Brian: if one calls write multiple times without zeroing first,
  should be handled at a higher level than this interface is intended
  to be.

- Jett: Which layer should HILs be at? This is trying to be as low
  level as reasonable.

  Agreed that this interface can be used incorrectly. Should the
  protections not go on top of it?

- Phil: Not all HILs are at the lowest layers. Some can be very
  low-level, some higher up. However the API should always be
  consistent. Individual calls need very clear defined semantics.

- Hudson: does this HIL provide sufficient abstractions to allow
  clients to be independent of the underlying hardware?

- Leon: at the very least would require a method to query page sizes,
  or even translate addresses to page banks and numbers.

- Brad: the write issue is where a lot of concerns have come up in the
  past. There has been demand for this interface and we should
  introduce it. We have always been hesitant with interfaces which are
  very easy to use incorrectly.

- Leon: key is documentation, especially for edge cases. For instance,
  does the offset point into the page or the buffer. Or what happens
  when a page is written twice. It's important that all
  implementations behave according to the specification.

- Brian: will incorporate updates. Also trying to publish more
  information about the way this interface is used internally.

- Phil: should the HIL should be parameterized on the page size? If I
  want a buffer which is a full page in size, how do I determine that?

- Hudson: could use associated constants for that.

- Leon: would give the option to pass in fixed-size arrays instead of
  slices, if that turns out to be beneficial.

- Leon: should this be an unsafe interface?

- Phil: could be writing code.

- Brian: making it unsafe opens Pandoras' box of accidentally doing
  potentially dangerous operations down the line.

- Phil: if this can write code space, it has to be unsafe.

- Hudson: or region of flash we hold Rust references to.

- Leon: can make use of the unsafe-creation safe-usage
  paradigm. Creating some instance implementing this interface would
  be unsafe, whether the API would be safe.

- Hudson: if there is an interface which can just overwrite code, that
  interface needs to be unsafe.

- Leon: correct, would need to be documented that this interface must
  not be used when the accessible memory overlaps with program memory
  or references. The instance implementing this interface would
  enforce bounds on the accessible flash window, which is set during
  instantiation of the implementation (an unsafe operation).

- Brian: we could add this as a directive on when to use this
  HIL. Trying to come up with an idea of how this interface could
  enforce it directly.

  However, even if we limit the accessible flash in the initialization,
  we still need to trust this to be done correctly.

- Leon: that's fine, because it's going to be an unsafe operation.

- Hudson: we could encode these bounds through an associated constant
  or const function that returns which addresses are allowed to be
  read or written.

- Leon: we could also encode them in an instance of a particular
  struct which can only be created in an unsafe manner. In that sense,
  holding a reference to this struct allows access into a particular
  region/window of flash.
