use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // No canonicalize(): on Windows it yields a verbatim `\\?\` path, and
    // MSVC cannot resolve `#include "util/..."` against such an include dir.
    // CARGO_MANIFEST_DIR is already absolute.
    let libobs = manifest
        .ancestors()
        .nth(2)
        .expect("rust/obs-c-oracle has a repo root two levels up")
        .join("libobs");

    let mut oracle = cc::Build::new();
    // Rust never fuses `a * b + c` into an FMA, but GCC/Clang may (clang's
    // default -ffp-contract=on does so on aarch64), which would make float
    // oracles such as vec2_norm (`x*x + y*y`) differ in the last bit from the
    // Rust port. Pin the C oracle to unfused IEEE operations. MSVC does not
    // contract under its default /fp:precise.
    if !oracle.get_compiler().is_like_msvc() {
        oracle.flag("-ffp-contract=off");
    }
    oracle
        .file("oracle/bitstream.c")
        .file("oracle/path_extension.c")
        .file("oracle/array_serializer.c")
        .file("oracle/darray.c")
        .file("oracle/crc32.c")
        .file("oracle/lexer.c")
        .file("oracle/dstr.c")
        .file("oracle/vec2.c")
        .include(&libobs)
        .std("c11")
        .compile("obs_c_oracle");

    // util/text-lookup.c needs <uthash.h>; oracle/include holds a test-only
    // stand-in, kept to this one build so no other oracle sees it.
    cc::Build::new()
        .file("oracle/text_lookup.c")
        .file("oracle/test_platform_io.c")
        .include(&libobs)
        .include(manifest.join("oracle/include"))
        .std("c11")
        .compile("obs_c_oracle_text_lookup");

    // Test allocator, whole-archive so bmalloc/bfree resolve regardless of
    // link order relative to obs-util.
    cc::Build::new()
        .file("oracle/test_bmem.c")
        .file("oracle/test_dstr.c")
        .include(&libobs)
        .std("c11")
        .link_lib_modifier("+whole-archive")
        .compile("obs_c_oracle_bmem");

    println!("cargo:rerun-if-changed=oracle");
    for header in [
        "util/bitstream.c",
        "util/bitstream.h",
        "util/path-extension.c",
        "util/array-serializer.c",
        "util/array-serializer.h",
        "util/darray.h",
        "util/serializer.h",
        "util/bmem.h",
        "util/crc32.c",
        "util/crc32.h",
        "util/lexer.c",
        "util/lexer.h",
        "util/dstr.h",
        "util/dstr.c",
        "util/text-lookup.c",
        "util/text-lookup.h",
        "graphics/vec2.c",
        "graphics/vec2.h",
        "graphics/math-defs.h",
        "graphics/math-extra.h",
    ] {
        println!("cargo:rerun-if-changed={}", libobs.join(header).display());
    }
}
