# Tock Meeting Notes 02/16/24

## Attendees

- Branden Ghena
- Hudson Ayers
- Leon Schuermann
- Jonathan Van Why
- Andrew Imwalle
- Tyler Potyondy
- Brad Campbell
- Philip Levis
- Pat Pannuto
- Alex Radovici


## Updates
### Certification and Unit Tests
* Alex: Started writing unit tests for Tock (in the process of certifying). We're not sure how to do this, and posted an issue looking for help/advice: https://github.com/tock/tock/issues/3854
### Board with Display
* Brad: PR on makepython-nrf52840 board with screen drivers needs reviews. Board isn't very interesting, but the capsules matter: https://github.com/tock/tock/pull/3817


## Async Process Loading
* https://github.com/tock/tock/pull/3849
* Brad: This is a refactor on process loading/checking to make it all asynchronous. It's been tricky to handle the synchronous code that exists with the rest of the asynchronous Tock stuff, specifically you don't get errors along the way, just one error for the whole thing. So the PR spells out some details on why this refactor helps
* Brad: The major change is that we currently load anything that looks like a valid TBF into a full process object, then decide if it's valid. So we committed a bunch of resources to something that might not be credentialed. So by splitting the tasks of checking and creating, we can stop it short and skip over things that are never going to be valid. This is particularly helpful on the path towards dynamic process loading, for loading new processes at runtime
* Phil: One point you made there, is the idea that you have a process that's parsed and syntactically valid from a TBF perspective, but we haven't loaded it so we don't know if it has a proper ID. If I want to check a signature, do I just check once, or each time I want to run it?
* Brad: Just once. Between parsing the binary and creating a process standard object in the processes array.
* Phil: What happens if I have two images, 1 and 2, and both of them have TBFs checked. I want to load version 1, then later want to stop it and load version 2 into that slot. Then later want to go back to version 1. Do I have to check the signature every time? Or just once? It's probably not a show-stopper, I'm just trying to understand.
* Brad: The primitive that's in the PR right now is that once a process is in the processes array, it has a valid credential. If you modify the binary, that would not be true. Other than that, the credential should still pass.
* Phil: So, three steps, checking loading and running. The credentials are part of the loading step. So if you have a shortage of process array elements, then re-checking signatures when swapping could be an issue. Although that's very hypothetical.
* Brad: There are two ways to think about that. Yes, you could end up having to recheck. But part of the implementation adds a process binary array, and that holds the object after parsed, but before loading into a full process.
* Phil: So you'd have unloaded, but parsed, binaries.
* Brad: Yes, you could do that
* Phil: So there are 4 levels: exists, parsed, loaded, running. That makes sense
* Brad: I had hoped that we could get away without parsed, but then I don't know how to do version checking. You need to hold everything you might want to run, so you can choose the best one to run.
* Phil: Something to think about for async is what are the operations that are async. It sounds like parsing is now async. Really, what's the granularity of async operations in the new approach?
* Brad: It's really just the credentials checker. Then the entire loading process happens as an async thing. You could a bunch of deferred calls, but you don't have to. There is one though, so we start on an async and can do callbacks. So a drop-in replacement could read from an external chip.
* Phil: That makes sense to me. This is a good idea. It was something I struggled with and was trying to figure out how to make it async, but the PR was so big already
* Brad: Definitely
* Brad: The other change that falls out of this: the core kernel loop treats the processes array like it did before credentials checking. Anything in that array is totally valid to execute. So all of the checking happens before the array is populated.
* Phil: That cleans up the loop. That's very nice
* Brad: Yes. That removes overhead if you don't want to do checking
* Phil: My one comment: now that process loading/checking is complicated with a four-stage state machine. It would be good to write a document describing it, as it will be totally non-obvious. What are the states, how do they transition, etc.
* Brad: Yes, good point
* Brad: Last thing, which will maybe be in this PR. What does happen if you want to dynamically swap processes. The idea that everything in the processes array is valid would no longer be true, and we'd need some way to check for uniqueness at that point. I'm still figuring that out.
* Brad: If you did a process update while the system is running, you have a new binary and would like to stop the old and load the new. But we need to make sure the uniqueness doesn't ever get violated.
* Phil: I thought you checked that it's unique before making a new process?
* Brad: Right. That was easy, but is hard in this PR. Because the kernel no longer has the checking mechanism to do that check. No reference to a checker.
* Phil: It'll probably need a reference. Whatever handles a call to transition a process will need that.
* Brad: Right now, if you only consider the boot case, you can do this once. But if there's a way to add a new process you need to do it again. And how that should work is a bit tricky.
* Phil: It just needs a reference, right? Or something else can mark a process as "has clearance to run". Which must be something that can assert uniqueness.
* Brad: Yup. I got to this stage yesterday or so. Still considering it.
* Phil: If you don't want the main loop to have a reference to the checker, you could add a new process state about whether a process is cleared as unique. And the kernel will only start those that are cleared.
* Phil: I am happy to continue to be a sounding board for this. I'm not good at tracking the github stream though. Send me an email about it please and that'll go faster.

