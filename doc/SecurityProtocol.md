# Tock Security Vulnerability Handling Process

## Overview

This document outlines internal procedures for handling security vulnerabilities in Tock.

## Roles and Responsibilities

### Triage Coordinator

The Triage Coordinator (currently @charles37) responsibilities:

- Initial review of security reports within 5 days.
- Categorization and assignment of vulnerabilities.
- Coordination of response team.
- Communication with stakeholders.
- Tracking progress and ensuring resolution.

### Security Response Team

Consists of subsystem maintainers called upon to address vulnerabilities.

## Subsystem Contacts

| Subsystem  | Primary Contact    | Secondary Contact  |
| ---------- | -------------------| ------------------ |
| Kernel     | @alevy             | @alexandruradovici |
| Drivers    | @alevy             | @bradjc            |
| libtock-c  | @brghena           | @ppanuto           |
| libtock-rs | @jrvanwhy          | @hudson-ayers      |
| Build      | @ppannuto          | TBD                |
| ARM        | @ppannuto          | @alevy             |
| RISC-V     | @lschuermann       | TBD                |
| x86        | @alexandruradovici | TBD                |

## Vulnerability Handling Process

### 1. Reception and Initial Assessment (5 days)

1. All security vulnerabilities received via **security@lists.tockos.org**.
2. Acknowledge receipt and assign unique ID.
   - ID assignment: `uuidgen | cut -d'-' -f1` (example IDs: `C2C47E9A`, `97C24006`)
3. Initial assessment:
   - Severity (Critical, High, Medium, Low)
   - Affected components
   - Priority (P0, P1, P2, P3)
4. Create private GitHub Security Advisory if valid.

### 2. Assignment and Response Planning (5 days-1 week)

1. Assign vulnerability to subsystem maintainers.
2. Establish response team channel if serious.
3. Define resolution timeline by severity.
4. Notify stakeholders confidentially.

### 3. Remediation Development

- Develop and test fixes privately.
- Conduct thorough code reviews.
- Validate fix without new issues.
- Document vulnerability internally.
- Prepare disclosure notes.

### 4. Public Disclosure Preparation

- Determine disclosure date.
- Prepare comprehensive advisory:
  - Detailed description
  - Affected versions
  - Mitigation measures
  - Upgrade instructions
  - Reporter acknowledgment
- Issue security advisory.
- Prepare patched release.

### 5. Release and Disclosure

- Merge fix to main branch.
- Release patched version.
- Publish advisory.
- Notify:
  - Public security-announce mailing list
  - Reporter
  - Stakeholders
- Update public documentation.

### 6. Post-Disclosure Activities

- Monitor fix for potential issues.
- Conduct retrospective.
- Update documentation.

## Communication Templates

### Initial Acknowledgment

```
Subject: [Tock Security] Acknowledgment of Security Report #[ID]

Dear [Reporter Name],

Thank you for reporting this potential security issue to the Tock team.
We take all security reports seriously and will investigate promptly.

We have assigned this report ID: #[ID]
Please include a reference to the ID for all future communications regarding this report.

We have received your report and have begun our initial assessment.
We will keep you updated on our progress and may reach out if we need additional information.

Best regards,
[Coordinator Name]
Tock Security Team
```

### Status Update

```
Subject: [Tock Security] Status Update on Report #[ID]

Dear [Reporter Name],

We wanted to provide you with an update on the security vulnerability you reported.

Current status: [Assessment/In Development/Testing/Preparing Release]

Estimated resolution timeline: [Date]

[Additional details as appropriate]

Thank you for your patience as we work to address this issue.

Best regards,
[Coordinator Name]
Tock Security Team
```

### Disclosure Notification

```
Subject: [Tock Security] Security Advisory Publication Notice

Dear [Reporter Name],

We will be publicly disclosing the security vulnerability you reported on [Date].
The fix is included in version [Version Number], which is now available.

The CVE assigned to this issue is [CVE-ID].

We would like to acknowledge your contribution in discovering and responsibly
disclosing this vulnerability.  Please let us know if you would prefer to
remain anonymous or if you would like to be credited differently than your
submitted name.

Thank you again for helping improve the security of Tock.

Best regards,
[Coordinator Name]
Tock Security Team
```

## Tools and Resources

- GitHub Security Advisories.
- Private mailing list: **security@lists.tockos.org**
