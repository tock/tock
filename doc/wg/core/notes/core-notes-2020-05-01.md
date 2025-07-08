# Tock Core Notes 05/01/2020

Attending:
 * Branden Ghena
 * Johnathan Van Why
 * Alistair
 * Leon Schurmann
 * Hudson Ayers
 * Amit Levy
 * Samuel Jero
 * Philip Levis
 * Brad Campbell
 * Vadim Sukhomlinov
 * Pat Pannuto
 * Garret Kelly
 
 
## Updates
 
### Tock Releases
 * Brad: released Tock 1.5 this week! Thanks to everyone who helped. Overall, no huge bugs or problems we ran into. Got it done in April too!
 * Brad: Then I merged like 10 PRs that were waiting to come out after the release. Now we're back to keeping an eye out for the next release.
 * Amit: Shift focus back to Tock 2.0 now. Is there anything people feel strongly about, that wouldn't necessarily be part of 2.0?
 * Phil: Yes. Particularly degree of testing, what needs to be carefully tested.
 * Pat: We also need to fix timers with some priority. Especially if 2.0 is going to be delayed.
 * Phil: I have a branch for that. It's just number 3 on my queue.
 
### Tock CI
 * Hudson: Leon and I have been testing github actions CI in a fork. Seems to work pretty well so far. Takes about 4 minutes right now. We'd like to start running that on the main branch. I'm going to submit a PR.
 * Brad: That's great.
 * Amit: What is the experience like and what can we expect?
 * Leon: We tried to replicate existing Travis. A few checks on the PR run in parallel. Benefits: reliable, speedy, artifacts (binaries) associated with PRs, we can also use this stuff with the planned size checks
 * Brad: With github actions, can we not run a monolithic check?
 * Hudson: Yes. We already broke it up into compiling, formatting, and checks.
 * Brad: Alistair's QEMU checks would be another good separate check.
 
### Libtock-rs
 * Johnathan: Prototype for lightweight design for libtock-rs. Plus, want to show that apps can be migrated to futures easily if desired. Looks like it's working in my testing so far. I'm writing up the investigation of possible code structures with learnings.

### HiFive RevB
* Brad: Found a hifive revB and tested Tock. It kind of does. There are going to need to be some changes though. Once again, I cannot find the bootloader source, so it's hard to know what it's doing.

