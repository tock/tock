# Tock Meeting Notes 2026-10-14

## Attendees

- Branden Ghena
- Amit Levy
- Alexandru Radovici
- Leon Schuermann
- Brad Campbell
- Johnathan Van Why
- Pat Pannuto


## Updates
* Johnathan: Current status of Tock-Registers is that all open PRs have been merged. I'm still waiting for Leon's feedback on two things.
* Johnathan: There's a discussion on Matrix of releasing a 0.12 that hard deprecates old APIs. I'll probably move forward with that if no one disagrees


## Migrating to new tock-registers APIs
* Amit: Discussion here is "how to do this" and whether to do it piecemeal or all at once
* Johnathan: There are effectively three versions of API. Old-old with repr(c) structs. Old API with register structs macro. New API with register map. We want to move everything off of both the old APIs because they're unsound. But we didn't decide if we want to move stuff by hand on a case-by-case basis or if we should make a project out of moving things off of old APIs.
* Johnathan: Apart from soundness, there's also a Rust lint that would flag old APIs that I want to add. Downside is that it's a big project to port everything
* Johnathan: Brad + LLM made a tool to migrate everything. Seems like a good start, but it's a big tool and I don't fully understand or trust it. Concerned about feasibility of humans reviewing all of the code that comes out of that.
* Johnathan: Is there some other solution? Should we be rubber-stamping the results?
* Amit: Your concern is that if we do some mass migration that it would fundamentally require careful manual review?
* Johnathan: Not necessarily. We could just give it a skim. And/or we could trust the tool for correctness. Most importanty, the tool just needs to error if it encounters something weird.
* Leon: A cheap check would be to compare the hashes before and after the migration. Since the change should be a no-op right?
* Johnathan: That might be a really good technique for the repr(c) to register map port. But I think renaming things will result in changes causing the hash to change
* Amit: Why is it a separate question for repr(c) vs register structs?
* Johnathan: They're separate code that both need to be ported. Same tool can probably do both. We do have a tool for moving repr(c) to register structs.
* Amit: That's a deterministic tool, right? So in principle it's error-free. Assuming we do a good job of reviewing the tool
* Johnathan: If I wanted to review the tool, my inclination would be to rewrite things and edit it to make it more readable.
* Amit: That seems fine.
* Brad: Why did we discard the idea of just reviewing the changes?
* Amit: I think we plan to do that too. Just understanding options
* Brad: I'm inclined to just review changes and delete the tool afterwards
* Branden: I think this tool is a one-time use tool. We're just going to port everything once, so it doesn't have to be super well written.
* Amit: Could be useful for downstream chips. So we might want to provide it
* Amit: I also don't _mind_ it being well-written. Understanding that it does the right thing seems most important
* Amit: Presumably the argument against reviewing each change is that it's a lot of work and we'll do it sloppily
* Johnathan: Yeah. I think the truth is that they won't all get a quality review. We'll all rubber-stamp it, or it'll linger forever. I'm not saying that rubber-stamping is the wrong move here, but only if we make it a willful choice to do that.
* Amit: Does Brad have reservations about using the tool?
* Brad: I don't know. Claude seems to do a good job, and the tool errors when it encounters something odd. And many peripherals are just a series of registers so it only asks you to count by 4. So it doesn't seem worth making this tool better
* Amit: So you would use the tool, you just wouldn't worry about validating/improving it, and would prefer to spend time reviewing the results
* Brad: In fact, I've already applied it. I've got branches with output with everything ported.
* Amit: So we just have to decide if we want to review or establish more trust in the tool
* Leon: I took a first look at the tool just now. It's doing a lot more low-level string and token manipulation than I hoped. Doesn't make me super confident. But if it looks fine for a reasonable set of examples, I'm not too worried about rubber-stamping this
* Pat: We should consider how thoroughly we reviewed the source material that it's translating anyways. Someone wrote down a list of these and got it mostly right we hope. And the ones they used were tested. So I'm sure there are already issues and there will be more issues
* Amit: Our merging of those originally did rely on the assumptions that someone ran the code to test it
* Pat: Fair. We don't have person-power to do anything other than automate this change, and check the output a few times to attempt to validate it
* Leon: I concur with Pat's opinion. Some of our less-well-supported chips have had various issues in their register maps. We shouldn't systematically introduce new issues, but the source material is far from perfect to start with
* Johnathan: We have code to do repr(c) to register structs. Moving to register maps is more complex because usage of the registers changes. So I'd propose that for now we move everything from repr(c) to register structs. Then we can create a tool to move all register structs to register maps.
* Amit: Why go through register structs to begin with?
* Johnathan: We don't know if a tool to port to register maps is easy to write. At least that tool will only need to read in one variant.
* Branden: We could chain the tools together when needed though. I'd like to avoid committing register structs if we want to replace it
* Brad: We shouldn't have the repr(c) stuff in here. This would be a step forward
* Branden: Why take the temporary step? Is it going to take forever to take the step to register maps?
* Brad: I'm worried it might. And it's a small step to move to register structs now. Smaller PRs are better
* Amit: Well, it's massive PR changes either way
* Brad: It is different. The change to register maps is easier if everything is already in register structs. The diff is smaller.
* Amit: You think each of the two diffs is smaller than the one diff
* Brad: Probably about the same size
* Amit: So we'd be replacing two small PRs each with one small PR. It's not a bigger PR.
* Brad: Going from registers version 0 to registers version 2 becomes smaller if we do the conversion now, which we arguably should have done years ago
* Amit: My contention against it is that we wouldn't be solving a soundness problem, we could be introducing semantic incorrectness with porting bugs. And then we'd port a second time. So the gain we'd get from porting to register structs is small compared to the risk and overhead. We'd have to review the PRs twice for correctness. Branden's framing seems right: if we're two weeks away from just going to register maps, then we shouldn't do this step. If we're a year away, it's a different story
* Johnathan: Changing to register structs does an indentation change, which adds a diff for every line in the thing. There's no indentation change for register structs to register maps, so that's less change.
* Johnathan: Refactoring PRs do make it hard to go through git blame and figure out why something changed. So more PRs here would affect us for years into the future
* Amit: I think we haven't resolved the issue of how far away we are from register map.
* Johnathan: We have two issues. One of which might be easy to resolve, or could be an unsoundness issue that's a month or two out
* Branden: If there wasn't an unsoundness issue, would we be ready to port all of Tock, or still be a while out
* Johnathan: I suspect someone will push back on some part of the API. So we might want back-and-forth on changes
* Branden: So it will be several months until all of Tock migrates
* Amit: And if it's going to be a while, it might be good to do the repr(c) port now. And then it'll be easier to do the full port later
* Johnathan: It does take more engineering/reviewing effort to do it now
* Amit: I think a reliable tool could help. If the tool was as reliable as cargo format, we'd run the tool and hit merge.
* Johnathan: I agree with that
* Amit: There's no reason we can't piecemeal migrate things, right? And those migrations could be individual PRs where the things in the PRs were originally proposed by claude or the tool claude built. As long as it's low volume, that's not a problem.
* Amit: So my suggestion is, let's start with a few PRs that move repr(c) to register structs for a few small things.
* Amit: Then I will, this week, try to work on this tool and see where we get to. Either the reviewing is easy, the tool becomes reliable, or we reevaluate the plan.
* Johnathan: Release timeline for register maps depends a bit on engineering effort and time.
* Leon: I can commit some time right now to just pushing this forward a bit
* Brad: PRs are up per Amit's plan.


