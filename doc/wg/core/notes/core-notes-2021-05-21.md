# Tock Core Notes 2021-05-21

Attending:
- Alex Radovici
- Amit Levy
- Andrew Malty
- Brad Campbell
- Gabe Marcano
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Philip Levis
- Vadim Sukhomlinov

## Updates

### Allow buffer overlap handling
 * Phil: I sent out the latest numbers I have for doing runtime checks for
   buffer overlap. Given the [discussions on
   tock-dev](https://groups.google.com/g/tock-dev/c/cY0-eKc6aos), we can figure
   out whether they are necessary or desirable. Basic summary is they're in the
   range of 70 - 300 cycles depending on the number of outstanding buffers. I'm
   only looking at 0-8 outstanding buffers. Most of the overhead, looking at the
   instructions, is register spillage. For example, the "insert zero" case is
   about 68 cycles, but if you refactor the code so it doesn't have to pay the
   same preamble overhead as other cases it is 17 cycles. So there are tradeoffs
   there. Do we care more about smaller or bigger cases? I want to lean towards
   bigger cases because it seems we should care more about maximum possible
   overhead than average. My guess is I can't squeeze too much more out without
   doing assembly programming, which I don't want to do because of portability.
 * Hudson: Leon and I have a working Rust playground that implements the slice-of-cells
   approach to give an API that's somewhat close to AppSlice. It no longer
   provides the kernel Rust references to the underlying memory; it makes all
   changes go through functions like copy-to-place and copy-from-slice in a way
   we believe to be sound. Overhead is very low, main limitation is you can't
   pass a slice of cells the same way you can pass a normal buffer. Necessitates
   some copying.
   - Vadim: What would it look like if you need to call a C function that takes
     several buffers together.
   - Hudson: I think you would need to copy into a Rust buffer for it to be safe.
   - Amit: Why is that the case?
   - Hudson: That's a good question.
   - Amit: The representation is transparent, so it's guaranteed to have the
     exact same representation as a slice of bytes.
   - Hudson: Yeah you're right.
   - Amit: FFI is already unsafe, so you already need to reason anyway.
   - Hudson: Amit is correct. You can pass a reference to a slice of cells to C
     functions directly.
   - Vadim: So for C functions there are no big difference compared to previous
     implementation and the real soundness come from checking the buffers do not
     overlap.
   - Hudson: Our implementation allows buffers to overlap. The fact these things
     are wrapped in cells informs the Rust compilers it cannot make the same
     aliasing assumption.
   - Phil: Can we go back to the overlap. Why do we want multiple read/write
     references to the same data? I agree that if we can do it in a way where we
     don't have a runtime overhead and it won't violate Rust safety, that would
     be better, but what are the cases where we want a process to pass multiple
     overlapping read/write buffers to the kernel?
   - Vadim: Yeah the use case is when say you make encrypt of a large block and
     you don't want this encrypted block to go into another memory allocation
     you can do it in place. You're processing in blocks of sixteen bytes used
     by the cipher. You read 16 bytes encrypt it and write it back so if you
     have to process ??? bytes of data it's a waste of memory having another
     buffer.
   - Phil: Let's go over this in email. I don't understand, but let's not waste
     call time on it.
   - Amit: Maybe we can follow through on a separate call for those interested.
   - Hudson: Leon's planning to schedule one but he is on vacation this week.
 * Amit: My student Anja has started working on an implementation of lightweight
   contexts for Tock userspace. Lightweight contexts is this work from (SOSP?)
   for FreeBSD from a few years ago that provides an abstraction of different
   protection domains to userspace without separate threads. There's some use
   cases where it's interesting to Tock. Vadim chimed in he had some use cases
   too. She's exploring that design and probably other design because
   lightweight contexts seems to be high-overhead for our scenario.

## Process Console Extension presentation
 * Phil: To introduce Andrew, he is working on his senior project for some fun
   implementation systems work. We're talking and Brad mentioned that improving
   the process console and making it more featureful is useful. Andrew has been
   working on it the past 8-ish weeks. He'll talk about what he's done and some
   small implications to the kernel.
 * Andrew: Process console started without a true writer, it used the debug
   writer. It used its own writer just to add new lines at the end of user
   input. I fully featured the writer to the debug statements. I added a queue
   for the writer so that it doesn't lose data. I added a state machine for
   large prints. I added a couple commands to make large prints. They would have
   otherwise needed a large queue, instead we use a state machine to allow the
   queue to be small without dropping packets.
 * Andrew: Added a command to print the memory map for a process. This is a
   large print so we had to use a state machine to print out several parts
   rather than one large chunk.
 * Andrew: Added a kernel command that has more of an effect on the kernel,
   mostly to get information to print things out. Added the ability to print the
   kernel map of the kernel, and a macro to see which drivers are available in
   the board structure. Could not think of any approaches other than a macro in
   the board's main file, so we decided to use a macro. Macro seems to work well
   but isn't ideal.
 * Andrew: End of presentation, questions?
 * Vadim: In Chrome OS, we have similar changes to implement a crash log stored
   in flash. That allows us to print the crash log on a subsequent boot. Another
   interesting things is tracing kernel functions using compiler sanitizer
   support. Allows you to retrace code execution. For us it was very helpful to
   debug some issues in kernel in early days.
 * Andrew: The problem Brad pointed out in the pull request is it forces the
   kernel to implement a lot of features. One change we may make is to push them
   downstream, so the board chooses whether to implement them rather than the
   kernel implementing them for all possible processes.
 * Amit: I'm wondering if you considered alternatives to this macro, like a
   derive.
 * Andrew: I did. The only reason I didn't try to implement a derive beforehand
   is I couldn't find one that looks similar. I could have created my own
   derive. I asked Phil if it was good to pursue and he said it probably wasn't
   necessary.
 * Amit: It seems generally very good. I haven't been able to grok if you
   absolutely have to use the macro. What happens if a board doesn't use this
   macro -- does it simply lose the process console? What is the opt-in/opt-out
   status of this?
 * Andrew: The process console can fully function without it. It probably should
   be an optional input. It's structured internally as an optional element, but
   it currently is not, but should have no impact if the user doesn't add it.
 * Amit: The macro's maybe not so bad because you only wrap boards in it if you
   actually use it.
 * Phil: I think it is important it is optional, because there is a chunk of
   overhead.
 * Alex: I assume you're printing the fields of the platform structure. Would it
   be better to print the actual data type? Boards can define the fields with
   any tame, and if the fields don't have distinctive names it could be hard to
   understand. Can you print the driver number as well?
 * Andrew: Printing the type is something we can do. I printed out the field
   name because I thought it was more consistent and shorter. Printing out the
   driver ID, I think I could.
 * Amit: That seems tricky because currently the driver number is encoded in the
   control flow of the Platform trait instance. By convention we store that in
   the capsule's `DRIVER_NUM` constant but that's not required. Technically a
   board could use a different number.
 * Phil: It's just the match statement, right.
 * Leon: Returning the type is less useful than returning the field name because
   the field name is more unique. It is guaranteed to not be duplicate.
 * Amit: The benefit of the type would be that regardless of the field name the
   board gave the printout would be the module name of the driver which is
   clear. The downsides would be the types may be longer and/or not unique (if
   you have two timers or two consoles).
 * Andrew: You could also uniquely print them if it is not meaningful that there
   are multiple.
 * Amit: I don't think we have any cases where there are multiple cases of the
   same driver. I expect if there were, it would probably be meaningful. This
   also seem like somewhere where we could extend the macro with a different
   variant to print out types.
 * Andrew: It would be a very minor variation. Just a question of making it a
   thing you change in the code or a thing you can request from the process
   console.

## [libtock-rs Exit API discussion](https://groups.google.com/g/tock-dev/c/zlyLPvMZd8g)
 * Johnathan: Within the Tock TRD, several system calls have sub-variants. Memop
   has sub-variants (as it did in Tock 1.0). Yield has them, and Exit has them.
   The Tock TRD defines two variants of Exit. One is terminate, and one is
   restart. Both variants take exactly one argument. Sort of implicitly, it's
   specifying a single function that takes one argument specifying what happens
   after Exit and another specifying the completion code. However, that's not
   the way the TRD is structured so in the future we may see new Exit types
   added that have different arguments. The question is what should the
   `libtock-rs` Exit API look like.
 * Johnathan: Idea 1 is to have a single function that takes two arguments --
   the first specifying whether to terminate or restart, and the second
   specifying the completion code. The advantage of that idea is if somebody
   wants to write a library with configurable exit behavior, they can pass the
   exit behavior enum through. The downside is the API may stop making sense if
   we add a third type of Exit call to the TRD.
 * Johnathan: Idea 2 is to make the two types of Exit two separate functions.
   You would have `exit_terminate` and `exit_restart` and each would take a
   completion code.
 * Johnathan: Idea 3 from Amit is to do both and guide people who don't need the
   functionality of the two-argument function to use the individual functions.
   The idea is if a third type of Exit call is added in the future we could
   delete and replace the single exit function and leave the `exit_terminate`
   and `exit_restart` functions in place so code that use those doesn't break.
 * Johnathan: It's not clear to me what people's opinions on them are except
   that nobody has a particularly strong opinion. Let me know if you have an
   idea you strongly like or dislike let me know.
 * Vadim: As a result of the function the process would be terminated and either
   restarted or not restarted. From that standpoint I don't see a reason to have
   separate function we can just make a variant one just the function we should
   exit and the kernel may or may not restart the process. I'm not sure this
   function would be very common, it's mostly to implement panics and things
   like that. Most processes are resident and have to live as long as possible.
 * Johnathan: I agree it would be rare. The advantage is "what happens if a
   third type of Exit is added"?
 * Vadim: Enum can be extended and that's it and it won't effect most
   applications they will just use the previous types of enum.
 * Phil: The issue is what if a new type is added that has parameters and
   changes the signature. You're right if it's just adding an element to an
   enum, but what if it takes a parameter.
 * Leon: If it's a completely disparate variant that is not compatible with the
   others, such as one that can fail, we could add a separate function and still
   have one that covers both of the variants we currently have.
 * Johnathan: That still ends up being awkward because if I were naming a
   function now I would call it `exit` so you would end up with `exit` and
   `exit_advanced` or something like that. That would be a bit weird. We'd get
   into renaming functions, but that's still breaking changes.
 * Leon: Then I would side with having two function because I think one of the
   core ideas is that most of the syscall operation types we don't want to bind
   ourselves to the type of behavior it implements. I could imagine that in the
   future we might add calls that are entirely different from what we have now.
 * Phil: My glib comments about GCC aside, I think Amit threaded the needle
   pretty well. I think we can rely on the Rust compiler properly inlining.
   Ergonomically having two different functions are much better. Easier to
   remember a method name than a method name and enum names, and for folks who
   want to do a dynamic choice we can have the underlying function.
 * Vadim: I can hardly imagine we would need that function to be used in
   application logic, it would be hidden somewhere. I don't have any strong
   preference to any implementation because it's not something we are going to
   use directly.
 * Amit: I suspect from an engineering perspective it makes sense to have a
   common underlying function anyway because it's going to be written in
   assembly. You're going to have to write implementations for each architecture
   and doing that once rather than twice is less bug-prone.
 * Johnathan: This is already higher level than assembly. The underlying
   assembly function is "make a system call with two arguments", and that is
   shared with Memop. We're already one layer above that, so we're potentially
   talking two more layers above that which would trivially inline.
 * Amit: I can imagine cases wher you want the first variant in runtime code
   that is taking an error return value that specifies what kind of termination
   you want. If we're worried about future-proofing we can mark it `unsafe`. I
   personally am not that swayed by the argument of there being a problem if we
   ever decide to extend the system call interface.
 * Johnathan: Leon, are you happy with the plan of having both APIs? Flexible
   exit and wrapper functions.
 * Leon: I'm fine with that. It's not a really big deal for me personally
   because the added type safety Rust gives up makes refactoring the changes if
   we introduce breaking changes easy to implement.
 * Johnathan: I'll put together a PR.
