# Tock Core Notes 2022-11-11

Attendees:
- Amit Levy
- Chris Frantz
- Hudson Ayers
- Brad Campbell
- Johnathan Van Why
- Philip Levis
- Vadim Sukhomlinov
- Leon Schuermann

# Updates
 * Phil: submitted a PR that addresses some concerns that Alyssa had about how
   userspace and kernel writes to the console can be interleaved in an order
   that is different than the order those debug statements were made.
 * Chris: I sent in a PR that details how Google's Titan chip is used in
   Google's datacenters, and describes how some of the features and planned
   features of Tock might be used in this context. I have a question about how
   this sort of document, which is not formal Tock documentation but more
   intended to illicit discussion, should be submitted
 * Brad/Amit: The place you put it (in the OT working group folder) seems
   appropriate for now, we could always move it eventually
 * Amit: The document is really great

# Significant Pull Request Rundown
 * Brad: I don't have a full list in front of me, we just had Phil and Chris
   mention two newly opened ones, and there have been some new PRs on the OT
   side of things, but not sure there is anything to discuss there.

# [PR #3318](https://github.com/tock/tock/pull/3318) Continued Discussion
 * Johnathan: Looking at the PR, there are currently three approvals out of the
   working group and a lot of unaddressed comments. I am not sure if it is too
   soon to reach out to the lowRISC legal committee, but I think I may do that today
 * Hudson (chat): What I had in mind when I added this to the agenda was going
   through all of the unresolved comments and coming to a consensus among the
   people on this call how we should resolve them.
 * Amit: I think many of the unaddressed comments are more a reflection of Pat
   being busy, and anyone else can go through and apply some of them. I can go
   through the comments though.
 * Johnathan: (Read one of his PR comments)... we can make the tool explicit
   about license headers
   added by Tock contributors, and ignore any copyrights / license notices
   already existing in files that we pull in. Pat was trying to be explicit to
   make the tool more simple, but I think it is more important that we make it
   easy to pull in code with existing licenses/copyright notices.
 * Amit: I think it is important to note that it is OK for the tool to be less
   restrictive than what the TRD specifies, the tool is just helping us to
   enforce our best practices
 * Johnathan: I think to the extent possible the TRD should document what the
   tool will enforce. But the TRD can have some ambiguity to make the tool
   work, such as in the case of being vague about exactly how near the top of a
   file a license header needs to be.
 * Phil: The TRD specifies the bounds of what you can do according to the
   project, but individuals can take a more restrictive approach. We just have
   to accept contributions that may not be as restrictive as, say, what we
   normally do, so long as those contributions meet the requirements of the
   TRD.
 * Amit: This can be tricky with a vague specification -- what if someone
   submits a contribution with a license header further down than we normally
   do, we ask them to change, and they are resistant. We can't clearly prove
   that the contribution violates the best common practice.
 * Leon: I think it is unrealistic for the first version of the tool to handle
   all the cases we will need to handle long term anyway
 * Phil: My one significant comment on the documet: I read this, and does this
   tell me what a file should look like, or what we have to do (e.g. we should
   not remove a copyright). I am not sure about the latter one, because it
   means the TRD is making a statement not about the documents themselves, but
   about actions people take.
 * Amit: Can you clarify that distinction?
 * Phil: I think there is a difference between saying "Copyrights should be at
   the top" and "you should not remove a copyright". The first is about what a
   compliant document looks like (the artifact), the second is about the
   actions that can be taken by an individual.
 * Amit: What is the takeaway here?
 * Phil: I am unsure whether this document should do the latter.
 * Leon: I consider the latter as arguably more important.
 * Johnathan: My concern with the latter is getting too close to giving legal
   advice
 * Phil: I think that is a good articulation of my concern. It is fair to say
   that we do not expect developers not to remove copyrights, but that is
   different from saying `SHOULD NOT`.
 * Amit: I am personally swayed by that argument
 * Leon: I think the current document is written using explicit RFC style
   language to perform or not perform any actions. I am indifferent as to
   whether we should be doing that.
 * Phil (chat): SHOULD NOT: This phrase, or the phrase "NOT RECOMMENDED" mean
   that there may exist valid reasons in particular circumstances when the
   particular behavior is acceptable or even useful, but the full implications
   should be understood and the case carefully weighed before implementing any
   behavior described with this label.
 * Leon: It is best for this document to not establish some new legal framework
   and instead just state best practices
 * Amit: I agree, this should be a statement of our expectations, nothing more.
 * Amit: OK, lets quickly go through the outstanding comments on the PR like
   Hudson suggested
 * Phil: Yeah I think hammering through these seems productive.
 * Amit: Should we update from "exactly one" blank line after license/copyright
   to "at least one"?
 * consensus: yes
 * Amit: Johnathan separated notices into two distinct groups, Tock project
   added notices and 3rd party notices. Do we agree that we should limit our
   formatting requirements and automated enforcement to the former group? I
   believe we do?
 * Leon: How are we going to enforce this in practice? Via an allowlist?
 * Johnathan: The tool will look for a Tock project notice at the top of the
   file in a particular format. If it does not find it it complains, if it does
   find it, it stops looking.
 * Johnathan: The document should be clear that these requirements only apply
   to Tock project notices.
 * Amit: I will make a note of that to add as a comment at the end
 * Phil: So the resolution here is that the formatting expectations are about
   the Tock project license headers/copyright notices.
 * Leon: That is nice because it means this document does not cover copyright
   enforcement
 * Johnathan: My next comments says that if another project's license header
   is similar we should be willing to accept theirs rather than nearly
   duplicating it at the top of the file
 * Amit + Phil: I basically agree with that
 * Johnathan: I think it is fair for the document to acknowledge that in cases
   when files are pulled in from other projects, the format of those notices
   might deviate slightly.
 * Amit: So we can modify license headers to make them compatible, but we do
   not need to if it is close enough?
 * Johnathan: I don't think we can modify them.
 * Leon: How will we specify which files are "Tock project files" to avoid
   adding our license headers to other project files
 * Leon: For example, if we pulled in a git submodule. Would we have a mechanism
   to make our tool aware that this might not follow our guidelines?
 * Johnathan: Google's practice is to have a directory called `third\_party`
   that covers all code that might be under different licenses. we could copy
   that practice, and the tool could ignore it. We might be able to get away
   with skipping submodules.
 * Leon: If we have any such mechanism, does this comment still apply?
 * Johnathan: Yes, because we might copy in other files directly into the main
   source e.g. driver files or vendoring code into libraries, and in those
   examples we want to preserve original headers but also have a copyright that
   reflects the possibility for continued additions by Tock contributors.
 * Amit: OK, our conclusion is that if they are different enough we would want
   to modify we should create a new license, but the tool can be liberal about
   accepting license that are close to our expectations to avoid needless
   additional headers.
 * Amit: What about files that do not support comments, e.g. markdown? The
   current distinction of code vs. not code might be ambiguous.
 * Amit: I proposed any file type that allows comments should have a license
   header
 * Hudson: How precise a category is that? Does a text file "allow" comments?
 * Johnathan: We obviously should not require people to add metadata in PNGs --
   are those comments?
 * Phil: The statement says code artifacts
 * Johnathan: That is vague
 * Johnathan: I think license headers in all textual files that allow comments
   should suffice
 * Amit: Sounds good *adds that comment to the PR on his shared screen*
 * Amit: I don't think that the license that applies to input that affects a
   binary should carry the same license as documentation. Does it even make
   sense to licenses for a README?
 * Johnathan: We didn't specify that our documentation is under a different license,
   so it is under the same license. Using a different license for our documentation
   would be weird because we copy code between our documentation and implementation.
 * Amit: *updates comments on PR to reflect earlier discussion of not using RFC
   language to specify actions of individuals*
 * Leon: My next comment is that our current suggested approach feels weird
   because it is different than what many other projects do with regards to
   putting license headers in parent-module doc comments.
 * Amit: I agree with Johnathan we should pick one or the other
 * Leon: Sure, if so I think we should be careful the ambiguity of the text
   does not allow either approach.
 * Amit: Yeah, with a 1-3 line license header it seems that should go first.
   This isn't a problem for the compiler right?
 * Leon: Yeah, Rust allows comments above module documentation (but nothing
   else)
 * Phil: Sounds like Rust thought this through!
 * Amit: I am gonna say that this is redundant since we are going to be more
   liberal with code imported from third-parties.
 * Johnathan: It sounds like we still want to follow a normal practice of
   putting the header above module documentation:
 * Amit: I will add that to the comment
 * Amit: What do we do about tools? Two questions -- first, other repos that
   are not the kernel, second, what about tools in the kernel repo (under
   `tools/`). What if someone contributes a tool with a different license
   header.
 * Amit: A tricky bit is something that is AGPL, which is fine to use as a
   tool, if it is not linked against.
 * Johnathan: AGPL stands out as one license that Google's lawyers really do
   not like, so they are very conservative about it.
 * Amit: An example is like a stack-analysis tool. We could pull it in using a
   shell script and run it, is that different than including it in the
   repository?
 * Phil: I think this is a lot easier if stuff within a particular repository
   has a very clear license.
 * Johnathan: I am really annoyed at picolibc because they removed all the
   GPL/LGPL code, but added some GPL/AGPL binaries, which still makes it hard
   to follow company policies with automated license headers. We (Google) might
   fork it and delete those 3 files.
 * Amit: Conclusion: It is easier if everything in a given repository is under
   the same/compatible license. I think this document, for now, should just
   cover Tock/tock, and we should update the language to reflect that. Is that
   OK with other people?
 * Johnathan: I will want to apply this tool to libtock-rs as well. Where
   should it live?
 * Amit: Seems fine to gradually expand the scope of this doc and tool.
 * Leon: We want to make it specific that the license must apply to these
   files, but not the formatting guidelines.
 * Amit: Agreed.
 * Hudson: I vote that the tool live in tock/tock/tools
 * Johnathan: That means the CI for other repos will have to reference the
   kernel repo, but that is not really a big deal.
 * Brad: Everyone take a look at the PR for the agenda item we did not get to.
   We will not meet next week because of Thanksgiving.
 * Johnathan: If anyone knows of any other alternatives to newlib besides
   picolibc, please let me know.
 * Vadim: Do you have a list of functions that you need from newlib? I am
   working on a bare metal C library for Ti50 to remove newlib dependency.
 * Johnathan: No I do not.
 * Phil: But that is definitely something of interest! I will try to find a
   list of what we need from newlib.
 * Amit: Regarding other options, MUSL/bionic do not work, they are too linux specific
 * Phil: We need printf.
 * Vadim: But how does it print?
 * Phil/Amit: we implement write/read/putstr, printf calls those
   implementations that we define.
 * Phil: I will look at the symbol tables to try to get a complete list of what we need from newlib
