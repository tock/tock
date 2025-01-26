# Tock Meeting Notes 2024-10-11

## Attendees

- Branden Ghena
- Amit Levy
- Leon Schuremann
- Benjamin Prevor
- Brad Campbell
- Hudson Ayers
- Kat Watson
- Johnathan Van Why
- Pat Pannuto
- Tyler Potyondy
- Chris Frantz


## Updates
### Viewing Treadmill CI Runs
 * Leon: How to see Treadmill workflows right now
    * Leon: Go to a particular merge commit from a PR. For example: https://github.com/tock/tock/commit/948bfe02936d47e38e5ebe8e2d5ba87a862e3381
    * Leon: At the top of the page to the right of "alevy authored ..." there should be a count of CI checks passed
    * Leon: Clicking that will take you to a _full_ list of CI checks. Which includes the Treadmill execute checks
    * Leon: You can hit details there for more information
### Alarm Bugs
 * Amit: Chasing down alarm bugs in libtock-c and kernel boundary. They are subtle. I think I've finally found the issue, but still working on a correct fix.
 * Amit: Between the decision to left-pad values less than 32-bit and the logic for dealing with timers that last longer than the maximum number of ticks, there was some subtlety that wasn't handled correctly
 * Amit: I have "fixed it", but I'm worried my fix breaks normal thing
 * Amit: One example bug was that while the kernel system call driver rounds up `dt` which is the time from the reference to the alarm, it fails to round down the reference properly. In result you can end up with a total expiration that is still too early. That one was more simple to fix
 * Amit: In userspace, the subtle thing is that when we iterate through outstanding alarms, we use the current counter value to do comparisons, rather than when the alarm was scheduled to fire. That is correct because now might proceed as we iterate, but on the other hand if the timer overflows and I had a long timer, all of a sudden this timer appears to not have fired. For example, if the reference is 2, firing is 1, and current time is 3. It looks like it's brand new instead of recently expired. I think we can check for this, but it needs more testing.
 * Tyler: What's your use case that's causing you to chase down these bugs?
 * Amit: I've been working on the SMA_Q3 smartwatch platform. Literally writing a clock right now with adjustments based on the GPS. I've still not really set really long wrapping timers, but I encountered a different bug where inserting ordering was wrong. Then I was writing more tests and found this too.
 * Leon: I thought my tests in the kernel covered every case. Did I miss one?
 * Amit: I suspect it was just missed. And I'll add one for it
 * Brad: Are those tests on LiteX?
 * Leon: No, they're capsule tests
 * Brad: And those have smaller sizes?
 * Leon: 32, 24, and 64. Came out of the major rewrite where I ripped out the timer logic from the userspace boundary
### Non-Volatile Storage
 * https://github.com/tock/tock/pull/4109
 * Brad: PR from UCSD for process-specific non-volatile storage. And that's starting to be important for many things, like OpenSK. Also motivated by Thread. So I've been pushing on that
### Process Debug Data
 * https://github.com/tock/tock/pull/4188
 * Brad: Inspired a while ago to see if we could make the memory usage of the process struct smaller. The low-hanging fruit was the debug information, so there's a PR to split that debug info into a trait so some, all, or none could be included. As Amit pointed out, that's another configuration option which gets pretty messy and hard to express.
 * Brad: So, now I've been looking into machinery for components to standardize configuring boards. Still iterating on this with new ideas.
 * Leon: I have been trying to make it so the core kernel logic doesn't need to care about process-specific debugging info. That created so many issues requiring generic types literally everywhere in the codebase. It's still worth pursuing disentangling process debugging and the core kernel, but this PR is still a great first step.
 * Brad: What I've found is that types are tractable as long as everything is for ProcessStandard, but not for other things. That does make different implementations more complicated, but simplifies most of our code
### Nightly Update
 * https://github.com/tock/tock/pull/4193
 * Brad: This PR has some static mut issues. Amit says they're "easy to handle" and he'll help.
 * Hudson: We could also make a partial jump if we needed, to somewhere less than here
 * Leon: The warnings in this nightly are not too bad, but they haven't gotten the point of checking board-tests, where there's a ton of mutable statics still
 * Hudson: Notably, I think warnings in tests don't block the build right now
 * Leon: Oh, that's on me to get to


