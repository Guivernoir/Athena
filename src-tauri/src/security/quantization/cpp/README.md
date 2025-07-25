# faiss_wrapper

Minimal, **static-only** C-API over FAISS Product Quantization.

- Target: mobile (iOS 13+, Android API 24+), ≤ 5 MB stripped
- Dimensions: 48 \* k ≤ 1024
- Output: 48-byte codes (8-bit × 48 sub-quantizers)

## Build

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```
