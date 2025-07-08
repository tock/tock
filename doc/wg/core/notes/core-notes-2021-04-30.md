# Tock Core Notes 2021-04-30

Attending:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Gabriel Marcano
- Pat Pannuto
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why
- Alexandru Radovici
- Philip Levis


## Updates

### New participant
 * Amit: Alexandru is joining these calls.
 * Alex: I'm Alex, a professor at polytechnic university in Bucharest, Romania. I'm contributing to Tock because it's fun and I'm using it in my embedded systems classes. I also have a small spinoff company that's sponsoring some students to contributed to Tock. I Tock as a nice change to OS.
 * Alex: Our focus has been the Raspberry Pi Pico working in Tock. We're now at the minimum Tock requirements, although there's still a lot of stuff to do. I'm thinking about how to use multiple cores. Also identifying some bugs, like the I2C error handling PR.

### Callback swap prevention
 * Leon: approach in the PR required us to move the process held state of all non-virtualized capsules into a grant region. A lot of work by Brad, Hudson, and others has helped. I'm going to spend this weekend refining and rebasing before merge.

### Libtock-C Github Actions
 * Hudson: Switched over libtock-c CI to Github actions. Which we use in the Tock kernel and libtock-rs. It seems to be working fine, but keep an eye out for things that work locally but not in CI for a few weeks.
 * Pat: The folks on hardware CI stuff are making a little headway and thinking about config. Hopefully they'll have an example next week. Chatting is going on in slack in the CI channel.
 * Amit: This might be premature, but would the plan be to host a bunch of stuff at UCSD?
 * Pat: The goal is to do the whole federated hardware idea, trying to produce an image for a RPi that you can attach hardware to. So with that idea, I'd support the nRF52 remote runner at UCSD.
 * Amit: Then others of us could host different boards based on the RPi image.
 * Johnathan: How are you dealing with security concerns?
 * Pat: At the moment we are not. It's just running on an isolated RPi on an isolated network. It's out of scope for the undergrads this quarter, but something we could think about.
 * Hudson: Is the concern that someone could submit malicious PRs to run stuff on the device?
 * Johnathan: Yes. There's no way to avoid that.
 * Amit: And we'd have to solve this for Google to someday host OpenTitan, for example.

### Old RFCs
 * Brad: I went through and poked old RFCs to move them to tracking issues. Since the discussion has stopped for many.


