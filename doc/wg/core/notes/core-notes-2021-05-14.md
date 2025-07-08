# Tock Core Notes 05-14-2021

## Attending
 - Pat Pannuto
 - Amit Levy
 - Leon Schuermann
 - Johnathan Van Why
 - Brad Campbell
 - Hudson Ayers
 - Vadim Sukhomlinov
 - Alexandru Radovici
 - Philip Levis
 - Gabe Marcano

 ## Updates
 - Hudson: I merged master into the callback swapping PR, it was a pretty hairy
   merge, so would appreciate another set of eyes on the kernel crate.
 - Phil: Students at Stanford still working on code size, not a lot of
   progress, but digging into a lot of strings that are pretty strange.
 - Amit: Do the strings make sense?
 - Phil: Some of them -- others are a lot of repeated special characters -- we
   are digging into this.

 ## AppSlice Aliasing / Soundness
 - Amit: Leon shared in the ML a document that pretty thoroughly laid out the
   options available to us for this issue, and put it on the agenda today.
 - Leon:  I put this on the agenda because I am not sure about what the best
   next steps are.
 - Leon: A brief summary is that aliasing of buffers shared from userspace
   processes can violate the assumptions that Rust makes leading to soundness
   bugs. The ML lays out several potential solutions to this issue and some of the
   associated tradeoffs.
 - Leon: In particular, I think we got some really interesting feedback from
   Miguel on the mailing list, as he confirmed that using an immutable
   reference to a slice of cells should allow for us to manage these memory
   regions safely/soundly.
 - Leon: I did not want to go into the deep technical details, as it can get
   pretty confusing especially for a call this large. I may try to organize a
   smaller call of people who are deeply involved in this particular issue to hash
   out some of these low level technical details. However I did think that a
   discussion of how we should approach the larger technical issue would be
   appropriate for today's call.
 - Phil: I would like to participate in that technical call
 - Amit: Yeah same
 - Amit: Broadly it seems that I think we agree that all of these approaches
   would be safe / solve the soundness issue. So the question becomes: "What is
   the right interface from the performance/ergonomics/flexibility perspective?"
 - Leon: Agreed, and those considerations include the runtime cost of accessing
   a buffer from within a capsule, and the runtime cost of allowing a buffer,
   and the runtime cost of using a buffer that has been accessed.
 - Amit: Also a consideration is the effort required to port all system call
   capsules, as some of these solutions (namely the slice-of-cells approach)
   means we can no longer use appslices as regular rust slices.
 - Leon: Fortunately in most places we do not use rust slices as regular rust
   slices. So this may be a little less effort than you imagine, but I need to
   investigate this more cleanly.
 - Amit: So there would be a helper function or something that would replace
   copy_from_slice
 - Leon: Yes exactly. Also we can no longer use the normal indexing operations
   for manual copying etc.
 - Amit: If the ergonomic overhead of using slice-of-cells is relatively small,
   is there some other reason that approach is bad?
 - Leon: If we allow sharing of overlapping buffers from userspace now, we
   probably can't go back on it in the future. Whereas the opposite is not
   true.
 - Hudson: I think one other downside is that it makes it impossible to share a
   buffer from userspace and use that buffer to pass something down across a
   HIL for a synchronous write to hardware without a copy.
 - Hudson: Notably, for any HIL that is virtualized that copy is already
   required, so maybe this is just an edge case
 - Phil: Generally speaking, even if it is possible in some cases to assume a
   synchronous implementation, it may not be others, so HILs that assume that
is possible are probably bad.
 - Hudson: I think that generally the CRC HIL may just be bad, but I think that
   generally we should consider that this makes a sync, no-copy HIL using app
buffers impossible
 - Amit: Well, not impossible -- a HIL could take in a slice of cells, or an
   enum encapsulating a slice of cells or a normal buffer -- but it certainly
   gets messier
 - Amit: So implementing either of these options seems like a lot of work --
   who is gonna do that?
 - Leon: I want to take a heat check of what people prefer and then I will
   start working on that one, and if it seems to work out okay maybe that saves
   us from having to actually implement both.
 - Amit: Yeah this seems to me like something worth digging into more depth for
   the people that are interested. For example Phil thinks that checking on
   each allow might be implementable at surprisingly low cost. We should save that
   for the separate call.
 - Leon: Sure, one quick question -- I'm gonna email the mailing list to
   organize that meeting, should I include the helena-project list?
 - Amit: Don't do both, everyone on helena-project is on the Tock ML
 - Phil: I looked at the sam4l CRC code and it is broken -- it passes the
   non-static reference to a DMA engine and tells it to go, but this is
unsound.
 - Amit: Yeah, if that was a stack allocated buffer you would definitely get
   the wrong CRC.
 - Phil: As I have mentioned this looks really similar to graphics code, and I
   have been working on some assembly code that could make this really fast.
