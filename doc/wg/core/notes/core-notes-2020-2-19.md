# Tock Core Notes 2/19/2021

## Attending
 * Branden Ghena
 * Amit Levy
 * Leon Schuermann
 * Arjun Deopujari
 * Vadim Sukhomlinov
 * Gabriel Marcano
 * Brad Campbell
 * Pat Pannuto
 * Johnathan Van Why


## Updates

### QEMU Submodule
 * Brad: Removed QEMU submodule from Tock, which makes out-of-tree builds with cargo much easier. Cargo by default recursively clones everything, and for some reason fails on QEMU. We replaced it with checks in the Makefile.

### Tock Bootloader
 * Brad: We also have version 1.1.1 of the tock bootloader which works on the Micro:bitv2 board. That was built by someone other than me, which was good to see.
 * Amit: Any particular challenges there, or just pin mapping?
 * Brad: There was a bug in my code where I set a timeout value wrong for receiving bytes, which messed up over a serial connection. Other than that was straightforward.


## Releasing Tock on Crates.io
 * Amit: Now that we've been releasing Tock regularly, I wonder if we should publish them to Crates.io so we don't have to have people rely on git.
 * Leon: I would appreciate that.
 * Amit: We might have to rename the crates if we do so. "kernel" might be taken for instance...
 * Amit: This would also remove the dependency of things we want in the development repository from things people on out-of-tree boards need.
 * Johnathan: I've been prefixing libtock-rs names with `libtock_`
 * Brad: I think it's reasonable. I just wonder if the utility will actually be there. I find that I always need to make some modifications to Tock crates in order to use them. Cycle is that these changes become PRs and get merged, but that relies on git. Eventually there's a new release, but that's months later.


## 2.0 Release Candidate 1 requirements
 * Amit: Brad pointed out three PRs #2430, #2443, #2444

    * https://github.com/tock/tock/pull/2430
    * https://github.com/tock/tock/pull/2443
    * https://github.com/tock/tock/pull/2444

 * Brad: That's 1) the last capsule to switch to new driver 2) removes `successWithValue` 3) removes `legacydriver`. In my mind, those are the minimal set missing to make a release candidate. I don't think these are controversial PRs. I just wanted to see if people know other things that need to be merged into the first PR.
 * Leon: I think we need to go through all the code once more in the kernel crate and check the documentation, especially when there are changes.
 * Brad: That's a good point. Something we could do during the PR review.
 * Amit: That shouldn't block an RC though. It can be done concurrently.
 * Brad: Ehh... Important to do before the merge.
 * Amit: I thought we'd do rc1 in parallel with PR. So rc1 would be that branch. Then there would also be a PR.
 * Leon: We do keep merging master into the 2.0 branch.
 * Brad: I think other things could change in the meantime and it would get confusing.
 * Amit: My guess is there are outstanding bugs which will be hard to catch from just PR review.
 * Leon: I'm wondering what degree of stability the thing that lands on master should have. Right now, I don't trust in a high degree of stability.
 * Johnathan: Right now, we have tested 2.0 on a cortex-m3, but the RISC-V board doesn't even boot.
 * Leon: I've seen similar behavior. Lots of subtle bugs everywhere still. I want to know if it's okay to have this state on master, or if the pre-release should be a tag of the branch for testing before merging into master.
 * Brad: That could be the PR I'm talking about.
 * Amit: We might want to keep PRs to master being changes. And PRs to the branch to be bugfixes.
 * Brad: But master is going to keep advancing and it will be hard to reconcile.
 * Amit: But we do want to avoid merging something into master where a bunch of boards don't work. We don't want to break master for people who aren't doing 2.0 stuff.
 * Amit: I do expect that because 2.0 works on some boards for some drivers, the remaining bugs are hopefully not huge foundational things.
 * Brad: Hard to say.
 * Amit: So what's blocking these PRs?
 * Leon: What's going on with the size impact of the changes?
 * Amit: Brad's change which removed an unused enum variant inexplicably increased kernel size on a bunch of boards. This is really annoying. Hudson looked into it and found that making the `returnCode` enum a byte not a word made a little extra code to handle it in a lot of places since it goes in a register. The solution is just to mark it explicitly as an `isize` which solves the problem and results in a tiny reduction in size as initially expected.
 * Brad: I though `usize` was super bad.
 * Amit: They were the same in my testing.
 * Hudson: Mine too.
 * Amit: Anyways, the original change increased things by like 3 KB. Changing to `isize` reduces by 3.5 KB. Which is to say a net gain of half a kilobyte of space.
 * Hudson: Yes, that's what I found.
 * Amit: It is very frustrating. Maybe a good place for research. The compiler could be smarter here.
 * Amit: Okay, so there are approvals and all of these can be merged. I will do so.
 * Amit: The only other PR I saw was Leon's from a few week's ago adding a platform helper macro.
 * Leon: To explain, I just made a new PR that Johnathan wanted which is a draft of making sure callbacks aren't swapped. It adds a method to the platform trait, which allows the kernel to notify all drivers about events like process initialization. When looking at boards with a lot of peripherals, it's like 15 lines of just invoking drivers, which would have to go in each board's main file which didn't seem elegant. So this PR reduces the size of that and wraps things up. I do see Brad's concern that it complicates some things too.
 * Amit: I haven't looked yet. But in any case, this isn't a blocking issue for 2.0.
 * Leon: Not at all. We can even reopen it later in my new PR if we agree it's needed.
 * Amit: Okay, so these three PRs will be merged. Then we'll merge in master to the tock-2.0 branch. Then we'll open a PR merging tock-2.0 into master. That PR will be our chance to start testing.
 * Hudson: So Leon's new PR could go in after 2.0?
 * Leon: It can't go in after initial 2.0 release. Because there are important ramifications to userspace, especially libtock-rs. But it doesn't have to be in the initial merge to master, just before the release. I know the last PR didn't do enough, but I think this PR is a more uncontroversial change.
 * Hudson: I will take a look.
 * Brad: We can definitely merge it after rc1, but definitely before 2.0 release.


