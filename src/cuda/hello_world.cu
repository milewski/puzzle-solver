extern "C" __global__ void cuda_hello(int *out){
    unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;
    out[i] = i;
}