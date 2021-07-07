# Tock Core Notes 2021-06-25

Attending:
- Alexandru Radovici
- Arjun Deopujari
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Leon Schuermann
- Pat Pannuto
- Phil Levis
- Vadim Sukhomlinov
- Anthony Quiroga
- Amit Levy

## Updates

- Pat: I want to introduce Anthony.  He is going to be working over the summer on the CI work that was
  getting going.  You can expect a PR of how it is going to work later today after I read over it.
- Leon: We have been getting inquiries on releasing a new "Tock-registers" version.  Unfortunately, the latest
  Rust nightly build does break the current released version.  We should look into that.  One PR I released has some discussion
  and we should discuss how to continue from there and we can release a new version accordingly.
- Branden: If they are asking for an updated Tock-registers version, can't we just update Tock to that version?  This might
  solve everyone's problems.
- Leon: We have but it makes sense to release a new version of Tock-registers.  It refactored some bits but not the general 
  structure which doesn't make a ton of sense now.  There is already a PR out for which we need to have a loose resolution and then
  we can release the new version.  I created a PR which back-ports the essential change which is just renaming a Rust feature.  We can release
  that.
- Amit: Here is the PR Leon is talking about: https://github.com/tock/tock/pull/2618
- Phil: My update is that I have two undergraduates working with me this summer on code size. 
  We'll give updates on progress as they happen. One possible outcome is a tool that characterizes 
  embedded data and what code is referencing it.
- Phil: Just to be clear, embedded data does not refer to the ".data" section, it refers to data embedded
  in the ".text" section of a binary.
- Hudson: Working on an out-of-tree Tock code base, I ran into an LLVM bug.  If you replace the panic handler with an empty loop
  on any nightly Rust before this march, it breaks memory safety with Rust.  LLVM (version < 12) assumes panics can never happen and so removes all 
  checks.  So, for example, checks for buffer overflows will disappear.  All the checks in the binary where the code could panic would 
  be dropped.  This results in significant code size decreases.  This seems to be fixed in LLVM 12.
- Amit:  Is the fix incidental or an explicit one (this is ok to do now)?
- Hudson: It is not incidental but intentional I believe.

## App Slice Issue

- Leon: Last week, we talked about how we are going to approach the app slice issue after the TRD changes have been merged.  These
  changes allow for apps to allow overlapping buffers into the kernel as part of the `allow` mechanism.  That is, overlapping between the
  same `allow` call (read/write or read only) or between different `allow` calls and between different capsules.  This means we have to 
  handle this in the kernel and cannot use regular rust slices anymore or we'll see undefined behavior.  We need to handle this while
  maintaining an API that is easy-to-use and backwards-compatible with previous versions.  I've been implementing an API which I discussed with Hudson
  on my local repository.  Some networking drivers were difficult to port, however, due to encoding and decoding of headers passed from userspace to networking drivers in temporary buffers.  The CRC driver capsule is very challenging and I haven't been able to port it to the new API due to a DMA access.  This is the only capsule I have left and hopefully a PR will be out tomorrow.
- Amit: So this issue with rewriting the drivers, are you mostly finished with it?
- Leon: I finished porting and I can release the PR today.  The CRC driver is a little tough, however, so I removed it.  The CRC driver is more difficult
  than I expected and involves switching from a one-call, single-buffer interface to a chunked buffer interface.  I'll submit a PR without that to 
  elicit feedback.
- Amit: Ok.  Maybe someone else can handle the CRC driver.
- Leon: Sure.

## Tock 2.0 Release and Outstanding Issues

- Amit: Now, let's talk about releasing Tock 2.0 and where we stand with it.
- Amit: Here is the tracking issue: https://github.com/tock/tock/issues/2429
- Amit: Here are the outstanding tasks according to the issue:
              - `kernel` crate exports
              - dealing with `tockloader install`
- Leon: The callback-swapping issue is unresolved.  We don't need to solve this prior to releasing 2.0.  However, according to the TRD, it would
  be great to have prior to the release.
- Leon & Amit: "Appslices" is a blocking issue.
- Hudson: How critical is the `tockloader install blink` deal?  I think Brad worked on that and was convinced it was a harder issue than we had 
  previously thought.
