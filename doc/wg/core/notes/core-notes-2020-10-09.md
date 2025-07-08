# Tock Core Notes 2020-10-09

## Attending
 * Branden Ghena
 * Alistair
 * Leon Schuermann
 * Amit Levy
 * Johnathan Van Why
 * Philip Levis
 * Brad Campbell
 * Pat Pannuto
 * Vadim Sukhomlinov
 * Hudson Ayers
 
## Updates
### CI Tools
 * Johnathan: libtock-rs bors is now pointed at github actions rather than travis, and we're about to remove the travis config.
 * Amit: Are we totally off of travis?
 * Hudson: libtock-c is still on it.
### 6lowpan
 * Hudson: 6lowpan through UDP now works on nRF52840 with new PR
### USB on OpenTitan
 * Phil: Issues with USB on opentitan in hardware are pernicious. So opentitan people are looking into it. Either the way we're initializing it is wrong or it's a hardware problem. But you get unmaskable interrupts that you can only stop with the global interrupt disable. Definitely some error by them, even if we're doing something wrong too.

## USB Stack
 * Phil: So Alistair did a PR for a full USB stack for CTAP on top of HID. At the same time, it turns out OpenSK had a pretty full stack that they'd never merged. So now we have two parallel somewhat different PR implementations. First we agreed to settle on a HIL. (with much patience from Alistair). Settled on a HIL and both groups reimplemented. Now we need to pick one. I did a detailed read of both and my opinion is that #2125 had some better qualities and #2094 had a better syscall interface. So why not use the high level driver from one and the low level from the other.
 * Phil: Since then, Guillaume has raised an issue about what does the API do if USB is in a bad state. USB unplugged. Or unplugged and then replugged. His most recent comment is that we should really solve this question before moving forward.
 * Phil: So, what should we do about major question, and what do we want to put into 1.6 release?
 * Brad: These different states, are they just USB stack as a whole or this particular HID HIL?
 * Phil: The USB stack as a whole
 * Phil: Here is the link to my written thoughts: https://github.com/tock/tock/pull/2094#issuecomment-705809296
 * Phil: Here is the problem Guillaume set up: https://github.com/tock/tock/pull/2094#issuecomment-706092725
 * Brad: I'm not sure why OpenSK would want to accept a new syscall interface, since presumably they have code on top of theirs already that they don't want to break. We could merge both system call APIs.
 * Phil: I think that's fine. We could have alternate capsules, but it's also good to say which one Tock things should be the default.
 * Amit: Assuming we fix the lifecycle thing, presumably that might impact the systemcall interface anyways. And OpenSK may want to make userspace robust to this as well (at least I think they would). So arguably, once all of this is fixed OpenSK may be changing userspace anyways, so a new syscall interface might not be so unpalatable.
 * Amit: So the main question is whether we should not include any of the new USB stuff in 1.6 before the USB lifecycle issues are resolved.
 * Phil: Two questions 1) approach of using #2094 syscall (where including the #2125 syscall interface in addition is a good idea)  and then the bigger one 2) what should we do about the 1.6 release given this. Should we keep this stuff out if it's not exactly finished?
 * Alistair: I'm still unsure about what the problem Guillaume pointed out was
 * Amit: In general, the HIL and both system call interfaces don't address USB lifecycle events (plug, attach, unplug, etc.). So if USB isn't plugged in at boot and remains plugged in, there's no way to address this
 * Alistair: I don't see what that matters. Shouldn't the driver do that for you? If you're a userspace app waiting to receive data and you're not plugged in, you keep blocking until data is actually available.
 * Phil: I think it would be reasonable for userspace to know if USB is attached or not, and especially if the USB connection is reset.
 * Amit: For building a U2F app, if USB is reset in the middle, what does the protocol need to do? Is there state to reset/manage in the userspace application?
 * Alistair: You shouldn't have to do anything. There are timeouts that fail transactions in the userspace application. You can reset based on those and don't need USB state.
 * Amit: So all the USB connection stuff is encapsulated in the lower level drivers.
 * Alistair: Right. So with CTAP that would work. I can't speak for everything.
 * Phil: The challenge is if I have a USB connection and am doing stuff, then the connection goes down then back up, it's not necessarily going to stop responding for a long period of time. So userspace would keep going because the timeout isn't sufficient unless the driver intentionally times out the userspace.
 * Alistair: So CTAP only ever responds to the host. It gives data to respond to the host. If the host never requests, then it will never pull the data and will timeout.
 * Phil: What happens if the app doesn't have a send pending. It's done exchanges, but isn't actively in one.
 * Alistair: For CTAP it only ever responds. So it'll get a new initialize request and just drop everything it thought it was doing.
 * Phil: So it's always waiting for requests. And so if the request that comes in is the initialize request, than the disconnect is implicit.
 * Alistair: It seems way more work to have apps manage the state of the connection.
 * Phil: I think I'm buying this. But the point wouldn't be that every app would have to deal with this, but maybe some apps might need to be able to see reset events, even if CTAP doesn't care.
 * Alistair: That's possible. There could be some HIDs that do. But why don't we just extend the HIL when something is needed rather than before we have a use case.
 * Amit: Also this interface would be a strict addition, hopefully not a revision of the existing interface.
 * Alistair: Right. You already have a cancel event. So you could just add a new one for resets. It's just an addition.
 * Phil: I think that all seems right. One caveat is that receive/send should have an additional error state for disconnected.
 * Alistair: If I'm the CTAP app, I do want to hang forever on the receive (with a yield). All I care about is the Host sending requests. It can yield forever if there is no request.
 * Phil: This has been really helpful. So the answer for does the reset need to make it to userspace is No for CTAP. So the syscall API is fine. Now the USB HIL might still want to send connected/disconnected events capsules could work on, but that's separate. We could punt on it until after 1.6 and releasing what we've got now.
 * Alistair: I agree
 * Amit: Other USB devices, like keyboard and mouse, those are also things where maybe you're not just waiting for requests, but when some hardware event happens like pressing a key, if you're not connected then the event is just lost. So who cares.
 * Alistair: And in general, USB always has the Host request data.
 * Phil: Yeah, the key thing is that a connect/disconnect must cancel a waiting transmission. So a Host initialization request shouldn't get a buffered request that was prior-to-reset sending a key. That's got to be in the capsule.
 * Alistair: Also the app has to reset its state when getting a new message. But both are doable.
 * Phil: Okay, and after 1.6 we should add connect/disconnect to the USB HIL.
 * Alistair: Adding to a very long list of things to rewrite in USB

