#pragma once
#include <faiss/IndexPQ.h>
#include <memory>

struct FaissContext
{
    std::unique_ptr<faiss::IndexPQ> index;
};