- Amit: I don't think it will be a blocking issue for release.  Changing the name of the binary to blink 1.0 or 2.0 and it should work just fine.
- Hudson: I suppose we could check if any app that issues a system call which is the old `memop` number fails.  This is not going to be a fool-proof 
  check, however.  I would be fine with a "dirty hack" like what Amit proposed.
- Leon: We can always extend the attributes we store in an application's header and, as such, I do not think this will be a major issue for release.
- Brad: The issue isn't detecting the application, it is detecting the kernel.  Amit's idea on changing the binary names stored on the Tock website
is good.  We should make binaries work for 2.0 and have an exception for 1.0.
- Amit: We should have a PR for modulo app slices pretty soon.

## Callback-Swapping

- Brad: The basic idea is to store upcalls in the grant region for apps and make handling upcalls entirely the core kernel's responsibility and so 
  capsules would never directly use them.  Capsules would access callbacks the same way they access grants.  The core kernel would mediate access between
  capsules and upcalls.  This would also save code in capsules.  The grant code in the core kernel can store an upcall but needs to dynamically store the 
  number of upcalls and then access them on a path from userspace.  I haven't had time to implement this.
- Leon: This is a move in the right direction.  This implementation might be tricky to get right.  Several months ago,  we established that there should be 
  a guarantee that callbacks never get swapped by the kernel or capsules. Would we rather want something which works now and will be replaced later with a better version because it doesn't change the process-kernel interface at all?  The current status of this PR needs to be rebased and can be removed easily if we were to go with Brad's version.  This PR could be a placeholder while Brad's approach gets implemented.
- Amit: I'm a little confused on where we are.  Is there a concise version of this?
- Leon: Capsules can currently swap upcalls and can copy upcalls which means applications can never rely on an upcall it has unsubscribed from to never
  be called again.  This violates the syscall semantics which are specified in the TRD.  The pr that is going to be out as well as Brad's approach protect 
  againt this but do so via different mechanisms.
- Brad: Agreeing on how we view 2.0 will impact its release.  One view is that we have documentation on guarantees and syscall interface and we can
  implement these changes.  Another view is that we should treat this as a huge verion change and make a huge mess in terms of out-of-tree code.  Then, there
  would be a nice slow-down after releasing 2.0.  You would need to port code to 2.0 once but not a couple months after 2.0, for example.  The problem with a lot of these changes is they are pretty invasive in-kernel changes and so out-of-tree kernel developers would have to update their code if we do this after 2.0.
- Leon: Out-of-tree capsules and boards would need to change to comply with the changes.
- Brad & Hudson: Agreed.
- Amit: Which approach is better? Do we have a consensus?
- Hudson: Brad's approach is better if it can be implemented as we imagine it but it is easier to say the not-yet-implemented version is better because
  it hasn't had to make any concessions that often occur during implementation.
- Phil: We're starting to go down the path of unreasonable expectations of "guarantee".  and argue that the guarantee is based on the premise of the lack
  of bugs and using the rest type system is merely one very convenient way to prevent bugs.
- Amit: A caviat to that is that the kernel does make guarantees in lieu of bugs in capsules.  What guarantees can userland rely on even if capsules are
  "buggy"?
- Leon: The motivation for these protections is that capsules are untrusted kernel code we cannot rely on code review for capsules if the TRD explicitly
  states that callbacks are not swapped.
- Amit: I agree with that.  Capsules could guarantee that even if the kernel doesn't.  Userspace would have to rely on these capsules.
- Amit: An implementation is better than none.  That is my view.  I think we should have a deadline to implement the better verion and, if not, we can
  merge the worse version.
- Hudson: If we miss that deadline, we will have to merge the better version later which might be inconvenient for out-of-tree developers.
- Leon: If we don't implement either version and merge it after 2.0, this will be a major inconvenience.
- Hudson: If we merge one version and then the other, developers would have to change code twice.
- Leon: Either way is fine for me.
- Brad: I think it's relatively easy to implement this based on the prototype.  There might be more complexity with specific Rust details but it is mostly
  straightforward.
- Amit: Could another developer do this in your place?  Is a few weeks a reasonable deadline?
- Brad: Someone like Hudson could do this in 4 hours.  It is fairly simple I think.
- Amit: We will not block on this implementation for 2.0 so we will have a couple weeks for this implementation.
- Amit: We should finish with outstanding issues in the tracking issue and then start creating testing versions.
