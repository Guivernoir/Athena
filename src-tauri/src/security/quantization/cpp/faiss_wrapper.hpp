// Unified C-API header for FAISS Product Quantization
// Compatible with iOS 13+, Android API 24+, static OpenMP
#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // Opaque handle
    typedef struct FaissContext faiss_context_t;

    // Create / destroy
    faiss_context_t *faiss_create(int d); // dimension ≤ 1024, multiple of 48
    void faiss_free(faiss_context_t *ctx);

    // Training
    int faiss_train(faiss_context_t *ctx,
                    const float *vectors,
                    size_t n_vectors);

    // Single-vector encode/decode
    // Returns number of bytes written (always 48)
    size_t faiss_encode(faiss_context_t *ctx,
                        const float *vector,
                        uint8_t *out_codes);

    // Returns number of floats written (dimension)
    size_t faiss_decode(faiss_context_t *ctx,
                        const uint8_t *codes,
                        float *out_vector);

    // Thread-pool tuning
    void faiss_set_omp_num_threads(int n);
    int faiss_get_omp_max_threads();

#ifdef __cplusplus
}
#endif