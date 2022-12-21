# Root-of-Trust Security Model
## Datacenter Use Cases

<p>Author: cfrantz</p>
<p>Date: 2022-11-15</p>
<p>Status: Published</p>

## Abstract

This document describes the security model for a Root-of-Trust (RoT)
chip as currently deployed for datacenter use cases.  The deployment
described represents how the current Google proprietary root-of-trust chip
(publicly known as Titan) and its firmware work.  The exact nature of the
security model continues to evolve along with Google's production security
requirements and the capabilities available from Root-of-Trust chips.

The primary purpose of the current integrations of the Titan RoT into
server products is to maintain first-instruction boot integrity and to
grant the machine a valid identity in Google's production environment
(colloquially known as _prod_). Titan RoTs also mitigate DoS attacks by
enforcing secure firmware updates.

This document concludes with some alternative concepts for firmware
delivery considering Tock's process isolation model and Application
ID features.


## Background

Google's Platforms team designs and contracts out the manufacture of
the servers and peripherals (NICs, SSDs, custom accelerators) that
comprise its production environment.  Within Google, customers (ie:
product teams such as websearch, gmail, cloud, etc) purchase compute
resources to execute their jobs.

In order for a machine to run customer jobs, it must pass a series of
health checks and acquire a cryptographic identity.  A machine which
is able to present (or wield) its identity can interact with other
production services such as the cluster scheduler, storage services or
other internal  services (such as the web index, image recognition, etc).
A machine which cannot present its identity is only permitted to interact
with a limited set of services geared towards automated or manual repair
of broken machines.

The Google Titan chip is a Google-designed Root-of-Trust chip.
It features a 32-bit embedded ARM Cortex-M, internal flash and RAM
and crypto acceleration hardware, including a SHA hashing block, a key
derivation block and a bignum accelerator.


## How Titan is Integrated into the Datacenter

The Google Titan chip is integrated into server products such that it has
control over the machine's reset process and boot firmware.  Specifically,
the Titan chip can both monitor and drive the application processor's
reset signals and interposes on the SPI bus between the Application
Processor (AP) and its EEPROM. This design is applied to simple servers
with a CPU, to serveris with Baseboard Management Controllers (BMCs),
and to peripherals like NICs and accelerators.

Titan integrations into server peripherals are very similar to the
integration into a server product: Titan is given low-level control over
the peripheral's reset signals and is positioned such that it has control
over the peripheral's boot firmware.  There are often customizations
to the integration to meet certain requirements of the peripheral (such
as boot timing), but for the purpose of this document, the integrations
are basically the same.


## Titan Code Images

Titan has 3 distinct code images. The first, called the ROM image, never
changes. The second, called the bootloader, changes rarely. The third,
called the application firmware, is a monolithic image including the
kernel, hardware support code and application code.  Generally, new
application firmware is pushed to production every few months.


## Assumptions and Requirements

Google's production infrastructure makes the following assumptions:

*  The Titan silicon boot process is secure.  Titan will only boot
   firmware signed by Google engineers authorized to use the Google Titan
   signing keys.
*  The Titan signing keys are kept secure and are only accessible to
   an appropriately authorized quorum of engineers[^1].
*  The Titan hardware correctly enforces the key/firmware version binding
   requirements.  A Titan firmware upgrade allows a secure migration path
   away from keys bound to firmware that is bad.

Requirements:

*  Datacenter machines must wield their cryptographic identity to
   join prod.  A machine without a cryptographic identity is forbidden from
   joining production and must go through the repairs flow.
*  Datacenter machines must boot from the intended boot stack in order
   to access their cryptographic identity.  The Titan firmware is the root
   link in the chain of boot firmware and software.  Titan verifies its
   own firmware and the machine's first boot firmware.
   *  Titan is not required to verify anything after the machine's
      boot firmware: If the machine's boot firmware is trusted, further
      verification of the boot stack can be delegated up the stack.
*  Signing and verification of boot-stack items must not be onerous.
   Authorized principals from the team responsible for each item in the
   boot stack should sign that item.  The public keys for each item must
   be made available to the authors of the prior stage of the boot stack.
   *  Implementation details:
      *  Most of the items in the boot stack have their keys stored in
         Google's internal key management service.  Signing is an online
         process delegated to the role accounts responsible for building
         and releasing software in Google's release automation systems.
      *  The signing keys for Titan firmware are deemed too powerful
         to be stored in any online system.  The Titan firmware keys are
         stored in an isolated system  in a secure facility.  The Titan
         signing quorum has access to the secure facility and performs an
         offline signing ceremony as part of the Titan firmware release
         process.


