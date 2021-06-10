# Tock Core Notes 2021-06-04

Attending:
- Alexandru Radovici
- Andrew Malty
- Arjun Deopujari
- Brad Campbell
- Branden Ghena
- Gabriel Marcano
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Phil Levis
- Simon Tao
- Vadim Sukhomlinov

## Updates
 * Leon: I'm about to send out the email regarding the external discussion about
   preventing aliasing of Allow buffers or how we handle writes and reads to
   them in the kernel. I announced it a few weeks ago but it didn't really fit
   into the schedule. Everyone who is interested in that discussion should check
   out the Tock mailinglist where I put out a doodle to find a time which fits
   all of us.
 * Brad: This is to discuss AppSlices that overlap each other, right?
 * Leon: Right. Either how we handle AppSlices which overlap each other or how
   we prevent it from happening.
 * Brad: Part of that discussion might be naming around these things. We need
   terms for overlapping in terms of memory addresses and overlapping in terms
   of memory accesses. Can a process access at the same time-ish as the kernel?
 * Leon: That's part of today's discussion. I think the accessing in parallel is
   going to be a shared allow, whereas when I speak of overlapping allows I mean
   multiple allows from the same app pointing to the same memory.
 * Brad: I guess it's the "shared" word. AppSlices are always shared. It's
   confusing to have some be shared and some be shared.
 * Leon: I think there's been a lot of movement on this topic yesterday so I'm
   just catching up.
 * Johnathan: I've seen the term -- in terms of Rust's semantics -- the term
   use-based, to refer to things that only apply when things are used rather
   than things just existing. We may have a parallel we can draw from there.

