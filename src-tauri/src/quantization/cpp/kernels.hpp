#ifndef KERNELS_HPP
#define KERNELS_HPP

#include <cstdint>

// Efficient bit packing/unpacking operations
int pack_bits(const uint8_t* input, int length, int bits_per_value, char* output);
int unpack_bits(const char* input, int length, int bits_per_value, uint8_t* output);

// SIMD-optimized kernels (AVX2/SSE4.2 when available, scalar fallback otherwise)
void quantize_block_simd(const float* input, int length, float scale, 
                        float zero_point, uint8_t* output);
void dequantize_block_simd(const uint8_t* input, int length, float scale, 
                          float zero_point, float* output);

// Performance analysis utilities
float calculate_compression_ratio(int original_bits, int compressed_bits, int length);
void benchmark_quantization(const float* input, int length, int iterations);

// Utility functions
inline int calculate_packed_size(int num_values, int bits_per_value) {
    return (num_values * bits_per_value + 7) / 8;
}

inline int calculate_unpacked_size(int packed_bytes, int bits_per_value) {
    return (packed_bytes * 8) / bits_per_value;
}

// Memory alignment helpers for SIMD operations
inline bool is_aligned(const void* ptr, size_t alignment) {
    return reinterpret_cast<uintptr_t>(ptr) % alignment == 0;
}

template<typename T>
inline T* align_pointer(T* ptr, size_t alignment) {
    uintptr_t addr = reinterpret_cast<uintptr_t>(ptr);
    uintptr_t aligned = (addr + alignment - 1) & ~(alignment - 1);
    return reinterpret_cast<T*>(aligned);
}

// SIMD capability detection at runtime
bool has_avx2_support();
bool has_sse42_support();

#endif