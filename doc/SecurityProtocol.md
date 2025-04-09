# Tock Security Vulnerability Handling Process

## Overview

This document outlines the internal process for handling security vulnerabilities in Tock. It establishes clear procedures for managing privately reported security issues, ensuring timely responses, clear communication, and efficient resolution.

## Roles and Responsibilities

### Triage Coordinator

The Triage Coordinator (currently @charles37) is responsible for:

- Initial review of security reports within 48 hours (2 working days)
- Proper categorization and assignment of vulnerabilities
- Coordination of the response team
- communication with stakeholders
- Tracking progress and ensuring timely resolution

### Security Response Team

The Security Response Team consists of maintainers with expertise in different subsystems who can be called upon to address security vulnerabilities.

## Subsystem Contacts

| Subsystem | Primary Contact | Secondary Contact |
| --------- | --------------- | ----------------- |
| Kernel    | TBD             | TBD               |
| IPC       | TBD             | TBD               |
| MPU       | TBD             | TBD               |
| Drivers   | TBD             | TBD               |
| Apps      | TBD             | TBD               |
| Libraries | TBD             | TBD               |
| Hardware  | TBD             | TBD               |
| Build     | TBD             | TBD               |

## Vulnerability Handling Process

### 1. Reception and Initial Assessment (0-48 hours)

1. Critical security vulnerabilities are received via security@lists.tockos.org, Less critical security vulnerabilities are received via the [Report a vulnerability](https://github.com/tock/tock/security/advisories/new) tab on GitHub
2. The Triage Coordinator acknowledges receipt
3. Initial assessment is performed to validate the report and determine:
   - Severity (Critical, High, Medium, Low)
   - Affected components
   - Priority (P0, P1, P2, P3)
4. Create a private GitHub Security Advisory if the report is valid

### 2. Assignment and Response Planning (48-72 hours)

1. Assign the vulnerability to appropriate subsystem maintainers via the chart below
2. Establish a communication channel for the response team if deemed serious enough.
3. Define a timeline for resolution based on severity:
   - Critical (P0): Target resolution within 7 days
   - High (P1): Target resolution within 14 days
   - Medium (P2): Target resolution within 30 days
   - Low (P3): Target resolution within 60 days
4. Notify relevant stakeholders while maintaining confidentiality

### 3. Remediation Development (Timeline based on severity)

1. Develop and test fixes in a private branch
2. Conduct thorough code reviews
3. Validate that the fix resolves the vulnerability without introducing new issues
4. Document the vulnerability and fix in detail for internal tracking
5. Prepare release notes for eventual disclosure

### 4. Public Disclosure Preparation

1. Determine an appropriate disclosure date (considering fix readiness and user impact)
2. Request a CVE identifier through GitHub Security Advisories
3. Prepare a comprehensive security advisory including:
   - Detailed description of the vulnerability
   - Affected versions
   - Mitigation measures
   - Upgrade instructions
   - Acknowledgment of the reporter
4. Prepare patched release

### 5. Release and Disclosure

1. Merge the fix to the main branch
2. Release patched versions according to the release process
3. Publish the GitHub Security Advisory
4. Send notifications to:
   - The Tock security mailing list
   - The reporter of the vulnerability
   - Any other necessary stakeholders
5. Update public documentation if necessary

### 6. Post-Disclosure Activities

1. Monitor for any issues with the fix
2. Conduct a retrospective to improve the security response process
3. Update this document with any lessons learned

## Communication Templates

### Initial Acknowledgment

```
Subject: [Tock Security] Acknowledgment of Security Report

Dear [Reporter Name],

Thank you for reporting this potential security issue to the Tock team. We take all security reports seriously and will investigate promptly.

We have received your report and have begun our initial assessment. We will keep you updated on our progress and may reach out if we need additional information.

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

We will be publicly disclosing the security vulnerability you reported on [Date]. The fix is included in version [Version Number], which is now available.

The CVE assigned to this issue is [CVE-ID].

We would like to acknowledge your contribution in discovering and responsibly disclosing this vulnerability. Please let us know if you would prefer to remain anonymous or if you'd like to be credited differently than your submitted name.

Thank you again for helping improve the security of Tock.

Best regards,
[Coordinator Name]
Tock Security Team
```

## Tools and Resources

- GitHub Security Advisories
- Private mailing list: security@lists.tockos.org
