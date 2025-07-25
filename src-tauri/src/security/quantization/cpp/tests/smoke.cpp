#include "../include/faiss_wrapper.h"
#include <vector>
#include <cstdio>

int main()
{
    constexpr int dim = 384;
    constexpr int n = 1000;

    // Dummy data
    std::vector<float> vecs(n * dim, 1.0f);
    std::vector<uint8_t> codes(48);
    std::vector<float> out(dim);

    auto *ctx = faiss_create(dim);
    if (!ctx)
    {
        std::puts("create failed");
        return 1;
    }

    if (faiss_train(ctx, vecs.data(), n) != 0)
    {
        std::puts("train failed");
        faiss_free(ctx);
        return 1;
    }

    faiss_encode(ctx, vecs.data(), codes.data());
    faiss_decode(ctx, codes.data(), out.data());

    faiss_free(ctx);
    std::puts("smoke test OK");
    return 0;
}