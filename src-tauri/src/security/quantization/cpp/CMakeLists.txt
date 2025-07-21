cmake_minimum_required(VERSION 3.16)
project(faiss_wrapper)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)

# FAISS static
find_package(faiss REQUIRED CONFIG)

# OpenMP (static)
find_package(OpenMP REQUIRED)
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} ${OpenMP_CXX_FLAGS}")
set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} ${OpenMP_CXX_FLAGS}")

add_library(faiss_wrapper STATIC faiss_wrapper.cpp)
target_link_libraries(faiss_wrapper PRIVATE faiss OpenMP::OpenMP_CXX)
target_compile_definitions(faiss_wrapper PRIVATE FAISS_USE_FLOAT=1)