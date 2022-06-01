# Attendees
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Brad Campbell
- Branden Ghena
- Jett Rink
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Vadim Sukhomlinov

## Updates

- Pat: Hudson is presenting the threat model paper at EuroSys, correct?
- Amit: He already presented, is flying back.
- Pat: I'd like an update on how that was received when possible.
- Amit: I can relay what he said to me. It went pretty well, I think. EuroSys is
  not a super embedded-familiar audience, so had to answer a few questions about
  why we don't use ASLR etc.. We also got some good questions from people who
  thought the challenge of having to trust liveness is interesting. Also got
  some questions after the talk about how we compare to Redleaf and whether
  Redleaf's fault-recovery mechanisms could work for our purposes. Some
  questions about whether there is interest in developing some public collection
  of safe-Rust-only crates with other systems like Redleaf and Theseus and
  working with them on requiring safe-Rust-only app code, which is an
  interesting idea.
- Amit: Are people familiar with Redleaf and Theseus? These are two research
  operating systems for not-microcontrollers also requiring Rust.
- Alyssa: I have not heard of them.
- Branden: [Linked https://mars-research.github.io/projects/redleaf/ in chat.]
- Amit: Expect to hear from Hudson next week. I think generally the topic of an
  ecosystem of safe-only core crates is a pretty interesting one. There are
  questions on if systems are similar enough to rely on that. My very limited
  experience on looking into that is that the differences between Tock and other
  things are not just in relying on unsafe, but also in not doing general
  dynamic allocation in the kernel. Probably a lot of things that would be
  reasonable dependencies would look pretty different if you can or cannot use
  the heap. Even if that is the case, maybe there is a set of related stuff that
  is relevant to the Rust userspace that has similar safety requirements,
  generally, but is allowed to do heap allocation and looks similar to what the
  other Rust kernels are doing.

[Editor's note: The meeting agenda was empty, and there was no topical discussion]
