# Agent instructions

This repository is being converted from C/C++ to Rust while keeping the public
API (libobs C API/ABI, obs-websocket v5 protocol, frontend and scripting APIs)
compatible.

## Mandatory reading

Before porting any code, writing any test, or touching `rust/`, `Cargo.toml`,
or `test/`, read and follow
[docs/rust-port/testing-policy.md](docs/rust-port/testing-policy.md) exactly.
It defines the four required test tiers (Rust unit tests, unchanged C tests
against the Rust implementation, differential tests against the original C,
and black-box protocol tests) and the per-unit workflow.

## Quick rules

- Use `soldr cargo ...`, never bare `cargo`.
- Never edit existing C tests or public C headers to make a port pass.
- Every port starts RED (failing test against stubs) and ends GREEN.
- Do not add new GitHub Actions workflow files without explicit sign-off.
