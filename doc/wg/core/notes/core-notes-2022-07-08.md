# Tock Core Notes 2022-07-08

Attendees:
* Branden Ghena
* Amit Levy
* Alexandru Radovici
* Johnathan Van Why
* Brad Campbell
* Alyssa Haroldsen
* Pat Pannuto


## Updates
 * Brad: https://github.com/tock/tock/pull/2958
 * Brad: Hudson and I met to talk about grants. Goal to save space by refactoring grant layouts. Tried to clean up structure of grant layout as well.
 * Brad: Also we need to make sure there are no references to a grant after its left. We came up with a solution that marks "leave_grant" as unsafe, which puts the onus on the caller to make sure there are no lingering references. That PR is ready for final review.
 * Alyssa: What was the circumstance that would violate safety?
 * Brad: The kernel would have to be implemented poorly. Something in `grant.rs` where this type is created and it's maintained after the file calls `leave_grant()`. It's not something where a malicious capsule could exploit it, but instead we want the grants structured so it's really hard to make this mistake.

 * Alexandru: https://github.com/tock/tock/pull/3077
 * Alexandru: Vadim had a concern about wanting to pack multiple system calls into a single context switch. I submitted a PR that follows Leon's idea for how to do so. It's still a work in progress.
 * Branden: A technical question: a context switch causes a single system call to happen. For packed system calls, how do we "loop" back to cause the next one to occur?
 * Alexandru: When the special system call occurs, it tracks how many system calls are packed. When `switch_to_process()` is called, it checks and just executes the next system call if there is one.


## Tock History
 * Johnathan: Part of an internal history document. I was curious about the interactions between Helena, Secure Internet of Things Project (SITP), and Tock.
 * Johnathan: To start, I think SITP funded some of early Tock?
 * Amit: Only Phil would be exactly sure about that answer. Various people were working on it, and were funded in multiple ways. So it's not always clear exactly what funds were funding which person.
 * Pat: I know I had some paper that acknowledged SITP funding.
 * Johnathan: What year did Tock begin? It looked like 2014?
 * Amit: The repository that we have now started later. There was an earlier repo, which was just a prototype. So late 2014.
 * Johnathan: And Amit interned on OpenTitan once in 2016?
 * Amit: Sounds right.
 * Johnathan: Do you remember when Phil was brought on to the project at Google?
 * Amit: I don't know. Sometime after that.
 * Johnathan: One question I was answering was when Google became involved in funding it. Either SITP or Amit's internship.


## TockWorld Agenda
 * Amit: https://www.tockos.org/tockworld22/agenda (may have been modified since the discussion)
 * Amit: I sent out a proposed agenda for TockWorld. I was hoping we could use some time today to get feedback. In particular, if there are other talks that aren't included, possible remote presentations, etc.
 * Amit: To start, does this make sense?
 * Branden: One question is how talk time and discussion time splits up?
 * Amit: For 30 minute slots, I was expecting 20 for presentation and 10 for discussion. My expectation was that the presentations would partially inform later-on discussions. We have plenty of time to discuss later. So if the talks get long, we can come back to the discussion.
 * Johnathan: A comment on the OpenTitan talk. I think I can give it, but I'm not sure yet. And the other people are out of office this week.
 * Brad: We could ask Alistair to give one. He's been leading development of Tock on OpenTitan, visibly at least. We could have multiple talks on OpenTitan from different perspectives.
 * Pat: State of RISC-V would also be really good. What's needed to get the rest of that fully functional and usable.
 * Brad: Yeah, something spanning both maybe? What support they need, what the features are, roadmap.
 * Branden: I could see two OpenTitan talks: one on the goals of OpenTitan generally and one about Tock and OpenTitan together.
 * Johnathan: One concern is time to put something together. I'm limited on preparation time. We can definitely talk at a high level about it. I'm not sure how substantive it can be.
 * Alyssa: I've got similar concerns about Ti50.
 * Amit: We can defer this a little bit, sorry to put you on the spot. In both of these cases, I think there is value in the rest of us knowing a bit more, or hear again, about the projects even if at a high level.
 * Alyssa: I think we can discuss individual technical problems and some general stuff.
 * Amit: Yeah. I'm wondering if it's reasonable to talk about "What is Ti50" and "What is OpenTitan".
 * Alyssa: Yes. We do have security requirements that aren't entirely Tock's goals.
 * Amit: Okay. Also Alexandru, you have some students coming. Is there a talk outside of teaching that you or your students would like to give?
 * Alexandru: Teaching is fine. A little bit of research is fine. We also have a commercial project in mind, but I don't know if I can talk about it yet.
 * Brad: One comment I have. I think the discussions section on day one could be broadened to have one and three-year vision. Those have been really useful discussions.
 * Amit: That's great. Do you see that as a separate section or just opening up one of them?
 * Brad: I'd say rather than focusing on pain points, we could focus on where we want to go, which should hopefully address the pain points too.
 * Branden: FYI, there's also breakfast in the morning before we start. And there is lunch on day two.
 * Alyssa: What do you want to hear about Ti50?
 * Amit: What it is and how it's going for sure. Hopes and dreams and important goals for the project.
 * Branden: And where Tock is or isn't doing well at supporting the project.
 * Alyssa: Yeah. It's complicated since it's in flux and pretty agile. It's a very different development paradigm from Tock.
 * Amit: That would be great too. It doesn't need to be limited to technical stuff, and this is true for everything. As we're looking to make the community project more formalized and sustainable, there are many definitions of working and not working: documentation, outreach, governance, etc.


## Next week's call
 * Amit: We should skip next week's call since it's so close to TockWorld. 