## Kernel Testing
* Phil: How can we make it really easy to run kernel tests? Particularly, testing kernel APIs. Currently we have tests commented out in main.rs. You can uncomment them to run the tests. If I want to run suite of tests, there's lots of manual effort. And when testing is hard, testing doesn't get done. Want to have lots of tests that run automatically.
* Phil: Need additional boards or reset handlers or something to run these tests. Want thoughts on how we should structure and organize it. And what are the criteria for useful solutions.
* Hudson: Number 1 is that if it has new boards, that we don't have to keep them in sync.
* Phil: Fact that we only have one board usually per platform is problematic. 
* Phil: What triggered this for me were bugfixes for alarm virtualizer. Proposed fix would have introduced other bugs. We should set up a good mechanism for testing that kind of stuff.
* Amit: So a unit test like setup. Where we generate independent binaries for each test?
* Phil: I don't see another way to do it.
* Branden: Why do they have to be separate binaries?
* Phil: To keep them independent from lingering state from prior tests. Plus you could run them in parallel.
* Phil: This reinforces that we have very few boards, and not much configuration for boards.
* Amit: We could imagine a setup for one particular piece of hardware, where like the nrf52dk, we can pass in things in. The test would just set up the capsule you want to test and pass it in. Maybe we can get away with that without solving more general configuration problem.
* Phil: All the tests in one place, and you configure a board to run one, is to me not a good solution. Really want the tests to be isolated and look simple. A bunch of separate boards that are simple would be better, and easy to create lots of boards.
* Hudson: It makes it a pain: if I have 15 imix boards that each run a tests, and I change something in the kernel, then I have to propagate that change into each board.
* Phil: Yeah, but that's why you should have stable kernel APIs.
* Hudson: But load processes, for example, changed multiple times in the past month.
* Phil: Yeah, we'd need to get away from making changes that effect boards like that.
* Brad: On that point, I would not like to see what happened with components happen again. The components that were board-specific made developing PRs that touched the board folder very very difficult. Now it's much easier that they're shared in one spot. That was very difficult to manage.
* Phil: Conversation was, these are super useful and we weren't sure yet how to make them platform generic. So the pain was a forcing function.
* Amit: What do you think the result of this should be? Board configurations or futures or whatever, we ideally want to generate a kernel for a test. So is then a good place to start manually and then try to automate it?
* Phil: Yes. I do agree with Brad's comment that board-specific was a huge pain. But where we ended up was ultimately a great solution. I remember what main.rs files were like without components. So we should start writing tests for individual parts of the kernel. And you have to run the tests if you change things in the kernel. As the scope of bugs starts getting big, the probability is that any bug fix might introduce a new bug. The mechanism to fix that is through testing.
* Brad: Right now we include a test in an existing board file. So as the board changes, it includes those new things in the system as the test is run. Would that still be true for what we want to have?
* Phil: No. We want to be independent from other tests.
* Branden: Well, we want unit tests and integration tests. So another criteria is that the testing solution support both.
* Phil: Distinction between the two is fuzzy in this space. If I'm going to test SPI, I have to go through the pinmux. Testing virtual uart tests deferred procedure calls. So I think if you test high enough layers, you build up to integration tests.
* Amit: High level question is whether we can imagine testing without relying on large stack of hardware-specific drivers. Could imagine testing virtual timer by mocking out a timer.
* Johnathan: Actually need to run on real hardware. Because they depend on architecture-specific code.
* Amit: Not the stuff in capsules.
* Phil: I think you could have pure software tests. You would need very precise specifications of HIL APIs, which you could test both sides of. I could mock out SAM4L timers in software if I can specify API. I feel like we don't have that degree of precision.
* Brad: So back to original question: including tests in the full boards as they exist now, I could see a way to do that. Don't run the process console, do start whatever the test is. Doesn't test specific configurations though. Given that board main.rs is so flexible, I don't see a nice way of doing this. Not saying there isn't one. It's just not immediately obvious to me anyways.
* Amit: Maybe the answer is to just start writing some tests. And see how painful it is. One issue is which hardware it runs on.
* Brad: A good question. Lots of types of tests. Some that require a particular chip and some that are software abstractions. Maybe to start they just run on imix and nrf52
* Branden: Then we would need to run tests on them more.
* Phil: They would have to have that platform.
* Hudson: We recently had a time HIL submitted by someone with only an STM board.
* Samuel: I would be sad if the solution required you to have some specific board.
* Leon: Long term that could be CI on real hardware.
* Amit: The board being emulated would help. Take something in QEMU.
* Phil: The challenge here, with the STM example, someone proposes changes to time HIL and tests on STM board. Then to accept it, one of us have to run tests on other boards, but we can't debug the PR. So it gets to be weird. Maybe it's fine to require no changes to HILs if you only have access to one piece of hardware.
* Amit: Seems reasonable. If you're not touching things likely to mess stuff up, then merging the PR we want to check that tests pass, but don't worry about it so much. And if you're touching something like a HIL or fixing virtual timer, then we do want to encourage that, but realistically we need someone with access to hardware that we really care about to be involved anyways.
* Leon: This could prevent people from getting involved in Tock.
* Phil: To be really hardware independent, we would need to do a better job. Some things are and some aren't.
* Phil: To flip it around, instead of a Hail test and an nRF test, we really want test code that can get incorporated into the boot for any board. Some way to write a test that can get installed on any board.
* Amit: Timers are straightforward. Things that touch pins, maybe less so. And timers _are_ what we care most about right now, honestly.
* Phil: I also care about the UART. But maybe the virtualizers are the right place to start on this.
* Samuel: So part of the challenge is that the board file is which architecture and low-level details for booting and which capsules are included and how. So maybe those two should be separated.
* Phil: I'll do more thinking on this. A lot has to do with what it means to be a board.

