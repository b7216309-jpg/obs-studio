# Rust Port Testing Policy

**Status:** Mandatory. Every change that ports C/C++ code to Rust MUST follow
this policy exactly. If a rule cannot be followed for a specific unit, stop and
record the exception in the tracking issue under `## Decisions` before writing
code; do not silently deviate.

## Goal

OBS Studio keeps its **existing public API**, but its internals are rewritten
in Rust. Concretely, the following stay compatible:

- The **libobs C API and ABI**: every `EXPORT` function signature, every public
  struct layout, and every public header under `libobs/`. Existing plugins and
  the frontend must compile against the unchanged headers and work unmodified.
- The **obs-websocket v5 protocol**, byte-for-byte on the wire.
- The frontend API (`obs-frontend-api`), scripting bindings, and the
  `window.obsstudio` browser-source API.

Tests exist to prove two separate things: that the Rust code is correct, and
that it is a drop-in replacement for the C code it replaces. One kind of test
cannot prove both, so every ported unit carries the four tiers below.

## The four tiers

### Tier 1 — Rust unit tests (correctness of the Rust core)

- Written in Rust with `#[test]`, against the **safe Rust API** (no `unsafe`,
  no FFI types).
- Every assertion in the existing C test for that unit (if any) is ported 1:1
  into Rust, in a test whose name references the C test it mirrors.
- New edge cases are added here.
- Run with `soldr cargo test`.

### Tier 2 — Existing C tests against the Rust implementation (API/ABI compatibility)

- The existing C tests (`test/cmocka/*.c`, and any other C/C++ test for the
  unit) are **kept and never rewritten or edited** to accommodate the port.
  They are the compatibility check.
- The Rust crate exports the original C symbols through a thin
  `#[no_mangle] pub extern "C"` shim with `#[repr(C)]` structs. The shim only
  converts types and delegates to the safe core; it contains no logic.
- The **existing C header is the source of truth**. Do not generate a
  replacement header and do not edit the public header as part of a port.
  Public struct layouts MUST match the C layout exactly, including any
  awkward field types (e.g. `uint8_t pos` stays `u8` at the C boundary even if
  the safe core uses `usize`).
- A layout test MUST assert `size_of`, `align_of`, and every field offset of
  each `#[repr(C)]` struct against values measured from the real C header
  (via the oracle crate, see Tier 3).
- The CMake build gains an option (`ENABLE_RUST_LIBOBS`, default `OFF`
  until the port is accepted) that drops the C source from libobs and links
  the Rust implementation in its place. The unchanged C test MUST pass in
  **both** configurations.
- Tests for the C boundary are written in C (they already are). Do not add
  C++ tests for C APIs.

### Tier 3 — Differential tests against the original C (behavioral parity)

- The original C source is compiled into the Rust test build as an **oracle**
  through the `rust/obs-c-oracle` crate (a dev-dependency only, built with
  the `cc` crate). Oracle symbols are renamed with an `oracle_` prefix by a
  wrapper `.c` file that `#define`s each exported name and then `#include`s
  the original source file. The original file is not copied or modified.
- A `proptest` test feeds random inputs to both the oracle and the Rust shim
  and asserts identical outputs and identical observable state.
- Known, intentional differences from the C behavior (bug fixes) MUST be
  listed in the test file and in the tracking issue, and the property test
  must exclude exactly those inputs, no more.
- The oracle for a unit is deleted only after the C source itself is deleted
  from the tree, in that same change.

### Tier 4 — Black-box protocol tests (external network API)

- Applies to obs-websocket and any other network-facing surface.
- Written against the running server over the wire, independent of the
  implementation language, and run unchanged against the C++ and the Rust
  servers.

## Required workflow for each ported unit

1. **RED:** commit the Tier 1 port of the existing test (and new edge cases)
   against stub implementations (`todo!()`), and show `soldr cargo test`
   failing.
2. **GREEN:** implement the safe Rust core until Tier 1 passes.
3. Add the C shim, the layout test, and the Tier 3 differential test.
4. Wire the `ENABLE_RUST_LIBOBS` CMake path and show the unchanged C test
   passing with it `ON` and `OFF`.
5. Record every intentional behavior difference in the tracking issue.

A port is not done until all applicable tiers pass. "Tier 1 passes" alone is
not done.

## Layout and naming

```text
Cargo.toml                    # workspace root, members = ["rust/*"]
rust-toolchain.toml           # pinned toolchain (required by soldr)
rust/
  obs-util/                   # ports of libobs/util/*
    src/bitstream.rs          # safe core (Tier 1 target)
    src/ffi/bitstream.rs      # extern "C" shim, #[repr(C)] types (Tier 2)
    tests/bitstream.rs        # Tier 1: 1:1 port of test/cmocka/test_bitstream.c
    tests/bitstream_layout.rs # Tier 2: struct layout vs. C header
    tests/bitstream_parity.rs # Tier 3: proptest vs. C oracle
  obs-c-oracle/               # dev-only: original C compiled with oracle_ prefix
    build.rs
    oracle/bitstream.c        # #define renames + #include of libobs/util/bitstream.c
```

New libobs areas get their own crate under `rust/` named after the libobs
directory (`obs-util`, `obs-graphics`, `obs-media-io`, ...).

## Rules that apply everywhere

- Use `soldr cargo ...`, never bare `cargo`.
- `soldr cargo clippy --all-targets -- -D warnings` and `soldr cargo fmt --check`
  must be clean.
- `unsafe` is allowed only in `src/ffi/` shims and in `obs-c-oracle`. Each
  `unsafe` block has a `// SAFETY:` comment.
- Do not add new GitHub Actions workflow files without explicit sign-off from
  the maintainer. Hook tests into existing workflows instead.
- Do not edit a public C header, an existing C test, or an exported function
  signature as part of a port. Changing the public API is a separate,
  explicitly approved decision with its own issue.
