# Tock Meeting Notes 2025-04-09

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Brad Campbell
 - Benjamin Prevor
 - Hudson Ayers
 - Amit Levy
 - Pat Pannuto
 - Kat Fox
 - Viswajith Govinda Rajan


## Updates
 - Brad: Tockloader 0.14 is released now. People should update and try that out
 - Amit: There is an x86 libtock-rs PR which should be userland for the x86 kernel PR. Very exciting
 - Amit: On Treadmill, Ben has multi-board BLE tests and most of the Thread tests that Tyler was running up and running on Treadmill. That'll be a PR soon
 - Benjamin: Confirmed
 - Amit: In working on a system for SOSP, Tyler got shortcuts working for the nRF 15.4 radio which makes it quite a bit faster, which is pretty cool. Anecdotally using those was a big blocker for robust BLE support. So good side progress on wireless networking front


## Security Bug Report Process
* Amit: We've recently gotten some security-related bug reports from Vivian at UCSD who has been doing verification stuff. So that raised the issue of having a process for handling bug reports, especially those with potential security vulnerabilities and those that could affect downstream. So we thought we should formalize and document a process for that
* https://github.com/tock/tock/pull/4401
* Amit: PR 4401 is a proposal for this process
* Benjamin: In our discussions, we thought there would be two main ways to report a vulnerability. One of the tabs in the Tock repo is the security tab, which gives you a template for information to provide and discusses how to handle certain levels of critical-ness.
* Benjamin: We also made a triage role for security vulnerability reporting, which I will be doing first. Then there will be contacts for various subsystem contacts who the issue will be farmed out to. I'll contact them saying that the issue exists and placing an initial estimate of severity.
* Benjamin: If it is deemed to be a critical security issue, we'll publish a github security advisory for the repo and also send an email to a security mailing list. We'll also identify if there are any downstream users we're aware of who need to be notified early
* Brad: How do we issue a "security advisory and CVE"?
* Benjamin: Github has an option for this for a repo, I believe. For issuing a CVE, I'm not entirely sure. I'll look into that
* Amit: We need to register as a partner with the CVE org
* Leon: More anecdotally, the author of CURL has various grievances with the CVE process and has documented what to do and what not to do. They have some thoughts we should consider
* Pat: We need two separate lists for notifying of bugs and announcing bugs publicly (general agreement)
* Brad: What happens if you do not have a critical or sensitive vulnerability? Those instructions should be in this writeup too
* Benjamin: I'll add that
* Brad: And what makes something critical and sensitive versus not? Do we assume people will know?
* Benjamin: If you're already not posting it as an issue you probably know it's a security vulnerability. But we should probably have some guidance and criteria
* Brad: Why use the security tab at all? Is that more public which is why we'd use it?
* Benjamin: The security tab goes to the repo maintainers
* Leon: I had the same question. Historically, we haven't been good about hygiene about people who have the ability to contribute to the Tock repository. That's something we could more easily control with a dedicated security list
* Brad: So maybe this is something to look into. This seems overly complicated. Why not just have one method for reporting?
* Benjamin: That makes sense. It should probably be the security tab if possible since it's more central.
* Brad: It's also fine to have the email for people who aren't sure
* Branden: I might have the opposite opinion. We should use email instead. How does the Github tab work? Is that archived somewhere?
* Benjamin: Good question. Email could be better
* Amit: I'm also sympathetic to the issues of access control. Security vulnerabilities which haven't been patched and could affect deployed systems are probably more sensitive than adminstrative access to the github org
* Brad: Not clear which one (email or github) is easier to control access on
* Leon: One of the advantages of email for all its faults, is that everyone can send an email. Makes it more accessible
* Hudson: It also allows us to have a single email for security reports for both tock and userlands and all the tools
* Benjamin: We could add this SECURITY.md file which populates the Github Security tab for all our repos
* Pat: Vivian actually expected the Github Security flow to exist and was surprised it didn't. Which is what prompted this work.
* Amit: Okay. So broadly is there agreement with the high-level process? With the caveat that we'll direct people to one channel for reporting, likely email.
* Branden: This PR was made like 15 minutes ago. So I'm not really sure. I agree with what was said here
* Leon: So steps for merging this. One issue will be who the subsystem contacts are. How do we determine primary and secondary contacts for these?
* Branden: There aren't that many people who are likely to be contacts for these. Just a few of us who work on Tock a ton
* Amit: The volume will be low for now hopefully
* Leon: It's important that they feel responsible to act on these