## Fixes for AppSlice aliasing
 * Hudson: Leon and I met after last week to lay out solutions. The issue is that today in Tock one soundness bug that exists is that a userspace app could allow two separate overlapping memory regions, and by doing so it makes it possible for a capsule to have two mutable references to the same memory location. That's a big problem in Rust.
 * Hudson: We discussed several solution approaches. One Amit suggested is treating any app-shared memory region as a volatile cell. Either a buffer of `VolatileCell<u8>` or a VolatileSlice. The advantage of this is that there's no need to even worry about overlapping regions anymore. The downside is that we would have to change every use of allow in the kernel as they exist today. We just use normal Rust mechanisms to access the slices today and that would have to change to get/set calls. Also, this could get tricky for HILs that receive buffers with non-static lifetimes. Like the CRC HIL today which can pass a buffer from an app down to the CRC. That works fine today, and would have to change to accept a slice of volatile cells, or separate HILs for from-app rather than from-capsule (which wouldn't be volatile cells).
 * Hudson: Second idea was to have a data structure to track allowed slices and detect overlaps. The real issue here is that anytime a process calls allow on a new AppSlice, you have to iterate all AppSlices allowed for any capsule, not just the using capsule. Which could be quite a lot of overhead. For example, Imix could have 30 AppSlices, which would be hundreds of instructions. Data structure could be sorted, but it's unavoidable that there would be overhead added to each allow.
 * Hudson: That's a basic overview. I want to hear thoughts from everyone.
 * Leon: I'd emphasize that the first approach, volatile writes, we would only need to change synchronous HILs. CRC HIL takes a buffer and returns a result immediately. For asynchronous HILs, you need a buffer with a static lifetime, so you already couldn't pass in AppSlices. So in those cases we'd only have to change the syscall driver.
 * Phil: I suspect that the cost of volatile writes when you're messing with the buffer a lot could be higher than the overhead for checking. Volatile is not cheap. The second one worries me that with the buffer swap semantics of allow, you are likely to be doing lots of allows to swap buffers.
 * Johnathan: They don't have to be volatile. Just an unsafe cell.
 * Leon: Isn't that volatile underneath?
 * Johnathan: No
 * Leon: I did a survey of other OSes, and they stored the raw pointer and just used a volatile read/write whenever accessing the pointer. I'm not sure if the compiler can optimize around that though to enter on the first byte.
 * Johnathan: The Tock kernel shouldn't need a volatile operation, just a memory barrier when transitioning between apps. Unsafe cell should handle this.
 * Amit: If we used Cell, which has unsafe cell under the hood, will that work?
 * Johnathan: Yes, I think so. Unsafe cell is just a magic type that the compiler knows about.
 * Hudson: So to check, the reason this would work is because Tock is single-threaded, having two cells with overlapping memory regions isn't actually unsafe. (yes)
 * Amit: It does still mean that processing, say a packet header, whenever you access the buffer you'll do a memory read, rather than caching values in registers if they are accessed again and again.
 * Johnathan: No. Because the reads don't have to be volatile, it can be cached within a function.
 * Amit: If you have two cells that point to the same value, the compiler can't know that.
 * Johnathan: Right.
 * Amit: You are doing a cell.get(), so you could manually copy stuff to variables. But you would have to do it, not the compiler. If you do the naive thing, it would effectively do memory reads everywhere. It wouldn't insert memory barriers at least.
 * Johnathan: That's if you're parsing one buffer and writing to another that could overlap. In that case you'd have to be careful because the compiler won't for you.
 * Leon: Reading through the docs, I don't think unsafe protects against aliasing structs.
 * Amit: That's the point of cell, that it allows shared mutability.
 * Johnathan: This might be obvious, but if we go the route of cells, then this solution could also be the solution for DMA from app-provided buffers. Which could be a two-birds-with-one-stone kind of solution.
 * Amit: I think it's the case that lifetimes still make that tricky. We've got to require that buffers have a static lifetime, but processes can't provide that because they could be stopped.
 * Leon: This also plays with when an app could get its buffer back. The hardware would have to give it up somehow. There's a lot of work between where we are and DMA with userspace buffers.
 * Amit: Looking at how a relatively expensive allow/unallow would work, it might not be a big deal for the common case, which is allowing once and using it a bunch of times. In practice though, the synchronous APIs on top of asynchronous APIs effectively allow/unallow on each call. We've got an assumption everywhere that allow and subscribe are pretty cheap.
 * Johnathan: And we can't fix that in Tock 2.0, which forces userspace to allow/unallow frequently.
 * Phil: Do we have numbers? Both methods have a cost, and it would be good to quantify them. Sometimes compilers and libraries surprise you.
 * Hudson: We don't have numbers. I think the second one depends heavily on log(n) of the number of AppSlices allowed.
 * Johnathan: If we don't check it and use a cell approach, apps could come to rely on sharing overlapping buffers. Then swapping later would break those apps. Where going the other way would be unlikely to break apps.
 * Leon: I think we might have overlapping buffers as a common case, where one capsule receives a packet and then that data is shared with another capsule, for example ethernet or uart.
 * Phil: My intuition is to go with number 2 so that there's a clear solution: make the check as quick as possible. That seems tractable to do.
 * Hudson: Even if we make it as fast as we can, there's going to be a fundamental limit.
 * Phil: True, but a logarithmic tree search has the sorting question, but seems straightforward at least. I'd want to see the numbers though. I could totally be wrong about which way will win.
 * Amit: I think the networking stack could be a decent litmus test for both. How reasonable would it be to benchmark on that?
 * Leon: I imagine there will be different drivers looking at both ways. Console will use two allow operations each time.
 * Amit: Actually, I think printf is non-blocking. Right?
 * Leon: No, we can't modify the buffer while it's in flight.
 * Gabe: printf definitely blocks. There is another put that is asynchronous.
 * Amit: So if this is a question of performance, do we have a path to benchmarking?
 * Hudson: Benchmarking the second is totally reasonable. Gives a cost for allow/unallow based on number of total AppSlices.
 * Hudson: The first is tricky. The networking stack is probably a fine stress test.
 * Brad: Is there an issue that describes the two options?
 * Hudson: Just a doc for now: https://docs.google.com/document/d/1PoPjnKX3tMBtPEd7xi_O7Ff80UoK2uiWVg3UnfVV48g/edit
 * Leon: If we go with the first approach, I'm wondering if it will lead to behavior that we don't want
 * Johnathan: I think we have a reasonable argument to go with option 2 for now, and if someone actually wants to optimize the kernel, they could argue for the first option.
 * Phil: We should at least check that the overhead is reasonable first.
 * Amit: In practice, there's not really good use cases for sharing overlapping buffers currently?
 * Leon: I think there is, with the network stack for example. Maybe others will disagree.
 * Vadim: There's an example where I use crypto on a buffer and then pass that data to something else. Technically how we do it has many allow calls anyways. So it wouldn't hurt much if we couldn't have overlapping buffers.
 * Johnathan: Right now, I could see overlapping read buffers. Overlapping write buffers I don't see much use for.
 * Amit: Does that help in any way? That it's read-only?
 * Hudson: I think it's fine to use overlapping buffers in allow read-only
 * Vadim: Read-only allow buffers can come from flash, right? (yes)
 * Leon: Read-only allow would maybe make console easier.
 * Johnathan: So we only need to watch out for mutable allows.
 * Leon: No. We need to watch out for aliasing between one mutable and one read-only allow
 * Amit: It had seemed intuitively that there should be use cases where sharing buffer concurrently would be good performance-wise. And it should be possible theoretically. And it sucks to have a restriction just because it's gross to make the Rust types work. But if the only cases are read-only allow, then we can just make that case work and disallow mutable aliasing.
 * Leon: If we were to go with the first option, then being able to mutate a buffer wouldn't really ensure that read-only buffers are consistent. Since they could overlap the same memory location. We can't just pick handling a single case.
 * Alex: Is it a problem if the app shares buffer with the same driver, or still an issue if it has them with different drivers? An app is unlikely to share the same buffer twice with a single driver, but maybe with two different drivers.
 * Amit: I think the problem exists in both cases. Because ultimately the issue is having two mutable slices that are pointing at overlapping memory.
 * Leon: I think it's the likely case that an app sharing overlapping buffers would do so with two drivers
 * Alex: If it allows them with the same driver, I see the issue. Is it a problem if different drivers have the buffer? If not, the search space is smaller.
 * Amit: I think it's a problem in both cases.
 * Hudson: Yes. Drivers can directly call each other, so that would be an issue still.
 * Johnathan: It makes the data structure design interesting though, if you're putting it in grants. You either have to split it up or have a max number of allows. Or put it in grant, but not a capsule-specific part of the grant.
 * Hudson: I think the soundness issue remains that it's undefined behavior to have two buffers pointing at one location.
 * Leon: Just guaranteeing that two drivers don't overlap would be confusing on its own.
 * Amit: Okay, so we should get some numbers to make a better decision here. It think choosing one and seeing how it goes is fine, as long as we can reject it if there's too high of overhead or too large of restrictions.
 * Leon: I think either case as a proof of concept could be a lot of work. Might be days-to-weeks.
 * Hudson: We could do the first with just one capsule to measure overhead. Then the second we could implement generally.

## Kernel organization discussion
 * Brad: PR 2551 https://github.com/tock/tock/pull/2551
 * Brad: Take a look. We want to agree on something, make changes quickly, and then not think about this for a while.
 * Amit: We'll put this on agenda for next week if it doesn't resolve itself by then.