## Machine Boot Process

The following sequence details the per-power-cycle machine lifecycle:

1. Machine powers on; the power sequencer releases resets.  Independent
   from the power sequencer, a simple RC circuit charges and releases Titan
   from reset (typical values for the RC circuit are R = 10KΩ and C =
   0.1μF; time constant ≈ 10ms).  The power-on state of Titan's GPIOs
   are such that although the power sequencer has released resets, Titan
   is still holding the server in reset.
2. Titan exits reset and the Titan's ROM executes.  The ROM finds the
   Titan's bootloader in its internal flash.  Titan cryptographically
   verifies the bootloader against a set of keys stored in ROM.  If the
   verification passes, execution jumps to the bootloader; if verification
   fails, Titan resets itself.
3. The bootloader performs initial configuration of the cryptographic
   hardware and low-level security features of the chip (such as locking
   certain flash areas and configuring intrusion detection features).
   The bootloader then scans the internal flash looking for the Titan
   application firmware.  The application firmware is cryptographically
   verified against a set of keys stored in the bootloader. If the
   verification passes, execution jumps to the application firmware; if
   verification fails, Titan resets itself.
4. The Titan application firmware performs additional hardware
   configuration, such as configuring GPIOs and other IO peripherals.
5. The application firmware, still holding the machine in reset, scans the
   discrete SPI flash part looking for valid AP boot firmware.  Once found,
   Titan cryptographically verifies the immutable portions of the AP boot
   firmware against a set of keys stored in Titan's application firmware.
   Titan records the validity of the AP boot firmware and then releases
   the AP from reset.
6. The AP boots. In cases where the AP is a server CPU that runs multiple
   boot layers (e.g. UEFI, kernel, etc.), the CPU follows a measured boot
   flow; each layer that runs is first measured by the preceding layer. Titan
   behaves like a TPM, and only releases the machine's cryptographic identity
   if the CPU boots through its intended software.
7. The machine boots.  A full description of the server boot process
   is beyond the scope of this document.  The short version is: the BMC
   boots while holding the machine's main CPU complex in reset.  The BMC
   scans the machine's BIOS and verifies it against public keys stored in
   the BMC firmware, records the validity of the BIOS and then releases
   the main CPU complex from reset.  The main CPU boots, finds the OS,
   verifies and records its validity and boots the OS.
8. Once the machine's operating system has booted, the security daemon
   will collect the statements of validity of each of the boot stages and
   present them to Titan.  If Titan can determine that the entire boot
   stack was the intended boot stack, Titan will grant the machine access
   to its cryptographic identity and the machine will join prod.  If Titan
   cannot determine the validity of the boot stack, it will refuse to grant
   the machine access to its identity and the machine will be forced to go
   through the repairs process.
9. If at any time during machine runtime Titan detects a reset, Titan
   will hold the machine[^2] in reset and the boot process will repeat from
   step 5.
10. The Titan application firmware monitors the SPI bus during runtime
    and enforces a write-protect policy on the SPI EEPROM (the firmware
    contains a descriptor that specifies its write-protect policy, including
    protected region offsets).
11. All firmware updates must be validated by Titan -- the machine is
    not permitted to write directly to flash.


## Notable Features of the Titan Server Firmware

The following are notable features of the application firmware for the datacenter use case:

*  It attests to the validity of the machine's boot firmware from the
   first instruction.
*  It securely guards the machine's cryptographic identity; the identity
   is only granted to a machine that booted the intended boot stack.
*  It has fail-open failure modes: excepting Titan itself, any failure
   in the integrity of the boot stack still permits the machine to boot
   and restricts its access to automated repairs systems.
*  It provides machine resilience through flash write protection and
   Managed firmware updates: Titan maintains a write-protect policy over the
   EEPROM and manages firmware updates.  A feature of Titan's SPI interposer
   permits A/B firmware updates such that an update preserves the existing
   firmware as a fall-back should the new firmware fail[^3].
*  It can tie cryptographic keys to major application firmware versions.
   In the event of a serious bug being found in the Titan firmware, a new
   application firmware (major version) can be released and new keys issued
   tied to that application firmware.  Newer application firmware can access
   and wield keys tied to older firmware, but older firmware will be unable
   to access the new keys[^4]

## Tock and AppID

The migration to OpenTitan represents a radical change from how the
current datacenter Titan firmware is developed and delivered.  This change
allows us to re-examine the assumptions and requirements of the current
system and make improvements.

### Firmware Delivery

#### Today

