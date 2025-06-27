#include "kernels.hpp"
#include <cstring>
#include <algorithm>
#include <cstdint>
#include <memory>

// SIMD headers - conditional compilation for maximum tactical flexibility
#ifdef HAVE_AVX2
#include <immintrin.h>
#elif defined(HAVE_SSE42)
#include <smmintrin.h>
#endif

// Platform-specific CPUID support
#ifdef _MSC_VER
#include <intrin.h>
#elif defined(__GNUC__) || defined(__clang__)
#include <cpuid.h>
// Define __cpuid for non-Windows platforms
inline void my_cpuid(int cpu_info[4], int info_type) {
    __get_cpuid(info_type, 
                reinterpret_cast<unsigned int*>(&cpu_info[0]),
                reinterpret_cast<unsigned int*>(&cpu_info[1]),
                reinterpret_cast<unsigned int*>(&cpu_info[2]),
                reinterpret_cast<unsigned int*>(&cpu_info[3]));
}
#else
// Provide a stub or error for unsupported compilers/platforms
#error "CPUID not supported on this platform/compiler."
#endif

int pack_bits(const uint8_t* input, int length, int bits_per_value, char* output) {
    if (bits_per_value == 8) {
        // Direct copy for 8-bit values
        std::memcpy(output, input, length);
        return length;
    }

    int output_bytes = (length * bits_per_value + 7) / 8;
    std::memset(output, 0, output_bytes);

    int bit_position = 0;
    
    for (int i = 0; i < length; ++i) {
        uint8_t value = input[i];
        
        // Ensure value fits in specified bits
        uint8_t mask = (1 << bits_per_value) - 1;
        value &= mask;
        
        int byte_index = bit_position / 8;
        int bit_offset = bit_position % 8;
        
        // Handle values that span multiple bytes
        if (bit_offset + bits_per_value <= 8) {
            // Value fits in current byte
            output[byte_index] |= (value << bit_offset);
        } else {
            // Value spans two bytes
            int bits_in_first_byte = 8 - bit_offset;
            int bits_in_second_byte = bits_per_value - bits_in_first_byte;
            
            output[byte_index] |= (value << bit_offset);
            output[byte_index + 1] |= (value >> bits_in_first_byte);
        }
        
        bit_position += bits_per_value;
    }
    
    return output_bytes;
}

int unpack_bits(const char* input, int length, int bits_per_value, uint8_t* output) {
    if (bits_per_value == 8) {
        // Direct copy for 8-bit values
        std::memcpy(output, input, length);
        return length;
    }

    int input_bits = length * 8;
    int num_values = input_bits / bits_per_value;
    uint8_t mask = (1 << bits_per_value) - 1;
    
    int bit_position = 0;
    
    for (int i = 0; i < num_values; ++i) {
        int byte_index = bit_position / 8;
        int bit_offset = bit_position % 8;
        
        uint8_t value = 0;
        
        if (bit_offset + bits_per_value <= 8) {
            // Value is contained in single byte
            value = (input[byte_index] >> bit_offset) & mask;
        } else {
            // Value spans two bytes
            int bits_in_first_byte = 8 - bit_offset;
            int bits_in_second_byte = bits_per_value - bits_in_first_byte;
            
            uint8_t first_part = (input[byte_index] >> bit_offset) & ((1 << bits_in_first_byte) - 1);
            uint8_t second_part = input[byte_index + 1] & ((1 << bits_in_second_byte) - 1);
            
            value = first_part | (second_part << bits_in_first_byte);
        }
        
        output[i] = value;
        bit_position += bits_per_value;
    }
    
    return num_values;
}

