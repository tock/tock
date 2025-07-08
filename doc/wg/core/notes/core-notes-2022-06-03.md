# Tock Core Notes 2022-06-03

Attendees:
- Alyssa Haroldsen
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Philip Levis
- Vadim Sukhomlinov

## Updates
 * Leon: Small update from my side. I've been working on the DMA buffer story
   and the process buffer story. Trying to reconcile the two. Writing Miri tests
   to make sure the solutions are sound in the Rust sense. Implementing the
   `LeasableBuffer` abstraction we spoke about last week as part of the DMA
   buffer. Hopefully there will be more news and actual code output next week.
 * Hudson: Amit is not here but he submitted a PR this past week that addressed
   some of the concerns with `VolatileCell` that Alyssa had brought up. Alyssa,
   I hope you're happy with how that ended up?
 * Alyssa: Yes, it's much better.

## TockWorld
 * Johnathan: I wanted to check in and make sure we're ready to invite people.
 * Hudson: Yes, we are ready to send out invites. I'll send out an RSVP form to
   share. Were you planning to personally invite some of the people you asked to
   invite?
 * Johnathan: Yes, that was my plan.
 * Hudson: Okay, that makes sense.
 * Johnathan: It sounds like I should wait for the form then I can send invites.
 * Hudson: I'll paste it into chat.
 * Phil: It sounds like we are more than just ready to send out invitations --
   we can send out invitations. [ed: some emphasis was lost when transcribing to
   text]
 * Hudson: Yes
 * Hudson: Johnathan, so we don't duplicate invites, who are you planning to
   send invites to?
 * Johnathan: I was planning to invite all the invitees I suggested.
 * Hudson: Alyssa or Vadim, would you like to send invites to the other Ti50
   folks?
 * Alyssa: Yeah, I can. What do I need to get to them, logistically?
 * Hudson: There's an RSVP form, and you can add some descripton.
 * Alyssa: Can I send an open invite to the team, so that anyone who wants to
   come can come?
 * Hudson: We definitely had a desire to not have too many total people. There
   was a limit above which Google people would be unable to give talks. I don't
   want to say no, don't do that, but we didn't really talk about it.
 * [Some discussion omitted for privacy. Conclusion: Johnathan will invite the
   people he recommended inviting, Alyssa will invite Ti50 team members]
 * Hudson: Alyssa, if you want to invite more people from Ti50 that's probably
   fine but I think Amit should have final say on that and he's not here at the
   moment.
 * Alyssa: Okay
 * Branden: Logistically, we have more space, but I'm concerned that if there
   are a bunch of random people who are not really interested in Tock it may not
   be interesting to them.
 * Phil: If it isn't useful, they might note come. Another concern as somebody
   who sits on both sides, I think it's important to avoid having too many
   people from any one side. So for example, a couple more people from Ti50 --
   especially if they are working on different parts of it -- is fine, but we
   wouldn't want 50 people from Stanford to show up and drown out the
   discussion. I'm not saying a couple more people is an issue, but we don't
   want the meeting to be dominated.

## UART HIL TRD PR (#3046)
 * Hudson: A group of students at Stanford spent some time this quarter to port
   the sam4l and nrf52 chips over to the new UART HIL that Phil proposed a few
   months back. This PR is currently in draft, because it cannot pass CI until
   all the other chips and capsules are ported to use the new HIL. Only two of
   the four capsules that use UART have been ported, and there's a healthy
   collection of chips that need to be ported over. I believe that yesterday the
   TRD for the new HIL was merged. It can still change if people run into
   problems with implementations. Have not run into problem yet so that's a good
   sign. I wanted to bring this up as a call in case anyone has one of these
   chips. It would be great to get help porting them over, as unfortunately this
   has to be an atomic update porting all these over at once.
 * Phil: We do have people who are in charge of each chip, right?
 * Hudson: Yes, with the exception of the ones that are maintained by the core
   team blob.
 * Leon: How do we propose updates? Do we just push to the branch, or send PRs
   onto the branch?
 * Hudson: I asked Colin to to give us write access to the PR, so we can push
   directly. If you don't have permission, you can send a PR to the branch.
 * Hudson: Try not to be too intimidated by the amount of changes in the sam4l
   and the nrf, because those are two of the more complex UART implementations.
   For a lot of these other chips, there's no support for receive or aborting
   transmissions etc so the changes will be very simple. Most of the changes are
   around how to correctly handle aborts. For chips where aborts aren't already
   supported, changing to the new HIL should be pretty mechanical.
 * Leon: On perhaps this is a chance to properly support aborts.
 * Hudson: That's true. Given this has to be atomic, and touches enough files
   that there is significant maintenance burden, I think there is incentive to
   get a minimum change in soon. If that additional work takes a lot longer, it
   creates a lot of maintenance work for Colin or myself to do the rebasing.
 * Leon: That makes sense.
 * Phil: Hudson, can I make a suggestion? Can you -- by each chip -- put, with a
   question mark, who is nominally in charge of that chip. A question mark
   because we want to check with them. Then I can reach out to everybody and let
   them know that we want to do this sooner rather than later. If somebody says
   I can't do that, we can remove them.
 * Hudson: Sure, that's a good idea.

