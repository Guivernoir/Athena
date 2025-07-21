// Unified mobile + desktop wrapper for FAISS Product Quantization
// Target: f32[384], up to f32[1024], no OPQ, raw byte arrays
// iOS 13+, Android API 24+, static OpenMP, ≤ 5 MB binary

#include "faiss_wrapper.hpp"
#include <faiss/IndexPQ.h>
#include <cstring>
#include <memory>
#include <omp.h>

// ----------------------------------------------------------
// Compile-time arch flags
// ----------------------------------------------------------
#if defined(__aarch64__)
#define USE_NEON
#elif defined(__x86_64__)
#define USE_AVX2
#endif

// ----------------------------------------------------------
// Static helpers
// ----------------------------------------------------------
namespace
{

    constexpr int MAX_DIM = 1024;
    constexpr int BYTES_PER_CODE = 1; // PQ byte size per vector
    constexpr int M = 48;             // #sub-quantizers for dim 384
    constexpr int KS = 256;           // 2^8 centroids per sub-q

    // Validate dimension compatibility
    inline bool is_dim_supported(int d)
    {
        return d <= MAX_DIM && d % M == 0;
    }

} // namespace

// ----------------------------------------------------------
// Context object
// ----------------------------------------------------------
struct FaissContext
{
    std::unique_ptr<faiss::IndexPQ> index;
};

// ----------------------------------------------------------
// C-API implementation
// ----------------------------------------------------------
extern "C"
{

    faiss_context_t *faiss_create(int d)
    {
        if (!is_dim_supported(d))
            return nullptr;
        auto *ctx = new FaissContext;
        int nbits = 8; // 256 centroids per sub-q
        ctx->index = std::make_unique<faiss::IndexPQ>(d, M, nbits);
        return reinterpret_cast<faiss_context_t *>(ctx);
    }

    void faiss_free(faiss_context_t *ctx)
    {
        delete reinterpret_cast<FaissContext *>(ctx);
    }

    int faiss_train(faiss_context_t *ctx,
                    const float *vectors,
                    size_t n_vectors)
    {
        if (!ctx || !vectors)
            return -1;
        auto *c = reinterpret_cast<FaissContext *>(ctx);
        try
        {
            c->index->train(n_vectors, vectors);
            return 0;
        }
        catch (...)
        {
            return -2;
        }
    }

    size_t faiss_encode(faiss_context_t *ctx,
                        const float *vector,
                        uint8_t *out_codes)
    {
        if (!ctx || !vector || !out_codes)
            return 0;
        auto *c = reinterpret_cast<FaissContext *>(ctx);
        const int d = c->index->d;
        c->index->sa_encode(1, vector, out_codes);
        return static_cast<size_t>(M); // M bytes per vector
    }

    size_t faiss_decode(faiss_context_t *ctx,
                        const uint8_t *codes,
                        float *out_vector)
    {
        if (!ctx || !codes || !out_vector)
            return 0;
        auto *c = reinterpret_cast<FaissContext *>(ctx);
        c->index->sa_decode(1, codes, out_vector);
        return static_cast<size_t>(c->index->d);
    }

    // Thread-pool control
    void faiss_set_omp_num_threads(int n)
    {
        omp_set_num_threads(n > 0 ? n : omp_get_max_threads());
    }

    int faiss_get_omp_max_threads()
    {
        return omp_get_max_threads();
    }

} // extern "C"