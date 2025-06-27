#ifndef QUANTIZER_HPP
#define QUANTIZER_HPP

#ifdef __cplusplus
extern "C" {
#endif

// C interface for Rust FFI
void* create_quantizer(int bits, int block_size);
void destroy_quantizer(void* quantizer);
int quantize_weights(void* quantizer, const float* weights, int length, char* output);
int dequantize_weights(void* quantizer, const char* quantized, int length, float* output);

#ifdef __cplusplus
}
#endif

#endif