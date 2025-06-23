#!/bin/bash

# Llama.cpp Setup Script
# 
# Strategic deployment script for llama.cpp tactical assets.
# Run this to establish your AI operational infrastructure.

set -e  # Exit on any error - no compromised deployments

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LLAMA_DIR="$SCRIPT_DIR/deps/llama.cpp"

echo "🚀 Initializing llama.cpp tactical deployment..."

# Function to detect system architecture
detect_system() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to check if CUDA is available
check_cuda() {
    if command -v nvcc &> /dev/null; then
        echo "✅ CUDA detected - GPU acceleration available"
        return 0
    else
        echo "ℹ️  CUDA not found - CPU-only deployment"
        return 1
    fi
}

# Function to setup llama.cpp
setup_llama_cpp() {
    echo "📦 Setting up llama.cpp..."
    
    if [ ! -d "$LLAMA_DIR" ]; then
        echo "📥 Cloning llama.cpp repository..."
        mkdir -p deps
        git clone https://github.com/ggerganov/llama.cpp.git "$LLAMA_DIR"
    else
        echo "📦 Updating existing llama.cpp installation..."
        cd "$LLAMA_DIR"
        git pull origin master
        cd "$SCRIPT_DIR"
    fi
    
    cd "$LLAMA_DIR"
    
    # Configure build options based on system capabilities
    CMAKE_ARGS=""
    MAKE_ARGS=""
    
    SYSTEM=$(detect_system)
    echo "🖥️  Detected system: $SYSTEM"
    
    case $SYSTEM in
        "linux")
            if check_cuda; then
                CMAKE_ARGS="-DLLAMA_CUDA=ON"
                export LLAMA_CUDA=1
                echo "🔥 CUDA build enabled"
            fi
            ;;
        "macos")
            CMAKE_ARGS="-DLLAMA_METAL=ON"
            export LLAMA_METAL=1
            echo "🍎 Metal acceleration enabled for Apple Silicon"
            ;;
        "windows")
            if check_cuda; then
                CMAKE_ARGS="-DLLAMA_CUDA=ON"
                export LLAMA_CUDA=1
            fi
            ;;
    esac
    
    # Build llama.cpp
    echo "🔨 Building llama.cpp with optimizations..."
    
    if command -v cmake &> /dev/null; then
        # CMake build (preferred method)
        mkdir -p build
        cd build
        cmake .. $CMAKE_ARGS -DCMAKE_BUILD_TYPE=Release
        cmake --build . --config Release -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
        cd ..
    else
        # Fallback to Make
        echo "⚠️  CMake not found, using Make fallback"
        make clean
        make -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4) $MAKE_ARGS
    fi
    
    echo "✅ llama.cpp build completed"
    cd "$SCRIPT_DIR"
}

# Function to download the Qwen model
download_model() {
    echo "📥 Setting up Qwen 2.5 model..."
    
    MODEL_DIR="$SCRIPT_DIR/llama/models"
    MODEL_FILE="qwen2.5-0.5b-instruct-q5_k_m.gguf"
    MODEL_PATH="$MODEL_DIR/$MODEL_FILE"
    
    mkdir -p "$MODEL_DIR"
    
    if [ ! -f "$MODEL_PATH" ]; then
        echo "📡 Downloading Qwen 2.5 0.5B model (this may take a few minutes)..."
        
        # Using huggingface-cli if available, otherwise curl
        if command -v huggingface-cli &> /dev/null; then
            huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct-GGUF \
                "$MODEL_FILE" \
                --local-dir "$MODEL_DIR" \
                --local-dir-use-symlinks False
        else
            echo "⚠️  huggingface-cli not found, using direct download..."
            curl -L "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/$MODEL_FILE" \
                -o "$MODEL_PATH"
        fi
        
        echo "✅ Model download completed"
    else
        echo "✅ Model already exists at $MODEL_PATH"
    fi
}

# Function to set up environment variables
setup_environment() {
    echo "🔧 Setting up environment variables..."
    
    # Create .env file for development
    cat > "$SCRIPT_DIR/.env" << EOF
# Llama.cpp Configuration
LLAMA_CPP_DIR=$LLAMA_DIR
QWEN_MODEL_PATH=$SCRIPT_DIR/llama/models/qwen2.5-0.5b-instruct-q5_k_m.gguf

# Optional: Enable specific features
$([ "$LLAMA_CUDA" = "1" ] && echo "LLAMA_CUDA=1")
$([ "$LLAMA_METAL" = "1" ] && echo "LLAMA_METAL=1")
EOF
    
    echo "✅ Environment configuration saved to .env"
    echo "💡 Add 'export LLAMA_CPP_DIR=$LLAMA_DIR' to your shell profile for permanent setup"
}

# Function to verify installation
verify_installation() {
    echo "🔍 Verifying installation..."
    
    # Check if libraries exist
    if [ -f "$LLAMA_DIR/build/libllama.a" ] || [ -f "$LLAMA_DIR/libllama.a" ]; then
        echo "✅ libllama.a found"
    else
        echo "❌ libllama.a not found - build may have failed"
        exit 1
    fi
    
    if [ -f "$LLAMA_DIR/build/libggml.a" ] || [ -f "$LLAMA_DIR/libggml.a" ]; then
        echo "✅ libggml.a found"
    else
        echo "❌ libggml.a not found - build may have failed"
        exit 1
    fi
    
    # Check if model exists
    if [ -f "$SCRIPT_DIR/src-tauri/src/llama/models/qwen2.5-0.5b-instruct-q5_k_m.gguf" ]; then
        echo "✅ Qwen model ready for deployment"
    else
        echo "❌ Model file missing"
        exit 1
    fi
    
    echo "🎉 Installation verification completed - all tactical assets operational!"
}

# Main execution
main() {
    echo "=== Llama.cpp Tactical Deployment ==="
    echo "Setting up AI inference capabilities for your backend..."
    echo
    
    setup_llama_cpp
    download_model
    setup_environment
    verify_installation
    
    echo
    echo "🎯 Deployment completed successfully!"
    echo
    echo "Next steps:"
    echo "1. Source the environment: source .env"
    echo "2. Build your Rust project: cargo build"
    echo "3. Run your tests: cargo test"
    echo
    echo "Well, that was quite the strategic decision, wasn't it? 🎪"
}

# Execute main function
main "$@"