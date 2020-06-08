# Tock Core Notes 06/05/2020

Attending
 - Brad Campbell
 - Alistair
 - Leon Schuermann
 - Pat Pannuto
 - Samuel Jero
 - Johnathan Van Why
 - Philip Levis
 - Vadim Sukhomlinov
 - Branden Ghena
 - Hudson Ayers
 - Garret Kelly
 - Guillaume
 - Andrey Pronin
 - Amit Levy

## Updates
 * Brad: USB serial working! Can read and write over USB virtual serial device.
 * Amit: What is the USB profile?
 * Brad: Communications Device Class Abstract Control Model.
 * Amit: We could conceptually have multiple of those interfaces and support separate consoles per application. That would be neat.
 * Branden: That could work, but we'd be limited to 2-4 at most depending on how many endpoints it takes and are available in hardware.
 * Leon: Porting tock to the stm32h743. It is a pain, but the board is affordable and cheap. With 2 MB of memory.
 * Phil: Also have a student working on nRF53 series. Has a coprocessor for BLE and trustzone-m support!

## Application ID
 * Johnathan: Want to allow multiple application ID types to satisfy different use cases. TBF entry for application ID would have two sets of data. First is the app ID itself and the second is verification data to verify the app ID. For example, if the application ID is a public key, then the additional data would be a signature signed with the private key. Other types of app ID would not need this. Hash of the application binary, for example, or just a number. This becomes complicated because the kernel needs to support multiple options and each needs to have a format and details about how its implemented. Asymmetric crypto for example, we'd have to pick a choice for curve or RSA etc. Two directions include first define all possibilities upstream in the tock repo and the kernel knows about them to be the same across boards. So multiple different boards could support the same choice. A different option is that we could kick it up to the board, and the core kernel wouldn't understand it but the board file would have to do platform-specific verification. But then apps with verification are board specific, so I don't know if this is in Tock's design. Need thoughts on complexity versus portability tradeoff.
 * Amit: Could you give more context on when these names would be checked and what might happen if the check fails?
 * Johnathan: They would be checked between when the TBF headers are read and when the app is started. The data is in the TBF headers, but needs to be checked before we run. If an app has an invalid header, then it should be considered faulty and not launch. If it was defined in the board, we would need to launch faulty apps or else apps wouldn't be portable across boards.
 * Leon: There's a distinction here in that when you are referring to an app ID you're coming from the security perspective. There's also just persistent IDs for consistent access to resources across the board for a specific application. When using public keys for that we may have the issue of having arbitrary, possibly very large formats with a lot of overhead. I think we should have an app ID for resources that is very small, but then additional verification methods on top of that could be executed once when the application loads.
 * Amit: Maybe that's still an independent question. Johnathan is asking not how these are encoded, whether it's the kernel or not, but is it reasonable for the board not the kernel to be responsible and is it also desirable.
 * Brad: I think it is reasonable for the board to be responsible, and it would be a design change for Tock to have the kernel be responsible. Boards just hand buffers to the kernel and say load it. For interoperability, I think most boards will use a simple version of this where apps will just work. For apps that need this though, it seems like they're not going to be portable anyways because they're going to be board-specific.
 * Amit: The current design is that boards are responsible for loading processes. Most boards do that with a kernel utility function, but they could be doing it on their own. We do want shared code for verifying applications, which may or may not live in the kernel crate. Also, if we do have several choices, some boards will just be more restrictive. A board that cares whether the titan team has signed apps, just won't run otherwise portable applications. Boards that aren't restrictive like a dev board should be able to simply not verify and just use whatever identity and take it on faith. So apps for more-restrictive boards should always be portable to less-restrictive boards. But if we did have straight-up different verification strategies that would be bad.
 * Johnathan: If a app has an invalid application ID, the board could just give it a new one. Also, secure boot is a separate question from application IDs.
 * Andrey: Is there already a case where some boards require more strict verification?
 * Johnathan: Not yet, but I expect it for opentitan.
 * Andrey: I think OpenTitan won't need it because OpenTitan knows exactly which applications exist at boot and doesn't have loadable applications.
 * Johnathan: Not all use cases are that way though.
 * Garret: The long-term plan is to eventually be able to address applications like that, but the near-term plan is to use a monolith like Andrey is saying to be able to deliver everything together and know what is there. So just a tag ID is enough without having secure properties with it.
 * Andrey: Yeah, there is a long-term plan, but if it isn't going to be soon, it might be too early to influence the kernel for something that might never happen.
 * Johnathan: I would rather allow the design to be open for many possibilities now to reduce churn.
 * Samuel: One thing missing is what the goal of application IDs are. I assume its for associating with persistent storage. I suggest that you really want the kernel involved in providing the ID such that the process struct has an ID and the kernel is checking it. Maybe the board chooses the ID.
 * Johnathan: I see four use cases. Secure boot, Storage, IPC, and crypto subsystems.
 * Amit: Also access control in the kernel.
 * Johnathan: If you're thinking permissions, such as to load apps or talk to peripherals, yes.
 * Andrey: I'd like to separate the use cases. Open Titan currently expects a set list of applications such that an app knows what index it is. Or a loadable case, where it needs to be proven cryptographically. I wouldn't overcomplicate the first case where we can just do this simply. The second case does need additional steps, which can still be a simple ID, but with a check first. I'm not sure the second case is very well defined yet.
 * Leon: For the first case, we shouldn't overcomplicate it, but we do want the same approach to work for both. Goals are that we can persistently and safely identify the apps, and that the ID is efficient to deal with i.e. that we don't have to verify every time we use it.
 * Amit: It seems like if its possible, that leaving it up to the board would be the best of all worlds.
 * Johnathan: Okay. I'll draft a proposal that leaves it up to the board.
 * Amit: And if it seems possible to do the verification during loading or after loading, then great. And then its simply a matter of, because the identities are central to access control, then the board should be communicating which ID it's assigning to a process.
 * Leon: Going back to Brad's no-one-size-fits-all approach. I think there does need to be a board-specific way, but I would love to see the extensible nature of the headers in the tock binaries used there. If we could all agree on using them, we could define a standard trait that is called on process load by the kernel to verify it first. Returning "valid" or "invalid" based on the binary and the headers. Then we could easily write different approaches if we want in the future.
 * Andrey: Most straightforward approach would be for the application to be signed, right?
 * Vadim: And that could be handled at load time. When the kernel is initialized and the processes are loaded it can verify signatures at that time. If the signature doesn't match then it won't load it. In this case, this check wouldn't be done on every board.
 * Leon: So that was the idea of having a second ID which is used when the crypto verification succeeds.
 * Vadim: It could be part of that signature.
 * Amit: You probably want the verification to be a signature of the hash of the application, which would include some crafted string or number ID.
 * Vadim: Yes. This part could be board-specific. So one board would use RSA or SHA or whatever depending on capabilities. Kernel needs to provide the common mechanism when we create a process we allocate an ID somehow and the kernel should be responsible to that.
 * Leon: Why we might need different verification processes, for example, I could see not every board wanting to implement asymmetric crypto. Just a hash would be enough for some boards.
 * Andrey: Yeah, you just want to check that this was the app you thought it was in some cases.
 * Leon: That is my use case, actually.
 * Johnathan: Someone mentioned a use case that had a fixed-size application ID on the open titan call. Anyone remember that?
 * Vadim: That was me a long time ago. Maybe not originating from me. The idea was that if App ID was 32 bits, then you could compare it in one comparison, whereas a slice of u8 would be more complicated and adds some overhead. It's a performance requirement. u32 should be more than enough.
 * Johnathan: Making it a fixed size might require another hash in the verification step which would be more expensive.
 * Andrey: Why? We're only talking about IDs, right? What verification you do before you assign the ID is tangential.
 * Samuel: I think we need to split verifying the ID from using/handling the ID. The ID has to be used in the kernel, IPC, crates, etc. You need a standardized ID. How you want to verify the ID is board-specific. But the ID seems to strongly need to be standardized in the kernel.
 * Leon: Probably don't want to derive the ID, but have it be statically located in the app.
 * Samuel: Probably. Although that assignment could be board-specific too.
 * Leon: But does that make sense. Having a persistent ID for non-volatile board is useful for every board.
 * Amit: A reasonable way for creating concise IDs at load time though, could just be creating a hash of the app.
 * Andrey: But those app IDs should not be board specific, but application specific. The application decides what IDs they should take and use. Applications as in processes here. Some apps on the board might be loadable but others on the same board could be a static image with a well-known ID. A short, concise ID is the common denominator. How to get from a process image to that ID is board-specific.
 * Leon: That makes sense. So a standard in-kernel format. Probably also a standard way its assigned and used for the standard boards that all do app loading the same way. Not every board needs to develop its own thing if it doesn't have a requirement.
 * Amit: It sounds like a resounding consensus to not make the kernel do this, except for the application ID itself. But how to get them and verify them should be left up to the board, but with a strong understanding that for the moment most common use case will be "no verification". Because things might be done out-of-band with a secure loader, or dev boards that don't care. But we do want to support future use cases.
 * Johnathan: Sounds good.
 
 ## Unallow/Unsubscribe Loophole
 * Amit: https://github.com/tock/tock/issues/1905
 * Guillaume: Credit to Johnathan and Vadim who raised the issue. The subscribe/allow syscalls that give callbacks or slices and can later unallow or unsubscribe with a null pointer. The assumption in libtock-rs is that once you unsubscribe you have ownership again. I think this is going to be important for libtock-rs as its using lambda callback functions. And local buffers for slices. If we can't get back ownership, the lifetime is likely gone. The app might exit the function and the memory may not exist anymore. The kernel could end up corrupting the stack. So it's important that unallow exists. But the Tock kernel doesn't actually enforce that capsules do the right thing and give back ownership. I'll focus on slices but the problem is similar for both. We have to declare clear semantics about who has ownership of a slice. For example, a USB driver that wants to receive packets into userspace, usually you would allow a slice from userspace and subscribe to a callback then call a command to start it. Then the kernel would fill the slice and call the callback. After that userspace can interact with the data. It's important to define who owns the slice at what point in these operations. Because the kernel can't overwrite the slice with the next packet while the userland is interacting with it. So the solution in userland seems to be to allow and disallow slices in a double-buffer sense. But there is nothing enforcing that a capsule properly gives back the memory right now.
 * Johnathan: I'll add that in libtock-rs, we're accounting for the kernel's current behavior with the idea that this isn't going to be a quick fix in the kernel. If I'm wrong, then we should make changes kernel side sooner rather than later, because it will change the libtock-rs design.
 * Leon: I've been stumbling on the same issue when interacting with DMA operations. Knowing who owns what, when, could allow us to reduce the number of copies with DMA operations.
 * Guillaume: I could imagine that, although it isn't used in practice right now.
 * Leon: Yeah. It would be nice to pass that slice around in the kernel.
 * Amit: An important issue is that if we allow hardware resources to touch application memory, then that would prevent us from deallocating application memory if there was a fault, until you can make sure that all of that memory has been returned from various parts of the kernel.
 * Leon: Yes, I agree that there are many issues, but I see ownership as an opportunity to explore in that direction.
 * Guillaume: For libtock-rs: if you consider capsules one-by-one, most capsules behave pretty well. RNG for example, handles disallowing slices correctly. Because the kernel doesn't have safeguards doesn't mean libtock-rs has to be too careful about it.
 * Johnathan: I'm worried about trust model though. I'm not trusting that capsules are well-behaved.
 * Guillaume: I think we have to define behavior carefully.
 * Amit: I think we have in Tock a relatively good notion about which things in capsules are trusted and which aren't, although we may not have written this all down. If you give a capsule memory, you have to trust it with that memory but not with other memory. And today unallow isn't part of that trust model. Because in C you typically don't unallow or unsubscribe because you have static buffers. It's not clear that that's desirable, but that is the notion that currently exists.
 * Guillaume: Capsule trust has been mostly focused on unsafe. But this notion of who owns a slice would be good to formalize.
 * Leon: I'm not sure if the threat model puts it this way, but capsules are untrusted within their bounds, the bounds of Rust limitations. But if we add extra constraints, that would allow us to reduce these bounds and how much we have to trust capsules.
 * Guillaume: That's why I think it's important to redesign this part. So that capsules can only hold on to references while they are needed and not just given away.
 * Amit: I definitely think that we should discuss this more. My interpretation so far of the issue, although I haven't read the proposed solution in depth yet, but it strikes me that there are a set of tradeoffs to drive this decision. One question is whether it is reasonable to have non-C runtimes, particularly Rust, designed in a way that doesn't require the kernel to enforce more things like this. But, how complex is it for the kernel to enforce this instead of it just being a convention. Finally, how much does this hamper the ability for applications to be performant if they do trust the capsule to do the right thing. An example for the Bluetooth case, if I do trust the capsule to not overwrite my buffer until I ask, then I can save a system call and not have to deal with unallowing a buffer.
 * Guillaume: I think there will end up being extra checks or extra syscall overheads. Although, I don't think that the rust runtime is too different from C. Apart from lambda functions on the stack.
 * Leon: It's not that this affects only currently shared buffers. A malicious capsule could collect a bunch of app slices and then conceptually write to several of them at once to do "malicious" things.
 * Amit: If the semantics are that once you've given a buffer to a capsule, you are trusting it for ever, then the malicious behavior isn't possible. But it does require apps to manage that and would be nice not to.
