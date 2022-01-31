# Tock Core Notes 2022-01-28

Attendees:
- Hudson Ayers
- Branden Ghena
- Philip Levis
- Leon Schuermann
- Pat Pannuto
- Vadim Sukhomlinov
- Johnathan Van Why
- Alyssa Haroldsen
- Brad Campbell
- Alexandru Radovici
- Amit Levy
- Jett Rink

## Updates

### ufmt library
 * Hudson: On uFmt stuff. I've completely implemented width specifiers, so you can pad numbers with zeros or any type with spaces, at least on the right side of the type. Left spacing for non-numeric types would take a re-architecture. I've gotten the size overhead down to like 1.5 KB out of 13 KB for the applications I'm looking at. So now they only save 11.5 KB by changing to this uFmt solution. Still a pretty big win.
 * Alyssa: Does ufmt support lowercase x or uppercase X or both? Would only one save a non-trivial amount of space?
 * Hudson: It only supports lowercase, but I think uppercase would be a trivial cost.
 
### Hardware CI
 * Pat: Hardware CI pull request finally showed up. Seems to work in the base case so people should start taking a look.
 * Phil: Now all we need is to duplicate on many systems.
 * Hudson: What's the base case again? One platform (nRF52840) and some base tests (UART, GPIO, and not SPI).
 
### Trustzone and Tock
 * Phil: A student rotating with me has picked up the M33 code for Tock. So trustzone-m on the nRF53. Some interesting thoughts on how trustzone could better secure tock. Particularly against capsules that could abuse safety issues in libraries. So basically you put some part of code in trustzone so a malicious capsule can't corrupt, say, core crypto.
 * Vadim: How do you determine which memory accesses are allowed and which are not?
 * Phil: When it boots, the code in trustzone says which memory isn't accessible.
 * Vadim: But it's not dynamic, right?
 * Phil: It's decided at compile-time, but happens at run-time.
 * Vadim: A long time ago I was working on sandboxing individual functions. I'm very interested in applying it to Tock.
 * Phil: The whole port is up to Tock 2.0 now. And more work is on-going.

### Libtock-RS 2.0 and Result
 * Alyssa: I was proposing adding must-use to command return. It's easy for a command to return and not be used.
 * Hudson: For background, this is similar to forcing results to be used. I support this.
 * Phil: There are some narrow cases where you don't care. But those are rare and you can be explicit about ignoring
 * Alyssa: I've got some code for turning command returns into actual results.
 * Phil: I'd love to hear about that. One of the goals of the way the systemcall ABI was designed was towards compatibility with result.
 * Johnathan: Oh! If I had known that, I would have had some comments on it. There are three cases now, counting "unintended result".
 * Leon: We could always represent unintended result as a failure, but it's also possible that we didn't design it right based on what the userspace might need
 * Johnathan: If it's a failure with a u32, what do you stuff in result? (Maybe just zeros)
 * Leon: The idea was to collapse into an error when passing up to higher level code.
 * Alyssa: The logic right now is generic over all success and error types. When you call into result, it checks if it's an unexpected variant, and it uses errorcode::fail if it needs to, filling in zeros for the other stuff. We could use the bad return value error code that's in TRD104 instead. It's not in the enum right now because the kernel can't return it, but the libtock-rs API might still want it.
 * Phil: Agreed
 * Leon: I think we expected it to be added to the enum for userspace. Just not in kernel-space. So we should update the userspace enum
 * Johnathan: It might also be a separate enum that has that type, rather than the errorcode enum. We could return a different type instead.
 * Alyssa: I'm not sure whether having the restricted enum in userspace is beneficial. Would it be easier to just have errorcode be a struct wrapper over u16.
 * Johnathan: It would be easier. Just loses efficiency in some use cases, but we likely don't care.
 * Alyssa: There's already one niche case with non-zero u16.
 * Johnathan: And there are many special cases the way it handles it now.
 * Phil: How would userspace respond to this anyways? If it returns a different type, you can't recover.
 * Alyssa: Right, it means your library is out-of-step with the capsule (out of date or a bug)
 * Johnathan: In a lot of cases, within the userspace API, I think it will be obvious what to do. But the generic command return path doesn't have anything obvious to do.
 * Alyssa: I think the answer is going to be panic or propagate.
 * Hudson: Propagate. If you app wants to be super reliable, then it can try to soldier on.
 * Alyssa: And I want to avoid errorcode::fail, because it's unclear what it means
 * Johnathan: Maybe add a new errorcode for it that handles "extreme failure"
 * Phil: Libtock-c does propagate badrval. It's part of the enum there.
 * Alyssa: Okay, I think I'm going to add it to libtock-rs
 * Alyssa: Generally, I'm thinking about how to structure into result to save code size too. I'll experiment and see what works best.

