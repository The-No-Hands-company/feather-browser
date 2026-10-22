# Contributing to Feather Browser

Thank you for your interest in contributing. Feather Browser is developed by The No Hands Company under the AGPL-3.0 license.

## Before You Start

Read the [README](README.md) to understand the project's philosophy. Contributions that introduce telemetry, tracking, enshittification, or closed-source dependencies will not be accepted — this is a hard constraint, not a preference.

## How to Contribute

### Reporting Bugs

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md). Include:
- Your OS and distribution
- GTK4 and WebKitGTK versions (`pkg-config --modversion gtk4 webkit2gtk-4.1`)
- Steps to reproduce
- What you expected vs. what happened

### Suggesting Features

Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md). Describe the privacy and sovereignty implications of the feature — every addition is evaluated through that lens first.

### Submitting Code

1. **Fork** the repository and create a branch from `main`.
2. **Write tests** for any new logic. The codebase uses Rust integration tests (`tests/`).
3. **Run the test suite** before opening a PR:
   ```sh
   cargo test --no-default-features
   cargo test --features feather-renderer
   cargo check --features webkit-renderer
   ```
4. **Follow the code style:**
   - SPDX header `// SPDX-License-Identifier: AGPL-3.0-or-later` on every `.rs` file
   - Conventional Commits for commit messages (`feat:`, `fix:`, `chore:`, etc.)
   - No `unsafe` without a `// SAFETY:` comment explaining the invariant
5. **Open a pull request** against `main`.

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions will be licensed under AGPL-3.0-or-later.
