# Tock on OpenTitan Roadmap

- P0 On-Device Testing Framework
	- Coverage reporting for on-device apps and kernels.
	- Fuzz testing for on-device apps and kernels.
- P0 On-Host Testing Framework
	- Ability to run and test applications on host.
	- Coverage reporting for on-host apps and kernels.
	- Fuzz testing for on-host apps and kernels.
- P1 TBFv3/Automation for building multiple apps at static addresses
	- Build applications at fixed addresses, load them there.
- P1 Integrate DIF drivers for peripherals
	- Use DIFs for drivers when they exist, use Rust implementations in the
		far-term.
	- PLACEHOLDER: Place new peripherals into this list as they become known, so
		that they can be marked as done.
- P1 Render Aid To libtock-rs Development Efforts
	- jrvanwhy@ leading this in Tock, if we can get tasks from him to help we
		should.
	- Complete testing of libtock-rs (and libtock-c), included as part of
		CI/regression.
	- Workspace-ify the libtock-rs repo?
- P1 Investigate performance of system calls and IPC.
	- Determine where we are spending most of our time, and remediate if
		required.
- P1 Persistent Key Value Store
	- Back storage of values on to flash, or any other persistent technology,
		regardless of topology.
	- Isolation between apps, apps only allowed to read and write own key space.
		This is transparent to the apps, no knowledge of namespacing.
- P2 System Updater Application, supporting A/B update scheme.
	- Give Tock the ability to update a complete system image, not just a single
		app.
- P2 Crypto Driver Abstractions
	- Driven by use cases, but covering the RoT use case first and foremost.
	- Hardening probably occurs here inside the peripheral drivers.
- P3 Fit the entire OpenTitan Tock kernel in 20kB.
	- Aspirational target.  Do not sacrifice safety or reliability for this.
