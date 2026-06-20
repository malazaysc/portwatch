# Security Policy

## Supported versions

portwatch is distributed as a rolling release; only the latest published
version receives security fixes.

| Version | Supported |
| ------- | --------- |
| latest  | ✅        |
| older   | ❌        |

## Reporting a vulnerability

Please report security issues **privately** — do not open a public issue.

- Preferred: open a private advisory via
  [GitHub Security Advisories](https://github.com/malazaysc/portwatch/security/advisories/new).
- Alternatively, email the maintainer at alesi.metal@gmail.com.

Please include reproduction steps and the affected version. You can expect an
acknowledgement within a few days. Once a fix is available it will be released
and the advisory published.

## Scope notes

portwatch inspects local listening ports and the processes that own them by
shelling out to system tools (`lsof`, `ss`, `ps`, `nettop`, `docker`, `git`)
and can send signals to processes you select. It runs with your user's
privileges and performs no network I/O of its own. Findings most relevant to
this project include: incorrect exposure classification (a network-exposed
port shown as local-only), unintended process termination, and any path that
lets untrusted process metadata influence command execution.
