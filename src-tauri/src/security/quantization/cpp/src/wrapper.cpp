#include "../include/faiss_wrapper.h"
#include "wrapper.h"
#include <faiss/IndexPQ.h>
#include <cstring>

namespace
{

    constexpr int MAX_DIM = 1024;
    constexpr int M = 48;   // 48 sub-quantizers for dim 384
    constexpr int KS = 256; // 8-bit codes

    inline bool is_dim_supported(int d) noexcept
    {
        return d <= MAX_DIM && d % M == 0;
    }

} // namespace

extern "C"
{

    faiss_context_t *faiss_create(int d) noexcept
    {
        if (!is_dim_supported(d))
            return nullptr;
        auto *ctx = new FaissContext;
        ctx->index = std::make_unique<faiss::IndexPQ>(d, M, 8 /*nbits*/);
        return reinterpret_cast<faiss_context_t *>(ctx);
    }

    void faiss_free(faiss_context_t *ctx) noexcept
    {
        delete reinterpret_cast<FaissContext *>(ctx);
    }

    int faiss_train(faiss_context_t *ctx,
                    const float *vectors,
                    size_t n_vectors) noexcept
    {
        if (!ctx || !vectors)
            return -1;

        // Let the caller catch any exception; we keep noexcept
        reinterpret_cast<FaissContext *>(ctx)->index->train(n_vectors, vectors);
        return 0;
    }

    size_t faiss_encode(faiss_context_t *ctx,
                        const float *vector,
                        uint8_t *out_codes) noexcept
    {
        if (!ctx || !vector || !out_codes)
            return 0;
        auto *c = reinterpret_cast<FaissContext *>(ctx);
        c->index->sa_encode(1, vector, out_codes);
        return M; // 48 bytes
    }

    size_t faiss_decode(faiss_context_t *ctx,
                        const uint8_t *codes,
                        float *out_vector) noexcept
    {
        if (!ctx || !codes || !out_vector)
            return 0;
        auto *c = reinterpret_cast<FaissContext *>(ctx);
        c->index->sa_decode(1, codes, out_vector);
        return static_cast<size_t>(c->index->d);
    }

} // extern "C"