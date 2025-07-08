# Tock Core Notes 12/10/2021

Attendees
 * Branden Ghena
 * Leon Schuermann
 * Alexandru Radovici
 * Johnathan Van Why
 * Phil Levis
 * Jett Rink
 * Pat Pannuto
 * Hudson Ayers
 * Amit Levy
 * Brad Campbell
 * Vadim Sukhomlinov


# Updates
 * Phil: Two updates. 1) there was some discussion about use cases for things like OTBN and accelerators. As background, OTBN has a data memory and code memory space. When you want to do a big number operation, you load code and data and trigger the action. In some cases you load code once at boot and it's static. But there's a question whether that's the expected use case or if code will be dynamically loaded at runtime. I check in with Jade and Arun and they thing dynamic is an important use case. The number of instructions to possibly accelerate is larger than memory. Important for abstractions.
 * Phil: 2) We're good with the UART TRD, so I'm going to work on that over the holidays.
 * Alexandru: Regarding WiFi, we managed to scan networks. It works on the RPI board and we pushed a PR we want feedback on. We renamed the HIL to be chip-specific for now, since there's just one chip supported. For now it just scans networks. https://github.com/tock/tock/pull/2625
 * Jett: Moving allow read only and allow read/write into core kernel. PR is failing some CI tests, but we're looking into why. We had to submit a change to libtock-c, which we did. But I think that will break CI for everything right now. It's the LiteX simulation. So FYI, that might be unreliable until my PR lands. https://github.com/tock/tock/pull/2906
 * Leon: I think this is a proof of the value of LiteX, that we had a breaking change and it's signaling that it's broken.
 * Hudson: To give some background, the CI uses a pinned version of libtock-c, so it won't break everything. I updated your PR to move CI to a new pinned commit. It still fails, so that might indeed be a real bug. Definitely needs looking into.


# Userspace Readable Allow
 * Jett: Continuation of my update. With the PR getting close to landing, there are some cleanups. We can remove the default impl for the process buffers and we can remove an option that the process points to and make it a straight process ID. They're blocked because of the userspace allowable buffer. It's effectively the same as the allowed read/write buffer and it's type aliased to it. So, I only moved the allow read/write and read-only, not the userspace one. So I can't make these cleanups as the code is. Wondering if this is something we can simplify.
 * Jett: Right now, this looks like an API contract thing. When the userspace shares a buffer, it can't assume that it can access the buffer anymore. The kernel can't assume that the userspace isn't looking at it. That's still the contract though. The userspace share explicitly allows the userspace process to access the buffer while shared, but they're treated the same on the kernel side.
 * Jett: Looking at the way things are implemented, I don't know if the kernel will ever be able to take advantages of the differences here. The kernel has to assume that things could be modified, even if the application promises not to. The only exception would be if there was hardware protection, in which case the kernel could relax this. But all of our hardware will never have this advance MPU stuff. So the lowest common denominator would be assuming that the process could touch the memory, since we can't rely on that hardware everywhere.
 * Jett: It seems like this is something we could potentially drop and make the API documentation for read/write allow. If we can't just drop it, then maybe we could do something for the read/write allow and use the top bit of the length to determine if it's a shared userspace readable buffer. So this concept is now something different that we need to discuss. Multiple ways forward are 1) drop the concept and change the docs or 2) can we encode the concept in a different way? Then in either case, dropping it allows a big process cleanup.
 * Phil: I didn't 100% follow everything. You're asking if we can unify read/write allow with userspace readable allow?
 * Jett: Yeah, can we drop the separate userspace version of it? We could say in docs, that processes could access the data inside a read/write allow.
 * Phil: The intention had been that the kernel could revoke access to something you allowed, say through an MPU. It doesn't today and isn't feasible on platforms, but it could and the userspace should assume that it does.
 * Jett: But every Tock isn't going to have that feature, so the kernel MUST assume that it doesn't have the protection.
 * Phil: That's true, but the next step of your argument is that because the kernel MUST assume this, we should say that all buffers are readable?
 * Jett: I think everything you pass to the kernel with a read/write allow, you should be able to read.
 * Phil: That's not the semantics today.
 * Jett: Right. I'm saying we merge the two semantics and change the docs to say this is how read/write allow works.
 * Phil: But it would mean that kernel could NEVER protect those buffers.
 * Jett: It could protect against write, but not read.
 * Phil: Is the motivation implementation simplicity?
 * Jett: One aspect is cleaning up code. But it's also good to simplify the API.
 * Phil: Barring extreme cases, it should be semantics first and not implementation first.
 * Phil: And you're right that it simplifies the API, but in making a more general syscall that has more options.
 * Jett: I think removing the third option of allowing makes things less confusing. The userspace doesn't have to use new functionality or change their code, but can use new features if it wants.
 * Phil: But there will no longer be the possibility of a syscall where you pass a buffer and then lose access to it.
 * Jett: Do we need that feature?
 * Phil: It's the general assumed behavior. When you pass a buffer you lose ownership until it's returned to you. You can't touch the buffer while the call is outstanding.
 * Jett: Indeed libtock-rs wouldn't be able to take advantage of this bonus semantic. But other languages could.
 * Johnathan: With the current design, it can do both. Other designs wouldn't do both under the same system call number though.
 * Phil: It seems to me that the idea of changing semantics of memory sharing to simplify some implementation is bad.
 * Jett: I think the backstory isn't the only point here. The API view is also meaningful. I didn't see big use cases for this option, although maybe it will be usable in the future.
 * Phil: The userspace readable buffer was the special case. That's what we thought of as a narrow case. Generalizing it to everything seems weird.
 * Jett: Will that restriction be useful though?
 * Hudson: It seems to me that actual implementations are unlikely to ever diverge. Until every platform Tock supports can protect the buffers, we can't take advantage of it. And if we had that feature, we might change many other API things too. So just having two calls with different docs to support a hypothetical future doesn't seem useful.
 * Leon: But some platform might have the capability. And if we change the call, then we lose that capability.
 * Brad: I want to push back on the idea that we can't implement this. We're hamstrung by IPC right now, which limits our MPU. If we change IPC, then we get enough MPU regions that we could implement faults on access to allowed buffers.
 * Leon: And FPGA implementations could trivially have 64 MPU regions.
 * Jett: Okay, so it sounds like we want to keep the concept. We could still change how we express the concept, where it's a flag instead of a syscall number. That might make the API more complicated, but would allow some kernel cleanup.
 * Phil: Lots of system calls isn't necessarily more complex if they're simple.
 * Jett: This PR moved stuff into the core kernel so the grant manages it. There's machinery in code to handle it. If we wanted to make the userspace call, we would have to also make that code handle that as well. That could potentially lead to another usize usage, even if you don't use this concept. We'd have to expand it for everyone. So all three calls are similar. Or we can leave it how it is and have it be separate and be this thing that's in your "type T" interface and still be on the syscall driver trait. It seems that all three should be handled similarly though, since they're similar concepts. Maybe that's not something that we want to do.
 * Leon: It sounds to me like we're trying to force three concepts into the same system call.
 * Jett: No read-only allow would be its own system call, for sure.
 * Brad: I'm sympathetic and agree that the current implementation is problematic because for every grant region we have to store the number of upcalls, the number of read-only allows, number of read-write allows, number of readable allows, even if they're all just zero. But we're never going to have 4 billion allows. Should we just divvy up the number of allows into different bit widths and just have one value?
 * Phil: 256 seems like a great number
 * Jett: I thought that could be a good optimization. I think that could work
 * Hudson: It does mean that every capsule has to specify the number of userspace readable allows it has. It felt, when we added it, that it was zero cost. Now it's starting to feel like a cost even though only one upstream platform uses it.
 * Phil: I'm less concerned about the grant region allocation than I am about code size. We can compress those numbers. Kernel code size, especially for specific use cases, matters.
 * Brad: The code size might go down if we improve read-only and read-write allows and not having to duplicate machinery to support userspace readable allow.
 * Jett: It sounds like we want to keep the concept. So I'll think other changes.


