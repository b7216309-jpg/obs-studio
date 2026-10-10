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
        // libobs/util/sse-intrin.h takes SSE from SIMDe outside MSVC. The
        // stand-in keeps `cargo test` free of a SIMDe install.
        oracle.include("oracle/sse-shim");
    }
    oracle
        .file("oracle/bitstream.c")
        .file("oracle/path_extension.c")
        .file("oracle/array_serializer.c")
        .file("oracle/darray.c")
        .file("oracle/crc32.c")
        .file("oracle/vec2.c")
        .file("oracle/nal.c")
        .file("oracle/encoder_packet.c")
        .file("oracle/hevc.c")
        .file("oracle/graphics_math_axisang.c")
        .file("oracle/graphics_math_bounds.c")
        .file("oracle/graphics_math_math_extra.c")
        .file("oracle/graphics_math_matrix3.c")
        .file("oracle/graphics_math_matrix4.c")
        .file("oracle/graphics_math_plane.c")
        .file("oracle/graphics_math_quat.c")
        .file("oracle/graphics_math_vec3.c")
        .file("oracle/graphics_math_vec4.c")
        .file("oracle/base.c")
        .file("oracle/base_drive.c")
        .file(libobs.join("util/base-variadic.c"))
        .file("oracle/file_serializer.c")
        .file("oracle/file_serializer_host.c")
        .file(libobs.join("util/dstr.c"))
        // dstr.c's conversions: platform.c's verbatim, over the real utf8.c.
        .file("oracle/platform_conv_host.c")
        .file(libobs.join("util/utf8.c"))
        .file("oracle/video_fourcc.c")
        .file("oracle/task.c")
        .include(&libobs)
        // obs.h (for obs-hevc.c) needs the CMake-generated obsconfig.h.
        .include("oracle/obsconfig")
        .std("c11");
    // task.c calls pthread_mutex_*/pthread_create plus the os_event/os_sem
    // helpers from threading-*.c. On Unix both come from the real sources
    // and the system pthread. MSVC has no pthread: libobs builds against
    // deps/w32-pthreads, so the oracle compiles its single-file build and
    // the real threading-windows.c instead.
    if oracle.get_compiler().is_like_msvc() {
        let deps = libobs.with_file_name("deps");
        oracle
            .include(deps.join("w32-pthreads"))
            .define("PTW32_STATIC_LIB", None)
            .file(deps.join("w32-pthreads/pthread.c"))
            .file(libobs.join("util/threading-windows.c"));
    } else {
        oracle.file(libobs.join("util/threading-posix.c"));
    }
    oracle.compile("obs_c_oracle");

    // Test allocator, whole-archive so bmalloc/bfree resolve regardless of
    // link order relative to obs-util.
    cc::Build::new()
        .file("oracle/test_bmem.c")
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
        "graphics/vec2.c",
        "graphics/vec2.h",
        "obs-nal.c",
        "obs-nal.h",
        "obs-hevc.c",
        "obs-hevc.h",
        "obs-encoder.h",
        "graphics/math-defs.h",
        "graphics/math-extra.h",
        "graphics/math-extra.c",
        "graphics/axisang.c",
        "graphics/axisang.h",
        "graphics/bounds.c",
        "graphics/bounds.h",
        "graphics/matrix3.c",
        "graphics/matrix3.h",
        "graphics/matrix4.c",
        "graphics/matrix4.h",
        "graphics/plane.c",
        "graphics/plane.h",
        "graphics/quat.c",
        "graphics/quat.h",
        "graphics/vec3.c",
        "graphics/vec3.h",
        "graphics/vec4.c",
        "graphics/vec4.h",
        "util/sse-intrin.h",
        "util/base.c",
        "util/base.h",
        "util/base-variadic.c",
        "util/c99defs.h",
        "util/threading.h",
        "util/file-serializer.c",
        "util/file-serializer.h",
        "util/dstr.c",
        "util/dstr.h",
        "util/platform.c",
        "util/utf8.c",
        "util/utf8.h",
        "util/platform.h",
        "media-io/video-fourcc.c",
        "media-io/video-io.h",
        "media-io/media-io-defs.h",
        "util/task.c",
        "util/task.h",
        "util/deque.h",
        "util/threading-posix.c",
        "util/threading-posix.h",
        "util/threading-windows.c",
        "util/threading-windows.h",
    ] {
        println!("cargo:rerun-if-changed={}", libobs.join(header).display());
    }
}
