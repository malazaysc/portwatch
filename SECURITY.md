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

### Handling of untrusted process metadata

A port's owning process chooses its own name, command line, and working
directory, so all of those are untrusted input. portwatch treats them
defensively:

- **No shell interpolation.** Every external command is invoked with discrete
  arguments (`Command::arg`), never via a shell, so crafted process names or
  paths cannot inject commands.
- **Exposure never downgrades.** When the same port appears multiple times
  (e.g. IPv4 + IPv6), portwatch keeps the *most*-exposed bind, and any
  unrecognized bind address is treated as exposed rather than local — it fails
  toward over-reporting exposure, never under-reporting it.
- **Bounded metadata reads.** Tech detection reads project files
  (`package.json`, `Cargo.toml`, `requirements.txt`, …) from process-controlled
  paths. These reads are restricted to regular files within a 1 MiB cap, so a
  hostile process cannot point detection at a device, FIFO, or huge file to
  hang or exhaust memory (see `detect::read_metadata_file`).
- **Detection labels are display-only.** The detected technology and project
  grouping affect what you see, never which PID a kill action targets — that is
  always the highlighted row's own process, and the kernel enforces signal
  permissions.
