# Tock Core Notes 2023-09-15

## Attendees
- Branden Ghena
- Pat Pannuto
- Leon Schuermann
- Hudson Ayers
- Alexandru Radovici
- Ionut (Johnny)
- Felix Mada
- Johnathan Van Why
- Amit Levy
- Chris Frantz
- Brad Campbell
- Phil Levis
- Tyler Potyondy


## Updates
 * Leon: From OpenTitan call, Brad and I chatted about `set_client()` issues in various HILs. In HMAC, we have a couple of combined traits and each have a client. We also have a client that's a supertrait across multiple. This completely destroys composability of HILs on our end. Depending on what the underlying code implements, you have to use a different `set_client()` call. So you have to be aware of the specific implementation beneath you, which is less than ideal.
 * Hudson: So this is the higher-layer client that's being set. I agree that seems bad
 * Leon: We can split things up into multiple different clients and remove all super-clients. Or alternatively, get rid of the individual clients and just have one client trait with methods for all possible events.
 * Branden: What's the downside of having one client trait?
 * Leon: If you are making a userspace driver, your implementation has to hit all of the traits for callbacks, even if you don't need that callback at all.
 * Hudson: Is this going in an issue somewhere?
 * Leon: It's across a couple different PRs, but we should create a single tracking issue.
 * Branden: In C callbacks, often they take an event-type argument. So you can just have one callback, and you can ignore types you don't expect or can't handle. Would that work here?
 * Leon: I think that's not too different from multiple callbacks with no-op function for callbacks. I'm not sure of the size difference of vtables versus match statement.
 * Branden: And rust loves to be pedantic with match statements so you'd likely have all cases anyways.
 * Hudson: I think it might be easier to have no-op functions, where it's clear we don't care about it (rather than forgot it).
 * Chris: For OpenTitan clock frequencies, we did commit it. But we found our CI doesn't share artifacts for commits on branches. So the bitstreams aren't published, even though the code is there. So the change in Tock to adjust FPGA clock rates and ratios isn't something we can test yet.


## ClockInterface Trait
 * https://github.com/tock/tock/issues/3671
 * Alex: Followup from discussion in Networking group. Many peripherals need a way to find out what frequency their clock is running out. For the rp2040, we built our own infrastructure and added a function to the ClockInterface trait. But every board will have to implement this. So we would have to change all of the boards (and we can't do that because we can't test all of them). We could instead of returning a number return an Option and have the default return `None`. But what's a peripheral supposed to do with that?
 * Pat: We have a formal definition of Frequency for the timer subsystem. They aren't the core CPU frequency, but are a notion of a discrete number of frequencies that HW could support. I wonder if it should be similar here, where it's an associated type? Actually, probably no since the clock can change frequencies.
 * Hudson: We could have a trait object.
 * Alex: I think the frequency trait has a static function.
 * Alex: A problem is that many peripherals have many options. The RPi has 32 dividers, so many different options. We also might need to change over time. For example: low power versus high power modes.
 * Pat: Frequency as a trait isn't a static function. It just returns the frequency. So it feels like you could return a struct that returns the frequency dynamically.
 * Branden: Why over-engineer this? Why not just return a number? For the boards they can't test, we could just have a static number, since they almost certainly don't change their clocks.
 * Pat: Don't know the difference between capital F Frequency and lowercase f frequency. When should we use each? I don't remember why we even have Frequency. I should review TRD105.
 * Alex: Should we defer this for next week?
 * Branden: Well, let's not defer unless there is a clear plan. I don't want us holding you for a week, and then coming back in the exact same spot.
 * Alex: So the main question is how do we build an infrastructure so a peripheral can figure out the frequency the clock is at?
 * Pat: And I think the architectural question is how does that relate to our time definitions in TRD105? Should we be trying to adapt/reuse that? Or is this orthogonal and should be different?
 * Hudson: I agree that the question is why we should use Frequency at all. So we can see if we need to use it here. I suspect Phil will know the best.
 * Pat: We could _remove_ the Frequency trait from the time HIL. But we should do something with this code that theoretically attacks the same problem.
 * Leon: Some of our chips do something really weird with the trait, which has a hardware register that returns this. At the time, Rust wasn't capable of doing this with const generics. So maybe returning a number is just the right way to go. Not having to be generic over a frequency trait which returns a number.
 * Alex: When adding boards, I several times had issues where I had to add Frequency traits to the kernel. Eventually we'll hit every possible frequency across enough boards...
 * Pat: Yeah, I think you've walked into a bigger problem here. We're just going to have more and more of these Frequency enumerations. On first read, it looks like we're trying to use traits to stop people from making clocks that can't be realized based on the underlying time source. Maybe there's a better way to do that?
 * Leon: I was thinking about PR https://github.com/tock/tock/pull/3333 as my attempt to touch this once before
 * Alex: The RPi PLL can indeed create just about every possible frequency
 * Pat: So how do we pass that into the Alarm requirement?
 * Alex: Indeed. We really hit this with Networking. We up the clock to high speeds.
 * Hudson: Do you need to change clocks, or just set the clock to a high speed at runtime?
 * Alex: We'll need a couple of profiles which we can switch between: high-speed, low-power, etc. So we would switch dynamically.
 * Hudson: This is starting to sound like PowerClocks. It got complex though. We had to add a lot of stuff for the SAM4L, because certain peripherals start or stop working based on the clocks. So you have to reconfigure certain peripherals each time you change clocks. And you have to change the clock speed based on user calls to service them properly, delaying them until the clock was ready.
 * Amit: PowerClocks never made it upstream because it was relatively involved and the SAM4L wasn't worth the effort since others weren't using it. PowerClocks did yield good results though.
 * Hudson: I thought it also required a lot of interfaces that only made sense if EVERY chip implemented it, and that was a huge lift.
 * Amit: And the nRF52 does that all magically in hardware.
 * Branden: Right, it doesn't let you chose at all, and just "does the right thing"
 * Amit: Maybe worth revisiting. Alex, maybe we can send you the paper and see if it matches what you guys want.
 * Alex: We need to include Felix here. He did the CAN stuff and had clock issues.
 * Alex: For us, this is a must-have. We have requirements where the chip has to go into low power.
 * Leon: Echoing this sentiment, PR 3333 does implement dynamic clock switching. That code could be useful.
 * Branden: A big thing I see here is that profiles could make this simpler than infinite modes. We don't need to handle arbitrary complexity, only a few discrete states.
 * Alex: Exactly, and boards could say that certain modes aren't available.
 * Hudson: Okay, we'll send over the PowerClocks paper and send over code, then we'll use that as a starting point for further thought.


