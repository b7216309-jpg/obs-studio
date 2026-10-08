# Links the Rust ports (rust/libobs-rust) into libobs in place of the C
# sources they replace. See docs/rust-port/testing-policy.md.

include(FetchContent)

FetchContent_Declare(Corrosion GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git GIT_TAG v0.5.2)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH "${CMAKE_SOURCE_DIR}/Cargo.toml" CRATES libobs-rust CRATE_TYPES staticlib)

# Whole-archive so every exported C symbol ends up in libobs, including ones
# libobs itself never calls (e.g. bitstream_reader_r16, used by plugins).
target_link_libraries(libobs PRIVATE "$<LINK_LIBRARY:WHOLE_ARCHIVE,libobs_rust-static>")

# ELF: Rust staticlib internals have default visibility; hide them so libobs
# exports the same symbol set as the C build.
# TODO(macOS): equivalent with -unexported_symbols_list once macOS is built.
if(OS_LINUX OR OS_FREEBSD OR OS_OPENBSD)
  target_link_options(libobs PRIVATE "LINKER:--version-script=${CMAKE_CURRENT_SOURCE_DIR}/cmake/rust-exports.map")
  set_property(TARGET libobs APPEND PROPERTY LINK_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/cmake/rust-exports.map")
endif()