## `LeasableBuffer`
 * Phil: To give some context, it has to do with the digest API and how it
   interacts with `LeasableBuffer`. I'd like to propose a change.
 * Phil: *Pastes
   https://github.com/tock/tock/blob/35cfc73e2b70023a91bbd550b6416366c3ca4911/kernel/src/hil/digest.rs#L74
   into chat*.
 * Phil: For `add_data`. To give some background. Digest has 3 traits. There's
   adding data to something you're going to digest -- a mechanism for taking a
   large piece of data and turning it into a small piece of data with status and
   integrity guarantees. It takes a `LeasableBuffer`. The idea is you could add
   some big thing like a process image, and use the `LeasableBuffer` to limit
   the size of stuff you're digesting over without messing with slices. As
   Hudson pointed out, technically the implementation can use the whole slice,
   so it doesn't provide privacy, but it's an easy way to couple the three
   values down: slice, start, and end.
 * Phil: The question I have is the return type of a success/OK. What the API
   does is if you get an `Ok`, you get a value back of how much is going to be
   digested. When you get a completion callback,
 * Phil: *Pastes
   https://github.com/tock/tock/blob/35cfc73e2b70023a91bbd550b6416366c3ca4911/kernel/src/hil/digest.rs#L14
   into chat*.
 * Phil: it doesn't give you a `LeasableBuffer` back, it gives you the actual
   slice. My thought is that one of these two things should be true: either
   `add_data` should operate on the entire active region of the `LeasableBuffer`
   or it's an error, so `Ok` should be a unit type, or `add_data_done` should
   pass back the `LeasableBuffer` with the active region being the data
   remaining.
 * Phil: This idea that I call it and need to track how much it was done and
   also track where I was in the `LeasableBuffer` is weird -- tracking
   `LeasableBuffer` state and replicating it repeatedly.
 * Leon: It makes total sense. I've encountered this essentially every time I
   try to use `LeasableBuffer`. The first time I saw similar code, I was
   surprised to see you were calling `reset` or transmutting it back into a
   mutable slice in the peripheral driver, while I was passing the
   `LeasableBuffer` back. That is definitely a part which is underspecified in
   the current API. I don't have an idea for what I think a sensible API would
   look like. Closing in on the discussion we had last week on a potential DMA
   buffer, we may want to extend this API to cover the functionality of
   `LeasableBuffer`, as it is solving a lot of these issues by its design. I can
   go into more details if you'd like, but we could continue discussing the
   current `LeasableBuffer` too.
 * Phil: Yes, it sound like you and I should chat. I was trying to find a
   surgical change to make this API more consistent as opposed to a more
   significant change.
 * Phil: Do people see what the issue is?
 * Hudson: I think there are two issues. One issue is it's not clear when you
   should be calling `reset` on the buffer and our HILs don't set a clear
   precedent for how they should be `reset`. The second problem is that often
   when you're using `LeasableBuffer`, you also want to be tracking how much of
   the buffer that was initially shared has been consumed, and it's not clear
   whether `LeasableBuffer` should be updated to track the amount of the buffer
   that has been consumed or whether `LeasableBuffer` is just for the higher
   layer to specify the subset of the buffer it wants to share.
 * Leon: I think that's a precise description of what's happening here. I always
   viewed `LeasableBuffer` as a mechanism just for the client to limit what
   region of the buffer is designated for the peripheral to operate on. I fear
   that if we are also using it as a mechanism to return feedback on what the
   peripheral has actually done, it will get more confusing because you have
   influences from both sides, which are fundamentally different aspects of the
   implementation. On one hand, you want to pass a buffer which is just an
   implementation detail of us using static mutable slices which we cannot slice
   easily, and on the other we want to pass some feedback on what data the
   operation has operated on. I don't feel comfortable mixing these two
   concepts. It also has the disadvantage of losing track of what you originally
   passed to the peripheral.
 * Phil: I would disagree on that point. I think it's a type. Just as we can
   pass a bunch of integers, which mean different things depending on their use.
   One answer is that the call to `add_data` doesn't return a `usize`, and an
   `Ok` then as well as a callback which indicates a success means that the
   entire active region of the `LeasableBuffer` as passed was added. You can't
   do like a partial add and return `Ok`, which is what happens now. That way
   it's then fine to pass back the slice or `LeasableBuffer`, that's less
   important because from the caller's standpoint you can do either one from
   either.
 * Phil: I think the idea that I call and get an `Ok` and get a type, and when I
   get the callback, the `LeasableBuffer` tells me how much I have left, that is
   valid. This data structure indicates how much is remaining. I kind of prefer
   the first approach -- a successful operation means the entire active region
   was added -- because it simplifies the client code. It forces a state machine
   at the bottom for hardware that can only digest a certain number of things in
   a particular interrupt, for instance OpenTitan, but it means that you don't
   have to have that state machine a layer above.
 * Leon: I agree. I think we're still trying to solve two different problems
   here. We have the problem of hardware potentially not supporting arbitrary
   buffer sizes in a single operation, and the problem of us using mutable
   slices which we can't reslice using Rust slicing operations. If we wanted to
   ignore that latter restriction and only focus on hardware not being able to
   operate on arbitrary buffers in a single operation, it becomes immediately
   obvious as to what the solution would be. Of course we need to pass back
   additional information on what was operated on or refuse operations with too
   much data to handle at once and pass back feedback on an appropriate buffer
   size. I think it's still fine to incorporate both use cases into a single
   type, but they must be explicitly designed into the API. Could use a type
   state construction which passes in one type which describes the region which
   is active, and would convert that into another one which says "I retain this
   designated region", but additionally pass back information on what I'm
   actually working on that. Having that implicit in resizing the buffer seems
   weird to me.
 * Phil: I view it as if I pass a list. Pretend we're not in Tock land and have
   `malloc`. I pass in a list, and it gives me a list back, which is what it
   didn't do. That seems like a valid use of the list. Regardless, an easy way
   to dodge this without adding types is to say that an `Ok` means that it did
   the whole active region of the `LeasableBuffer`, and the low level driver
   just needs a state machine.
 * Leon: I think I prefer that. For exceptional cases where that's unreasonable,
   we can still pass back some form of feedback about how much was actually
   processed.
 * Phil: You mean in an error? So if you do `add_data` and it gives `Ok` but it
   can't process the whole buffer, when you signal the error case in the
   callback you tell it how much was processed?
 * Leon: Not what I was going for, but a good idea. What I was going for was if
   we have a peripheral where processing the whole buffer at once isn't
   reasonable, we can add feedback to the callback to say "I'm giving you the
   original slice back, but I only took N bytes of that".
 * Phil: That seems like a really strange edge case. You can always do things in
   software. Fundamentally, digest operations should not be limited in how much
   data they can cover. In that case, you don't use the digest trait.
 * Leon: I was just trying to make sure that even if we were to go with the
   route of saying an operation is either okay or error with an entire slice
   processed, then that doesn't prevent us from ever processing partial slices
   in the future.
 * Phil: I see.
 * Hudson: I'm w/ Leon, that sounds right to me.
 * Phil: Lets talk more about this next week. That's it for me.