## Code size improvements
 * Phil: I would like to introduce Simon. Simon is an undergrad who has been
   working with me for the last quarter, looking at code size from several
   angles. He's required by his class to give a public presentation of his
   results. I sent a draft of his final report to the group -- maybe the best
   thing to do is have everybody take a look for a couple minutes.
 * (look happens here)
 * Simon: I am currently at the end of my junior year at Stanford. I'm excited
   to share what I've been working on along with my friend Eduardo.
 * Simon: The project that Eduardo and I worked on this quarter is looking at
   code size. Specifically, we were looking at the code size for nested
   closures. Generally Tock is used on embedded systems that are limited in
   size, so we want to use smaller code space.
 * Simon: There is a hypothesis that having nested closures increases code space
   due to needing to save various states. In Tock 1.6, there is the
   console.send() function which has a triply-nested closure. We compiled with
   Rust version 1.51.0, and used the Tock OS tool `print_tock_memory_usage.py`.
   In this function, we have `tx.buffer.take().map`, and inside this `.map()` we
   have a for loop that performs `.iter()` and `.enumerate()`. Currently, the
   function is 284 bytes, and the total kernel size is around 174 kB. The
   question is, is there any way we can rewrite this boxed portion of code to
   maintain functionality but at the same time reduce code space.
 * Simon: The first method we thought of was to replace the outer `.map`.
   Instead, we take the buffer and simply unwrap it. With this, the size of the
   `send` function increased by 8 bytes and the size of the embedded data by 6
   bytes, but the function `tock_cells` decreased by 14 bytes, resulting in no
   net change.
 * Simon: Next, we looked at removing the `.enumerate` call from the loop. We
   replaced the `.enumerate` call with a separate counter we incremented
   manually. This did not produce an overall change in the kernel size or
   embedded data size. We think `rustc` already optimized `.enumerate()`.
 * Simon: The third method is something similar. Something we noticed is this
   `buffer.len()` function is called repeatedly. We weren't sure whether the
   compiler optimized it. We wanted to test whether the compiler actually had
   code overhead for calling `buffer.len()` many times. There was no change in
   kernel size or data size, so we concluded it's already optimized by `rustc`.
 * Simon: Our last idea is to replace the for loop with `.copy_from_slice()`.
   `copy_from_slice` has the same functionality as the for loop. However, we had
   to split it into two cases based on the size of the buffer. What's
   interesting is that the size of `send` increased by 4 bytes overall, however
   the size of embedded data across the kernel decreased by 28 bytes. Under the
   hood, this function called `core::ops::Range`, which increased by 40 bytes.
   Therefore there is an overall change in kernel size of 16 bytes.
 * Simon: Overall, we found most of these changes don't change the kernel size
   or embedded data size, and when they do they are for the worse. Our
   conclusion is that having nested closures does not affect code size by a lot.
   However, I want to emphasize our experiment was performed on Imix, and this
   might change in the future. `rustc` may change optimization settings in the
   future, or it may be different on another board. For future work, it would be
   good to replicate this experiment with a different board.
 * Vadim: Have you make a breakdown of code size. Why is this `send` closure 284
   bytes in the first place? For the first one, you have method original of 284
   bytes, but what are the source lines these are attributed at? Can you trace
   where the code came from in the source code? Maybe the cost came from some
   other code?
 * Simon: Let me clarify the question: in the 284 bytes, where does the majority
   of the code size come from?
 * Vadim: Yes. If you enabled symbolic debug information you will have source
   line tracing, and see what source line resulted in each instruction.
 * Simon: I can say with confidence that most of the 284 bytes do come from this
   body. Most of this does not have a lot of (edge?) space. Based on the
   function itself and the assembly we analyzed
 * Vadim: I would advise you to take a look at what comes from each source line
   versus the closure itself. The compiler captures the context of used
   variables in the closure. That code is a dark matter because it is
   compiler-generated and it comes from the fact you are using a closure. It
   would be very useful to take a look at that because once you are in the
   closure and the context is captured you don't see any big difference in code
   size by changing how you use that data -- it's already well optimized. I saw
   the context capture itself is the problem, and if you capture the same of
   variables you will get the same overhead every time. The only difference you
   will see is if you reduce the amount of captured variables. It just captures
   the content of the app struct.
 * Phil: It sounds like your hypothesis is that if you take one closure you pay
   the cost, but if you take nested closures it doesn't add to the cost?
 * Vadim: It matters in the sense of what else is captured. It adds up, the
   context is basically appended. It doesn't multiply, but requires you to
   capture outside context which adds fixed cost, and increases the amount of
   structs you have to copy. That code is problematic. The problem is context
   capturing, not what is inside the closure.
 * Phil: That's good to know. One way to think about it is they're looking at
   ways to make the function shorter and found there's no silver bullet for this
   particular function.
 * Vadim: That's what I'm proposing -- try to attribute every instruction to a
   source line.
 * Phil: Yeah I understand. It's a little tricky when it's optimized.
 * Simon: We have tried to do that. For example, we found setting up the for
   iter enumerate takes 36 bytes. We found that setting up the (hedge?) space for
   `copy_from_slice` call from for else range takes 20 bytes compared to 36
   bytes. However, in the assembly, we found that this was done twice, resulting
   in the function being 4 bytes larger overall.
 * Vadim: One experiment you can do here is you see all the references outside
   the closure like `app.remaining`. Try to do the following: create variables
   `app_remaining = app.remaining` before the closure and then use these
   variables which copy content from the closure manually and see how that
   affects the cost. If these are primitive data types maybe copying these
   variables may be more efficient than copying the whole struct which contains
   something you don't use inside the closure.
 * Simon: I see, we can take a look at that.
 * Vadim: I don't know when changes to the Rust edition 2021 are expected to be
   available, but that's one of the things I am waiting a lot because edition
   2021 proposed changing semantics for closure captures so that only used
   fields are captured. It may automatically solve problems with closures and
   nested closures. I am not sure how to activate this edition in Tock OS and if
   it's even available.
 * Johnathan: Can you send a link to that to me?
 * Brad: It's also always good to see work where the answer is "actually this
   didn't have an effect".
 * Vadim's link:
   https://blog.rust-lang.org/2021/05/11/edition-2021.html#disjoint-capture-in-closures