## IPC
 * Amit: https://github.com/tock/tock/issues/1993
 * Amit: IPC is and has always been a hot mess. Porting to 2.0 highlighted some of the issues again, which are longstanding but we've never cared to deal with them since IPC isn't very commonly used. But if we're going to have it, we should have a good interface.
 * Amit: I think thinking about this in the context of libtock-rs would be really useful Johnathan.
 * Amit: To summarize what the interface is now: IPC has a client and a service. Services register callbacks with the IPC driver and allows other processes to share stuff with it and invoke that callback. Clients use the IPC driver to discover services by supplying a package name(the name from the TBF header) and get back a descriptor for the discovered service if it exists. Then clients can share blocks of memory with the service which will modify the MPU config for the service when that service is notified and can invoke the callback by calling notify which is a `command` which enqueues a callback for the service. The callback has a pointer to the shared block of memory and then the service can read and/or modify the block. Then can notify the client back that it's done.

#### Problems with IPC
 * Amit: And that's it. If one process dies, the API doesn't handle that. In fact, this shared memory might just disappear randomly including in the middle of the callback. Which could crash the service due to unpermitted memory access.
 * Amit: Order of loading apps also matters. If a client tries to discover a service that isn't loaded yet, it will fail and there's no way to get a notification when the service does become available. Just keep trying.
 * Amit: There's an issue where the way that memory is shared seems problematic for rust semantics too. It's not passing ownership or borrowing, both processes could modify it concurrently.
 * Amit: There's no notification if sharing memory fails. Due to process death. Not enough grant space on either app. Problem with alignment of memory can happen with MPU mapping. No errors there.
 * Amit: Interface also leaks information about order of process. So there's a lot wrong.
 * Johnathan: If you're sharing memory with another app, how do you make sure that memory lines up with MPU boundaries?
 * Amit: The client has to know the architecture of the hardware you're on and force alignment with GCC align directives.
 * Leon: I thought IPC was a good interface. But enforcing alignment is really really hard to debug. I spent hours on it.
 * Johnathan: That list of problems is important. Because that will be an argument for someone, maybe me, fixing IPC in the future.
 * Amit: There's some exhaustion problem Brad found too.
 * Brad: When you have a grant region, you call .enter, if it's there you enter, if not its created then you enter. Right now with the IPC implementation, if a process asks to notify another process as if it were an IPC server/client. The IPC will call .enter on both processes, causing an IPC grant to be allocated in the "target" process. Which may have not expected to use IPC but now has a kilobyte of grant space allocated from it.
 * Brad: So once we pick a design, we need to be way more careful about leaking things between processes and doing checks.
 * Amit: So this could be avoided with a `tryenter` or something like that. But we would need to do that.
 * Brad: Might not even need that. But some checking, for sure.