## STM32 Clock Management
 * https://github.com/tock/tock/pull/3528
 * Brad: This has been around for a while. It's adding features for the STM family for several different variants. The issue is that features don't work well for this use case. Features are supposed to be additive, not mutually exclusive. And chip variants are mutually exclusive. I'd like to get rid of the features and pass in the options to the crate.
 * Pat: Are you describing something similar to what Leon did for the frequency variant configuration in the OpenTitan space?
 * Brad: It could be. It would be at the chip level instead of board.
 * Leon: That should work, as long as you're using the specific chip variant.
 * Branden: Do you have a link? I don't know what this is?
 * Leon: https://github.com/tock/tock/pull/3640
 * Leon: One takeaway was that we wanted a good precedent of how to do features correctly. Type signatures do get long and unwieldy.
 * Brad: https://github.com/tock/tock/pull/3640/files#diff-2ebe726d4be8b51b1c35fbd4be6b1ef605e264656feeada21790ec4a755e43dbR82
 * Brad: That link has a specific use of the change to OpenTitan. Do we know how bad the type signatures would get?
 * Leon: It's pretty bad. You have to add the type to all uses of the chip, which itself is a generic argument for many things. I solved it with a type alias, that can be used.
 * Pat: A type alias makes sense. You're describing a discrete thing, which is a type of chip that specifies specific choices for the variant.
 * Brad: Why does that have to propagate up beyond the chip crate?
 * Leon: Oh, the chip crate can bind to the variant it wants?
 * Phil: Do we have a sense for how many layers of hierarchy there might be? STM32, STM32F4, variants so on?
 * Alex: They have a ton in the family, and they sort of do and sort of don't overlap. They have a manual for STM32F40x7 where the x can be anything, but they also have STM32F46x. We have some chips that differ by two registers and some by MANY.
 * Brad: Regardless we'll run into this problem.
 * Leon: I'd say A) we want to share peripherals whenever possible and B) we'd probably have a single STM32 trait which encapsulates the differences and lazily add things to the trait.
 * Alex: What do you mean by trait?
 * Leon: There's a file in the PR that isn't meaningful at runtime, but contains differences between chips.
 * Leon: https://github.com/tock/tock/blob/master/chips/earlgrey/src/chip_config.rs
 * Phil: Not having duplication is what we want to do. But when the hierarchy gets deep or complicated enough, you start chasing constants around and have to look at a ton of traits. Seeing things in one place versus de-duplication
 * Leon: I agree. It's a "noble goal" but we do have different drivers when necessary.
 * Pat: Tyler and I were talking about this. I wonder if it's a tooling issue. I wonder if the place to store this is in the tree, but there's a tool we write that shows you "here's where everything comes from for this chip"? But now I'm just reinventing device trees in a way.
 * Brad: But we're not super deep right now. It's four devices. We could work on something more complete outside of this.
 * Phil: For four, just flatten it. Works for any small number
 * Brad: So have four copies of the flash driver?
 * Phil: The constant definitions at least. The deduplication point is especially true for code, but not for constants.
 * Leon: I think we've only been meaning to talk about deduplication for code.
 * Phil: I might be confused here.
 * Pat: It's come up with constants more on the nRF52 side of things, where there's a base and configs add more registers to parts of the hardware. So really just the memory map extends.
 * Leon: To illustrate our point, for this particular STM case. I think Brad and I are proposing to create a trait that captures the relevant constants, then four implementations of the trait: one for each variant.
 * Pat: I like that solution here. Does that make sense to Alex and Johnny?
 * Alex: Yes. Similar to Leon's PR, right?
 * Brad: Yes. Just wouldn't be exposed to the board
 * Phil: I would say, the devil is in the details. If this doesn't seem to work when implementing it, we can circle back to the question again.
 * Alex: Okay. We'll modify the PR and move forward then.


