#include "quantizer.hpp"
#include "kernels.hpp"
#include <algorithm>
#include <cmath>
#include <memory>
#include <vector>

class Quantizer {
private:
    int bits_;
    int block_size_;
    std::vector<float> scales_;
    std::vector<float> zero_points_;

public:
    Quantizer(int bits, int block_size) 
        : bits_(bits), block_size_(block_size) {
        if (bits <= 0 || bits > 16) {
            throw std::invalid_argument("Bits must be between 1 and 16");
        }
        if (block_size <= 0) {
            throw std::invalid_argument("Block size must be positive");
        }
    }

    bool quantize_weights(const float* weights, int length, char* output) {
        try {
            // Calculate number of blocks
            int num_blocks = (length + block_size_ - 1) / block_size_;
            scales_.resize(num_blocks);
            zero_points_.resize(num_blocks);

            // Calculate scales and zero points for each block
            for (int block = 0; block < num_blocks; ++block) {
                int start_idx = block * block_size_;
                int end_idx = std::min(start_idx + block_size_, length);
                
                calculate_scale_and_zero_point(
                    weights + start_idx, 
                    end_idx - start_idx, 
                    scales_[block], 
                    zero_points_[block]
                );
            }

            // Quantize each block
            int output_offset = 0;
            for (int block = 0; block < num_blocks; ++block) {
                int start_idx = block * block_size_;
                int end_idx = std::min(start_idx + block_size_, length);
                int block_length = end_idx - start_idx;

                output_offset += quantize_block(
                    weights + start_idx,
                    block_length,
                    scales_[block],
                    zero_points_[block],
                    output + output_offset
                );
            }

            return true;
        } catch (const std::exception& e) {
            return false;
        }
    }

    bool dequantize_weights(const char* quantized, int length, float* output) {
        try {
            int num_blocks = scales_.size();
            int input_offset = 0;
            
            for (int block = 0; block < num_blocks; ++block) {
                int block_length = std::min(block_size_, 
                    static_cast<int>(scales_.size()) * block_size_ - block * block_size_);
                
                input_offset += dequantize_block(
                    quantized + input_offset,
                    block_length,
                    scales_[block],
                    zero_points_[block],
                    output + block * block_size_
                );
            }

            return true;
        } catch (const std::exception& e) {
            return false;
        }
    }

private:
    void calculate_scale_and_zero_point(const float* data, int length, 
                                       float& scale, float& zero_point) {
        if (length == 0) {
            scale = 1.0f;
            zero_point = 0.0f;
            return;
        }

        // Find min and max values
        float min_val = data[0];
        float max_val = data[0];
        
        for (int i = 1; i < length; ++i) {
            min_val = std::min(min_val, data[i]);
            max_val = std::max(max_val, data[i]);
        }

        // Calculate quantization parameters
        int max_quantized = (1 << bits_) - 1;
        
        if (max_val == min_val) {
            scale = 1.0f;
            zero_point = 0.0f;
        } else {
            scale = (max_val - min_val) / max_quantized;
            zero_point = min_val;
        }
    }

    int quantize_block(const float* input, int length, float scale, 
                      float zero_point, char* output) {
        std::vector<uint8_t> quantized_values(length);
        
        // Quantize values
        for (int i = 0; i < length; ++i) {
            float normalized = (input[i] - zero_point) / scale;
            int quantized = static_cast<int>(std::round(normalized));
            quantized = std::max(0, std::min(quantized, (1 << bits_) - 1));
            quantized_values[i] = static_cast<uint8_t>(quantized);
        }

        // Pack bits efficiently
        return pack_bits(quantized_values.data(), length, bits_, output);
    }

    int dequantize_block(const char* input, int length, float scale, 
                        float zero_point, float* output) {
        std::vector<uint8_t> quantized_values(length);
        
        // Unpack bits
        int bytes_read = unpack_bits(input, length, bits_, quantized_values.data());
        
        // Dequantize values
        for (int i = 0; i < length; ++i) {
            output[i] = quantized_values[i] * scale + zero_point;
        }

        return bytes_read;
    }
};

// C interface functions
extern "C" {
    void* create_quantizer(int bits, int block_size) {
        try {
            return new Quantizer(bits, block_size);
        } catch (...) {
            return nullptr;
        }
    }

    void destroy_quantizer(void* quantizer) {
        delete static_cast<Quantizer*>(quantizer);
    }

    int quantize_weights(void* quantizer, const float* weights, 
                        int length, char* output) {
        if (!quantizer) return 0;
        
        Quantizer* q = static_cast<Quantizer*>(quantizer);
        return q->quantize_weights(weights, length, output) ? 1 : 0;
    }

    int dequantize_weights(void* quantizer, const char* quantized, 
                          int length, float* output) {
        if (!quantizer) return 0;
        
        Quantizer* q = static_cast<Quantizer*>(quantizer);
        return q->dequantize_weights(quantized, length, output) ? 1 : 0;
    }
}