# Tock Meeting Notes 2025-12-18

## Attendees
 - Brad Campbell
 - Branden Ghena
 - Johnathan Van Why
 - Leon Schuermann
 - Amit Levy

## Updates
### Network WG
 * Branden: Network working group talked about the IPC RFC PR.
 * Johnathan: Provided some feedback on IPC. I'll join the call to hash it out verbally
 * Branden: Thank you!
 * Branden: Network working group also discussed, WiFi, which will be merged very soon.
 * Branden: Also, Tyler has put work into 15.4 and Thread tests, which he's mostly gotten working in hardware CI. This includes a new libtock-rs test for raw 15.4, which is still an open PR.


## Moving Schedulers Out of Kernel
 * https://github.com/tock/tock/pull/4640
 * Brad: Moves schedulers to the Systems capsules crate. Makes it a forbid-unsafe capsule crate. Requires some refactoring to remove a use of unsafe, and it's an opportunity to update board definitions with Type aliases for picking schedulers.
 * Brad: I think this is ready to go, but I wanted to check in
 * Amit: Okay, this has one approval from me for a while ago. Needs a second approval for anyone to hit the merge button
 * Branden: I'm still confused about what's going on with the discussion of capabilities and components. I don't understand the discussion.
 * Brad: That's relevant to keeping unsafe out of components.
 * Leon: Actually, this is different. We're adding the ProcessManagementCapability trait on a struct, but that struct is safe to construct. So visibility is the only thing keeping people from making instances of this struct. It needs an unsafe constructor so that making instances of it is unsafe. So what I was suggesting was a macro that makes the struct for you and makes it unsafe. Then we can use the macro and not have an issue
 * Brad: It's no more wrong than what we do in other places though. Really should be a separate refactor
 * Leon: Agreed.
 * Brad: I'm not using the create capability macro, because we need the name of the struct to pass into the macro.


