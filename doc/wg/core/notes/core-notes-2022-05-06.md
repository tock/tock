# Tock Core Notes 2022-05-06

Attendees:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

- Leon: Rebased the Rust toolchain update PR
  ([#2988](https://github.com/tock/tock/pull/2988), as the previous
  attempt at merging failed CI and has been waiting for some weeks. We
  should get that in reasonably quickly, to avoid collisions with
  other merged PRs.

- Hudson: If I recall correctly, those changes are pretty innocuous.

- Leon: Already had some approvals, shouldn't be a big deal.

## elf2tab / Tockloader / cargo Pipeline Application Credentials Integration

- Phil: *moving this to an email discussion with Brad*

## TockWorld

- Hudson: Talk about doing a TockWorld this year. People are somewhat
  able to travel again, can plan doing an in-person TockWorld at some
  university in the summer. We can talk about scheduling and location.

- Amit: We might want to clarify what a TockWorld is for people who
  have not attended the previous ones.

  It's been a while since the last one. It comprises the people on
  this call and perhaps a few more, such as ones starting to be
  engaged.

  We have used it as a focal point for some important discussions. For
  instance, last time we have used it to focus on having a Tock 2.0
  and which constraints should be integrated into the API.

  This time, there might be some different goals. The primary goal
  might be, given that Tock is growing, how to grow it in an organized
  way, not just shaped by those putting in the cycles in an ad-hoc
  way. Specifically how we can grow open source communities grown out
  of an academic project.

  Many people have not met in person, which is a change from previous
  time. We might want to invite interesting stakeholders which have
  not necessarily been actively engaged in Tock.

  A significant goal might be to develop a shared understanding in
  which directions the project should move, and as a result, what
  types of activities we should do as a community. Also, what type of
  governance would make sense going forward.

  We might have some more constraints this time given potentially two
  contributors would be arriving from Europe, and perhaps one from
  Australia.

- Phil: I thought that the last TockWorld was really successful in
  part because we had some clear agenda items regarding the major
  questions and challenges to sort out. Meeting in person is a good
  high-bandwidth way to reach a consensus.

*Personal details excluded from the meeting notes, location / date /
topic planning notes sent via Email.*

## tock-dev Google Group Mailing List

- Hudson: An issue ([#60](https://github.com/tock/tock-www/issues/60))
  was created on the `tock-www` repo stating that joining the
  `tock-dev` mailing list is not possible unless one provides a phone
  number to Google. This used to work, but it seems it no longer does.

- Brad: Did anyone tell them about Slack?

- Leon: I don't think Slack is any better than Google groups in terms
  of giving up information.

  Also, Google groups are pretty picky regarding spam filters, which
  is not great when you operate your own mail server.

- Amit: It might be that we can solve this issue for this particular
  person, but this raises some reasonable concerns. Google Groups is
  easy to setup and manage, and it has a nice interface generally, but
  there are alternatives.

  Will respond to that issue.

- Branden: It appears we can add individual people through their email
  address.

- Leon: Will reach out to the person in question.