## Process console features
 * Brad: I think it's primarily blocked on me. I read the latest comments, and I
   would like to propose that if I can remove the kernel symbols so the kernel
   crate isn't dependent on the linker script, the rest makes sense. We can open
   an issue for having a more principled way to create process debug info in
   strings. There's already been an interest in having smaller mechanisms, so
   that'd be good future work. It would be nice to not have the kernel crate
   depend on a particular linker script, but that's an easy change to make
   quickly.
 * Phil: I think that sounds reasonable. We disagree on the linker script side
   of things, because they seem like things the kernel should have, although
   perhaps not the particular name. Part of the challenge is why you gravitated
   towards that rather than Brad's proposal of passing it in.
 * Andrew: What was the question?
 * Brad: We're talking about the tradeoff between using linker symbols inside
   the kernel for getting access to the addresses versus having those passed in.
 * Andrew: The assumption was every kernel would have those exact addresses so
   there was no reason to make them something passed in. If that's not a valid
   assumption, I don't think there's any real problem with just passing them in.
 * Brad: Yeah, I guess the work on the host side testing makes me wonder if that
   will just work. My main hesitancy is those symbols come from our particular
   linker script and they're not particularly nicely named. We've mostly moved
   away from using them except at the highest layers.
 * Andrew: If that's the case I have no problem moving them. I thought this was
   more a question between you and Phil. I did not very much like put them in
   with much thought as to their effect on the overall Tock environment.
 * Phil (in chat): Cool, sounds like Andrew is good with it, so let's do it!

