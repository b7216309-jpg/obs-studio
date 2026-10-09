project(obs-cmocka)

find_package(CMocka CONFIG REQUIRED)

# Every test executable joins this target, so the Tier 2 validators
# (rust/tools/*-validate) build `cmocka-tests` instead of a hand-kept list and a
# new test cannot go unbuilt (and show up in ctest as "Not Run").
add_custom_target(cmocka-tests)

# Serializer test
add_executable(test_serializer test_serializer.c)
target_include_directories(test_serializer PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_serializer PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
add_dependencies(cmocka-tests test_serializer)
add_test(test_serializer ${CMAKE_CURRENT_BINARY_DIR}/test_serializer)

# darray test
add_executable(test_darray test_darray.c)
target_include_directories(test_darray PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_darray PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_darray)
add_test(test_darray ${CMAKE_CURRENT_BINARY_DIR}/test_darray)

# bitstream test
add_executable(test_bitstream test_bitstream.c)
target_include_directories(test_bitstream PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_bitstream PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_bitstream)
add_test(test_bitstream ${CMAKE_CURRENT_BINARY_DIR}/test_bitstream)

# OS path test
add_executable(test_os_path test_os_path.c)
target_include_directories(test_os_path PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_os_path PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_os_path)
add_test(test_os_path ${CMAKE_CURRENT_BINARY_DIR}/test_os_path)

# AVC header test. Where the toolchain supports it, obs-avc.c is compiled into
# the test with UBSan (no recovery) so undefined behavior fails the test
# (obs-rust/obs-studio#11); its definitions take precedence over libobs's.
add_executable(test_avc test_avc.c)
target_include_directories(test_avc PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_avc PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_sources(test_avc PRIVATE "${CMAKE_SOURCE_DIR}/libobs/obs-avc.c")
  target_compile_options(test_avc PRIVATE -fsanitize=undefined -fno-sanitize-recover=undefined)
  target_link_options(test_avc PRIVATE -fsanitize=undefined)
endif()

add_dependencies(cmocka-tests test_avc)
add_test(test_avc ${CMAKE_CURRENT_BINARY_DIR}/test_avc)

# Formatted filename test (obs-rust/obs-studio#21). platform.c is compiled into
# the test so the test's obs_get_video_info fixture replaces libobs's and no obs
# core has to be started; its definitions take precedence over libobs's. utf8.c
# is not exported by libobs, and platform.c needs it.
if(NOT MSVC)
  add_executable(
    test_formatted_filename
    test_formatted_filename.c
    "${CMAKE_SOURCE_DIR}/libobs/util/platform.c"
    "${CMAKE_SOURCE_DIR}/libobs/util/utf8.c"
  )
  target_include_directories(test_formatted_filename PRIVATE ${CMOCKA_INCLUDE_DIR})
  target_link_libraries(test_formatted_filename PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  target_compile_options(test_formatted_filename PRIVATE -fsanitize=address,undefined -fno-sanitize-recover=undefined)
  target_link_options(test_formatted_filename PRIVATE -fsanitize=address,undefined)

  add_dependencies(cmocka-tests test_formatted_filename)
  add_test(test_formatted_filename ${CMAKE_CURRENT_BINARY_DIR}/test_formatted_filename)
endif()

# CRC32 test
add_executable(test_crc32 test_crc32.c)
target_include_directories(test_crc32 PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_crc32 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_crc32)
add_test(test_crc32 ${CMAKE_CURRENT_BINARY_DIR}/test_crc32)

# UTF-8 test. utf8.c is not exported by libobs, so it is compiled into the test
# (non-Windows only: the Windows implementation wraps the Win32 API).
if(NOT MSVC)
  add_executable(test_utf8 test_utf8.c "${CMAKE_SOURCE_DIR}/libobs/util/utf8.c")
  target_include_directories(test_utf8 PRIVATE ${CMOCKA_INCLUDE_DIR})
  target_link_libraries(test_utf8 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  target_compile_options(test_utf8 PRIVATE -fsanitize=address,undefined -fno-sanitize-recover=undefined)
  target_link_options(test_utf8 PRIVATE -fsanitize=address,undefined)

  add_dependencies(cmocka-tests test_utf8)
  add_test(test_utf8 ${CMAKE_CURRENT_BINARY_DIR}/test_utf8)
endif()

# dstr test
add_executable(test_dstr test_dstr.c)
target_include_directories(test_dstr PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_dstr PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_dstr)
add_test(test_dstr ${CMAKE_CURRENT_BINARY_DIR}/test_dstr)

# lexer test
add_executable(test_lexer test_lexer.c)
target_include_directories(test_lexer PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_lexer PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_lexer)
add_test(test_lexer ${CMAKE_CURRENT_BINARY_DIR}/test_lexer)

# cf lexer test
add_executable(test_cf_lexer test_cf_lexer.c)
target_include_directories(test_cf_lexer PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_cf_lexer PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_cf_lexer)
add_test(test_cf_lexer ${CMAKE_CURRENT_BINARY_DIR}/test_cf_lexer)

# text lookup test
add_executable(test_text_lookup test_text_lookup.c)
target_include_directories(test_text_lookup PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_text_lookup PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_text_lookup)
add_test(test_text_lookup ${CMAKE_CURRENT_BINARY_DIR}/test_text_lookup)

# base (logging) test
add_executable(test_base test_base.c)
target_include_directories(test_base PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_base PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_base)
add_test(test_base ${CMAKE_CURRENT_BINARY_DIR}/test_base)

# bmem test
add_executable(test_bmem test_bmem.c)
target_include_directories(test_bmem PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_bmem PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_bmem)
add_test(test_bmem ${CMAKE_CURRENT_BINARY_DIR}/test_bmem)