## 1.6 Release
 * Amit: I think that ignoring USB and ignoring the Grant allocator soundness bug, that everything else for 1.6 is in and we are ready.
 * Brad: I agree
 * Leon: Do we have any boards using a 64-bit alarm in 1.6. (Yes) I just opened an issue that there is a problem with the alarm driver for 64-bit alarms. I have a 100 MHz clock, so I reach this problem quickly. https://github.com/tock/tock/issues/2143
 * Phil: I'll take a look into this. I was hampered by not having proper 64-bit hardware.
 * Amit: We do also have to decide what to do about these remaining two. The USB stack sounds like it requires some engineering work to combine these two PRs and isn't a huge deal. The soundness issue with the grant allocator might end up breaking capsule code that's doing unsafe stuff now and getting away with it. So it could be a lot of work. So do we want to block on either of these for 1.6, or should we move forward without them?
 * Alistair: I think USB still has decisions to make about what to do.
 * Hudson: I think we already decided to not block on USB for 1.6. Is that okay?
 * Alistair: I don't care if it's in 1.6. I just want it done.
 * Brad: I'm about to click go on release candidate one.
 * Phil: So do we block on merging USB until after 1.6?
 * Amit: If USB is merged between now and fixing the alarm issue, it's not going to break our tests, so we could pull it in late to 1.6 if it happens. We just don't want to block on it.
 * Phil: Even if we merge USB, it's not in boards yet, so that seems fine.
 * Amit: Sounds good. And the comfort level with this grant soundness bug is that it'll be fine to still in 1.6, and we'll fix it after that.
 * Hudson: Sounds fine
 * Brad: Works for me. I was more excited before this `iter` issue that complicates it
 * Amit: Yeah, it will definitely be quite a lot of work to do
 * Brad: Release candidate one is now tagged. Let the testing begin

