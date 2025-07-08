# Tock Core Notes of 2022-09-02

Attendees:
- Brad Campbell
- Alyssa Haroldsen
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why
- Philip Levis

## Updates

Phil: AppID

Brad: Support for parsing, adding, and removing credentials in tockloader
is in the master branch. There might be bugs, but it's working. There's
also progress in elf2tab, making it so you can build TABs with credentials
and footers. We want to make it possible for poeple to start using this
AppID ecosystem as soon as possible.

Hudson: That will be 0.11 for elf2tab?

Brad: No, I think 0.11 will be for 2.2. There are still some unknowns
on the exact formats of credentials. When those settle out and are
stable, if that's before 2.2, I don't see any reason not to do a 0.11
release.

Hudson: So people will have to pull and work out of git.

## PR Review

Hudson: There's an update to static macros, such that if the macro
is called twice, it panics rather than fail silently. Now that it calls
panic, I don't think static_buf needs to be unsafe, which means that
static_init doesn't need to be unsafe, so that is something to look at.
One issue is the extra code size for this additional function call is
1500 bytes for imix, which is significant. We could have a "safe" static
buf, which is checked, and an "unsafe" static buf, which doesn't include
this check. So that's 

Hudson: Another is 3221, converting ShortID to be an enum rather than an
Option. Also 3216, which fixes UART MUX to be able to create multiple 
MUXes, which inspired the changes to static_buf. Currently the UART MUX
component, because it called static_init locally, in finalize, a second MUX
would use the same memory which is of course unsound.

Hudson: THere's a README update for OpenTitan, which lets people use a
much newer version of OpenTitan. This is important because OT has moved to
a new build system, Bazel. This is part of a longer-term effort to update
Tock's support for OpenTitan.

Hudson: Three merged: 3191, board tiers, 3184, which defines the buzzer
HIL, and 3196, which added support for the particle boron board.

Hudson: What are the TODOs for AppID, blockers?

Brad: So one question that we had on Wednesday was is if two processes 
have the same application ID do they necessarily have the same short ID?

Phil: They do not necessarily have the same short ID.

Brad: Well, I would put that the other way and say, well if they're using the 
short ID locally unique, then they're using the app ID locally unique.

Phil: Okay, yeah, I think I think I buy it right that so basically what this means is 
that yes there is that this notion of just locally unique IDs.

Brad: Yeah, so, I guess. Okay. So there is this lingering question of if you write in 
app ID to short ID compressor, are you responsible for ensuring that if the app IDs 
are different, the short IDs are different.  And I would say yes, you are responsible for 
that and so therefore, we can just check the short IDs for uniqueness.

Phil: Yes.

Johnathan: Implication doesn't feel that way. If you have two apps AppIDs then they'll 
have different short IDs but that means you can have two apps with the same long ID.

Brad: No, that requirement is also fixed if you have the same app ID, you must have the 
same short ID and if you have different app IDs you must have different Short ID.

Phil: Also, I have two versions of the same binary version one version two. They 
might have different app IDs but they have the same short ID.

Leon: For them what, when this isn't bizarre behavior. How would I write a 
compression function which takes for instance, a signature over give them a 
binary and tries to do just to show that from that or is that just like an 
entirely unsupported use case and you should use locally unique IDs which already pulled up.
Because what this kind of tells me is that users of the boards or developers always 
responsible for ensuring that the show that the assignment reflect some properties they 
would have liked to have in the system with respect to for instant access control until 
that. What kind of make me feel like it is better to have apple IDs be assigned for 
instance statically through us for a tool to just talk to order and then store the 
headers instead of them being derived from some other.

Leon: I think I was probably just confused by the terminology of a compression. 
I remember in the sense that this is really not compressing along to a short one, 
but it's extracting a short that ID from a longer one.

Brad: There is the rule that the short ID has to be unique when running. So if you 
want to have a family of applications that work together they have to have separate short 
IDs. So you don't get the benefit of having the same access.

Phil: You would have to write it in the access rules that all of those Short IDs 
have these permissions.  For example, let's suppose that you have three processes 
that you want to all be able to work together. In some way they need to have unique 
short IDs cause they are unique applications. If you, for example wanted to make it 
that they all the same access permissions then you can structure the short ID space: 
they all have a most significant bit set to one. We talked about having the notion of 
families of identifiers and eventually said, a better way to do this is to do it this way 
and to allow the implementation to decide how it wants to structure the identifiers.

Alyssa: I'm worried a little bit about this confusing authorization and authentication.

Leon: Well, I think that it's still a well, a separate in the sense that the sort of 
these are still just identifiers, right? 

Leon: I have an application and I wanna create multiple instances of it like from 
that exact same binary. How would I come up with a mapping which assigns each of 
these instances of that exact same binary and individual engine and locally in 
each application I do Short ID.

