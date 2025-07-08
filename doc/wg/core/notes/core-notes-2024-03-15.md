# Tock Meeting Notes 03/15/2024

## Attendees

- Amit Levy
- Phil Levis
- Hudson Ayers
- Alyssa Haroldsen
- Pat Pannuto
- Johnathan Van Why
- Brad Campbell
- Andrew Imwalle
- Tyler Potyondy

## Updates

Johnathan: Working on third attempt at a sound and testable tock registers design. So far it seems promising.

## TockBot Check-in
Hudson: What are people's thoughts on PRs immediately being assigned to people? Should we alter the amount of time people need to respond within.
Hudson: We had originally said three days, but there was some pushback to that. 
Amit: Generally a week seems reasonable to at least triage.
Hudson: A week seems reasonably to me too if they are assigned immediately.
Hudson: Are we at all worried that this may lead to fewer people looking at these PRs since they will already see that someone else is assigned?
Amit: I think you are right, if a new PR is assigned to someone else, I am unlikely to look at it immediately unless I have a stake in it. We should have some mechanism that is equivalent to looking at unassigned PRs if a week passes without the assigned review activity.
Pat: Could this be the bot assigning a needs triage tag?
Hudson: What would looking more automated look like?
Amit: This could look like that the bot removes the "needs triage" label once the PR is assigned.
Hudson: This does feel like the sort of thing that is entirely possible. This seems reasonable.
Amit: Is anyone against immediate assignment? 
Alyssa: This seems to generally be a good idea. Is there a ping to create a notice if there is prolonged inactivity?
Amit: I am not entirely sure, but we need to do that anyway.

## Reviewed Count Requirement
Amit: Within the documentation working group, there have been a number of PRs that seemed obvious to approve. In the case of say Brad submitting a PR in the documentation working group, it seems only one approval should be required to merge.
Amit: I propose that we step back from the two approver rule. If the person submitting the PR is someone who can approve (core working group), than the first approver can merge the PR if they feel comfortable with the PR.
Phil: Is this something on a per working group policy?
Phil: It seems documentation is smaller so fewer reviews make sense.
Amit: Agreed, I am suggesting this for now for the core wg.
Hudson: I am in support of this.
Amit: To be clear, I am proposing that the approver can merge with one approval on trivial issues.
Amit: For example, if Brad submits a PR that adds a two line change, this should not need to block on two reviewers.
Pat: I believe our policy for upkeep PRs is already one approver.
Amit: Alright, this may be a moot point then.

## RSA2048 Credential Checker https://github.com/tock/tock/pull/3445 

Brad: This is blocking on the RSA library supporting none alloc environments.
Brad: There is currently a PR to change the RSA library which, in theory, it would allow us to rewrite the library in a way that allows for using a dynamically allocated type or a statically allocated type. From there, we then could complete this PR.
Hudson: It seems we should mark this as blocked upstream for now.
Brad: Yes, agreed.
Hudson: Unfortunately, it seems adding block labels after someone is assigned, does not unassign them.
Hudson: I think we want to keep the signal of people only being assigned to PRs that are ongoing. 
Amit: That's fair.

## 64 bit timer PR https://github.com/tock/tock/pull/3343

Amit: There were a number of comments on the PR.
Hudson: Phil, it seems you had a number of comments on this PR. I can look through this again.
Phil: Yes, I will go through this again to refresh on this conversation.

## cortex-m: Add initial dwt and dcb support https://github.com/tock/tock/pull/3246
Hudson: This is listed as waiting on the author. I suspect the author is no longer working on this PR based on the last message.
Amit: This seems relatively trivial to get working since it is on the nrf52840DK board. 
Hudson: I may be able to work on this.
Amit: It seems it may be best to close this. I have however used this in the past for benchmarking.
Amit: Occasionally someone comes along to ask how to do this which having this PR to point to is helpful.
Amit: Having the PR open does not cause this to be implemented.
Amit: I think the two paths forward are one of us completing this if someone has cycles, or closing the PR.
Hudson: I have worked on similar things before and I am familiar with this. I think this is useful so I will assign it to myself and see in the next few weeks if I have time to get to this.

## STM: Improve App Programming https://github.com/tock/tock/pull/3166
Hudson: This is also waiting on the author (Alex). We should touch base with Alex on this. 
Hudson: I am of the opinion that if a PR is stale for over a year, we should close the PR.
Brad: We should close this. This makes app loading very difficult. 

