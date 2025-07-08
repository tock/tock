Attendees:
 * Pat Pannuto
 * Amit Levy
 * Leon Schuermann
 * Philip Levis
 * Vadim Sukhonmlinov
 * Johnathan Van Why
 * Jett Rink

Updates
 * Johnathan: libtock-rs starting to move again; sent PR with exit system call and tests — tricky as it must also terminate the unit test process, so had to implement some meta process management machinery
 * Johnathan: Also a PR for the subscribe system call—intended audience is experienced Rust devs; I will find reviewers
 * Amit: mini-update, picked up USB stack earlier this week; awesome that it works, but current structure makes it hard to build on; e.g. CDC device must declare that thing which is attached is self-powered; hope is to have an initial re-design draft out in the next few days; will be looking for feedback from folks with more USB experience shortly

Update on AppID
 * Phil: Johnathan and Miguel got me in touch with Felix, who is an integrity/auth expert; conclusion from these conversations is that seems likely that you will need to change signatures, not certain that you will need to add new ones (e.g. resigning)
 * Phil: Will need to be able to modify signatures and credentials post-facto
 * Phil: Experience thus far suggests we will want to move from headers to footers—thus headers are always covered by integrity computatations, but footers are not; this allows to append signatures if you want
 * Phil: This helps tbf parsing code, could build tbf structure in memroy as a bunch of references to flash/app memory, or could create an in-memory representation. Current parsing does the latter. If we have headers, this will require re-parsing to piece out what's covered by integrity
 * Phil: https://github.com/tock/tock/blob/appid/doc/reference/trd-appid.md#4-credentials-in-tock-binary-format-objects
 * Phil: To support footers, now need to know object size
 * Phil: Need exclusively one of Main or Program header; backwards compatible with apps that doesn't care about this
 * Leon: Have you found any ways to deal with the MPU/alignment issues @bradjc raised?
 * Phil: You mean padding?
 * Leon: Yeah, if apps shouldn't read footer data, how do that work with MPU?
 * Phil: Depending on where you place the footers, it is very possible that the MPU will allow read access to the footers; that's not terrible; e.g. apps can read their own headers. If there is a security concern about processes being able to read their own footers, then we will need new layout and indicators. There is a padding footer; could pad to push other credentials forward; but that embeds platform information into the app binary
 * Amit: Unlike with headers, if footers are primarily signatures, it may even be less critical if an app can write. Worst thing an app can do is add a valid signature, which is fine, or corrupt itself (not great, but not a safety concern)
 * Leon: Right, so the forward-looking concern is that this could get really messy with Risc-V and the highly varying PMP implemenations
 * Phil: There are also some ARM platforms that vary MPU
 * Leon: I thought ARM was standard
 * Pat: ARM has a standard, but chips can also roll their own MPU if they want
 * Phil: I will circle back with Miguel and Felix to make sure there is no risk with allowing apps to read their own footer info
 * Phil: Also, note that nothing about this design prevents a future design that prevents footers from being readable
 * Phil: Next steps here will involve starting to actually check the signatures, which will involve playing with the RSA traits and figuring out some of the outstanding issues there