// Vectorized quantization - the real battlefield begins here
void quantize_block_simd(const float* input, int length, float scale, 
                        float zero_point, uint8_t* output) {
    
#ifdef HAVE_AVX2
    // AVX2 implementation - 8 floats at once, like a well-coordinated squad
    const __m256 scale_vec = _mm256_set1_ps(1.0f / scale);
    const __m256 zero_point_vec = _mm256_set1_ps(-zero_point / scale);
    const __m256 half_vec = _mm256_set1_ps(0.5f);
    const __m256i max_val = _mm256_set1_epi32(255);
    const __m256i zero_vec = _mm256_setzero_si256();
    
    int simd_length = (length / 8) * 8;
    
    for (int i = 0; i < simd_length; i += 8) {
        // Load 8 floats
        __m256 values = _mm256_load_ps(input + i);
        
        // Normalize: (input - zero_point) / scale
        values = _mm256_fmadd_ps(values, scale_vec, zero_point_vec);
        
        // Round to nearest integer
        values = _mm256_add_ps(values, half_vec);
        
        // Convert to integers
        __m256i int_vals = _mm256_cvtps_epi32(values);
        
        // Clamp to [0, 255] - tactical precision required
        int_vals = _mm256_max_epi32(int_vals, zero_vec);
        int_vals = _mm256_min_epi32(int_vals, max_val);
        
        // Pack to bytes using saturation (AVX2's secret weapon)
        __m256i packed_low = _mm256_packus_epi32(int_vals, int_vals);
        __m256i packed = _mm256_packus_epi16(packed_low, packed_low);
        
        // Extract and store - a bit of manual labor, but worth the performance
        uint64_t result = _mm256_extract_epi64(packed, 0);
        std::memcpy(output + i, &result, 8);
    }
    
    // Handle remaining elements with scalar precision
    for (int i = simd_length; i < length; ++i) {
        float normalized = (input[i] - zero_point) / scale;
        int quantized = static_cast<int>(normalized + 0.5f);
        quantized = std::max(0, std::min(quantized, 255));
        output[i] = static_cast<uint8_t>(quantized);
    }
    
#elif defined(HAVE_SSE42)
    // SSE4.2 implementation - 4 floats at once, still respectable firepower
    const __m128 scale_vec = _mm_set1_ps(1.0f / scale);
    const __m128 zero_point_vec = _mm_set1_ps(-zero_point / scale);
    const __m128 half_vec = _mm_set1_ps(0.5f);
    const __m128i max_val = _mm_set1_epi32(255);
    const __m128i zero_vec = _mm_setzero_si128();
    
    int simd_length = (length / 4) * 4;
    
    for (int i = 0; i < simd_length; i += 4) {
        // Load 4 floats
        __m128 values = _mm_load_ps(input + i);
        
        // Normalize
        values = _mm_add_ps(_mm_mul_ps(values, scale_vec), zero_point_vec);
        
        // Round to nearest
        values = _mm_add_ps(values, half_vec);
        
        // Convert to integers
        __m128i int_vals = _mm_cvtps_epi32(values);
        
        // Clamp to [0, 255]
        int_vals = _mm_max_epi32(int_vals, zero_vec);
        int_vals = _mm_min_epi32(int_vals, max_val);
        
        // Pack to bytes
        __m128i packed_low = _mm_packus_epi32(int_vals, int_vals);
        __m128i packed = _mm_packus_epi16(packed_low, packed_low);
        
        // Extract and store
        uint32_t result = _mm_extract_epi32(packed, 0);
        std::memcpy(output + i, &result, 4);
    }
    
    // Handle remaining elements
    for (int i = simd_length; i < length; ++i) {
        float normalized = (input[i] - zero_point) / scale;
        int quantized = static_cast<int>(normalized + 0.5f);
        quantized = std::max(0, std::min(quantized, 255));
        output[i] = static_cast<uint8_t>(quantized);
    }
    
#else
    // Scalar fallback - sometimes you fight with what you have
    for (int i = 0; i < length; ++i) {
        float normalized = (input[i] - zero_point) / scale;
        int quantized = static_cast<int>(normalized + 0.5f);
        quantized = std::max(0, std::min(quantized, 255));
        output[i] = static_cast<uint8_t>(quantized);
    }
#endif
}

void dequantize_block_simd(const uint8_t* input, int length, float scale, 
                          float zero_point, float* output) {
    
#ifdef HAVE_AVX2
    // AVX2 dequantization - elegant in its mathematical precision
    const __m256 scale_vec = _mm256_set1_ps(scale);
    const __m256 zero_point_vec = _mm256_set1_ps(zero_point);
    
    int simd_length = (length / 8) * 8;
    
    for (int i = 0; i < simd_length; i += 8) {
        // Load 8 bytes and convert to 32-bit integers
        uint64_t bytes;
        std::memcpy(&bytes, input + i, 8);
        
        __m128i bytes_128 = _mm_set1_epi64x(bytes);
        __m256i bytes_256 = _mm256_cvtepu8_epi32(bytes_128);
        
        // Convert to floats
        __m256 float_vals = _mm256_cvtepi32_ps(bytes_256);
        
        // Dequantize: quantized * scale + zero_point
        __m256 result = _mm256_fmadd_ps(float_vals, scale_vec, zero_point_vec);
        
        // Store result
        _mm256_store_ps(output + i, result);
    }
    
    // Handle remaining elements
    for (int i = simd_length; i < length; ++i) {
        output[i] = input[i] * scale + zero_point;
    }
    
#elif defined(HAVE_SSE42)
    // SSE4.2 dequantization
    const __m128 scale_vec = _mm_set1_ps(scale);
    const __m128 zero_point_vec = _mm_set1_ps(zero_point);
    
    int simd_length = (length / 4) * 4;
    
    for (int i = 0; i < simd_length; i += 4) {
        // Load 4 bytes and convert to 32-bit integers
        uint32_t bytes;
        std::memcpy(&bytes, input + i, 4);
        
        __m128i bytes_128 = _mm_set1_epi32(bytes);
        __m128i int_vals = _mm_cvtepu8_epi32(bytes_128);
        
        // Convert to floats
        __m128 float_vals = _mm_cvtepi32_ps(int_vals);
        
        // Dequantize
        __m128 result = _mm_add_ps(_mm_mul_ps(float_vals, scale_vec), zero_point_vec);
        
        // Store result
        _mm_store_ps(output + i, result);
    }
    
    // Handle remaining elements
    for (int i = simd_length; i < length; ++i) {
        output[i] = input[i] * scale + zero_point;
    }
    
#else
    // Scalar fallback
    for (int i = 0; i < length; ++i) {
        output[i] = input[i] * scale + zero_point;
    }
#endif
}

// Utility functions for performance analysis
float calculate_compression_ratio(int original_bits, int compressed_bits, int length) {
    return static_cast<float>(original_bits * length) / (compressed_bits * length);
}

void benchmark_quantization(const float* input, int length, int iterations) {
    // This would be useful for tactical performance assessment
    // Implementation left as an exercise for the strategically minded
}

// SIMD capability detection - intelligence gathering at its finest
bool has_avx2_support() {
#ifdef HAVE_AVX2
    // Check CPUID for AVX2 support
    int cpu_info[4];
    my_cpuid(cpu_info, 7);
    return (cpu_info[1] & (1 << 5)) != 0; // EBX bit 5 indicates AVX2
#else
    return false;
#endif
}

bool has_sse42_support() {
#ifdef HAVE_SSE42
    // Check CPUID for SSE4.2 support
    int cpu_info[4];
    my_cpuid(cpu_info, 1);
    return (cpu_info[2] & (1 << 20)) != 0; // ECX bit 20 indicates SSE4.2
#else
    return false;
#endif
}