## Final review of App Completion Codes
 * https://github.com/tock/tock/pull/2914
 * Hudson: I pushed a commit handling things from last time we talked about this. Want to make sure everyone is fine with the text as-is now.
 * Vadim: How should the kernel handle codes that are nonzero. One use case is that the kernel may try to restart an application or may not. How should this be indicated what action it should take for this app? In most cases you would expect either restart or non-restart.
 * Hudson: Right now in Tock, boards get to choose the restart policy. I wanted this doc to say it would be allowable for a board to choose to only restart apps with non-zero exit codes. But any board can choose any policy.
 * Vadim: Would it be reasonable to have an error code that indicates that there was an unplanned panic in the application?
 * Hudson: Currently, we report panics in libtock-rs via the low-level debug. C doesn't have an analog to that. TI50 has a custom error handler for apps. But libtock-rs still plans to support various panic handlers that apps can choose from.
 * Phil: Two comments. 1) in the table say "reserved" rather than "not defined". 2) in section four, we should document the Termination trait here. Since this is a documentary TRD, section four, which says to use a trait, implies that this document should _also_ document that trait.
 * Alyssa: I disagree. I think the trait is an example that implements the design described.
 * Phil: So what happens if the source code changes and the trait goes away?
 * Alyssa: It would be a broken link like any out-of-date documentation
 * Hudson: Well, it wouldn't break. You linked to a specific blob of code.
 * Phil: So, I think we should describe the trait here and the implementation. I think this doc should be self-contained, even if github stops existing in the future.
 * Alyssa: I think there's strong value of connecting the design doc and a concrete implementation. I find it very helpful when trying to learn the code base. So, I think removing it is bad.
 * Amit: Maybe what Phil is saying is that you can both keep the link and also import things from the link so the documentation is standalone.
 * Phil: Yeah, exactly. Self-contained.
 * Alyssa: Would this be a binding promise if we include it in TRD106 here?
 * Phil: There's a process for writing a new doc and deprecating an old one. Things can change and evolve.
 * Leon: I think this is a major concern though. It kind of seems like a big choice to say that the kernel must never do anything. So maybe we should say that if there's a contract between the kernel and the application, then it might apply semantic meaning to the completion codes. We definitely want to avoid misinterpretations of codes, because if there's no agreement by both parties than there should be no action.
 * Phil: Generally what I was getting at was that it would be good to have the termination trait here.
 * Alyssa: Okay, as long as we can have both the trait and the link, I'll do that.
 * Alyssa: Going back, should we really do not defined instead of reserved?
 * Phil: In my mind, reserved means that in the future we might specify.
 * Leon: TRD104 says some areas are reserved and others will never be used. So maybe reserved has the wrong meaning here.
 * Phil: Okay, I buy that.
 * Brad: With return code zero being a MUST. What happens if a process doesn't do that? Why is it a MUST?
 * Alyssa: My thought there is that any engineer who sees success printed by the kernel will be very confused why there are errors. Maybe the MUST is too strong of language, but I wanted to make it clear that returning error code zero on a panic is a bad choice. But it isn't breaking any invariants on the kernel, so maybe it should be a SHOULD.
 * Leon: I like having a specific number for it. There is a much more contained space for success than the possibility of errors. The app could still make multiple other success variants on other numbers, if it wanted to distinguish. So zero would always mean that the application succeeded.
 * Brad: That does make sense too.
 * Phil: Okay, I think this doc is good as-is now.