## Explicit System Call Types
* https://github.com/tock/tock/pull/4228
* Pat: This has been on my todo queue, but personal life pulled me away from it for a few weeks now. 
* Pat: Nothing much else to discuss here. I have a path forward and I'll ping folks for review


## Dynamic Process Loading
* https://github.com/tock/tock/pull/3941
* Brad: What's the bar for moving forward here?
* Leon: I haven't gotten to address the new responses here
* Amit: I haven't written comments, but my only significant concern is that there's a lot in the kernel crate which I'm pretty skeptical of whether it needs to be in the kernel crate. However, I don't really think that should block things. We can refactor later. So I should review this and avoid that problem
* Brad: That's a perfectly reasonable question/comment. I think I disagree, but we could try to figure out. It's up to you how important you think it is
* Amit: There's a bigger refactoring design question here. If I'm right that a lot of this could go outside the kernel, then great we should do that eventually. If I'm wrong, it's not as though we don't want to include this functionality. So despite wanting things to stay minimal in the kernel as much as possible. Upon reflection, I'm convincing myself that we want the functionality anyways so it's better to have it in the kernel for now than to not have it
* Brad: I will add that I don't think there is anything other than the connection to the sequential process loader, which is in the kernel, that would tie this to the kernel crate in a way that would be hard to change in the future. It doesn't use internal APIs
* Amit: That seems roughly right
* Amit: So Leon and I can commit to giving this a look soon
* Leon: I have spent quite a bit of time reasoning through this, but that's taking too much time. I'll focus on safety issues and as long as that looks good I'll approve
* Hudson: For the TRD PR, there are open comments
* Brad: We've been ignoring that PR for now, as the bar to merge that is even less clear than for this
* Leon: The only persistent comment is the tight but non-explicit coupling between the app flash and app loader traits


## CHERI RISC-V Support Assembly
* Leon: I don't think we need to go into deep details. But we are getting to a fundamental question. We're increasingly seeing a problem where we're targeting different chips which share a ton of implementation with small minor internal details which are different. We used to have internal if statements based on constant values for the target, and we decided against that because of the difficulty of understanding what's included in the code. We also had the Cortex-M variants for adding small differences.
* Leon: CHERI is pushing this further, adding internal switching based on the target in a way that's pretty invasive for RISC-V chips. This makes testing more challenging. So we need to decide if there's a clear line we can draw, or if we have to decide on a case-by-case basis
* Amit: That's pretty general. Is there a more specific decision we're making?
* Leon: A good example is if we have two chips and one needs an additional instruction in its trap handler. In many cases Tock has duplicated the code into two functions and picked which one to use.
* Leon: This CHERI example does the opposite. It's one big assembly block with a bunch of compile-time choices for choosing which assembly to generate for different targets
* Amit: Is the question "whether if configs should be used for similar assembly"? (yes)
* Brad: And "should a macro be used to generate assembly on a per-target basis"?
* Amit: There are some examples of this in the CHERI PR. https://github.com/tock/tock/pull/4365
* https://github.com/tock/tock/blob/1f5fe2b968fd7163155c57b4f7698f9a1570e716/arch/riscv/src/syscall.rs specifically
* Leon: I'll say I'm in awe of the engineering here. But it's quite challenging to read and understand
* Brad: I see this as a question around: is Tock the minimal set of code needed to support the platforms and use cases we want, or not? For me, I don't see that the minimal set of code is a worthwhile goal. So as we get closer to hardware the variations are biggest and things are least dynamic because hardware is fixed. So code duplication makes sense there because things shouldn't change often.
* Pat: I agree with you, but I'm mindful that we did mess up the Cortex-M0 context switch in a way that we had previously fixed in the Cortex-M4 version. And that lasted for years until Vivian found it
* Amit: Okay, I think we'll have to carry on this discussion next time with more synthesized thoughts

