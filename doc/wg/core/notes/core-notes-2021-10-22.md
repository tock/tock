# Tock Core Notes 2021-10-22

## Attendees
 - Amit Levy
 - Brad Campbell
 - Gabe Marcano
 - Hudson Ayers
 - Johnathan Van Why
 - Leon Schuermann
 - Pat Pannuto
 - Philip Levis
 - Vadim Sukhomlinov
 - Alexandru Radovici

## Updates
- Hudson: PR to rust-lang/rust for panic location detail control was accepted and is going through bors.
  Should be usable downstream (w/ a new nightly) in a few days, even if its not necessarily something
  that we want to use upstream for now.
- Johnathan: I have a design for allow/subscribe that I am confident is doable for libtock-rs 2.0 -- it is the
  Pin-based design that I described in issue #338. I'm going to go ahead with that design in the interest of getting
  the library working with Tock 2.0 as soon as possible. If we come up with a better design later I am all ears, and
  we could always switch later. I plan to submit a PR with the full implementation next week.

## AppId -- Padding Credentials TLV
- Phil: Met w/ someone who is very involved in the kind of authentication needed for different embedded use cases
  centered around root of trust chips etc. One thing that came up is the ability to add / check credentials after-the-fact.
- Phil: Not absolutely sure that this is necessary to be able to add and check credentials later, but it seems
  likely that it is. So we want to be able to add a TBF object that can reserve space in it, then insert
  credentials later
- Phil: Alex had an idea that worked differently, with basically a dynamically sized list, which I am open to.
  But I am worried that it adds a lot of complexity compared to this padding approach.
- Phil: To make my approach work, I add a padding field to the credentials type. The spec clearly states that
  credentials are not included in the integrity check. So this way we just add one header type and by definition
  modifying it will not change the result of the integrity check.
- Brad: So the padding field -- would there be one TLV with multiple signatures, or that the TLV would show up multiple
  times, some with padding of 0 and a credential sig, and some with non-0 padding and including no credential signature
- Phil: Multiple TLVs -- either a TLV is all padding or its not padding and is something else.
- Vadim: Is the particular algorithm that should be used specified?
- Phil: Yeah it is specified in the PR. There are a couple now, we could add more.
- Leon: So the basic idea is to append the signature or credential, you replace the padding with a header of a different
  type that is not padding?
- Phil: Yeah, so take a padding credential that is 6kbytes long. You replace that padding credential with a 4096 bit
  RSA signature, then add a new padding credential that is 6kB - (4096/8) bytes long.
- Phil: The last paragraph of section 3 specifies this.
- Leon: Ok, that makes sense. I was worried about a certain class of attack but this seems reasonable.
- Alex: So the binary size is fixed?
- Phil: Yep, if covered by an integrity check
- Alex: Do you need to fill with 0s for the integrity check?
- Phil: no, it is unchecked
- Phil: I wanted to put in there that we could have credential TLVs which are then covered by other credentials. This
  would take the form of a new header.
- Leon: is it worth introducing that complexity now instead of needing two header types for these scenarios?
- Phil: The problem is ordering, and what you do when you don't want both headers
- Alex: Say I have a binary with a normal credential and a padding credential. The normal credential includes
  a signature over the size of the whole TBF header including the padding, and the current integrity check.
  How does this work?
- Phil: Say you are doing a hash for simplicity. Then you have a credential which has a hash in it. It is computed
  over the TBF, the full TBF object, with the exception of any credentials headers which are considered to be all 0s.
  It works the same for a signature. Note that the TBF header checksum is only over the header.
- Alex: I understand -- I thought the hash was computed over the entire TBF, but actually it considers the credential headers
  to be 0s. Sounds fine
- Phil: It is important that it is 0s, not that you skip it. We don't want to do that.
- Alex: The tradeoff is between allocating a fixed space, or using an append based approach which would require recomputing stuff
  all the time
- Pat: Flash is cheaper than memory, easy yes here
- Phil: Code size does matter!
- Leon: Only concern here is that we leave out a mechanism to say that one credential covers another, then we will need to implement
  two different headers, each with their own parser, which is bad for flash use. If we are confident we will need that, its cheaper to
  just do it now by writing one that handles both cases. Because once this is specified we can't modify it to handle this new case.