Phil: We're so, I guess what I would I to see, can you write down this use case exactly 
what's needed cause that's part of like it's very easy to come up with. You got the 
spaces of what we might need to do is huge.

Phil: I'd like to move on, this is much more than 2 minutes. And I know that Alyssa had 
something she wants to talk about, which I think is probably more useful. This
discussion could go on for a very long time ago and so maybe it's something to push to the PR.

Hudon: Ok, yeah, Alyssa how did you want to talk about this?

Alyssa: I wanted to talk about the best way to structure static_init. I
really want niche optimization to be possible, but I can't figure out a 
way to do it. I can't figure out a way that the Option isn't all 0s.

Hudson: The most recent commit doesn't use an option.

Alyssa: On 3219? I want niche optimization to work. There's no way to
ensure that the niche is 0. There are ways to make it 0, but that disables
the optimization. Two issues: guaranteeing the safety of this, should be
able to handle thread-safety scenarios, so you need some kind of 
"once mut" that keeps track. Or you need an atomic bool, on platforms that
support it.

Hudson: How do we implement this on platfors with atomic bool, we check
this once, while ones that don't have it we know it's single threaded.

Alyssa: There are ways to communicate this with a config flag. Or I believe
they added that you can check if atomics exist.

Hudson: True, but some of our single-threaded platforms have atomics.

Aylssa: We could configure this in a central location. We could, say, have
one implementation or antoher based on whether you are single or
double threaded.

Hudson: Main motivation for multithreaded environments is for testing?

Alyssa: Yes, testing in general. So you're not writing something that
isn't actually safe.

Hudson: So some arbitrary Rust user can use it safely.

Alyssa: Whole bunch of, multiple scenarios. There needs to be some way
to communicate whether the system is thread-safe. We should not be
bundling the bool and the buffer together. due to alignmenty requirements.
We want to put the bools together, without padding on the buffers.

Hudson: I like that. The second thing is inoffensive, no problem there. The
first thing, is trying to find something that isn't non-ergonomic. Or
we could just leave it as unsafe, and if we want to transition to safe
cross that bridge when we get to it.

Alyssa: We should keep a static unchecked around.

Hudson: Keeping them makes sense. The problem for a user who doesn't want
to pay for the code size is that at some point components will have to 
make a decision.

Alyssa: All of our static_init occurs in our kernel setup function.

Hudson: Some of the helpers do it. There are some examples when there
are calls to static_init in the finalize method.

Alexandru: There are more problems than just UART.

Alyssa: Component helpers exist due to unsafe?

Hudson: Component helpers exist to make it possible for components to
be reusable without requiring everything to be initialized  in main.
It's a way to take all of this code that can't be safely contained
within a function.

Alyssa: Yeah I would just make it an unsafe function.

Brad: Short answer: allows components to work across different
chips.

Hudson: Component methods are already unsafe. It's not a safety issue.
I don't recall all of the details.

Alyssa: It's a pattern I've been trying to unpack.

Hudson: General dissatisfaction with components.

Phil: They're always getting better and are never good enough.

Brad: One of the issues around components is that we've never made
clear guarantees on what they are, what they can do, what they can't
do. So different people have different ideas on what is possible, and
that leads to issues. So we need to have clear answers for the requirements.

Hudson: Alyssa, one of the reasons why we didn't do the component is
a function that you can only call once, is that we want them to be
used multiple times.

Alyssa: Why we don't have a more egonomic API for static initialization.
Just calling static_init directly might more ergonomic.

Phil: Only call once has issues with encapsulation.

Alyssa: Could we make that a concept, a function that can only run once,
that could give you zero cost at initialization. 

Hudson: Zero size cost at least. I would be interested to see code
that might achieve that.

Alyssa: I've been hammering my head at this problem for a while.
A function that returns a singleton is not a great design.

Hudson: Component functions also include calling set_client, passing
default parameters, buffer sizes, etc.

Brad: Helper macros for components are something I added because it was
the only way I could figure out how to do it to share components across
chips. But I am not a Rust expert, and Rust has come a long way. There
might be a much better way. It's not that I said "This macro is what
we really want."

Alyssa: Primary thing is initialize static inputs, I was going to suggest
we just do that, but statically creating tuples is less efficient
than individual statics. I might make it a procedural macro that's on
the component impl, which did not exist at the time.

Hudson: It may be the case, I remember compiler errors on generic
parameters in outer scope, we went to this macro approach instead, it
might be something that's totally fixable.

Brad: Other part of static init is how components can be used multiple
times in the same board. 

Hudson: Right the issue is people put static_init in finalize and then
it's not reusable.

Alexandru: I think this is not documented anywhere, that you should not
use it in finalize.

Brad: We never said "we intend components to be used multiple times on
the same board" and so it was never a design goal. 3219 will be a big
improvement.