## TRD 104 Updates
 * Phil: As I'm sure people have tracked somewhat, there's been all of this
   discussion about buffers that have been allowed. Is userspace stopped from
   accessing them, is userspace allowed to access them? This is tied in with
   Alistair's read only syscall approach of allowing userspace to read data
   that's updating by the kernel during a context switch. We were hammering away
   at specifying when you can do this, and it was getting pretty messy.
   Johnathan pointed out that allowing both of these access patterns will be
   very tough in Rust. If you provide both, the whole system can be `unsafe`.
   The current proposal is for read-write allows and read-only allows, userspace
   libraries must not access those buffers, and we'll have a separate system
   call for buffers that allow concurrent access between userspace and the
   kernel with its own rules for how they access it. Do people think that's the
   right summary or want to talk about some of the complexities?
 * Brad: I think that was helpful. The thing I'm curious about, and I'm having a
   hard time trying to distill it from the discussion, is that with the existing
   Allows you can have a buggy userspace app or an intentionally bad userspace
   app. As code reviewers, how do we think about that? Do we consider that a bug
   and maybe we catch it and maybe we don't? Is there a high level of importance
   for identifying those issues, or is it a "maybe sometime we can enforce this
   at runtime" thing?
 * Johnathan: There's an interaction here with `libtock-rs`' design. If we say
   userspace MUST not access the buffer, that allows us to design `libtock-rs`
   so that you cannot -- in safe Rust -- access the buffer after it has been
   allowed. Practically speaking, that means capsules cannot ask apps to do
   that, because apps written in Rust that try to use the capsules will be
   unable to do so. The other side is capsules must be tolerant of that anyway
   because a malicious app might do so anyway. We would consider an app
   accessing data that Allow says it can't access to be a bug, and we cannot
   have the kernel rely on that for correct function, but it may exist from a
   threat modelling standpoint.
 * Leon: If I understand Brad correctly, he was mostly concerned with userspace
   writing to a read-write allowed buffer.
 * Brad: I don't think I've thought through it to that level. What I'm trying to
   reconcile is the difference between something specified in code that we can
   test -- or which can panic during development -- versus something only
   identifiable in a code review.
 * Leon: With regards to the kernel, I think the latest discussion is that
   having exclusive access to allowed buffers in the kernel is the easy way out.
   That would imply that as long as the buffer is allowed the kernel can always
   write to it. We can't prevent apps from reading intermediate states, because
   they always have read and write access to the allowed buffers. The kernel has
   to always expect that apps might modify the buffers in flight, so from a
   safety perspective it is important for capsules to not rely on the fact that
   data may change. When it comes from capsules wanting to write data to
   read-write allowed buffers, there is nothing we need to watch out to prevent
   it, because capsules by definition have exclusive access. If userspace relies
   on accessing intermediate states that's their fault.
 * Brad: I think that makes sense. We're saying a capsule has to be defensive,
   because the kernel isn't enforcing it. That addresses correctness issues. The
   only remaining problem is if a capsule author wanted apps to access the
   buffer, and then didn't use the Rust runtime, and they can do whatever they
   wanted.
 * Leon: I think this is the main difference between what we've been discussing
   the past few days and the current proposal. There are these optimizations we
   might want to make where userspace apps can -- or are supposed to -- read
   intermediate states without unallowing/allowing a buffer. Potentially in the
   other direction, writing a value it expects the kernel to immediately see.
   Taking the route towards having two cases, we don't need to specify when it
   is safe for a capsule to write a buffer. In the current proposal, a capsule
   can always write to a read-write allowed buffer, whereas in the new class of
   allow we need to carefully design the bidirectional interactions between the
   capsule and userspace. We should define points where the capsule can and
   cannot write to the buffer, and apps can and cannot write to the buffer. For
   the majority of cases, we have this read-write allow that is conceptually
   much easier to understand and we don't have to trust capsules as much.
 * Phil: It's from both sides. A capsule must assume a buffer can change at any
   time -- a process can die and the AppSlice goes away. At the same time,
   userspace code and capsules should not assume they're accessing it during
   normal operations. It really lowers the bar with respect to the degree of
   care with which you have to look at the system. If we shove it all into Allow
   then any userspace could modify these things and we need to look at the
   capsule very carefully. A big concern is letting userspace trigger panics by
   writing to allowed buffers. It's clear there are use cases where we need more
   permissive sharing of buffers. For those, let's keep TRD 104 as-is and we can
   create a new TRD to add a new system call and support the use cases we're
   seeing.
 * Leon: From past discussions, if we have interactions between kernel and
   userspace where both have simultaneous access to a buffer with no exclusivity
   we really want to have a formal definition for when it is safe for either
   side to access the buffers in each way. We can essentially not have capsules
   written in arbitrary ways and instead have a formal specification on a
   per-capsule-API basis and we can check that what we're doing is safe and
   leads to consistent buffer states.
 * Brad: Do that for both cases or only the new proposed case?
 * Leon: Only the new proposed case. In the read-write allow case we have now,
   we don't need any specification on the kernel side because by definition once
   the buffer is granted the capsule can do anything with it until it is
   un-allowed again.
 * Brad: So from an education perspective, if you're a capsule author, are we
   basically saying "you almost absolutely want to use RW and would only use the
   new version if you have a very specific use case". Maybe we have a couple
   criteria we develop which would say "if you meet one of these then you want
   to use this". Is that fair?
 * Leon: I think so. I'd even go so far as to say that most of these use cases
   could be represented by having a wrapper around the shared AppSlice which
   handles the touch mechanics. For instance, there are established techniques
   for having lock-free bidirectional ring buffers between the kernel and
   userspace. We could implement this once so capsules don't need to implement
   it themselves.
 * Phil: When we write up this new form of Allow we don't want to be
   exclusionary. If somebody concludes they need to use it, then that's fine.
   The general motivation will be performance -- if you need to do lots of small
   reads or small writes and don't want to pay for a system call every time you
   do that.
 * Brad: Okay. This is important because if Tock is successful, capsule owners
   won't want to know every nuance of Allow. When people look at pull requests,
   this is helpful to see. If a PR is just using the new one that raises an
   alert, let's make sure that was intended.
 * Phil: Or they require much more careful code review, because it's a tricky
   thing.
 * Leon: I think it's another concern, because a shared allow is a superset of
   what you can do with read-write allow.
 * Phil: 104 should say "if read-write allow works, you should use read-write
   rather than shared allow"
 * Johnathan: If they use the Rust userspace -- and that's a big if -- that will
   provide encouragement to avoid the fully-shared allow.
 * Brad: I think that's a very helpful illustration that makes it clear.
 * Phil: I think there was a point where we were trying to squeeze this into
   read-write allow and read-only allow where we said that no capsule API should
   require concurrent access. It should be just an optimization. The canonical
   case can be something like Console.send or receiving packets, where if it's a
   performance problem you can do this concurrent access under restrictions but
   you can always do the swap. Then you need callbacks to tell you when it's
   ready to do the swap.
 * Brad: Has anybody looked at `libtock-c` and thought about if the APIs there
   make this at all intuitive? Is there a way to make it intuitive?
 * Leon: I suppose because C doesn't have the utilities that Rust has to
   represent the exclusive access to the memory region, it would behave
   similarly to what's currently in a read-write allow. As with everything in C,
   you would have to be careful how you use it.
 * Johnathan: It actually seems scary because I think you would need all
   accesses to the buffer from userspace side to be volatile which I think is
   trickier to ensure in C.
 * Leon: That's a problem with read-write allows anyway. We have the notion of
   there being no access from userspace, while in C there's nothing preventing
   you from just reading the buffer.
 * Phil: This gets back to Brad's original point. You use specifications at the
   point where you need a manual check. If you can statically check it, we can
   do it through APIs. We would be stating that the guarantees we give you is
   based on following these rules, and if you don't follow these rules, there
   you go.
 * Leon: It at least justifies documentation on `libtock-c`'s side.
 * Brad: I'm not referring to doing the memory accesses, I'm referring to doing
   the swaps. I imagine that every library in `libtock-c` is not doing this
   correctly.
 * Leon: I think currently every library is relying no the fact the buffer is no
   longer accessed by the application until it's given another buffer, which is
   an undocumented assumption.
 * Phil: I think Brad is probably right. In a lot of `libtock-c` code, if you
   call send, it does an Allow to give the buffer to the kernel, it does a
   subscribe, it does a command, it does a yield, it doesn't do another Allow to
   get the buffer back. That's a pretty simple thing to go through.
 * Brad: My question was, is there anything we can do -- wrap this is something
   -- to make this more intuitive that you've given up something and to give it
   back you have to do it again? It wouldn't be enforced -- it's not Rust -- but
   like the wrappers around the syscalls which make it harder to check them
   correctly.
 * Leon: I suppose there's macro magic but there's not the kind of closures we
   would use in Rust.
 * Phil: You could pass a function pointer with the parameters to command and it
   gets coupled with Allow/Command/Allow or something.
 * Branden: Create your own smart pointer type out of a struct and boolean or
   pointer or something. Even that's only a rule as long as everyone follows it.
 * Brad: Of course, there's not going to be any guarantees. I think that's fine.
   Most people are going to copy stuff, and if there's a nice API that's a lot
   less work we have to do checking to make sure there's Allows when we review.
 * Hudson: Most of this should happen in the libraries not the apps. For
   everything that's synchronous, you can wrap it.
 * Alexandru: One quick note, as someone who has used `libtock-c`, I think the
   rule about not using the buffer is not clear. I did not follow this rule
   because I did not know about it.
 * Phil: It's a new rule
 * Alexandru: I understand why I should do this, but I did not at the time. I
   think the idea of using a type instead of a raw pointer is better. Maybe
   accessing the raw pointer via a function like `get_pointer` or something like
   that. Can create an error if somebody does it wrong.
 * Brad: I agree
 * Hudson (in chat): +1
 * Alexandru: Question about the shared buffer, should there be separate shared
   read-only and shared read-write? I don't know if that matters or not.
 * Phil: Great question, we need to sort it out. I think to do is start with the
   use cases that are most pressing -- shared read-write where the userspace can
   read from what the kernel is writing -- and go from there.
