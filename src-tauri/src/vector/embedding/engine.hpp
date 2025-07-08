#ifndef BGE_ENGINE_HPP
#define BGE_ENGINE_HPP

#ifdef __cplusplus
extern "C"
{
#endif

    /**
     * BGE Embedding Engine C Interface
     *
     * Strategic interface for vectorial intelligence operations.
     * Now we're dealing with semantic vectors instead of creative text -
     * a more precise weapon in the information warfare arsenal.
     */

    void *bge_engine_create(const char *model_path);
    void bge_engine_destroy(void *engine);
    float *bge_engine_embed(void *engine, const char *text, int *embedding_size);
    void bge_free_embedding(float *embedding);
    int bge_engine_is_loaded(void *engine);
    int bge_engine_get_embedding_dim(void *engine);
    const char *bge_engine_get_model_info(void *engine);

#ifdef __cplusplus
}
#endif

#endif // BGE_ENGINE_HPP