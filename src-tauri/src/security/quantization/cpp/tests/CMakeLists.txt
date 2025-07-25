add_executable(smoke smoke.cpp)
target_link_libraries(smoke PRIVATE faiss_wrapper)
enable_testing()
add_test(NAME smoke COMMAND smoke)