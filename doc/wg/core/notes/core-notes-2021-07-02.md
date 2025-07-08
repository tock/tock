# Tock Core Notes 2021-07-02

Attending:
- Alexandru Radovici
- Amit Levy
- Anthony Quiroga
- Arjun Deopujari
- Brad Campbell
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Philip Levis

## Updates
* Johnathan: Unfortunately this only applies to RISC-V, at least with the
  current plans. On behalf of OpenTitan, LowRISC is working on an embedded
  position-independent-code spec for RISC-V that they're contributing to LLVM
  for both `clang` and `rustc`.
* Amit: That's awesome. Do we know if it's going to support the kind of
  PIC we rely on right now in ARM for C?
* Johnathan: The current plans are such that it will initially look a lot like
  that, but then it will change to be somewhat more efficient and eliminate the
  global offset table. It was specifically designed for Tock's usage.
* Amit: That would be huge, that's great.
* Alexandru?: I have a grad student trying to implement low latency in Tock.
  Currently he is trying to implement an eBPF driver. I'll keep you posted on
  how that works. He'll probably send the PR in the next week.
* Amit: Very cool, so it's an interpreter or something for BPF?
* Alexandru?: For now, it's just an interpreter. We'll see if we can transform
  it into a JIT at some time. It's a bit faster than running in userspace, but
  we're still working on optimizing it. We have a GPIO test frame -- if the rule
  is in userspace it is about 100 microseconds, in eBPF in a capsule it is about
  70 microseconds. We think we can reduce it to 30 microseconds. If the code is
  native in the capsule, it is 30 microseconds.
* Amit: So that is a lower bound.
* Alexandru?: At the moment yes. Tock does all bottom halves, so the time is
  probably lost in the time it takes to execute the bottom half in the kernel.

## PR #2462 vs. #2639
* Amit: Unfortunately, Leon is not here, but I think at least Brad and I might
  be able to provide that perspective.
* Amit: As of two days ago, there are now two competing designs for enforcing
  upcall swapping. One is Leon's implementation, which we have discussed a
  couple times in the past. The other is a new one Brad opened a PR for a few
  days ago. Brad, can you summarize where the implementation ended up?
* Brad: In v4, the core kernel itself handles the entire subscribe call. A
  userspace process calls subscribe, the kernel stores the upcall, and
  guarantees it provides the old upcall. Just like capsules access grant regions
  through a handle that they `enter`, the capsule gets a handle it can use to
  schedule an upcall. The core kernel does all the checks to ensure it is a
  valid upcall. At no point does the capsule hold or store an upcall object.
* Brad: The major issue that came up in the implementation is "what happens if
  the very first thing the process calls for a driver is subscribe". At this
  point, the grant has not been created, this process has never used the capsule
  in the past, so the kernel has nowhere to store the upcall that came in from
  the subscribe call. We talked about a range and there's a comment in the
  source that explains all the options we thought of to address it. The option
  we took was to add a new API to the Driver struct that is an `allocate_grant`
  function capsules have to implement. That way the kernel can ask a capsule to
  allocate its grant, so the kernel has a place to store upcalls. This is a
  minor function, pretty much boilerplate.
* Brad: A few other bumps came up, but we've smoothed those out. I think it's a
  full implementation that would be ready to merge.
* Amit: Do folks have thoughts about these two competing implementations?
* Johnathan: I like the idea of having the core kernel handle subscribe, i.e.
  Brad's implementation, and I'm happy to see it results in a size reduction as
  well.
* Amit: Leon, it looks like you just joined. We're talking about the two upcall
  swapping designs: your implementation and the PR Brad opened a few days ago.