## Tock 2.0 Allow Semantics
 * Phil: One of the major changes in the syscall ABI is exactly how Allow works. In 2.0 when you call allow you get a slice back (a pointer and length). So when you allow a buffer you get the previously allowed buffer back. To unallow, rather than passing NULL, you'd pass something of length zero.
 * Phil: The question is, can the capsule say no to an unallow request? Currently, it can say EBUSY and not give you your buffer back. There's some pushback about whether a capsule should be able to. A buggy/malicious capsule could steal your memory. I think it's necessary because otherwise it gets really complicated with what happens if the capsule is right in the middle of using your buffer when the app wants it back.
 * Phil: Guillaume has made the case that allow is when the copy to kernel happens, and the kernel should never actually hold app memory. I think this would end up with big mostly-unsued kernel space buffers. Because they'd need to hold a big buffer for the maximum sized allow case.
 * Phil: So the question is, are people comfortable with the idea that if an app allows a buffer to a capsule, the capsule can refuse to give it back?
 * Johnathan: I'm comfortable with it.
 * Leon: A capsule today can deny service in many way, and this is only another denial of service. Capsules should give buffers back eventually, but guaranteeing it immediately sounds tough.
 * Phil: So if there's no outstanding operation, a capsule must give it back. If you just allowed the buffer without a subscribe, then it must return it.
 * Johnathan: Sounds like a fine policy. Very hard to enforce technically.
 * Phil: Right.
 * Brad: I agree that allow has to be able to return an error when unallowing. I don't understand the idea that an app could make the best of it if the kernel is untrustworthy, but I am interested. So maybe looking more into what it would mean for the kernel to not be trusted by apps and how to be more nuanced there could be interesting questions.
 * Phil: Here it could be a bug, not a lack of trust. So it's not precisely a malicious capsule, but preventing a buggy capsule from starving you of memory would be nice.
 * Amit: Yeah, use case could be if you had an app with very small amount of memory that it is timesharing between drivers. If one driver holds on to that memory, it takes the whole app down. But it doesn't seem like a worthwhile tradeoff for the complexity.

