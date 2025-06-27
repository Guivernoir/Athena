use std::env;
use std::path::PathBuf;

fn main() {
    tauri_build::build();
    setup_llama_cpp_integration();
}

fn setup_llama_cpp_integration() {
    println!("cargo:rerun-if-changed=src/llama/engine.cpp");
    println!("cargo:rerun-if-changed=src/llama/engine.hpp");
    
    // Detect llama.cpp installation
    let llama_cpp_dir = find_llama_cpp_installation()
        .expect("Llama.cpp installation not detected - set LLAMA_CPP_DIR environment variable");
    
    println!("cargo:rustc-link-search=native={}/lib", llama_cpp_dir.display());
    println!("cargo:rustc-link-search=native={}/build", llama_cpp_dir.display());
    
    // Link against llama.cpp libraries
    println!("cargo:rustc-link-lib=static=llama");
    println!("cargo:rustc-link-lib=static=ggml");
    
    // System dependencies
    link_system_dependencies();
    
    // Compile C++ bridge
    compile_cpp_bridge(&llama_cpp_dir);
    
    println!("Llama.cpp integration configured successfully");
}

fn find_llama_cpp_installation() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(path) = env::var("LLAMA_CPP_DIR") {
        let path = PathBuf::from(path);
        if validate_llama_installation(&path) {
            println!("Found llama.cpp via LLAMA_CPP_DIR: {}", path.display());
            return Some(path);
        }
    }
    
    // Check common locations relative to Tauri project
    let common_paths = [
        "./deps/llama.cpp",
        "../deps/llama.cpp", 
        "./llama.cpp",
        "/usr/local",
        "/opt/llama.cpp",
    ];
    
    for path_str in &common_paths {
        let path = PathBuf::from(path_str);
        if validate_llama_installation(&path) {
            println!("Found llama.cpp at: {}", path.display());
            return Some(path);
        }
    }
    
    None
}

fn validate_llama_installation(path: &PathBuf) -> bool {
    let has_headers = path.join("include/llama.h").exists() || 
                     path.join("src/llama.cpp").exists();
    
    let has_libs = path.join("build/libllama.a").exists() || 
                   path.join("lib/libllama.a").exists() ||
                   path.join("libllama.a").exists();
    
    has_headers && has_libs
}

fn link_system_dependencies() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=m");
            
            if env::var("LLAMA_CUDA").is_ok() {
                println!("cargo:rustc-link-lib=cuda");
                println!("cargo:rustc-link-lib=cublas");
            }
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=Accelerate");
            println!("cargo:rustc-link-lib=framework=Foundation");
            
            if env::var("LLAMA_METAL").is_ok() {
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=MetalKit");
            }
        }
        "windows" => {
            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=advapi32");
        }
        _ => {}
    }
}

fn compile_cpp_bridge(llama_dir: &PathBuf) {
    let mut build = cc::Build::new();
    
    build
        .cpp(true)
        .std("c++17")
        .file("src/llama/engine.cpp")  // Note: adjusted path for Tauri structure
        .include("src/llama")
        .include(llama_dir.join("include"))
        .include(llama_dir.join("src"))
        .include(llama_dir.join("common"))
        .flag_if_supported("-O3")
        .flag_if_supported("-march=native")
        .warnings(false);
    
    // Platform-specific flags
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "linux" | "macos" => {
            build.flag("-pthread");
            if env::var("LLAMA_CUDA").is_ok() {
                build.define("GGML_USE_CUDA", None);
            }
            if target_os == "macos" && env::var("LLAMA_METAL").is_ok() {
                build.define("GGML_USE_METAL", None);
            }
        }
        "windows" => {
            build.define("_WIN32_WINNT", "0x0601");
        }
        _ => {}
    }
    
    if env::var("PROFILE").unwrap() == "debug" {
        build.define("DEBUG", None);
        build.flag_if_supported("-g");
    } else {
        build.define("NDEBUG", None);
    }
    
    build.compile("qwen_engine");
}