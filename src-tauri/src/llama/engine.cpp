#include "engine.hpp"
#include "llama.h"
#include <string>
#include <memory>
#include <cstring>
#include <iostream>

/**
 * LLM Engine Implementation
 * 
 * The engine room where llama.cpp does the computational heavy lifting.
 * This is where the magic happens - and by magic, I mean carefully orchestrated
 * matrix multiplications that would make a mathematician weep with joy.
 */

struct QwenEngine {
    llama_model* model;
    llama_context* ctx;
    std::string model_path;
    bool is_loaded;
    
    QwenEngine() : model(nullptr), ctx(nullptr), is_loaded(false) {}
    
    ~QwenEngine() {
        if (ctx) {
            llama_free(ctx);
        }
        if (model) {
            llama_free_model(model);
        }
        llama_backend_free();
    }
};

static bool backend_initialized = false;

void ensure_backend_initialized() {
    if (!backend_initialized) {
        llama_backend_init();
        backend_initialized = true;
    }
}

extern "C" {

void* qwen_engine_create(const char* model_path) {
    if (!model_path) {
        std::cerr << "Model path is null - quite the strategic oversight." << std::endl;
        return nullptr;
    }
    
    ensure_backend_initialized();
    
    auto engine = std::make_unique<QwenEngine>();
    engine->model_path = model_path;
    
    llama_model_params model_params = llama_model_default_params();
    model_params.n_gpu_layers = 0; 

    engine->model = llama_load_model_from_file(model_path, model_params);
    if (!engine->model) {
        std::cerr << "Failed to load model: " << model_path << " - the asset appears to be compromised." << std::endl;
        return nullptr;
    }
    
    llama_context_params ctx_params = llama_context_default_params();
    ctx_params.n_ctx = 2048; 
    ctx_params.n_threads = 4; 

    engine->ctx = llama_new_context_with_model(engine->model, ctx_params);
    if (!engine->ctx) {
        std::cerr << "Failed to create context - tactical failure in initialization." << std::endl;
        return nullptr;
    }
    
    engine->is_loaded = true;
    return engine.release();
}

char* qwen_engine_generate(void* engine_ptr, const char* prompt, int max_tokens, float temperature) {
    if (!engine_ptr || !prompt) {
        return nullptr; 
    }
    
    auto* engine = static_cast<QwenEngine*>(engine_ptr);
    if (!engine->is_loaded) {
        return nullptr;
    }
    
    std::vector<llama_token> tokens;
    const int n_prompt_tokens = -llama_tokenize(engine->model, prompt, strlen(prompt), nullptr, 0, true, true);
    tokens.resize(n_prompt_tokens);
    
    if (llama_tokenize(engine->model, prompt, strlen(prompt), tokens.data(), tokens.size(), true, true) < 0) {
        std::cerr << "Tokenization failed - intelligence gathering compromised." << std::endl;
        return nullptr;
    }
    
    llama_kv_cache_clear(engine->ctx);
    
    if (llama_decode(engine->ctx, llama_batch_get_one(tokens.data(), tokens.size(), 0, 0)) != 0) {
        std::cerr << "Failed to evaluate prompt - computational assets under stress." << std::endl;
        return nullptr;
    }
    
    std::string response;
    
    for (int i = 0; i < max_tokens; ++i) {
        llama_token next_token;
        
        if (temperature <= 0.0f) {
            next_token = llama_sampler_sample_greedy(engine->ctx, 0);
        } else {
            auto* candidates = llama_sampler_chain_init({});
            llama_sampler_chain_add(candidates, llama_sampler_init_temp(temperature));
            next_token = llama_sampler_sample(candidates, engine->ctx, 0);
            llama_sampler_free(candidates);
        }
        
        if (llama_token_is_eog(engine->model, next_token)) {
            break;
        }
        
        char token_str[256];
        int n_chars = llama_token_to_piece(engine->model, next_token, token_str, sizeof(token_str), 0, true);
        if (n_chars > 0) {
            response.append(token_str, n_chars);
        }
        
        if (llama_decode(engine->ctx, llama_batch_get_one(&next_token, 1, tokens.size() + i, 0)) != 0) {
            std::cerr << "Decode failed during generation - tactical retreat initiated." << std::endl;
            break;
        }
    }
    
    char* result = static_cast<char*>(malloc(response.length() + 1));
    if (result) {
        strcpy(result, response.c_str());
    }
    
    return result;
}

char* qwen_engine_chat(void* engine_ptr, const char* system_prompt, const char* user_message, int max_tokens) {
    if (!engine_ptr || !user_message) {
        return nullptr;
    }
    
    std::string full_prompt;
    if (system_prompt && strlen(system_prompt) > 0) {
        full_prompt = "<|im_start|>system\n" + std::string(system_prompt) + "<|im_end|>\n";
    }
    full_prompt += "<|im_start|>user\n" + std::string(user_message) + "<|im_end|>\n<|im_start|>assistant\n";
    
    return qwen_engine_generate(engine_ptr, full_prompt.c_str(), max_tokens, 0.7f);
}

void qwen_engine_destroy(void* engine_ptr) {
    if (engine_ptr) {
        delete static_cast<QwenEngine*>(engine_ptr);
    }
}

void qwen_free_string(char* str) {
    if (str) {
        free(str);
    }
}

int qwen_engine_is_loaded(void* engine_ptr) {
    if (!engine_ptr) return 0;
    return static_cast<QwenEngine*>(engine_ptr)->is_loaded ? 1 : 0;
}

const char* qwen_engine_get_model_info(void* engine_ptr) {
    if (!engine_ptr) return "Engine not initialized";
    auto* engine = static_cast<QwenEngine*>(engine_ptr);
    return engine->is_loaded ? "Qwen 2.5 0.5B - Tactical AI Asset Deployed" : "Asset offline";
}

} 