* Leon: I am generally in favor of the other [ed: Brad's] approach now that I
  have seen how it works out. I think the core kernel handling it is just fine.
* Amit: It seems like, importantly, there is also not a restriction I was
  worried about where a driver can only have one grant. That is not the case
  here, right?
* Brad: That is still the case.
* Amit: What enforces that?
* Brad: When a capsule goes to allocate its grant, if the driver number that was
  passed in when the grant was initially created was already used, the grant
  will not be allocated.
* Amit: I'm confused. `enter` hasn't changed, so if I call `<grant type>::enter`
  what prevents it from being allocated as a separate grant?
* Brad: You'll get an error. What prevents it is when the kernel asks the
  `Process` implementation to allocate space for that grant in the process'
  grant memory, the `Process` will say the driver number has already been used,
  so it can't be created.
* Amit: That's only for callbacks.
* Brad: It's all the same. The `Process` implementation does not care how
  anything else uses the memory allocated in the grant region. It's given a
  length and alignment and will return some space. It does know about the driver
  number and is promising that the same driver number will never be used for
  multiple grants.
* Amit: I see, because the grant now has a driver number.
* Phil: It seems to me the major distinction is that Brad's approach really
  enforces the semantics in TRD 104. With Leon's approach, if a capsule does
  something wrong, we kill the process.
* Leon: That is correct. We would have some form of error handling -- killing
  the process, panicing the kernel, and disabling the capsule are all possible.
  With Brad's approach, we have the significant advantage of there not being any
  chance for things to go wrong assuming the kernel's implementation is correct.
* Phil: With Leon's approach, you can detect a bad capsule but there isn't a
  great answer for what to do.
* Amit: It seems that the v4 upcall swapping restrictions imply a tighter
  coupling between grants and drivers than was required in the previous design.
  The good thing is that is how we use grants in practice so maybe that's a good
  thing.
* Brad: And that seems to be fundamental. Despite Hudson and I arguing for a
  month, we could not come up with a way where that was not a fundamental
  requirement.
* Phil: Just to check, this requires that every driver have a grant region?
* Amit: It requires that every driver that has Subscribe has a grant region.
* Phil: What if there's a driver that doesn't do anything for Subscribe, what
  happens?
* Brad: The subscribe is rejected and userspace will get an error.
* Phil: So it does not require every system call driver capsule to have a grant
  region.
* Brad: In fact, we can't enforce that a capsule will correctly implement
  `allocate_grant`. There's not a default implementation so you have to provide
  something, but if you throw an empty `Ok` and don't do anything the kernel
  still won't be able to store an upcall. That will result in sending an error
  back to userspace.
* Leon: I want to emphasize the fact that we need to use grant for subscribe is
  shared with my approach as well. That seems to be fundamental and also a good
  property, as it makes it way harder to write non-virtualized drivers.
* Brad: So Amit, to follow-up on these multiple grants, if you use a different
  driver number in your second grant, you wouldn't be able to store upcalls for
  your actual driver number. I believe you could still use the grant.
* Amit: Yeah, you'd have to allocate a dedicated driver number for it.
* Amit: I think this changes how we want to talk about drivers and grants. In a
  sense, with this design, what is a driver? A driver is potentially the only
  component that can allocate memory from a process in the grant region and also
  interacts over a system call interface, which makes sense. Before, it was the
  case in practice that only drivers used grants but that was not strictly
  necessary.
* Phil: Would it be possible to have a capsule that does not handle system
  calls, but has a driver number and allocates grants?
* Brad: I think you could set that up.
* Amit: I wonder if there's a very similar version of this that separates grants
  and upcalls.
* Brad: I think there are more options here. We could have `Grant` take an
  optional driver number, giving us two versions that can and cannot store
  upcalls. Another extension is if you want to use upcalls but not have a grant,
  the kernel could still handle that, it'd just be more code and be
  inconvenient.
* Leon: [tries to speak but signal cuts out]
* Amit: If we separated upcalls and grants, we would need an additional table to
  store the upcall list in each process' grant region. With this implementation,
  we're piggybacking on the grant pointer table, and we just know the offset of
  the upcalls relative to the grant itself.
* Brad: It's pretty explicit -- we have to keep a number around so we know how
  to calculate the offset. There is something a little more subtle here as well.
  We could have two pointers in the grand index list: one that goes to type `T`
  and one that goes to the upcall array, except we don't know how many upcalls a
  particular capsule wants.
* Amit: I'm suggesting something else. At the top of the grant region, we have a
  list of grants. We know at boot time how many there are. I'm suggesting we
  could have an additional list that points to upcall lists, one for each
  driver. We also know the number of drivers at boot time, so those are two
  independent lists that could have a different number of elements in them.
  Often they'll be the same number of things. When a process calls Subscribe on
  a driver, then we'll allocate the entry for that driver in the list.
* Brad: The issue with that is we do not know the number of upcalls that driver
  wants.
* Amit: `allocate_grant` could become `num_upcalls` or something.
* Brad: Right, we would still need some mechanism.
* Amit: Or maybe it could be a type parameter, like how `num_upcalls` in now a
  type parameter for `Grant`.
* Alexandru: How do you know the exact number of drivers?
* Phil: It does know the number of grants allocated.
* Alexandru: If grants, yes, but of drivers, no.
* Brad: That's correct. We would need some new way of counting that for what
  Amit's describing.
* Phil: I think the approach Brad has seems generally simpler. I prefer that the
  allocation is simpler and has simpler data structures. Having just had to deal
  with rewriting the process loading and restart code, I think simpler kernel
  process data structures are good.
* Amit: Aside from the tangent, I think I'm hearing that we prefer version 4,
  and that's pretty close to what we would want to actually merge. Does anyone
  disagree?
* Brad: Does anyone have lingering concerns? We haven't really talked about the
  downsides. There is a fair bit of new pointer complexity. I've tried to be
  overly verbose in describing what I'm doing. That code is harder to maintain
  than it used to be, so that's a major drawback.
* Phil: My one comment is the comment for `allocate_grant` is remarkably long
  and could be trimmed down. It goes into other comments rather than just what
  it does.
* Brad: But that's an internal? comment, not the public documentation.
* Leon (in chat): I think that the benefits of v4 are really good and important.
  It's a bit more complex, but the motivation for v1-3 was for them to be easy.
  Now that we know that v4 works and is reasonable, it seems good.
* Alexandru: My only comment is the driver number needs to be stated in two
  locations. There's a runtime check, but the ergonomics are not great. I don't
  have another solution.
* Brad: That's a good point. I haven't really thought through what happens if
  you do that incorrectly. If a board's `main.rs` uses one driver number for the
  syscall mapping and another for the grant, then subscribe would fail for
  processes. It would come in on one driver number but the kernel would not find
  a grant with that driver number, and it would not be able to create one,
  because there is no grant with that driver number. All it would be able to do
  is send back an error to userspace.
* Brad: Luckily that would be unconditional; any time you try to call subscribe
  it doesn't work.
* Alexandru: The problem is it can be silent. If someone makes the mistake at
  the beginning, it will be a silent failure. Maybe panicking the kernel, or
  documenting it might be good.
* Brad: What do you mean by silent? Userspace might not check the return codes.
  In `libtock-c` we've made that a little bit more difficult.
* Alexandru: It will be an error that somebody new to Tock will spent a lot of
  time debugging. Finding the error may be difficult. That's why I'm saying
  maybe document this really well.
* Brad: So "Your subscribe failed, here's the most likely reasons why".
* Alexandru: What if both drivers exist and somebody swaps the numbers when
  initializing them, what happens there?
* Amit: It would fail for the first driver. The second would succeed, wouldn't
  it?
* Alexandru: I think the check in the code checks the driver number from the
  grant with the driver number that it received, if I'm not mistaken.
* Amit: By the time you do the second, the first driver will allocate a grant
  for the second one and subscribe will have failed. On the second driver it
  will find the first grant region.
* Alexandru: That's a big problem. That means one driver could allocate a grant
  for another driver. Data types could mismatch. Wouldn't that result in memory
  corruption?
* Amit: I don't think so. The safety here is still preserved, the only thing
  that gets confused is where are upcalls stored and who might call them. The
  negative thing is one driver would be able to invoke upcalls on a process that
  were registered for another driver.
* Alexandru: This could be prevented with a runtime check, maybe. Actually no.
* Amit: It's tough because the driver doesn't know its driver number.
* Alexandru: Or it's a malicious driver. Actually it can't because the number
  comes from the board implementation. It's still a really hard error to debug,
  and an easy mistake to make.
* Brad: Definitely this could have some very strange behavior, that matters on
  ordering. Both subscribes could succeed, or one could fail. I think our basic
  answer is "components make it easier, hopefully".
* Amit: Right, ideally this is something that is resolved in the board
  configuration. We could make the board configuration do this automatically.
* Alexandru: I don't see how. You have the `with_driver` function that has
  nothing to do with the components. Then you supply the driver number to the
  component, so I don't think components solve the problem.
* Amit: I agree components on their own don't. My dream for several years is a
  macro for specifying a driver that compiles into the spaghetti code we have
  now.
* Brad: This is a fundamental issue to both designs, choosing v3 or v4 does not
  address it.
* Hudson: Sorry, I joined a little late. The problem you are discussing is if a
  different driver number is passed to `with_driver` and to `create_grant` that
  you will get potentially cryptic and weird bugs? For v4 the check is currently
  at runtime and it just returns an error.
* Brad: Correct
* Hudson: Could we panic when that happens? Then it would be easy to debug.
* Brad: Then you would be panicing the kernel.
* Hudson: I think it is okay, because it is trusted code in `main.rs`. A capsule
  can't do it.
* Phil: How would this work? Where would this panic check go?
* Hudson: When Subscribe into a grant it passes the driver num it is expecting
  the grant to have and if they don't match, there's a panic.
* Brad: Wait, when?
* Hudson: Where we currently return an error in the code.
* Brad: Sorry, I'm not quite sure. Is a capsule triggering this, is the kernel
  triggering this?
* Hudson: No, it would be in the kernel code.
* Brad: Right, but I'm still not following when. What series of events have
  happened?
* Hudson: The series of events is a capsule has called subscribe.
* Brad: Capsules don't call subscribe, processes do.
* Hudson: Yeah, so a process has called subscribe.
* Brad: But then how does the kernel check?
* Hudson: I added this check yesterday...
* Brad: Let's talk about this offline.

## Code size reduction effort update
* Hudson: I sent a PR which I don't expect to be merged until after the 2.0
  stuff settles. The high-level takeaway for the PR is we have a lot of
  high-level functionality in methods on `Grant` that are generic over the type
  in the Grant and/or closure. E.g. every time you call `Grant::enter` you
  duplicate the logic in the binary. It's generally inlined into the use site,
  and for boards like Imix that means 50 or more copies of the identical code.
  This PR moves the portions of the functions that do not rely on the generic
  parameters into non-generic functions that are marked `#[inline(never)]`. I
  then call those from the generic functions. For code that doesn't need to be
  duplicated for the generic types, you only have a single instance of those
  function. As written, this saved about 7300 bytes on Imix. Functionally it is
  identical, except it might result in slightly-worse performance. A high-level
  takeaway is we should be careful to do the minimal amount possible in generic
  functions we expect to be instantiated multiple times.
* Phil: Hudson has had great success, and I haven't. I've been trying to reduce
  the size of the core kernel itself, the stuff that's always there and doesn't
  grow with the number of drivers. I looked at process loading. There were
  comments about code duplicated between process creation and restart. I
  refactored it, and it doesn't reduce the code size. It makes the code size a
  bit bigger for reasons I haven't gotten to the bottom of. Generally the
  compiler does better than we can, and I don't think there's that strong a
  coupling between the number of lines of code you write and what is generated
  because of generics. The places I think we should look are places like what
  Hudson's looking at. We should look carefully at how we're using generics and
  what that means for code duplication. That being said, I have a rewrite of the
  process loading. It does reduce the overall size of the kernel. There's a
  slight increase in code size and a significant decrease to embedded data size.
  I need to figure out what's going on there.
* Brad: That's really interesting. In the upcall swapping PR, we're trying to
  document we're doing all this `unsafe` stuff, what I've found is that code you
  might otherwise write in a single line is better split up so you can comment
  each step. That adds lines of code you wouldn't otherwise have if you were
  just writing C code. In Rust there's a lot of reasons why lines will not match
  compiled code.
* Phil: That's right, for example there used to be a `create` and a `restart`
  call. `restart` did a subset of `create` with a little tweaking. I pulled the
  stuff out and put it into `restart`. Now when you call `create`, you still
  allocate the grant region. The actual allocation of the flash region, the
  clearing of pointers and similar, are all done in `restart`. Now `restart` is
  called by `create` and on a restart, so now it's not inlined. I think there's
  a lot of bookkeeping added that's adding code size. A lot of stuff is put into
  a structure and `restart` loads it.
* Phil: One other observation: process loading has great line-by-line comments,
  but the big picture is a bit weak, so I added more comments. A funny bug I had
  was I wasn't increasing the kernel allocation enough, overlapping the grant
  region and process structure. Suddenly your upcalls were getting corrupted by
  the process structure, and you were jumping to nowhere and getting faults.
* Brad: Those bugs are great in a language that's not supposed to allow you to
  do that.
* Phil: Once you do that one minus correctly, then you're fine.

## Free-for-all Allow PR
* Amit: Brad, do we have enough time, or should we shelve?
* Brad: Alistair asked me to bring it up, because this meeting is at a time that
  doesn't work for him. We need to decide whether we do or don't like the
  concept, and whether this should be pre- or post- 2.0.
* Phil: My thought is this should be post-2.0. This should be a separate TRD
  from 104. Generally we should do it, but I'm looking at the back-and-forth
  that happened with Allow, and I wouldn't want 104 to block on that.
* Amit: I agree with that. We should circle back to this -- and look at it
  offline as well, because it's been hanging for a while.