The current datacenter Titan application firmware is a monolithic software
image.  The implementation consists of three main components: a kernel &
drivers component, a hardware integration component (referring to the
specifics of Titan's control over the server system) and a cryptographic
& identity services component.  The boundaries between these components
are somewhat blurry and there is no strong separation between the kernel
and application components.

The components of the monolithic image are maintained by different teams
within Google.  The platforms team maintains the kernel, drivers and
hardware integration components.  The prod-identity team maintains the
cryptographic & identity services component.  As one might expect, having
multiple teams contributing to a single monolithic firmware image has been
a source of complexity that has occasionally led to confusion or delays.


#### The Glorious Golden Future

The following sections of the document describe hypothetical use cases or
hypothetical modifications to existing use cases.  They assume OpenTitan
is the Root-of-Trust and adopt OpenTitan terminology.  It is assumed that
the OpenTitan chip boots securely and configures the chip appropriately
before booting the Tock kernel.

Tock provides a kernel and userspace boundary and process isolation.
Tock also permits applications to be delivered and loaded separately from
the kernel payload.  It will be possible for the datacenter firmware to
be delivered as individual components with different access controls on
each component.


*  The hardware integration component does not need access to the
   cryptographic hardware within the chip.  Its main concerns are performing
   reset control and managing access to the SPI flash chip.
*  The cryptographic services component, likewise, does not need access
   to the IO interfaces of the OpenTitan chip.

Google platforms team, as a <code><em>silicon_owner</em></code> (in
OpenTitan terminology, <em>silicon\_owner</em> is the purchaser of the
OpenTitan chip or devices containing an OpenTitan chip), can sign and
deliver the Tock kernel for datacenter integrations.  The platforms team
can also sign and deliver the hardware integration application which
provides the lowest level of machine control services for an OpenTitan
integration.

Google's production identity team can sign and deliver the cryptographic
services application which will be responsible for guarding and
maintaining the machine or subsystem identity and attesting to the
validity of its boot firmware.

These applications may be signed by distinct keys, allowing independent
operational security and release processes for the different
applications.  The application signing keys are distinct from the
kernel signing keys and each of these keys may have different key
storage requirements.  For example, the kernel key may be  considered
a high-value resource restricted to an offline Hardware Security Module
(HSM) whereas application signing keys could be considered safe enough
to be stored in Google's online key management service (because they
can be rotated as part of a new kernel release).  This separation of
authority and permissions can allow the platforms and prod-identity
teams to independently develop their respective applications as well as
sign and deliver those applications without need of an offline ceremony
(whereas, kernel upgrades or deployment of new application signing keys
would require an offline ceremony), thereby lowering the cost of feature
additions and other maintenance work.


### Delegation of OpenTitan Resources

The separation of kernel versus application signing authority also allows
for new modes of service delivery for OpenTitan-enabled platforms.

Examples:



*  A hardware peripheral in a server may wish to obtain an identity
   or keys which are tied to the server platform mainboard.  The team
   responsible for that peripheral could write their own identity-granting
   Tock application and deploy it to the motherboard's RoT (sometimes called
   a Platform Root-of-Trust or PRoT).  When in service, the peripheral can
   establish communication with the PRoT and acquire its mainboard-derived
   identity.
*  Secure logging: Server platforms (or a subset of servers deployed in
   prod) may require a cryptographic record of log messages authenticated
   by or stored inside the RoT's tamper-resistant perimeter.  An optional
   secure logging application could be deployed to the RoT that generates
   logging tokens needed by the secure logging subsystem.
*  Cloud end-customer applications: Although this is (IMHO) a bit of
   a wild use case, infrastructure providers could conceivably choose to
   sell or otherwise make available execution time on the RoT and allow
   end-customers to deploy their own Tock applications.  We will have
   to establish what sorts of Tock services and syscalls are available
   to end-customer applications, but this could potentially allow cloud
   customers the ability to perform certain operations within a secure
   element on their purchased compute resource.

<!-- Footnotes themselves at the bottom. -->
## Notes

[^1]: The exact method of securing the keys is beyond the scope of this
   document.  No engineer has unilateral access; access is granted via an
   M-of-N quorum authenticated through multiple factors.

[^2]: There are some variations or exceptions.  In all cases, Titan has
   control over the machine or peripheral's boot process.

[^3]: TL;DR: the flash part is twice the required size and Titan can
   choose which half to show to the machine.

[^4]: In the event that an attacker could force-downgrade Titan to the
   known-bad firmware, attempts to access or wield the newer keys will be
   denied by the hardware.