Should Tock allow re-sharing?
 * Jett: If an app has access to some memory (via any means, so may not be in its own app memory / flash), should it be allowed to share non-owned memory with the kernel?
 * Jett: context https://github.com/tock/tock/pull/2875
 * Jett: We have a ZeroCopyBuffer downstream which does this
 * Jett: Could address this via copying, but there's a philosophical question of whether Tock should allow this?
 * Leon: During development of 2.0, we determined that the kernel should not be able to use app-provided buffers in an async manner (across the scheduler loop); it is really hard to think about what should happen when the owner of a buffer dies especially with respect to process resource cleanup
 * Leon: If we take an example of re-sharing the buffer from another app shared by IPC
 * Jett: In the IPC case, App A shares a buffer to B via IPC; while B is processing it dies; restarted B shouldn't have access to A; so problem of cleanup exists 
 * Leon: right, but when talking about IPC, need to recall that our current IPC is not considered a final design; e.g. right now, it's not a memroy handoff, both apps have access to the buffer in parallel
 * Jett: ahh, that's why there's no IPC in Rust userspace..
 * Phil: Why can't we just reverse the current mechanism? i.e. kernel can share something to userspace, which it can then unallow?
 * Jett: That might be similar to what we have. ZeroCopyBuffer resides in kernel SRAM, and use the MPU to give access temporarily and exclusively to the application until the app is done processing the request; and then the access is revoked once app is done
 * Phil: Yeah, you get an upcall with a memory reference that expires once the upcall completes
 * Jett: That's exactly what we have, but you can't allow that memory back to the kernel
 * Jett: Imagine the upcall is a large I2C packet, and then we want to write some of that data to flash during that upcall dispatch with some of the data
 * Phil: Async or sync? Kernel pushes upcall stack frame. You have access to memory as long as the stack frame is alive. If you issue a blocking call to flash, you're okay.
 * Jett: We do a blocking call now
 * Phil: So this stack-based approach should work becuase then we also never have to worry about userspace passing back, etc
 * Jett: You're right that this should work with what we do now; it's just that currently the allow buffer within that stack frame fail
 * Leon: I think these semantics which implicitly rely on app stack frame being very challening to program; imagine a large buffer that an app must chunk
 * Phil: As long as you're blocking it's fine; it's a challenge when doing asynchronously
 * Leon: I think working in chunks is common in Tock
 * Phil: That's okay you can loop
 * Leon: Right, just trying to come up with potential issues
 * Phil: Jett's point of I have this buffer and then I need to share it to the flash driver is about the mechanism
 * Leon: Right, so the operation down into the kernel might be sync, but in the kernel it'll be async
 * Amit: Maybe the stack-based control is a red herring as it's not clear we can enforce that anyway; will need a separate and explict signalling mechanism, and rules for relinquishment / enforcement
 * Jett: The PR does cover some of what would likely be needed to make this bulletproof; hopefully at some point another Googler or intern might be able to pick this up
 * Jett: Might imagine doing something similar to what was done with SyscallDriver, pull the core allow management logic into the kernel itself, so the kernel controls allow-management
 * Jett: I think it can be done, but want to establish philosophically whether it is something Tock wants to do
 * Leon: Still concerned about IPC case; it's an example of possibly complex dependencies; does it work with the Tock goal of mutually distrustful applications; complex topic
 * Amit: Definitely complex; if two processes are talking over IPC, it's probably unreasonable for them to be totally distrustful, they do share state; but do want to understand what we can promise apps
 * Jett: Not wholly familiar with IPC, but inferring there is a share/unshare, but share doesn't cause loss of access; refinig this should be feasible to enable cleanups
 * Phil: I think if this results in something that requires an explicit unshare by an app, it goes against Tock's philosophy; the kernel must have final say on who has access to what memory
 * Phil: Let's step away from IPC; imagine just a big memory pool, e.g. for crypto; can get passed around among processes, just have to ensure limits on fate-sharing
 * Amit: If the kernel were able to revoke access to a buffer arbitrarily, there's no way to ensure that an app won't just bork unexpectedly when buffer disappears; and maybe that's an okay design
 * Jett: Yeah, this is is part of what the stack frame is hoping to protect
 * Phil: Right, but what if an app `loop {}` that? Never gives back buffer
 * Amit: Right, so might need some time-based policy, etc
 * Phil: Right, but then there's no way to gaurentee that a process won't crash
 * Jett/Phil/Amit: There are some limits to the execution model; nothing like a signal, only push upcalls in response to yield, so limited opporunity to notify
 * Phil: One challenge here is what revocation will look like in Rust
 * Jett: At a certain point, if an app is seeming malicious (or erroneous), just shut down the app
 * Leon: That is a very dangerous interface
 * <yeah>
 * Jett: The goal is to make sure an app can't starve / lock a shared resource; eventually have to kill them
 * Amit: If only we had virtual memory
 * <yeah>
 * Leon: Is memory that's re-allowed into the kernel, is that a kernel resource in that timeframe? Could it then re-allow this buffer into yet another application?
 * Jett: Kernel to app is now app-ish memory, so when app allows to kernel, it looks like regular app-allowed memory
 * Leon: And the kernel can't allow the original buffer to another app?
 * Jett: Right, kernel has given up the buffer
 * Phil: Whatever this mechanism is, it should look like IPC; it's shared memory with the kernel
 * Phil: It's likely a success if IPC is this same mechansim
 * Jett: What happens today with IPC out curiousity? If A shares to B, can B share A's region to C?
 * Amit: Belive IPC checks that a share is from app-owned memory
 * Jett: Makes sense
 * Amit: And just that friendly old reminder, that IPC is what it is to get a paper out the door, not because it was actually the right design
 * Leon: One challenge is that most of the UKB is async; so the stack frame mechanism would likely be hard to implement in practice; it might be that use cases for this are very narrow
 * Jett: Agree, that most interfaces are async, but they are logically sync
 * Leon: That's right, but if I get a, say, 500 byte buffer, but flash only accepts 100 byte writes, the interface requires that I yield and go to different stack frame; so monitoring the stack will not work
 * Jett: So, taking your example with 100 byte chunks, this could all work off the same nested dispatch upcall, no?
 * Leon: Oh, yeah, but does that get really hard for the kernel to enforce
 * Jett: Don't think so, once the dispatch stack frame is popped, the buffer is returned to the kernel
 * Phil: Think the takeway is that so long as kernel has final control, there's nothing philosophically opposed, but it's going to be very subtle / tricky to implement
 * Jett: {missed}
 * Phil: If the shared buffer is associated with a particular device ID, and only available to that device ID, then it gets easier
 * Phil: What gets tricky is if it is an unlabeled buffer, and moved around
 * Jett: I think if allow ro/rw moves into the kernel core, then this simplifies
 * Leon: Don't have to do exhaustive searches if it's time-based, because if we decide to forcefully kill the process in case it violates these constraints, we can make use of the regular process cleanup mechanisms (e.g. `ProcessBuffer`s won't be accessible any longer)
 * Jett: That simplifies, then can just rely on time
 * Jett: If we were to look at implementing this, and moving ZCB upstream, there's nothing upstream that uses it; is that okay?
 * Leon: It is nicer if CI, etc can test; is there nothing you could reasonably upstream
 * Jett: It's driven by the dispatcher, if this is useful, could upstream
 * Leon: Think it will be easier to reason about if there is a concrete use case upstream
 * Jett: Okay; thank you everyone for the discussion; this is plenty to think on, will circle back

Multiplexed Serial?
 * Phil: Big mistake if we require an integrated vertical toolchain to talk to a device
 * Phil: Descriptors are super useful however
 * Amit: Temperature check, different impls of console syscall driver with same interface
 * Phil: Why wouldn't it be virtualized?
 * Amit: Instinct is that it will be overly complex; but need to play with this more
 * Amit: I'm starting with USB stack anyway, but long-burn think on this
 * Phil: Sounds good; the big concern is no custom tooling
 * Amit: What about requiring bi-directional, e.g. requiring Ctrl-F7 to pick which stream; knocks out Atmel's unidirection micro UART thing
 * Phil: For me so long as something like screen works it's a probably a good choice
 * Pat: Can we literally use the same interfaces as 70s/80s-era terminals?
 * Amit: Quite possibly yes; will look into this more