## 64-bit RISC-V ABI TRD
* https://github.com/tock/tock/pull/4906
* Johnathan: This is a TRD. Two of us approved it. We wanted to wait for the Tock 2.3 release. I wanted to know if we can merge it now
* Amit: Branden noted that the core team should have notion of consensus and sign-off on this. So I suggest we read it right now on the call and see if there are any flags preventing us from merging it right now
* We read for a while
* Pat: Comment, do we want the upper 32 bits be zeros or undefined?
* Amit: Similarly, handling i32s has the same question. It's really just about the code that extracts data from them
* Leon: We don't want a decision we regret in the future, but this does match how RISC-V 64-bit would run 32-bit code. It just ignores the upper 32 bits. So this sort-of matches the architecture
* Amit: Does the architecture set those upper bits to zeros?
* Leon: Yeah
* Branden: Even for signed values?
* Leon: Unsure
* Branden: It doesn't matter though, as the userspace interprets the bits however the rules say
* Amit: So we want to store a signed 32-bit value in the lower 32 bits and extract that by masking out those bits and casting to whatever type is relevant. So signed is just two's complement 32-bits in the lower 32-bits. And unsigned is unsigned in the lower 32-bits
* Amit: We could say that the upper 32 bits SHOULD be zero, but not MUST.
* Leon: My only argument for leaving the upper bits guaranteed as zero is that code which doesn't care about signed values can be more efficient on a fairly hot code path
* Amit: I don't understand that
* Branden: I think it's a cost either way. Either the kernel assures the upper bits are zero, or the userspace does.
* Leon: When you load a value into a register in risc-v, that clears the upper bits. So the kernel side doesn't have a cost
* Amit: Does anything guarantee that occurs though?
* Leon: If we read values from memory, then the upper bits are zeroed. If we move the value from one register to another, it's highly unlikely that the compiler doesn't leave the upper bits zeroed. RISC-V always zeros them out
* Amit: What's the example where having the upper bits being undefined hurts?
* Leon: You have to do extra instructions to remove stuff.
* Branden: What's the upside to leaving upper bits undefined?
* Amit: Other architectures could have advantage from it possibly
* Amit: In practice, maybe it's not a big deal to just always set the upper bits to zeros.
* Leon: Is there any architecture where when you read a 32-bit values, the upper bits don't get zero'd?
* Branden: Signed values sign extend up to 64-bits often.
* Leon: I think the i32 values we handle as "cast to u32, send in u64, convert back to u32, cast to i32"
* Amit: Why would we do that?
* Brad: Because we have to handle any smaller types. So to simplify it we make it 32 bits of space.
* Leon: I think we should add to the current document that all values transferred are unsigned. And that signed values have to be packed and unpacked properly.
* Brad: And that's what we actually do, and what this document intends, and I think what it says
* Amit: So when we say packing and unpacking, we mean "stick an i32 in the lower 32bits of a u64"
* Amit: An upcall can't have signed values? That seems not correct.
* Johnathan: They're all u32 values.
* Amit: That seems like a bug.
* Leon: Why is it undesirable?
* Amit: Because the values sent could be negative
* Leon: For an actionable suggestion, this document for command arguments should NOT say that it supports signed values. Our system call interface does not have signed values. We encode signed values as unsigned, and send them over the interface.
* Amit: But different capsules shouldn't have different conventions for how they pack signed values into unsigned storage. We should have some directive for how it's done
* Leon: I agree that there should be a convention.
* Amit: So what we should do is, in 3.1 line 77 it should refer to value32 instead of u32. And more importantly, in 4.3 argument0 and argument1 should be value32. And we should have a separate section for recommending how capsules should pass non-u32 values in commands and upcalls.
* Pat: I might push back on that approach. TRD104 has a sentence dealing with "values smaller than the native register type" that's the issue we have here. We can say that the value you're passing across the 64-bit ABI is restricted to the 32-bit range. The way that you pass it, is to cast your value to a u64 or i64 so that it matches the native register type when going across the ABI. Then the other side casts it back.
* Leon: Are we sure that when we direct case from signed to unsigned there's no sign extension? Also, is casting well-defined across languages?
* Johnathan: That would make things tricky for userspace, as you can't pack a i64 into a u32 as it doesn't fit. So casting to a u32 then zero-padding is actually easier. That was you can type erase it on the userspace side.
* Amit: So what's the takeaway there?
* Johnathan: Userspace APIs still only see u32, but the recommended strategy for encoding signed value is, first scale to i32, then cast to u32, then pass that value in following the ABI rules.
* Pat: So we should add a sentence explaining that cast flow to this TRD
* Leon: I think this should apply to TRD104 and it's an oversight there.
* Brad: I'm worried about new things here that aren't also in TRD104.
* Pat: I think TRD104 does this correctly.
* Brad: Okay, then we're good if TRD104 says it.
* Amit: But TRD104 doesn't address the case here. How do we handle i64s?
* Johnathan: Passing an i32 in a u64 is poorly defined.
* Pat: For i64s the high half is signed and the low half is unsigned.
* Leon: Oh, we really need to document that.
* Brad: But that's a TRD104 issue
* Amit: But this is about splitting a 64-bit value across multiple registers. In TRD104, once you split it you're putting it in a 32-bit register filling the whole thing. In this TRD when we split things, we put them in bigger 64-bit registers and have to decide what to do
* Brad: Okay, I don't quite follow. Maybe someone should write it and that would help me
* Amit: It should just be defined. Once we split the value into low and high, we say that the type of those things is value32 and then we're done. We stick those in the relevant bits. We just have to say that. It doesn't go in TRD104 because it's not a question of what to do with _other_ bits in the register as we fill the whole register.
* Brad: Okay, makes sense. That's why I wrote the sentence you commented on. So we change that to value32 and then we're on the same page.
* Amit: We're talking about lines 198 and 199
* Brad: That to me would be a TRD104 issue. How does an i64 become argument 0 and argument 1. Once you have that, line 77 explains how you pack that into a larger register.
* Amit: But that doesn't compose as nicely with what Pat suggested of adopting the language from TRD104 for 64-bit. The suggestion is to replace argument0 and argument1 in 4.3 with their types being value32. Then separately we make a section with a convention for non-u32 values in command and upcall. So it would say "when passing values with sizes smaller than 64-bits... something something... for 64-bits we split between low and high and treat the resulting 32-bit values as value32" and that would be done.
* Brad: That sounds good, but that's a TRD104 issue, as it also has to handle i64 values. So in my mind, the path forward is to make the value32 change, and then modify TRD104 with the updated language. We don't want to specify stuff in this TRD that's unspecified in the other one.
* Amit: Agreed. This TRD defers to TRD104
* Amit: I'll take a stab at modifying TRD104 today


