# Tock Core Notes 2021-04-09

## Attending
 - Leon Schuermann
 - Hudson Ayers
 - Pat Pannuto
 - Brad Campbell
 - Amit Levy
 - Gabriel Marcano
 - Johnathan Van Why
 - Vadim Sukhomlinov
 - Branden Ghena
 - Philip Levis

## Updates

 - Brad: Working on error standarization. Hopefully at the end should be understandable. Fairly invasive changes.
 - Pat: Student taking a fresh stab at making a lora capsule. Undergrad asking if there's anything they can do. Dry-run at hardware CI server. Supposedly the students already have RPis. Minimum viable product is to get stuff from github, flash program, verify GPIO is toggled and report that back to CI.
 - Amit: PhD student, working on process security, based on previous lightweight context research. Use case, when unwrapped a key for an endpoint (plaintext encryption for private key), switch into a context with limited access to memory and narrow interface to back out to return results, and limited access to hardware, hopefully can make a stronger argument to limit the leaking of the private key across contexts. Vadim talked about some similar work/implementation/use case leveraging drivers controlling access to specific flash pages and some other interfaces.

## Testing expectations for PRs before 2.0 release releated to release that are really big
 - Hudson: We have a a bunch of big PRs, replace return codes with Result, callback swapping, app slices, among others. A lot of changes across a lot of files. A lot of changes across many captules. Running the full testing suite might not be possible after every PR. Should we do one test before the final release? Or something else?
 - Leon: Maybe we should differentiate between massive rename PRs, and other ones adding/modifying behavior.
 - Hudson: Sounds about right. Even with Result PR, not everything was automatic, risk of error.
 - Amit: Not in favor of replicating the test suite that takes a week or so for each PR. Vote would be to not block each PR on comprehensive manual testing.
 - Philip: We talked about where testing is just people using it between alpha and subsequent work before 2.0 release. Amit remembers this also.
 - Amit: Any voices against? Cool, we're on the same page or don't care.

## Return code PRs
 - 2508, blocking other PRs
 - Philip: Wanted to talk about the PR and its implications. Good question about idiomatic vs effective Rust related to ? operator. Just wanted to get everyone on the same page before we pull the trigger, so to speak.
 - Hudson: Is the question how we use ? in the kernel?
 - Philip: What's our rough consensus how idiomatic we want to be? I think we're all on the same page, but we can have that discussion. Brad described the issues and tradeoffs pretty well.
 - Brad: Errors in Rust are a little tricky. With a lot of practice it would get pretty intuitive, but even with some experience it can be tricky and things have changed making it difficult to look stuff up online. Doesn't see a reason to be overly pedantic over this.
 - Leon: Wouldn't use ? if it doesn't make sense semantically
 - Amit: I think we are all on the same page. Reluctant to say using ? everywhere is standard Rust. Case by case basis on whether ? is the behavior that we want for some particular implementation.
 - Philip: Good distinction to make. Question is, does that matter to us (that something that's idiomatic or not)? Is idiomatic a neutral or positive attribute in Tock.
 - Hudson: For the ? operator, can't think of an example where it is possible to use it and preferable not to use it.
 - Leon: It could do the wrong thing, throwing the specific error might not make sense.
 - Amit: e.g. previously you'd do unwrap, but now you could do ?. Different behavior between unwrap and ?.
 - Leon: Perhaps enumerate all errors of the API (?)
 - Amit: Leans to idiomatic being a neutral attribute. Idiomatic code might be better optimized. Because kernel uses few external libraries, get little advantages from idiomatic code.
 - Philip: It's clear idiomatic is not a negative attribute.
 - Brad: Rust has a different philosophy on unsafe than we do. A lot of their standard API has hidden panics everywhere. Sometimes we shy away from idiomatic Rust to avoid cases where panics or unsafe code will be used.
 - Leon: Is idiomatic talking about library or language?
 - Hudson: let _ = to ignore results. For future PRs, do we want to keep doing this?
 - Leon: Doesn't think this is a good idea. let _ is greppable. There are to groups of ignored errors: 1. errors that there's no reasonable way to handle it (e.g. inside an error handler), 2. just being lazy to handle all errors. Did some fixups to UART.
 - Amit: Can you describe UART?
 - Leon: e.g. error handling in the kernel, a function returns an error. Not a lot you can do. In other cases, match on the error and return the appropriate error code to the caller.
 - Amit: Should look at places where using Result, but system is totally broken if we're returning an error.
 - Brad: Yeah, error is meaningless at that point.
 - Leon: Is there a way to mark those cases in the  code?
 - Amit: A standard comment format?
 - Leon: Yeah
 - A let underscore with a word
 - Hudson: \_word has a different meaning to the compiler than just \_
 - Brad: What does that mean for us as Tock developers?
 - Leon: We might end up with values on the stack that we're not going to use. Might.
 - Philip: Code size. If you do cover every single error, code gets bigger. When is it worth it to do full error handling, at the cost to code size? Where's that line? Maybe it is OK to panic some times.

 ## Talk about libtock-c PR
 - Brad: Passing status calls in upcalls (positive error values).
 - Hudson: have to make a bunch of changes in libtock-c. Brad made them and some other helpful changes. Makes it easier to write drivers. A lot of changes.
 - Amit: How can we divide up the review? Per example, or per driver?
 - Hudson: Should ask who has time this week.
 - Leon: Split up by driver and check all examples that use the driver.
 - Amit: That seems harder to split up.
 - Hudson: Most apps would use print driver and GPIO.
 - Amit: Not trivial to find all drivers being used.
 - Leon: No strong feelings about it. Could go by examples too.
 - Hudson: For examples that use a single driver that makes sense.
 - Amit: Can do review,
 - Hudson, Leon also, this weekend.

## Other things
 - Merged PR 2508, multiple people in favor of doing so.

## Phil's TRD (PR 2431)
 - Philip: Hoping someone that hasn't been involved in writing it to do a final careful readthrough
 - Pat was hoping to have time to do it early next week.
 - Philip: Amit did a good job last time, could do it again?
 - Amit: Yeah.

## Leon's PR on Tock registers using traits
 - Leon: PR opened about tock registers using traits. If people can take a look at it, core ideas not hard, do we want this, do we not want this? Goal is to make traits for all kinds of traits, just to reduce code duplication. Make it easier to implement register types.
 - This is part of the register crate
Amit: Tap person using RPi for OS