## Tags Versus Branches for Releases
* Johnathan: Using tags for releases, there's a freeze where nothing other than bugfixes could be accepted. Now that we pushed the release, it's hard to incorporate additional bug fixes into the release. Large projects usually branch for a release. Then you have to copy fixes over, which is some amount of overhead. But maybe we should evaluate the tradeoffs.
* Leon: I was wondering about this with Tock 2.0. I've seen some changes that would be interesting, and our current approach seems to be to have the PRs hang around for a while. A branched approach for 2.0 could move forward faster.
* Brad: Yeah, for 2.0 we have to do a branch. To the broader point, I'm not sure I understand what the negative effect is other than PRs aren't merged.
* Johnathan: That and we can't do bug fixes. I suppose we could branch off the release. We just had a bunch of PRs merged. If you want to do a bugfix, you'd be pulling in them too.
* Amit: You're saying we would just have to test the patch better. The tradeoff seems to be that doing that is a reasonable amount of overhead. This is starting to look like maintaining release branches, like linux does for backports. It's good that linux does it, but there are people just to manage and patch old releases.
* Leon: I think Johnathan's point wasn't that we commit to patching old releases, but that we could. And also that we don't have a freeze around release time.
* Amit: So we would branch off when we would have originally freezed. Continue to work on master and cherry-pick commits into branch when needed.
* Johnathan: Middle ground would be a one-week freeze of master, then changes after that get cherry-picked.
* Amit: So freeze for first RC.
* Johnathan: Yeah, because lots of little changes came in right away. Then a few later came slowly.
* Brad: Current solution has negative results that encourage people to get things done in a timely manner. As for a week merge freeze, 1.5 was 11 days, so we're close already.
* Amit: The delta is, that bug fixes that should apply to 1.5, then we don't really have a way to do that now. We could create a branch, cherry pick, and tag new release.
* Johnathan: I guess this is more of a long-term concern. Freeze window could become long and problematic in the future.
* Amit: Since we do have this option, then that means we can be a bit more aggressive about doing releases. So it could not require everyone to sign off and test only the main boards. Because we know that we can do patch updates.
* Branden: That all sounds bad to me.
* Pat: We do have a distinction of stable versus experimental boards.
* Branden: I think that not requiring testing on all the platforms seems like a bad idea
* Amit: We already do that for some platforms (e.g. launchxl this release)
* Brad; I agree that we're on a bad curve where testing is increasing. Hopefully we can automate more of this so release periods are manageable.
* Branden: To reduce the freeze is there any downside to tagging off the branch?
* Brad: Well it is very cut and dry in the current version, in the branch version PRs that need to get merged to both have to be more meticulously managed
* Amit: What do you mean?
* Brad: PR would go to master, and we'd have to pull it into the branch.
* Amit: In NixOS, PRs go to master. Then there are branches for releases. The only PRs accepted for those branches are those that cherry-pick merge PRs from master.
* Brad: Is that overhead reasonable?
* Leon: I think so in this case since it's done by a bot. So only things which build on master in NixOS get pulled into release. I would say overhead is less manageable for us.
* Brad: My concern would be that a major PR goes into master. Which makes the bug fix commit that's fine against release candidate not merge into master because something else changed. Extra layer of complexity.
* Amit: Proposal, it's easy to create a branch based on the tag. Let's do that and see if anything happens. If there really are bug fixes we want on 1.5.
* Brad: We have done that in the past. 1.4.1 is a tagged branch with bugfixes.
* Amit: So what is takeaway for now?
* Johnathan: Let's try it for 1.6, that we put a cap on freeze period.

