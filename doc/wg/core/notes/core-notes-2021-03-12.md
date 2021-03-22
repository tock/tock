# Tock Core Notes 2021-03-12

## Attending
 * Alistair
 * Brad Campbell
 * Gabe Marcano
 * Hudson Ayers
 * Johnathan Van Why
 * Leon Schuermann
 * Pat Pannuto
 * Philip Levis
 * Branden Ghena
 * Max
 * Grant Jiang
 * Vadim Sukhomlinov


## Updates
 * Phil: Tock 2.0 alpha1 is almost there. Comments outstanding from Pat on syscall TRD, which should be handled on that PR after the merge. There's a technical concern Brad has about the new allow systemcall trait and the passing of a closure. We should discuss that if possible.

 * Brad: All of LLVM ASM is gone from Tock and we're now just using the hopefully standards-tracked `asm!` feature. Good for the push towards `stable` Tock.


## Benchmarking Results
 * https://github.com/Pumuckl007/TockBenchmarking
 * https://github.com/Pumuckl007/TockBenchmarking/blob/main/reports/final.pdf
 * (Also saved at: https://github.com/tock/TockBenchmarking)
 * Gabe: I have a writeup, which was for class so is a bit overdone. Max also worked on it.
 * Phil: Can you link us to it?
 * Gabe: Right now it's in a private repo, which we're working to make public. And will email out soon
 * Gabe: We did testing on an FPGA with bitstream for OpenTitan from Alistair. The biggest changes we made to Tock were adding IPC to the earlgray board support file and we modified the number of memory regions available because the CPU supports up to 16 physical regions. We needed those regions for IPC for memory sharing.
 * Gabe: We wanted to do benchmarking in Rust and C, but we ended up just focusing on C. Especially since libtock-rs doesn't have IPC support.
 * Gabe: Three areas: CPU operations including syscalls, memory operations, IPC operations. I think our CPU and IPC tests are decent. We ran into problems with memory because memory operations are so fast. The benchmarking overhead is on the same order magnitude as the tests.
 * Gabe: Also for the commit of OpenTitan, there's a 4 kB i-cache, there's also a writeback stage in their CPU pipeline, which ibex actually doesn't document and made clear that their 3-stage pipeline is presently undocumented.
 * Gabe: There's a table at the very end of this doc with data at the very end. Generally, multiply microseconds by 10 to get cycles since we're running at 10 MHz.
 * Gabe: Another thing I added to Tock was a perf capsule to expose counters to userland. It's not great and doesn't follow a HIL, but it explicitly read/writes to CSR registers to read performance counters and return to userland via systemcalls. In the future, we should improve and generalize this. A future PR.
 * Gabe: One thing that stands out is system calls. Very long. 4300 cycles for a system call.
 * Phil: But there aren't quite enough instructions to account for the cycles.
 * Gabe: Integer instructions are single cycle. Branches and loads/stores are multi-cycle. I think that accounts for the differential.
 * Phil: But SRAM is one cycle in a different table here.
 * Gabe: I think the latency is on top of the SRAM delay. It looks like each load/store takes about 3 cycles to complete. 1 cycle to execute the function and 2 cycles of latency is our interpretation.
 * Gabe: This is a 3-stage pipeline, but the execute stage can take an arbitrary number of cycles depending on the instruction.
 * Gabe: We used perf capsule to measure syscalls. It is a syscall, so it was a fine thing to measure overhead for. About 450 microseconds, 4500 cycles, or 1690 instructions. We expected about 200 instructions for kernel dispatching and switching. We were very surprised to get to 1600.
 * Phil: You spill all registers too.
 * Gabe: That's 100-200 instructions. But way lower than what we expected. This jumped out at us the most because it was the biggest difference from estimate to measurement.
 * Gabe: Measurements were amazingly stable. Almost 0 standard deviation even for 32-1000 samples. In theory there's an icache, but it didn't act randomly. It's possible it wasn't even active.
 * Phil: I'm not so surprised since there's only one core.
 * Max: 1700 instructions is 2 syscalls! So 844 for one syscall. So less than what was just said. Still a lot.
 * Phil: For a course, difference between estimate and real is important. For our case, we do care most about what the actual number is. My sense is that spilling registers takes a lot, but a NULL syscall I would hope to be 100-200 cycles. So this is definitely something to look at.
 * Gabe: We also measured other things, which mostly hit expectations.
 * Gabe: Unoptimized procedure calls were about as expected except that that returns are faster than no-ops. Section 4.3.1 from paper.
 * Gabe: We looked at an unrolled loop and get bandwidth for CPU copy operation. Memory I/O is ~14 MB/s (that's Byte per second). That doesn't totally match up to our latency, so it's something to look into.
 * Gabe: The IPC stuff is interesting. We did tests like pinging back and forth. We also did how long it took to copy buffers over and find a service or fail to find it. That's in 4.3 of the paper. The numbers matched expectation in that there is a good bit of overhead. The direction of the timing mattered. Pat pointed out that the earlgray is running a priority scheduler based on load order, so the ordering of yields versus notifies could mean one way would load faster than the other. Client to server with server as first application is 1.9 ms (1900 cycles). Server to client was 2400 cycles, maybe due to scheduling. Table 11 in the paper. Finding a service takes longer than failing to find it, likely due to configuration on finding a service.
 * Gabe: That's most of the interesting bits. We wanted to look at flash performance, but there's a bug in opentitan where the flash controller doesn't update the status registers on interrupts. So we weren't able to test the nonvolatile capsule and overhead for read/write to flash. So we read from flash via memory mapped regions, but didn't notice it being slower than SRAM.
 * Gabe: We tried doing things to stress icache too, changing code size, but we were not able to see any changes. Unsure if methodology or something else going on.
 * Gabe: Most useful parts are measurements and discussion and the conclusion. We are planning on releasing all of the source code and the measurements we took. I'll send a message on the mailing list.
 * Max: Updating an MPU takes about 2x a syscall. Section 4.2.3. The overhead is much longer than just writing to registers. This is testing sbrk.
 * Phil: Yeah, there's a big sort here. We were thinking about whether we wanted to keep that.
 * Alistair: We actually removed that recently. We removed all of the sorting. There's a new one in the last few days that should change this behavior. https://github.com/tock/tock/pull/2420
 * Phil: Stepping up a level, what we care about the most is looking into syscall overhead.
 * Hudson: There are some ideas there.
 * Phil: And I know Vadim is very aware of these issues.
 * Vadim: Syscall performance and where it's spent. For a minimal syscall, you can go below 80 cycles, which is mostly context storing. The switching is not a big deal. I also found that typical syscall processing in Tock drivers is expensive due to idiomatic rust code like closures that capture context and is inefficient. Like a map of map, which can be a significant number of instructions. Plenty of things to look at. It's mostly what happens after the switch.
 * Phil: The really key one is command. Commands should be light weight. I was chatting with Amit about certain Rust idioms that lead to unforseen or unpredictable code size or efficiency hits. Depends on the compiler, which means little nightly changes can change things a lot. C has all of its problems, but it maps to assembly simply making it easy to reason about. Rust can do that too with good tooling, but we're not there yet.
 * Vadim: It's not Rust, it's how you use it. Some embedded guidelines could help here and explain issues you could face with different constructions. Mutable closure with mutable parameters is much more expensive.
 * Leon: We do have other tools. Alistair has a new verilator board, which is pretty easy to simulate various timings. That gives us options to simulate behaviors.
 * Phil: One of the conclusions I've reached is that our code size tool needs to be much much better. So we need to think about methodology there. In C basically the only thing is no big arguments on the stack. With Rust there are many of these things. But the challenge is if it's compiler instance specific. We'll see.
 * Gabe: In terms of compiler, we specified which nightly we're on. It would be good to compare against other things. It wouldn't surprise me for changes to exist. Even as we were testing, some of the ASM stuff standardized.
 * Max: ASM is standardized now I think.
 * Gabe: Yeah. So those changes will affect things.

### ChromeOS
 * Johnathan: They're improving kernel fault debugging. Kernel faults usually panic in Tock and panic prints out kBs of text. They don't want the print to happen immediately, but instead of the information to go into flash. Then a reboot, which might not even by the same version of the kernel, should unpack and print. Putting all the text into flash isn't practical due to size.
 * Johnathan: So they want a struct with all information printed during a panic as a `repr(c)` struct. Then the fault handler could retrieve that struct and store it before resetting the system. Does that seem like a reasonable design?
 * Leon: Those approaches have two issues. Versioning with different kernels and different internal representations. The second is how you would in a C-style struct represent things that are dynamic across boards. Some boards might have 2 elements and some might have 3.
 * Alistair: I was thinking that it would be nice for the debug to be more board specific. All the no-ops are really taking a lot of time on verilator, for instance.
 * Leon: I'm not against a canonical representation of the board state. I think it's a great idea. I just think having the single format will be quite tricky.
 * Johnathan: So maybe the struct is board specific.
 * Phil: Is there no reason to not always do it to flash and then retrieve from flash?
 * Leon: I don't want my flash to wear out for testing boards.
 * Phil: There are ways to prevent that, but fair.
 * Brad: Panic is board specific. We can add new helper functions to avoid long delays. Right now the process, architecture, etc. have a function that gets called when panic runs. One way to implement this is to have a UART writer and a buffer function. And you could choose which path to call. Something can be responsible for these buffers. Then that implementation could be pretty static.
 * Johnathan: So the implementation he wrote up writes into an array of usize which is the API. So that all seems reasonable. The versioning issue needs to be dealt with, pushing things to the board. Maybe there's a struct and each board picks what goes in flash? I'm not a fan of writing to flash every time.
 * Johnathan: That's all helpful feedback. I can move forward here.

### Tock 2.0 syscall API
 * Phil: Major remaining technical question Brad had. Let's see if it's quick to resolve.
 * Phil: https://github.com/tock/tock/pull/2446
 * Brad: How does the core scheduler and process.rs interact to implement a syscall. In this case the allow syscall. Right now scheduler calls process and in that call the scheduler passes a closure. The process sets up the allow buffer and handles the closure which calls the capsule which needs the buffer. This stood out to me because passing a closure in this way is not how the other APIs work and it adds complexity. My question is why we don't ask the process for an allow buffer instead. Then all the logic about semantics of the syscall are entirely handled in scheduler rather than split between scheduler and process. Right now we only have one process implementation, but we do want to allow for more than one implementation.
 * Brad:  https://github.com/tock/tock/pull/2446#discussion_r590415779
 * Phil: But if they want to check swap semantics differently, then processes can do that. Instead of having the scheduler determine.
 * Leon: I think I was the biggest proponent of this change. Really what the Tock 2.0 ABI emphasizes is that appslices are process resources, and it's difficult to keep track of where we allocate memory. We previously had overlapping memory regions, which is undefined behavior. Process.rs was the barrier for creating a new reference to an app in the kernel. So the process was the one thing responsible for allocating resources from the process. And either this is successful and the driver is called and can check the swap semantics. So that was the motivation. You've already convinced me to change it back and just do all of the things interacting with the systemcall in the scheduler. So the process wouldn't call the driver. That's not as clean, but it also resolves some of the issues with this approach. In summary, I think both aren't great, but I understand your reasons and intend to switch back.
 * Phil: I prefer the split closure approach. Some decomposition is good since scheduler is getting big. Maybe this isn't the best way to do this. I'm fine with the old way too.
 * Leon: There are two things to look at: the impact on performance, and the cohesion between those files which will happen when we stop aliasing. So there will be even more interactions that may get complicated.
 * Brad: I do agree that it would help to have that TODO block implemented. I think overall I would more convinced by the closure argument if the app slice was returned, but it gets stored in the capsule instead. So process memory is leaving process.rs.
 * Leon: One crucial issue is that we're enforcing that a capsule returns an appslice. When the scheduler does it, we temporarily have two appslices lying around. In this case, it creates and consumes one at the same time.
 * Brad: If all implementations do that. Which can be tricky.
 * Leon: I'll look into it tomorrow. Definitely want to measure performance first.