## Implications of App Credentials
 * Phil: One of the requirements of AppID is that two processes running at one time can't map to the same short ID. So an implication of that, is that two applications with the same Application ID (decided however you want) can't run at the same time. So this means that when we want to run a process, we have to check that no process has the same Application ID. Currently all the checks are in the process trait. So every implementation would need to handle this correctly.
 * Phil: So, I added a function to the kernel that is "submit a process to run". It checks if it is valid, and then marks it as runnable. This state transition would require a capability which only the kernel would have.
 * Phil: The way it's written, a ton of functionality is within process, but properties of processes for security reasons mean that some of that logic should move into the kernel. We don't want kernel guarantees reliant on specific process implementations.
 * Brad: I think it's a great point. We've always relied on a little of this, because the kernel loop uses internal process state to determine which things it can run and when. We've always relied on those states being valid and not crashing the kernel. I would definitely be supportive of policy decisions being extracted outside of any implementation.
 * Phil: It does mean more Capabilities. We also discussed in the past when to use a capability versus a trait. So we could have a trait that only the kernel has access to. The two things on process right now protected by capabilities are the method that says that the process integrity is verified (passes credential checks into the "checked state"), and then the second is whether a "check" process can be run.
 * Brad: I think using traits with process just _doesn't_ work with rust the way we set things up. So capabilities might be our only hope, just for compilation reasons.
 * Hudson: Yeah, we sure ran into a lot of issues when trying to do this for process printing. So long as we can keep generics off of the process type itself, we're okay.
 * Amit: Phil, can you clarify more what the question we're asking is?
 * Phil: I don't know that there is a question. I just wanted to let everyone know and check if anyone saw a major/minor issue with doing this. I wanted incremental updates so I didn't make a whole system and only at the end find out it's messed up. In this case, it's the refactoring of responsibilities between process and kernel and who can transition process states I wanted to check in on.
 * Amit: In that case, this totally makes sense to me.
 * Phil: Cool. I'll keep going, but people feel free to speak up if you realize an issue. I haven't fully done the short IDs part yet, but the system works in the sense that two processes with the same application ID returned by the application identifier trait, the kernel won't do it.
 * Hudson: I'll second that this sounds good. What I see in the changes looks good so far, but I'll keep skimming as you go and will speak up if I see an issue.