- Alex: Maybe padding can be a more general header than just tied to credentials
- Phil: Yeah I did that bc credential headers are already skipped over. We could modify the spec to say padding headers are also
  skipped over rather than considering them to be the same type.
- Phil: This sounds like a detail question, but sounds like I should just start implementing this and see what comes up.
- Phil: The broader question of whether this header should handle that use case can still be handled a bit later down the line. I
  will get started on hammering out the traits a little more and circle back in a couple weeks.

## TBF Terminology
- Phil: Yeah, in the current TBF document it talks about having a header, and there are header types, and there are also TLVs and
  elements. I want to know what we should call them. I am used to considering these things headers which contain TLVs within them
- Brad: I don't have strong feelings. I intended to not use the word header twice, bc header refers to the entire TBF header, that way
  TLVs are called something else, not also called headers, so there are fewer English word collisions.
- Phil: Right, so you are trying to distinguish the TBF header which has fields in it from the TBF header base?
- Brad: The whole thing is a header. Then there is a base which is required, and then the elements make up the header.
- Phil: Right. And by elements you mean just the optional structs? Or everything? So version is a u16 -- is it a field or an element?
- Brad: field
- Phil: but then the optional structs are elements?
- Brad: Correct
- Alex: But everyone calls the TBF elements headers...should we split the TBF into sections?
- Brad: What word for the collection of TLVs?
- Alex: What if we had a TBF file divided into headers, code, and padding sections. So the collection of headers in
  the headers section would be the individual TLV elements.
- Brad: Then what is a collection of TLVs?
- Alex: TBF section instead of TBF header
- Phil: Why are primitive types fields, but collections of primitive types are elements? Is it that they are optional? Why not call them
  optional fields?
- Brad: I guess we could say that is a base element and that would be fair.
- Alex: I assume that would be a base header and then we would have optional headers?
- Brad: That makes sense, but now that I am used to this it is always easy to talk about using context. I was just trying to avoid
  calling too many things headers.
- Alex: When I first read the TBF spec I did not figure out the difference between TLV headers and the other use of header. So I see
  what Phil is saying
- Brad: I suspect people might have different expectations here. The TBF header is a legacy of how the code evolved, so a namespace used to
  be convenient, but the names persisted. TBF header in those name refers to the fact that this is part of the TBF headers section.
- Phil: Header section / binary section / padding section. Within a header, there should just be fields.
- Brad: Then how do you refer to a TLV?
- Phil: It is a field
- Brad: So field is used for both the TLV itself, and the things in the TLV?
- Phil: No
- Brad: Then what is inside a TLV?
- Phil: Oh, I see...well what do other people do for this?
- Alex: I imagined a TLV should be called a header, and the values therein should be called fields
- Phil: So there is a TBF, then there are headers within it?
- Alex: There is a headers section, which contains several headers. These headers are actually the TLVs
- Leon: I find that confusing.
- Phil: Let me re-read this with the historical context from Brad, I understand why Brad wants different names for things
  at different levels in the hierarchy, and I will see what other people have done to address this.
- Brad: Sure. I didn't agonize over the naming at the type, so improvements are welcome.
- Alex: I am happy to help with this Phil.

## UART TRD
- Brad: UART TRD is still open. Should we be merging that?
- Phil: Yeah, I incorporated all the comments. We could merge it then implement, but I think we should implement then merge.
- Brad: That sounds fine. Just wanted to know the status.
- Phil: I get it -- I am just overloaded right now.
- Brad: Sounds good, wanted to understand who this was blocked on.

## mut_imut_buffer
- Brad: I don't see why to merge this until the RSA key stuff can use it. What if RSA ends up not using it?
- Phil: I think Alistair is being wonderfully patient. I think that if we can't come up with a better solution
  then we will have to merge it, but it feels like it goes against the Rust spirit and we should avoid it if we can.
  We have talked about it a lot in the OT call with Alistair.
- Brad: I definitely feel that we still have not quite wrapped our heads around it.
- Pat: I will leave a comment on that PR summarizing our current state on this.

## PR triage
- Pat brings up a few long standing PRs, we discuss them and where they stand, and merge a few.