## Grant Soundness Fix PR
 * Hudson: Last week we didn't have a fix. Amit made some changes and now we do. I ran a good number of tests on Imix and fixed everything that failed. I think we won't have a ton of failures and most of these are straightforward to fix. We really don't have many complex drivers, which is where grant `iter` or `enter` were getting called, which is what led to the soundness bug. So I think if we did merge this we wouldn't have that many problems. But waiting until after 1.6 is fine. https://github.com/tock/tock/pull/2137
 * Hudson: I want to make sure we settle on the right fix. I'm worried that the fix for `iter` right now that silently skips already entered apps, will break a lot of code. Skipping things can really break things. I think we could catch many of these really easily by panicking whenever there is a skip. But I think they might recur in the future without the panics. So I think either `iter` should always panic, or `iter` should return options so that when a grant is already entered, the caller would know it wasn't available and could handle that itself. The downside is that changing `iter` in every capsule is a pain.
 * Amit: I acknowledge those concerns. My question is if you have an interface that allows you to tell if something's already been entered, how would a capsule actually handle it? Is there any case where you want to do something other than skip over the already entered thing?
 * Hudson: I agree it's hard to think of those scenarios. But the panic occurring in the grant rather than in the capsule is rough to debug. So if the capsule needs to unwrap, then at least you can pinpoint the error quickly. And other capsules could avoid panicking but handle the error.
 * Leon: I think the timer example alone is enough to mean we have to do better.
 * Amit: Yeah, the purpose of the iterator is to be the simpler interface to use. But we have another interface, that could be improved, that is now implemented with `each` that would be an iterator over grants rather than appliedgrants. So maybe the grant type should have a method to check whether it's available to be entered.
 * Brad: Why do we need all of this complexity?
 * Amit: Because `iter` isn't transparent enough as is.
 * Brad: So if the old interface just stopped you from calling `iter` while inside the grant we'd have been fine.
 * Amit: There are multiple grant states. One is never entered and never allocated. And you don't want to allocated it. If you're looking for the next alarm, you don't want to unnecessarily allocated the grant region for apps that have never asked for an alarm and may never do so. You may want to allocate grant space when a process does ask for an alarm, which happens on first enter right now. There's also the state where something has been allocated and generally is available to enter, but is not presently because you're already entered. This was the soundness bug. Finally, there's the state where it's been allocated and you want to enter and you do.
 * Brad: I'm just questioning that the capsules should have to understand grants and their API. And they definitely should be able to make mistakes. The interface now in master just does the right thing, which is great.
 * Leon: The proposed change would make the simple case remain simple. But the proposal of iter silently dropping in the complex case could be a really subtle bug that's very hard to debug. So that would be the only added complexity for the programmer in the simple case.
 * Brad: I think the iter semantics shouldn't change.
 * Hudson: I think they must change.
 * Amit: Yes. If the grant has been entered, iter must not enter them again. Which the draft PR handles by silently not doing so.
 * Brad: I'd say that already is a substantial change. We don't want to change the number of processes you iterate over.
 * Amit: I'd be happy to not change the semantics of iter while also fixing this bug. But I'm not sure how.
 * Brad: That's what I'm asking. Do you have to be able to call iter while already in a grant.
 * Hudson: You're saying a compile-time error?
 * Brad: That would be the dream. Even if not compile time, having it error in a big way would be fine. We need an inverse capability that means you cannot call something
 * Amit: You're asking for a monad
 * Leon: I think iterating over options would be fine because the capsule can then choose how to handle it
 * Amit: So the capsule would just call map
 * Leon: And really confident capsules would just call unwrap, which would panic in the case of a bug
 * Amit: I think in practice to Brad's point, if you use iter in a reentrant way and you panic in the reentrant case, this will fail immediately.
 * Hudson: Or iter could return a result of iter or error.
 * Amit: Yes. But that's harder to implement because you'd have to iterate the processes first to see if one was entered.
 * Leon: So I propose we leave it up to the user. So one line with some clever iterators and mapping could fix this and can choose whether to ignore Nones or handle them.
 * Brad: Anything suggesting unwrap to capsule authors is a bad precedent and makes you have to think about any unwrap you see to decide if it's okay. So if iterate returns an option, you just call .map and go on your way. The option wouldn't be obviously signifying that the app is already entered. I'd have guessed it meant that the app just wasn't allocated. So I think that's a bad API.
 * Brad: What's the use case. Do apps want to ever update all grant regions except the one that they're in?
 * Hudson: They might often read from all the capsules except the one they are in. For example binding to a port requires checking all the other apps to see if they are already bound to it.
 * Leon: Or alarm does this, check if other apps should trigger sooner.
 * Hudson: The problem is that a helper function to check "is port bound" works sometimes, but then later when called in a different scenario silently fails or else maybe panics. And I'm worried that the default tests wouldn't find the error and so the panic is rare and only when doing something complex.
 * Leon: We should make this more evident in the type system. So the programmer has to handle these cases.
 * Amit: We'll have to cut off for now because we're over time. It's overall not as simple of a change as I hoped it will be and will need more iteration. We should not merge this in 1.6