## Treadmill Updates
 * https://github.com/treadmill-tb/tock/pull/2
 * Ben: I have a PR in progress that makes a testing framework for adapting existing Tock tests into Treadmill. It defines a board file and a testing framework file in Python. The Board file tells you how to get the UART and Board for programming. The framework shows the tests to run
 * Ben: The board harness has architecture, path in the Tock repo, ability to get UART port, and functions for flashing it
 * Ben: The testing framework such as the "c_hello.py" file, has a test, steps to load it on the board such as flashing the kernel and app or apps. Then for oneshot tests, which are the typical ones, they run once and collect info
 * Ben: Two types of tests, AnalyzeConsole expects output from serial ports. There's also a WaitForConsoleMessage test which instead just waits for a single message.
 * Ben: This can run on real hardware with Treadmill now. In the test-execute for Treadmill, the runner will load things, wait for a message, and then return success/failure
 * Leon: The basic idea here is that for any given test we want to combine a specification for interaction with a board with a specification for how the test works. So the board is interaction and the test is the strategy of when to flash things, when to flush buffers, when to expect output, when to use GPIO pins, etc. We can inherit parts of this to abstract away common choices here that many tests use. For example WaitForConsoleMessage is an abstraction that just takes an app and a string.
 * Leon: So the goal here was to balance expressiveness and abstracting detail away for simple tasks
 * Ben: And if we want the c_hello test, for example, on a different board, we just need a new board file, but the same test framework will apply. Separates tests and board specifics
 * Branden: What about tests that rely on certain resources such that only some boards can satisfy it?
 * Ben: Tests can require certain attributes from boards in the test
 * Leon: Maybe we need a different layer. Right now we have a board and test module combination, but the yaml file just has this fixed right now. When we have a set of boards available with varying hardware resources, we really need to make that decision before scheduling jobs that spawn resources next to hardware. Because these tests are python modules, we could add attributes to them, then in the github actions test-prepare phase, we could match boards and test attributes to come up with the testing plan
 * Branden: Yeah, that makes sense. So we'd need that list of attributes, but that can be figured out as we go
 * Ben: And the runner can double-check this too
 * Chris: This is great. I am impressed and enthusiastic here. I agree with Leon that boards should have tags about the features they support or depend on. We do something similar in OpenTitan, although we aren't yet at the detail of specific features like GPIO blocks or something because we only have the one chip right now. But we do have different kinds of execution platforms like verilator, FPGAs, and silicon chip. Within our test harness, we match platforms with known capabilities to the tests. So at the test harness level, the test can say "I require I2C" and if you try to run a test like that against a platform that doesn't support it, the test immediately errors out.
 * Chris: I think you'll get a huge benefit just off of console interactions. Many of our tests just run on the device and print out status with some common pass/fail strings. So all one has to do is write the firmware component of the tests and spit out the right codes. We also have regex matching for more complex tests. We also have another level of tests that require more than just the console, such as a GPIO test that sets up a pin mux and uses both console and GPIO. But we have the capability to use the test framework as a library and write these more complicated things
 * Chris: I think what you've described here is a very similar model. And the ability for the host side to interact with IO capabilities will be cool. This is great!
 * Leon: We have been informed by OpenTitan and other frameworks. We don't have standard pass/fail strings right now, but we should definitely add that.
 * Leon: An example of the tag matching, in treadmill-ci.yml, line 101 has a `--tag-config` which could be an arbitrary combination of tags to find a proper board
 * Chris: Cool. I think as you go down this journey of adding tests, I think a few common pass/fail codes will make configuration of the test "harness" (the component that runs on the host and interacts with the device under test). You want that piece to not vary a whole lot for the vast majority of tests. You do still want to let tests emit logging statements in addition to the pass/fail states. That'll be really helpful, and most tests can be self-reporting, without requiring the harness to change.
 * Chris: And in my OpenTitan experience, the more complex cases like GPIO or I2C interactions are composed with a UART that exchanges messages between the host and device-under-test. So for example, the host can inform the board that it's going to send an I2C transaction and the device will print back what it received. Then the harness can verify that for correctness or error.
 * Chris: If I were to give advice, focus first on the easy things which is just tests reporting status over console. That will be a LOT of value for less work
 * Leon: Thanks! That's helpful advice
 * Leon: For your info, I have been in contact with James Wainwright at lowrisc, who aren't so interested in all of Treadmill, but are interested in the central scheduler design for use with their own system. So maybe some day we can run the upstream bazel tests on this too.
 * Chris: There's CI with a bunch of tests that targets a container, the software build happens outside of the container, then the command to invoke tests goes into the container. James is the authority here on how that works though. I interact with it but haven't developed it.
 * Leon: Definitely exciting to see us moving in similar directions
 * Ben: For the upstream Tock PR, I really just need some small amount of cleanup. Probably a PR today or in the next few days
 * Amit: In other Treadmill updates, I've been using it as a local test platform. I am using Treadmill to test my Tock updates remotely, and apart from some simple ergonomic issues, it's been working great for me.
 * Leon: And we're happy to record ergonomic issues to do feedback-driven development
 * Leon: We also talked about doing a walk-through for interested users. Probably in a position to do that in the next one or two weeks.

