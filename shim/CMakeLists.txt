cmake_minimum_required(VERSION 3.18)
project(otio_shim)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Get the absolute path to the vendor directory
get_filename_component(OTIO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../vendor/OpenTimelineIO" ABSOLUTE)

message(STATUS "OTIO_ROOT: ${OTIO_ROOT}")
message(STATUS "OTIO include path: ${OTIO_ROOT}/src")

# Build OTIO as static libraries
set(OTIO_PYTHON_INSTALL OFF CACHE BOOL "" FORCE)
set(OTIO_DEPENDENCIES_INSTALL OFF CACHE BOOL "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(OTIO_SHARED_LIBS OFF CACHE BOOL "" FORCE)
add_subdirectory(${OTIO_ROOT} otio_build EXCLUDE_FROM_ALL)

# Build shim as static library
add_library(otio_shim STATIC otio_shim.cpp)

# Set include directories BEFORE linking to ensure they're available
target_include_directories(otio_shim
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
    PRIVATE
        ${OTIO_ROOT}/src
        ${OTIO_ROOT}/src/deps
        ${OTIO_ROOT}/src/deps/rapidjson/include
)

target_link_libraries(otio_shim PUBLIC opentimelineio)

# Install for Rust to find
install(TARGETS otio_shim opentimelineio opentime DESTINATION lib)
install(FILES otio_shim.h DESTINATION include)
