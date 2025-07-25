#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct FaissContext faiss_context_t;

    /* Create / destroy ---------------------------------------------------- */
    faiss_context_t *faiss_create(int d); /* d ≤ 1024, multiple of 48 */
    void faiss_free(faiss_context_t *ctx);

    /* Training -------------------------------------------------------------
     * Returns:
     *   0  OK
     *  -1  NULL context or vectors
     *  -2  Training failed
     */
    int faiss_train(faiss_context_t *ctx,
                    const float *vectors,
                    size_t n_vectors);

    /* Encode ---------------------------------------------------------------
     * Writes exactly 48 bytes into out_codes.
     * Returns number of bytes written (always 48).
     */
    size_t faiss_encode(faiss_context_t *ctx,
                        const float *vector,
                        uint8_t *out_codes) noexcept;

    /* Decode ---------------------------------------------------------------
     * Writes dimension floats into out_vector.
     * Returns number of floats written (dimension).
     */
    size_t faiss_decode(faiss_context_t *ctx,
                        const uint8_t *codes,
                        float *out_vector) noexcept;

    /* Thread-pool tuning -------------------------------------------------- */
    void faiss_set_omp_num_threads(int n) noexcept;
    int faiss_get_omp_max_threads() noexcept;

#ifdef __cplusplus
}
#endif