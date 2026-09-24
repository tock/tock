# Tock Meeting Notes 2026-09-23

## Attendees

- Alexandru Radovici
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto

## Updates

- Leon: About two years back, after the BLE hype died down, Nordic released a
  soft device that is explicitly compatible with existing OSes and doesn't do a
  hostile takeover of the hardware. That is the new canonical soft device used
  for BLE in their products. Its interfaces look like they may work well for
  Tock. Claude got this running with Tock by only modifying the board crate.
  Added 400 lines. Gives us basic support. Another 400 line capsule allows us to
  pass it through to userspace. The code's horrible, but it demonstrates that a
  day's worth of work can get BLE working by linking against this binary in the
  kernel.
- Pat: I was aware of the new stack, but not aware of how easy it is to use.
- Leon: I'm kinda cheating. It's dynamically modifying interrupt priorities, and
  patching the interrupt table in RAM. You tell Claude to not modify the chip
  crate and e.g. it creates a bunch of hacks rather than making sensible chip
  crate edits.

## Tock 2.3 Release

- Johnathan: Do we have a list of remaining issues?
- Leon: I have been tagging PRs appropriately. A quick recap of where we stand.
  We had a lot of momentum pushing for the release. We created a 2.3 branch,
  bumped `master` to 2.4-dev. During release testing, Brad found a longstanding
  haunted issue of tail interrupts skipping system calls. There's some debate,
  but I think it's a no-brainer that it's a release blocker. Another critical
  issue with upcalls being dropped. Now `master` has driffed far from the
  release branch. We can cherrypick, but we need to do that. We should re-do
  some release testing because of how much has changed.
- Alexandru: I had another bug fix in the mpu v8.
- Leon: We should mark that with a backport label.
- Leon: Hopefully this will be our last synchronous meeting before the release,
  so last chance to voice issue.
- Alexandru: An STM board is not showing panics.
- Leon: My bar for the release, given limited testing, is not that every board
  works. We can do a patch release for that if needed. I'm more concerned about
  the all-target releases.
- Leon: I don't hear much pushback on releasing with the current PR list. Lets
  look through them.
- Johnathan: Is every PR that is a release blocker labeled
  `needs-release-backport`?
- Leon: There is `release-blocker` and `needs-release-backport`
- Johnathan: Oh, `release-blocker` is on issues.
- Leon: #5191
- Pat: Branden's comment on that PR optimization which isn't critical. But we
  can merge.
- Leon: #5193
- Pat: That one's harder to handle.
- Leon: Yeah, I don't plan to push these all through during the call. But I
  think it's blocked on Pat.
- Pat: Yes.
- Leon: It's hard for me to convince myself that our PendSV PR is correct.
- Pat: I think ARM intended for the kernel to run as the lowest-level priority
  mode, and that is the switch we should eventually do.
- Leon: Yes.
- Pat: We used to use PendSV circa 2016. If I remember correctly we switched
  because we thought it would be nice to be able to run the kernel with the MPU
  enabled, and it wasn't obvious we could use all hardware protections when
  executing in handler mode. We don't do that today mostly because v7m MPU
  configuration is annoying, but we had the idea it would be nice to be able to.
  IIRC, we can't have MPU enforcement enabled on an interrupt handler. Amit
  probably remembers the details here better than me.
- Leon: That rings a bell to me.
- Pat: You can tell the MPU to be active when you're in privileged mode, but
  handler is a different state. Handler ignores a lot of architectural state
  configuration.
- Leon: ARM is really complicated. Independent privileged stack, privilege
  modes, MPU active, etc.. Some memory-mapped registers ignore the MPU, some are
  only allowed in certain modes. It's such a mess. That's why PendSV makes me
  skittish. I feel like I don't have confidence in my understanding of the
  architecture. But you're probably the resident
  expert on it.
- Pat: I'm reading it now, just give me five minutes.
- Pat: I think I'm happy with it.
- Leon: I think we found a couple minor issues. Wording around double-apply
  isn't really correct, because it's an interrupt handler and an svc handler
  rather than two svc handler invocations. But we can fix that.
- Leon: Cortex M0+ is also v6 or is it also v7?
- Pat: It is.
- Alexandru: Can back-port it and test in on a Pi.
- Pat: Will just have to use `movs` instead of `mov`; the reduced ISA omits many
  instructions that don't set condition flags, but we don't care here.
- Leon: I would've been very confused by that. I know what to do now, it seems
  doable.
- Leon: I think just one more, #5194. I think it should be easy.
- Pat: I looked at this on my phone. I think it's fine. I'll read it quickly.
- Leon: I will work on polishing these PRs and getting them across the finish
  line. Once they're merged, I'll do the backporting, and then prompt for
  another round of release testing. That test should presumably test at least
  one ARMv6m target and one ARMv8 target.
- Alexandru: No problem, ping me when I can do that.
- Johnathan: Hopefully I'll find time to look at Treadmill sometime, and we can
  get tests in place before next release.
- Leon: I'm working on a CI document.
- Pat: Why does ARM have different documentation on their website and their PDFs?
- Leon: I usually use the PDFs.
- Pat: And LLMs are trained on the website. Asking them to quote the PDF can
  change their answer.
- Leon: `<reads example of difference>`.
- Pat: I would trust the PDF, so assume all of the CFSR bits are sticky.