## ProcessBuffer & Raw Pointers
 * Leon: I've been looking into how to change the ProcessSlice API into one that uses raw pointers underneath. Sounds terrifying due to lots of unsafe code, but appears to be the only sound solution for the model we chose.
 * Leon: Here's a small writeup: https://gist.github.com/lschuermann/a51cbcf65f6315609361c0608452bd7e
 * Leon: We currently sort of abuse slicing in rust in that we use a slice of cells. A ReadableProcessSlice is a transparent wrapper around a slice and not a slice reference. ReadableProcessByte is a wrapper around a cell, and that's where the unsafety is because a cell would allow mutating memory and we get into issues of pointer prevalence. This structure we've chosen provides the Index trait which must return a reference to something whenever we have an indexing operation using square brackets. When we switched it over to the struct illustrated in the link, we have an actual instance of the struct lying on the stack. This makes it fundamentally incompatible with implementing the Index trait in Rust because it requires returning a reference to something. So, this makes our APIs rather inconvenient.
 * Leon: I've summarized what I think could be a solution in the "Proposed API Changes". We must implement three different methods because we can't overload them based on argument type. Slice, slice_from, slice_to, and finally an index method to get access to a ReadableProcessByte type, which is essentially the wrapper around a single byte we can access. That makes for a bit unwieldy of an API to be honest, and I'd rather not make these changes, were it not for soundness issues.
 * Leon: It gets tricky when you think about panicking/non-panicking APIs. Previously we were able to just provide non-panicking APIs next to the indexing operations, which DO panic if you specify out-of-bounds indices. But when we try to have panicking and non-panicking APIs on the same struct we have a collision of the method names. One thought I had was whether it is a good idea to provide panicking APIs at all. I asked Brad since he has experience. The point I'm trying to get at is that if we provided only non-panicking APIs, we'd shift all the panics that could happen explicitly into capsule code.
 * Leon: I have an example of what this looks like, which looks scary with all the unwraps. We might be able to optimize some of these cases where we essentially use the buffers shared with userspace as a configuration vector and index in at different offsets. A Macro could make these more elegant, but the macro_rules! are insufficient for implementing this. I could try to make this API nicer and look into procedural macros, but if that's still a no-go for us, then I'd rather not waste the time.
 * Alyssa: Why not have it do the same thing as index, where .get() can take a range, range-from, usize by having a trait bound. Then  if there's concern about overloading of the term, you can just change ProcessByte get into ProcessByte read.
 * Leon: I hadn't though of that. We should be able to bind that method on a trait and implement it.
 * Alyssa: That would simplify the API to look really similar to normal indexing.
 * Leon: Still the same issues with regards to panicking.
 * Alyssa: That's generally true. So get would be for the non-panicking API. In most cases where I'm dealing with it, I'm just going to put a question mark after each operation, so it's a little less unwieldy.
 * Leon: I have an implementation of the raw pointers lying here, but I haven't ported to all capsules yet. I could try with only the non-panicking APIs and see how messy it gets. And then report back on whether we want to invest in this and use macros to make it more comfortable.
 * Alyssa: It would make it really explicit where panicking can occur. So there is an advantage there.
 * Leon: Yes. Traditionally we've always been a bit concerned about processes exposing arbitrary data of arbitrary length to the kernel. This has always been a friction point of implicit panics in the kernel because we assume buffer sizes. So the code might look scarier, but now it's explicit unwraps.
 * Jett: I like the idea of only exposing the non-panicking versions. We don't want implicit panics based on what user code is sharing if we forget to check things. So we want that interaction really explicit and really checked.
 * Alyssa: A lot of time it can save a significant amount of code size to use the question mark operator and then panic in the outer layer as well. From what I've seen.
 * Leon: I'm just checking if people are generally open to this APIs, for now.
 * Alyssa: I like it
 * Hudson & Phil agree
 * Phil: We do want to see the implications before making a decision
 * Leon: The current capsules will be a bit ugly since I'm not refactoring. But new capsules it might be more elegant.
 * Alyssa: Can you also make sure you're using non-null and not just a raw pointer. Then you'll be able to have the same niche behavior as references. There won't be any ABI changes.
 * Leon: Makes sense
 * Phil: If the direct translation is kind of ugly, we don't necessarily have to do it for every capsule. Some are less common than others. But core capsules should be made clean. Timer, UART, GPIO, etc.
 * Leon: Do you want to do this in the step of introducing core APIs, or different PRs after the fact?
 * Phil: We should at least do one right away as a demonstration. So we can make a good decision.
 * Jett: We probably also want ok_or() rather than unwrap(). For general best practice
 * Hudson: The main problem with some of the existing capsules is that you have lots of functions where the only operation is accessing a processbuffer, which is just a panic operation doing it the easy way in the past. Refactoring for returning a result and propagating is, I'm guessing, what Leon is referring to about a re-org.
 * Leon: Right. I would like that more complex transform, but that's weeks of work.
 * Jett: We probably want to return no-mem, if you try to access something you were expecting. Maybe the next step, if we're doing this is a function that returns a Tock result, so you don't have to do ok_or(), and you can have a function just for that.
 * Leon: We've been trying to separate ABI results from internal tock kernel, because it would otherwise lead you to just always hand errors up to userspace. So an option would be a better solution.
 * Jett: Sounds good to me too