## Returning to Frequency Trait Discussion
 * Pat: We don't have anything in-tree right now dealing with dynamic frequencies of clocks. Holly's work doesn't exist in-tree, so we were discussing what stuff should look like for Alex and company. PowerClocks feels like it had full power, with infinite choices but lots of complication. But we discussed that we could do discrete profiles, like low-power, high-power, etc. which could simplify this?
 * Branden: We also discussed the Frequency HIL from the timer/alarm interfaces, and whether they would make sense here or not.
 * Phil: The key things from PowerClocks was knowing _when_ you can change clocks so-as to not disrupt an operation. You could definitely quantize things.
 * Amit: Our takeaway was that we would share PowerClocks as a reference for them, then jump off from there to think more deeply about what makes sense.
 * Phil: Makes sense. I think Holly had some great insights into the complexity here. I don't think any of us would argue the implementation was the perfect thing to do. Take inspiration from it, rather than treating it as the final word. We could even rope in Holly possible.
 * Amit: Power management and clock management is one of the original motivating factors for Tock, which was less important for the hardware and applications we've used so far, but it's exciting to see it as a real requirement again.
 * Alex: The whole discussion started in the peripheral knowing what it's clock frequency is.
 * Phil: I think you could certainly go with a simpler form of the mechanisms from PowerClocks. But it can be pervasive, with everything needing to know about it. And maybe some chips can implement it and others can ignore it, depending on need.
 * Alex: Another angle of this story was to add a function that returns a frequency. But should we return a number or the Frequency trait? Our PLLs can generate more-or-less any frequency possible in a range.
 * Phil: I'll need to refresh my knowledge of the frequency trait. Definitely tricky. No question that the time stuff was intended for static use, not dynamic use.
 * Alex: Even while not changing the frequency when running, we could have a different frequency per board based on which peripherals are needed. Ethernet or not, for example. So the exact same chip will need to run at different frequencies.
 * Phil: What's the issue with the Frequency trait?
 * Alex: Branden and Hudson said it adds complexity.
 * Phil: It does
 * Leon: We also have this Frequency trait type-system concept that returns a runtime number. It's a weird hybrid place where everyone expects a constant value, but you don't have to?
 * Amit: It was pre-const-generics
 * Leon: Yes. And const-generics break with some chips
 * Phil: So chips have no way of doing dynamic frequencies without callbacks. For example, a timer needs to know when clocks change so it can change counters and prescalers and stuff
 * Alex: I think that is version 2 of this. So version 1 is the peripheral knowing what frequency we're running at. Version 2 will be changing it.
 * Phil: Doesn't the Frequency HIL trait do that now?
 * Alex: We would have to add it in the board and export it
 * Leon: I think that's okay
 * Alex: How would the chip get something from the board?
 * Leon: Have the chip take a generic argument
 * Alex: So the chip would have a generic frequency parameter you can access?
 * Leon: Yes. And you could request a static number at runtime
 * Amit: I suggest we have a dedicated asynchronous or even synchronous discussion about this in the future
 * Alex: We can continue next week. We'll look at Leon's idea.
 * Phil: So Leon's idea is that there's a const-generic parameter that lets the chip look at the frequency?
 * Leon: Something that implements the Frequency trait anyways. So you could use this to call and request the frequency value.
 * Phil: I wonder what will happen compiler-wise with it passed as an argument to the chip. Will it be able to figure out it's a constant and do useful optimizations?
 * Leon: For sure it'll know this at compile-time. It'll be labeled as const.
 * Phil: But there could be a code-path in the board which could pass in different options.
 * Leon: It would monomorphize and create the options, as a generic parameter.
 * Alex: That would stop us from changing the clock in the future.
 * Leon: Our current infrastructure really can't support changing frequencies, until we add support for that.
 

