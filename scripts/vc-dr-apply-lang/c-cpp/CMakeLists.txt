cmake_minimum_required(VERSION 3.24)
project({name} VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 17)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT MSVC)
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# clang-tidy runs on every compile when it is installed.
find_program(CLANG_TIDY clang-tidy)
if(CLANG_TIDY)
  set(CMAKE_C_CLANG_TIDY ${CLANG_TIDY})
  set(CMAKE_CXX_CLANG_TIDY ${CLANG_TIDY})
endif()

add_library(greeting src/greeting.c)
target_include_directories(greeting PUBLIC src)

add_executable({name} src/main.c)
target_link_libraries({name} PRIVATE greeting)

enable_testing()
add_executable(test_greeting tests/test_greeting.c)
target_link_libraries(test_greeting PRIVATE greeting)
add_test(NAME test_greeting COMMAND test_greeting)

# `cmake --build build --target format` formats every source in place.
file(GLOB_RECURSE FORMAT_SOURCES CONFIGURE_DEPENDS
  ${CMAKE_SOURCE_DIR}/src/*.[ch] ${CMAKE_SOURCE_DIR}/src/*.[ch]pp
  ${CMAKE_SOURCE_DIR}/tests/*.[ch] ${CMAKE_SOURCE_DIR}/tests/*.[ch]pp)
find_program(CLANG_FORMAT clang-format)
if(CLANG_FORMAT)
  add_custom_target(format COMMAND ${CLANG_FORMAT} -i ${FORMAT_SOURCES})
else()
  add_custom_target(format
    COMMAND ${CMAKE_COMMAND} -E echo "format: clang-format is not installed"
    COMMAND ${CMAKE_COMMAND} -E false)
endif()
