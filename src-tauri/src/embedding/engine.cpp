#include "engine.hpp"
#include "llama.h"
#include <string>
#include <memory>
#include <cstring>
#include <iostream>
#include <vector>

/**
 * BGE Embedding Engine Implementation
 *
 * Strategic pivot from text generation to vectorial intelligence.
 * Now we're in the business of semantic understanding rather than
 * creative writing - a more tactical approach to information warfare.
 */

struct BGEEngine
{
    llama_model *model;
    llama_context *ctx;
    std::string model_path;
    bool is_loaded;
    int embedding_dim;

    BGEEngine() : model(nullptr), ctx(nullptr), is_loaded(false), embedding_dim(0) {}

    ~BGEEngine()
    {
        if (ctx)
        {
            llama_free(ctx);
        }
        if (model)
        {
            llama_free_model(model);
        }
        llama_backend_free();
    }
};

static bool backend_initialized = false;

void ensure_backend_initialized()
{
    if (!backend_initialized)
    {
        llama_backend_init();
        backend_initialized = true;
    }
}

extern "C"
{

    void *bge_engine_create(const char *model_path)
    {
        if (!model_path)
        {
            std::cerr << "Model path is null - tactical oversight detected." << std::endl;
            return nullptr;
        }

        ensure_backend_initialized();

        auto engine = std::make_unique<BGEEngine>();
        engine->model_path = model_path;

        llama_model_params model_params = llama_model_default_params();
        model_params.n_gpu_layers = 0;

        engine->model = llama_load_model_from_file(model_path, model_params);
        if (!engine->model)
        {
            std::cerr << "Failed to load embedding model: " << model_path << " - asset compromised." << std::endl;
            return nullptr;
        }

        llama_context_params ctx_params = llama_context_default_params();
        ctx_params.n_ctx = 512; // BGE models typically use shorter contexts
        ctx_params.n_threads = 4;
        ctx_params.embeddings = true; // Critical: enable embedding mode

        engine->ctx = llama_new_context_with_model(engine->model, ctx_params);
        if (!engine->ctx)
        {
            std::cerr << "Failed to create embedding context - initialization failure." << std::endl;
            return nullptr;
        }

        engine->embedding_dim = llama_n_embd(engine->model);
        engine->is_loaded = true;

        return engine.release();
    }

    float *bge_engine_embed(void *engine_ptr, const char *text, int *embedding_size)
    {
        if (!engine_ptr || !text || !embedding_size)
        {
            return nullptr;
        }

        auto *engine = static_cast<BGEEngine *>(engine_ptr);
        if (!engine->is_loaded)
        {
            return nullptr;
        }

        std::vector<llama_token> tokens;
        const int n_tokens = -llama_tokenize(engine->model, text, strlen(text), nullptr, 0, true, true);
        tokens.resize(n_tokens);

        if (llama_tokenize(engine->model, text, strlen(text), tokens.data(), tokens.size(), true, true) < 0)
        {
            std::cerr << "Tokenization failed - intelligence processing compromised." << std::endl;
            return nullptr;
        }

        llama_kv_cache_clear(engine->ctx);

        if (llama_decode(engine->ctx, llama_batch_get_one(tokens.data(), tokens.size(), 0, 0)) != 0)
        {
            std::cerr << "Failed to process text for embedding - tactical failure." << std::endl;
            return nullptr;
        }

        const float *embeddings = llama_get_embeddings(engine->ctx);
        if (!embeddings)
        {
            std::cerr << "Failed to extract embeddings - vector extraction failed." << std::endl;
            return nullptr;
        }

        *embedding_size = engine->embedding_dim;

        // Allocate and copy embeddings
        float *result = static_cast<float *>(malloc(engine->embedding_dim * sizeof(float)));
        if (result)
        {
            memcpy(result, embeddings, engine->embedding_dim * sizeof(float));
        }

        return result;
    }

    void bge_engine_destroy(void *engine_ptr)
    {
        if (engine_ptr)
        {
            delete static_cast<BGEEngine *>(engine_ptr);
        }
    }

    void bge_free_embedding(float *embedding)
    {
        if (embedding)
        {
            free(embedding);
        }
    }

    int bge_engine_is_loaded(void *engine_ptr)
    {
        if (!engine_ptr)
            return 0;
        return static_cast<BGEEngine *>(engine_ptr)->is_loaded ? 1 : 0;
    }

    int bge_engine_get_embedding_dim(void *engine_ptr)
    {
        if (!engine_ptr)
            return 0;
        auto *engine = static_cast<BGEEngine *>(engine_ptr);
        return engine->is_loaded ? engine->embedding_dim : 0;
    }

    const char *bge_engine_get_model_info(void *engine_ptr)
    {
        if (!engine_ptr)
            return "Engine not initialized";
        auto *engine = static_cast<BGEEngine *>(engine_ptr);
        return engine->is_loaded ? "BGE-Small-EN-v1.5 Q8_0 - Vectorial Intelligence Asset" : "Asset offline";
    }
}