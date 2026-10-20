# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| main    | ✅ Yes    |

Feather Browser is pre-release software. Security fixes are applied to `main` and released as patch versions.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities by emailing: **erichakansson84@gmail.com**

Include:
- A description of the vulnerability and its impact
- Steps to reproduce
- Any proof-of-concept code (kept confidential)

You will receive an acknowledgement within 72 hours. We aim to release a fix within 14 days for critical issues.

## Scope

We are particularly interested in:

- **Privacy leaks** — any code path that sends user data, browsing history, or identifiers to a third party without explicit user consent
- **URL handling** — injection or bypass in address bar logic or privacy parameter stripping
- **Ad blocker bypass** — techniques that allow blocked domains to load
- **Settings file** — arbitrary file read/write via `~/.config/feather/`
- **WebKit sandbox** — anything that escapes the renderer's process isolation

## Out of Scope

- Vulnerabilities in WebKitGTK itself (report to the [WebKit Security](https://webkit.org/security/) team)
- Vulnerabilities in GTK4 itself (report to [GNOME Security](https://security.gnome.org/))
- Denial-of-service via malformed web pages (expected behaviour — browser crashes are not security issues unless they leak memory)

## Philosophy

Feather Browser is built on the principle that privacy is a concrete engineering obligation. We treat any code that could compromise user privacy as a security vulnerability, even if it does not enable remote code execution.