# TRD for App Completion
 * Hudson: Brad had a decent comment I wanted to discuss here https://github.com/tock/tock/pull/2914
 * Brad: This would be a new TRD that would have some context and structure for app completion codes: a 32-bit number the process provides to the exit code that the kernel will store on behalf of the process. Right now we have no description about what the code means. So the proposal reserves regions for certain semantically meaningful completion codes.
 * Phil: One question I have is "why SHOULD".
 * Brad: I think the intention is for this TRD to be advisory in nature. It's not that the kernel will necessarily act on these values, which it could if they were MUST. The kernel wouldn't change control flow or act on the value based on its value. Just hold it.
 * Phil: You're saying the kernel shouldn't act upon it, leave it flexible to apps.
 * Brad: That's my understanding
 * Phil: Seems reasonable to me.
 * Brad: My comment was that this was about specifying process behavior, not kernel behavior. So should this doc be in the kernel at all? Especially since we don't say anything about how the kernel uses it.
 * Brad: Also the exit call already has a flag about exit terminate versus exit restart. And this doesn't say how it interacts with that.
 * Branden: Where else would you put it?
 * Brad: It could go in the libtock-rs or libtock-c communities. Constants in the code.
 * Pat: But I'm hesitant in encouraging differentiation between runtimes. Ecosystem should be faithful across languages.
 * Phil: But that's not what happens in posix. What matters is what it's codes are, not the language.
 * Leon: So the hesitation is that it's not the kernel, just a central authority. So this would define how applications act with respond to each other.
 * Phil: I like that this is a SHOULD. If you have no idea what to return, here's a default. This is a good approach. But if you've got your own ideas, you're welcome to. It's good to give guidance, as long as the kernel makes no assumptions.
 * Brad: Yeah. That's a good argument. Then in my opinion this TRD needs to go further and give more options. I could see that being useful.
 * Phil: More than just saying you "should do this"?
 * Brad: I think your example of writing a library and crashing is good. So you need to give a number. Not zero, but what?
 * Phil: It's tricky, because any level of the stack could exit, and somehow they need to align. If main uses some values and libtock-c uses others, how do you reconcile that?
 * Brad: I think we're agreeing! The TRD, as-is, doesn't actually provide a number though. Which the implementer needs rather than range.
 * Branden: On the documentation standpoint, I think ecosystem goes in Tock repo, even if it's not binding on the kernel.
 * Leon: I'm worried that the kernel will eventually make assumptions on these and make use of them. So the isolation isn't as clear. Fault handlers want to give info.
 * Phil: Right, so will the kernel try to infer a string from this?
 * Brad: I think that's problematic, but we just have to be rigorous about things the kernel relies on being part of the exit docs, not this advisory doc.
 * Phil: We could say "if the error condition aligns with an error code, then it should be a TRD104 error code, otherwise it should be something ELSE"
 * Leon: But the kernel might do something specific for some of these.
 * Phil: We can be rigorous about it being a SHOULD.
 * Leon: So it only makes sense in that other applications can use it? I'm worried that implicitly we'll do something in the kernel based on it.
 * Phil: So don't do that :)
 * Amit: So the proposed TRD seems to me to do as good of a job as it can to avoid that
 * Brad: I think it's missing a little that we discussed here. Motivation and expectations. But then it would be good.


