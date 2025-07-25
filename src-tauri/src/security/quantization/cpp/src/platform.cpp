#include "../include/faiss_wrapper.h"
#include <omp.h>

extern "C"
{

    void faiss_set_omp_num_threads(int n) noexcept
    {
        omp_set_num_threads(n > 0 ? n : omp_get_max_threads());
    }

    int faiss_get_omp_max_threads() noexcept
    {
        return omp_get_max_threads();
    }
}