cmake_minimum_required(VERSION 3.21)
project(faiss_wrapper VERSION 1.0.0 LANGUAGES CXX)

# ----------------------------------------------------------
# Global policies
# ----------------------------------------------------------
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)   # Rust FFI
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON) # LTO / IPO
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)

# ----------------------------------------------------------
# Options
# ----------------------------------------------------------
option(USE_STATIC_FAISS "Link static FAISS" ON)
option(BUILD_TESTS      "Build smoke tests" OFF)

# ----------------------------------------------------------
# Force CPU-only, minimal FAISS
# ----------------------------------------------------------
set(FAISS_ENABLE_GPU        OFF CACHE INTERNAL "")
set(FAISS_ENABLE_PYTHON     OFF CACHE INTERNAL "")
set(FAISS_ENABLE_RAFT       OFF CACHE INTERNAL "")
set(FAISS_OPT_LEVEL         "generic" CACHE INTERNAL "")
set(FAISS_ENABLE_C_BLAS     OFF CACHE INTERNAL "")
set(CMAKE_DISABLE_FIND_PACKAGE_BLAS   ON CACHE INTERNAL "")
set(CMAKE_DISABLE_FIND_PACKAGE_LAPACK ON CACHE INTERNAL "")

# ----------------------------------------------------------
# Pre-built FAISS (static)
# ----------------------------------------------------------
add_library(faiss STATIC IMPORTED)
set_target_properties(faiss PROPERTIES
    IMPORTED_LOCATION /home/guivernoir/faiss-cpu/lib/libfaiss.a
    INTERFACE_INCLUDE_DIRECTORIES /home/guivernoir/faiss-cpu/include
)

# ----------------------------------------------------------
# OpenMP
# ----------------------------------------------------------
find_package(OpenMP REQUIRED)

# ----------------------------------------------------------
# Library target
# ----------------------------------------------------------
add_library(faiss_wrapper STATIC)
target_sources(faiss_wrapper
    PRIVATE
        src/wrapper.cpp
        src/platform.cpp
)

target_include_directories(faiss_wrapper
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
)

target_link_libraries(faiss_wrapper
    PRIVATE
        faiss
        OpenMP::OpenMP_CXX
)

# ----------------------------------------------------------
# Compiler flags (size & speed)
# ----------------------------------------------------------
target_compile_options(faiss_wrapper PRIVATE
    -fno-exceptions
    -fno-rtti
    $<$<CONFIG:Release>:-O3 -DNDEBUG>
)

# ----------------------------------------------------------
# Tests
# ----------------------------------------------------------
if(BUILD_TESTS)
    add_subdirectory(tests)
    target_link_libraries(smoke PRIVATE faiss_wrapper faiss blas lapack)
endif()