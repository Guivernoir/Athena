#ifndef QWEN_ENGINE_HPP
#define QWEN_ENGINE_HPP

#ifdef __cplusplus
extern "C" {
#endif

/**
 * LLM Engine C Interface
 * 
 * Strategic deployment interface for llama.cpp integration.
 * Handle with appropriate tactical caution - pointers are live ammunition.
 */

void* qwen_engine_create(const char* model_path);
void qwen_engine_destroy(void* engine);
char* qwen_engine_generate(void* engine, const char* prompt, int max_tokens, float temperature);
char* qwen_engine_chat(void* engine, const char* system_prompt, const char* user_message, int max_tokens);
void qwen_free_string(char* str);
int qwen_engine_is_loaded(void* engine);
const char* qwen_engine_get_model_info(void* engine);

#ifdef __cplusplus
}
#endif

#endif // QWEN_ENGINE_HPP