#### Back to discussion on IPC
 * Amit: Part of the tricky thing is that we have a bunch of resource limitations that give us less flexibility than might be nice to design this interface. On ARM for example we have a limited number of MPU regions that can be active concurrently. That means that a service, which could in theory be processing calls for multiple processes, will be limited to only accessing memory from some clients. There are 8 MPU regions, 3 are used by process set up. Which means 5 concurrent clients.
 * Leon: Is it set in stone that IPC must be memory sharing?
 * Amit: Nothing is set in stone. We could do message passing with copies.
 * Leon: Not sure if it's better.
 * Branden: Right now we have a bad interface. So message passing would be a lot simpler to make good.
 * Amit: Yes. Sharing is more efficient though.
 * Amit: Right now we don't have a way to say that the kernel shares memory only for the lifetime of the callback. We don't have a mechanism for when the callback returns. If we were passing ownership from client to service, should we block the client process until the service calls notify back and returns ownership? Should the client lose that slice of memory with an MPU rule and just knows it shouldn't access it? It's unclear what would be good.
 * Brad: With the concern that a process could disappear at any time, is there any reliable way to do shared memory?
 * Amit: That's a concern. If we block the client, there's no way for that process to die. Probably.
 * Brad: The kernel could always remove it... How would we know when the server is finished?
 * Amit: That could go in the notify semantics. But it could _never_ notify. Which is why the callback finishing would be a nice signal. But that's inflexible and we don't have a mechanism for it.
 * Amit: One big problem that maybe message passing might help solve is that it's basically impossible to use IPC to mediate between different processes. Suppose you had a communication broker. Or a networking IPC service that two clients happen to be on the same device so it would pretend to go out to the network but actually share with each other. In the shared memory you have to have both client's regions concurrently. Or to be able to ask for access. Because you have to copy bytes from one to the other. In message passing you don't need extra access.
 * Leon: Given that we have preemptively scheduled userspace, what would stop any other process from writing to a region. There would need to be some kind of cooperative handshake which relies on good actors. So I would need to copy the buffer into a local non-accessible buffer anyways to have consistent state.
 * Amit: If the shared buffer semantics are allow clients to write to each other, then I'm copying bytes from a buffer to wherever.
 * Leon: But if an app relies on some state in the buffer, things go wrong.
 * Amit: Right. Which we deal with in the kernel by having a guarantee that the kernel will never get preempted. Looking at a buffer and branching on it without returning guarantees that the buffer can't change. (That's not true for applications)
 * Hudson: For the shared memory support. How would libtock-rs do the shared memory approach? Volatile access like with allowed buffers?
 * Johnathan: Yeah. Annoying but not awful.
 * Amit: Or we could somehow at the API level transfer ownership temporarily.
 * Leon: That would run up against MPU region limits though. My question is how much cost all these mechanism, like MPU configuration have versus just copying buffers in message passing.
 * Amit: MPU configuration is reasonably expensive.
 * Johnathan: Message passing also fits with other kernel APIs better. And ownership. Maybe we could have a copying API for people who do care about performance.
 * Leon: It is just a driver. It doesn't have to be deeply integrated into the kernel.
 * Amit: That's right.
 * Hudson: Is changing the way IPC something after 2.0? I agree that message passing sounds great.
 * Amit: IPC wasn't strongly considered. We picked some interface in an hour for the paper. So this wasn't well thought through. One consideration against IPC was that because you might an unpredictable number of processes using a service, the idea of translating grants into IPC sounds attractive. So the clients should lend memory to the server to use. Message passing would not allow this probably.
 * Leon: A service could block and refuse new operations until the old one is complete. It would just have to refuse receive operations due to lack of memory.
 * Amit: And a network service might have a packet filter for each client process. It can't be blocking because packets arrive whenever. But we maybe can't solve everything for everyone.
 * Amit: To Hudson's question, I think IPC can wait until after 2.0. IPC is not well documented anyways and isn't a stability guarantee. It would be bad to block on this, since it might take a while to think through the interface.
 * Leon: So should we remove IPC from boards by default, if not from the kernel entirely?
 * Amit: Sounds reasonable.
 * Hudson: There are a lot of capsules which aren't stable.
 * Leon: IPC is in the kernel though, which makes it look more serious.
 * Amit: You also build service on top of it, which is frustrating if the interface changes.
 * Amit: Okay. So the notes will have the issues of IPC. I'll take a task to list some requirements for IPC and think about whether message passing can get us there versus shared memory.
 * Amit: I do agree that if message passing works, it's simpler to get right.
 * Hudson: It seems like you could do message passing and still have memory coming from client grants.

