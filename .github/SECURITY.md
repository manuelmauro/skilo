# Security Policy

## Supported Versions

skilo is distributed on [crates.io](https://crates.io/crates/skilo) and as
prebuilt binaries via [GitHub Releases](https://github.com/manuelmauro/skilo/releases).
Security fixes are released against the latest published version.

| Version | Supported          |
| ------- | ------------------ |
| 0.11.x  | :white_check_mark: |
| < 0.11  | :x:                |

Please upgrade to the latest release before reporting an issue — the fix may
already be available.

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately using GitHub's **"Report a vulnerability"** button on the
[Security Advisories page](https://github.com/manuelmauro/skilo/security/advisories/new).
This opens a private advisory visible only to the maintainers and you, where we
can discuss and coordinate a fix.

If you cannot use GitHub's private reporting, email **manuel@nutsandbolts.dev**
instead. When sharing a proof of concept, please defang URLs and paths and avoid
including real secrets.

### What to include

- A description of the vulnerability and its impact.
- Steps to reproduce — ideally a minimal, sandboxed proof of concept.
- The affected version(s) and platform.
- Any suggested remediation.

## What to Expect

- **Acknowledgement** within 3 business days.
- An initial assessment and severity estimate soon after.
- **Coordinated disclosure**: we will work with you on a fix and agree on a
  disclosure timeline before any public announcement.
- **Credit**: with your permission, we will credit you in the release notes and
  the published [GitHub Security Advisory](https://github.com/manuelmauro/skilo/security/advisories)
  (and a [RustSec advisory](https://rustsec.org/), where applicable).

## Scope

This policy covers the `skilo` crate and CLI itself.

Note that `skilo add` installs **third-party skills** from sources you choose.
Those skills are not part of skilo and are outside the scope of this policy —
only install skills from sources you trust (see the Security section of the
[README](../README.md)).