## Fix Temperature Sign (bme280 / bmp280 drivers) https://github.com/tock/tock/pull/3112
Hudson: This is from Branden in 2022. It looks like the primary blocker was waiting for a breakout board with this temperature sensor. 
Amit: It also will need to be rebased.
Hudson: This seems like another example where we shouldn't assign someone to this (since it is blocked on the author).
Hudson: I am going to remove Johnathan as assigned.
Johnathan: I think it is entirely reasonable that Branden should be able to add me again once it is ready for review.
Hudson: I do not get notifications for all activity in my email, I only have activity specific to me (such as assignment) into my emails. I would prefer to only get emails when it is a PR I can do something about.
Amit: So perhaps we should have the bot send an email when you are assigned.
Hudson: Others may disagree with removing the assignee when waiting for the author since it may lead to a number of stale PRs.

## Tile on Display https://github.com/tock/tock/pull/3067
Hudson: It seems Phil was ready to merge, but it appears it blocked on merge conflict issues. It seems the original author has not been active recently.
Amit: I can work on rebasing this. You can assign this to me.
Phil: Feel free to ping me.
Brad: Displays are non trivial. There is a lot of diversity in hardware and there are implications for a number of software components we have. There is a more recent PR with an entirely new display HIL. 
Brad: Part of the issue with this PR was it was done somewhat piecemeal.
Amit: Yes, this is a very good point. Let's assign me to look into this. Perhaps the outcome is to not rebase and merge but drop this in favor of a tracking issue.

## OTA App Project https://github.com/tock/tock/pull/3068
Brad: This is still a work in progress. I guess this PR is still open as a placeholder. 
Hudson: Should we mark this as a draft or tag it as a work in progress?
Brad: I do not believe we will try to update this specific PR.
Amit: What should be the outcome then?
Hudson: It seems to me that this should be closed since you plan to replace this with a new PR.
Brad: There is a certain discoverability that is lost when contributors make PRs from their own repos. 
Amit: The PR remains, but is marked closed.
Hudson: I agree with Brad that marking this as closed makes it seem that someone is no longer working on it.
Amit: No one seems to be working on this. The last commit was a year ago.
Brad: This code is going to be a PR. 
Hudson: If this is still being worked on in the background. This should be converted to a draft.
Brad: I agree.

## Code Size Progress Report
Amit: Me and some others are trying to evaluate and characterize the code size issue. Obviously, we've heard code size is an issue in a number of cases.
Amit: When we wrote the SOSP paper, a full sensor network app was something on the order of 10-20KB. A blink app with just the kernel was under 5KB. Those were all manually optimized for the paper.
Amit: Now the vast majority of boards we have upstream are 100KB for the kernel.
Amit: The smallest are around 30KB.
Amit: We are investigating what the smallest possible Tock kernel we can achieve.
Amit: This would be a board with no apps that prints hello world to the console just in the kernel.
Amit: This is somewhat fuzzy as changes almost nothing in the kernel, but mainly fudges the chip code to remove unnecessary peripherals. 
Amit: This is the starting point for proving there is something still here to optimize.
Amit: The next goal is to do this in userspace. 
Amit: It seems promising to have minimum target high bar we can set and getting there in practice is likely going to require some software engineering to exclude unnecessary code and playing better with the LTO.
Amit: Some improvements may require more meaningful changes. For example, the nrf52840 possess a lot of peripherals. A number of these peripherals are unused an are quite expensive from a code size perspective.
Hudson: I think this is a solved issue (linked PR https://github.com/tock/tock-bootloader/pull/19/files).
Amit: Wow, I will look into this.
Hudson: I used this when writing my thesis.
Amit: Another preview as we get lower and lower down of future optimizations: the systick implementation takes ends up compiling as a u64 divide intrinsic.
Amit: We haven't tried to get rid of this yet, but this is an area to improve.
Hudson: I believe for RISC-V we did not see this since there was a different systick implementation. We manually went and edited the code in those places.
Alyssa: What is systick?
Hudson: Systick is the cortex-m peripheral used for keeping time/scheduling etc.
Alyssa: We currently have a list of symbols that we block. It goes through searching for and removing specific symbols.
Amit: That is the progress report. This remains high level for now. One thing I wanted to solicit feedback for now, is that a minimal blink application should be very small. 
Amit: I am interested in people's thoughts for examples of minimal code size applications. I think having a more fully featured would be helpful. 
Amit: I do not think full openthread support is feasible at the moment. 
Amit: Potentially a BLE test app would be good that implements a minimal feature set? 
Hudson: It sounds you are looking for a libtock-c application?
Amit: I am looking for a complete set of kernel and application targets. 
Brad: We have the HOTP demo and I am also working on a soil moisture demo.
Amit: This could be useful. Is that basically an ADC and sending that over the network? 
Brad: It will have one different apps doing Lora, BLE advertisements, and OpenThread.