## Signed Processes
* https://github.com/tock/tock/pull/3772
* Brad: It makes sense to have a trait per hash so we can keep track?
* Phil: Not per hash. Per signature algorithm. So you know which kind was used. And those types should define the size of the data and the contents
* Brad: Doing that elegantly doesn't seem possible right now. I might have a less elegant way to do it with a rust feature.
* Phil: Is the issue having two types?
* Brad: You can have the trait, and in theory there's just a constant attached to the trait which is the size. That's a nightly feature.
* Phil: Can't you associate a type with it?
* Brad: Yes, but not a constant one

## Libtock-C Revamp
* https://github.com/tock/libtock-c/pull/370
* Brad: I wrote a guide about how we could arrange the libtock C library to be usable but more predictable. Looking for comments there
* Hudson: I'll look into this
* Branden: I really strongly like this. I think it's a big step forward for libtock-c
* Leon: So this is mostly the status quo, but with a synchronous namespace?
* Brad: Two other things too. Requires wrappers for low-level syscalls. Second it is very prescriptive on what those names look like.


## Unit tests for Tock
* https://github.com/tock/tock/issues/3854
* Alex: We want a certified version of Tock. Needs unit tests for every single line
* Alex: But it's difficult to test free-standing functions. For example, the TBF library. When doing unit tests, we have to mock up various other functions. The only way we found to do this is configurations for testing/not testing. I'd love some thoughts on how to do this
* Alex: We want, long term, these tests to get back into Tock. I know Tock doesn't like conditional compilation
* Pat: We do have some of this already. Conditional compilation for testing was the one type we really were okay with. I think it's just in arch right now?
* Alex: We'll need it in the kernel too though. And it can't just go in the test suite, it's got to go in the main code because when we compile it for testing we have to pull in a different mocked-up crate. Something I know is giving correct or incorrect answers so I can do unit tests on a function-by-function basis.
* Hudson: Yeah, but every dependency in the kernel having a config for testing or not testing will be really ugly, right?
* Alex: That's the issue. For anything that has a trait or a generic works fine. But everything, like the kernel, that doesn't do this is not fine. We would either need to modify the kernel to take the tock TBF as a trait, which wouldn't be a code size increase at least. Or we need configure
* Hudson: Yeah, that would have it's own problem. It would add generics everywhere and explode a bit.
* Leon: Doesn't Rust support default arguments for generics? That would potentially mean we could add a limited set of generic parameters but other things wouldn't need to care?
* Alex: So we could add it to kernel resources, add a default associated type, and in testing we'd override the kernel resources?
* Leon: Yes? It's still not "nice", but at least there's an interface contract that you explicitly write out in the code. I'm still not sure this is the right solution. But it could be useful.
* Alex: We're willing to try some things and see if it works well
* Alex: Generally, we'll need to do this everywhere we use libraries. Particularly, everywhere we use Cells.
* Johnathan: That seems like a weird line. Cells seems like something that doesn't need to be stubbed. Not sure if it's a requirement.
* Alex: It's a might right now. We'll argue it's a low-level primitive, but certification may still require it
* Alex: Long-term if anyone wants to use Tock in safety-critical environments, or even IoT in EU soon, certification will be a must
* Leon: We'll take a look at Leon's suggestion though, and see how it goes