## Refactoring Panic Writes
 * https://github.com/tock/tock/pull/4684
 * Brad: Part of getting rid of static muts everywhere. I've been porting all boards to use SingleThreadValue. One big issue is having panic handlers, which do interesting things like printing the state of the system. That was all moved to SingleThreadValue. But in doing that, I realized that io.rs implementations use static mut all over the place in interesting and creative ways.
 * Brad: So this new PR is a proposed approach to handling static muts in io.rs. The reason those exist at all is that the handler needs a synchronous writer for printing error messages. Lots of copy-pasting with tweaks. Roughly 50 slightly different implementations of this.
 * Brad: The proposal Leon, Amit, and I talked about was adding a PanicWriter trait that the kernel defines and chips implement. UART would implement this and give you a writer object that can do a synchronous write of a panic message. Doesn't have to be UART, it could be anything. We could remove this from board authors, and keep this on a chip basis which lets us be more careful about state and share the code better.
 * Brad: For now, I implemented this for the nRF52 chip with UART. It's synchronous, so the expectation is that nothing will use this except for the panic handler.
 * Branden: Would this need to be implemented for _every_ chip for us to use it at all?
 * Amit: We could do it incrementally, in principle. The kernel could have both versions of how this works, at the cost of a static mut in the kernel trait, I think.
 * Brad: I think not. Boards that aren't converted would look exactly the same as today, where the board passes the object to the kernel panic handler. This new version, the board passes in a factory which can create the writer and the kernel makes the writer object itself.
 * Amit: That's right. Fairly simple to have both.
 * Branden: What I'm worried about is that chip-wide efforts often fall on one person to handle, and are a ton of work.
 * Amit: It's not trivial to implement this per chip. Right now we're not being careful about resetting UART state, which is maybe okay for development but not at all for production things. And all the chips are a little different. It's not terribly difficult, but not mechanical. This both means it's a particularly high bar to not merge until everything is transitioned, and also suggests that if we don't do that we won't ever fully get rid of the old stuff.
 * Brad: I did a couple. Apollo3 and SiFive as well. It's not hard to move what we have in io.rs and move it to UART. That's not safer, but not worse.
 * Branden: No worse than what we have now
 * Leon: At the very least, it warrants documentation saying that they were ported verbatim and require future reasoning about soundness
 * Brad: In any case, fixing this is a lot of work no matter what. Doing something is going to be hard regardless.
 * Brad: There could be other solutions. One challenge with this approach is how does the board tell the panic handler how to configure the UART. Right now we have an associated type that the board passes in, which gets routed through the kernel into the chip. A little bit of complexity there. So there are good questions of whether there's a better option, or if we think this is good enough
 * Leon: Is there a concern with keeping the panic UART and the main UART in main.rs in sync?
 * Brad: No, I think it's okay for boards to specify it in two places. We already do that with other things like blinking LEDs. Boards could make a shared type for it too.
 * Branden: Right. But it is important for the Board to specify how the UART is configured.
 * Brad: Yeah, a shared implementation in the chips crate would never be correct for all boards.
 * Leon: So you're asking if this design is right?
 * Brad: Yeah. Do we like that pattern, or is there some better method?
 * Leon: I don't want to rehash our full discussion, but to me it's counter-intuitive that in this pattern we have a sort of split-phase constructor, where we pass config in but then the constructor happens later. Just constructing the panic writer implementation in the board file and passing an instance to that into the kernel panic handler seems more straightforward. That's more of a nit-pick though, it doesn't fundamentally change how this works. Both solutions would seem fine to me
 * Brad: I do agree with you. The main benefit of the trait and extra complexity is that it would help us segment this as a non-normal thing that's for panic and nothing else.
 * Amit: You could have the unsafe trait in both cases. Would need it actually. What Leon is suggesting is that we don't need a writer config thing, and instead an already-instantiated instance of the trait should be passed into the kernel.
 * Leon: We'd create an instance of that kernel trait. The trait is a good centralized point for documenting the assumptions and guarantees for the writer. But we'd have an implementation of it in each chip. The trait wouldn't need a constructor anymore. Then Boards can just use a normal pattern for instantiating it.
 * Leon: I still think that's minor though
 * Branden: I do see the benefit of what Leon's describing, of having it just be instantiated in the board like anything else.
 * Amit: I think we need to be in a place where we think the pattern is generally right, and if there are any changes like removing the associated type, those changes could be mechanical.
 * Leon: Mechanical, but touching an enormous amount of code. I'm a little worried about the amount of work to change it
 * Amit: I'm not so concerned about amount of work, but we do really need to make forward progress on static muts soon. So I'm worried that chip-specific changes are needed. We can't even test all of the chips, and these are non-trivial changes that could be incorrect and break stuff. Versus changes that are essentially refactors at the type level, which could be cumbersome but aren't scary and don't require rigorous testing.
 * Brad: I agree with Amit. I want to make sure we like this general approach of having peripherals support panic output. Putting the onus on chip implementations to provide a panic writer implementation for that peripheral. I want agreement on that.
 * Brad: An alternative is that somewhere else we could have something taking in existing APIs and creating a writer, like a capsule or something.
 * Branden: I do agree with the chip implementation. A capsule "sounds" nice, but we need something that provides a synchronous API and knows about assumptions of peripheral state. I think a chip implementation is really the only way for this
 * Brad: Okay, we can move forward then. We can separate out the discussion about the exact instantiation.
 * Leon: So to move forward we have the pattern and implement it for just a couple of chips?
 * Amit: Yes
 * Brad: I can keep only things that I can test. Which is important. So nRF52 and maybe Arty. Then we'll separate out ones that we can't test and be careful about those.
 * Leon: We can further document about mechanical changes versus well-tested and thought-through implementations
 * Brad: One other thing to mention here. In the PR, I moved things out of debug.rs. Debug sounds optional, but traits that are implemented by chips seem like they should go somewhere else. To make sure people could remove debugging but have chips still build.
 * Amit: Agreed
 * Brad: Also, this fails on the nRF52 right now. We recreate the pinmux, which the logic checks for and refuses.
 * Amit: I think we might change what Pinmux takes, so then we no longer need to check it in the constructor. The Pin thing can guarantee uniqueness on its own.
 * Brad: Would that work?
 * Amit: Maybe. Maybe not. Pinmux really does that as its only thing. Well, there's a from_pin function that's unsafe and constructs it without checking. It's problematic, but should work.
 * Brad: Okay, so it won't panic as-is. Great.
 * Brad: This depends on https://github.com/tock/tock/pull/4678
 * Group reviews and merges it on the call

## Next Meeting Timing
 * January 8th is next meeting, still at this time
 * We will do another scheduling round for this meeting during the first week of January, since people's schedules may be changing in the new semester/quarter.


