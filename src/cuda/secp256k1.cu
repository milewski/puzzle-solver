#include <stdio.h>
#include <stdint.h>

// secp256k1 constants
__constant__ uint64_t SECP256K1_P[4] = {0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFC2FULL};
__constant__ uint64_t SECP256K1_N[4] = {0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFFULL, 0xFFFFFFFFFFFFFFFEULL, 0xBAAEDCE6AF48A03BULL, 0xBFD25E8CD0364141ULL};
__constant__ uint64_t SECP256K1_GX[4] = {0x79BE667EF9DCBBACULL, 0x55A06295CE870B07ULL, 0x029BFCDB2DCE28D9ULL, 0x59F2815B16F81798ULL};
__constant__ uint64_t SECP256K1_GY[4] = {0x483ADA7726A3C465ULL, 0x5DA4FBFC0E1108A8ULL, 0xFD17B448A6855419ULL, 0x9C47D08FFB10D4B8ULL};

// Utility functions for u256 arithmetic
__device__ void u256_add(uint64_t *res, const uint64_t *a, const uint64_t *b) {
    uint64_t carry = 0;
    for (int i = 0; i < 4; i++) {
        uint64_t temp = a[i] + carry;
        res[i] = temp + b[i];
        carry = (temp < a[i]) || (res[i] < temp);
    }
}

__device__ void u256_sub(uint64_t *res, const uint64_t *a, const uint64_t *b) {
    uint64_t borrow = 0;
    for (int i = 0; i < 4; i++) {
        uint64_t temp = a[i] - b[i] - borrow;
        borrow = (a[i] < b[i]) || (temp > a[i]);
        res[i] = temp;
    }
}

__device__ void u256_mul(uint64_t *res, const uint64_t *a, const uint64_t *b) {
    uint64_t temp[8] = {0};
    for (int i = 0; i < 4; i++) {
        uint64_t carry = 0;
        for (int j = 0; j < 4; j++) {
            uint64_t lo, hi;
            uint64_t a_part = a[i];
            uint64_t b_part = b[j];
            lo = __umul64hi(a_part, b_part);
            hi = a_part * b_part;

            uint64_t sum_lo = temp[i + j] + lo;
            uint64_t sum_hi = temp[i + j + 1] + hi + (sum_lo < temp[i + j]);
            temp[i + j] = sum_lo;
            temp[i + j + 1] = sum_hi;
        }
    }
    // Reduce modulo P
    for (int i = 0; i < 4; i++) {
        res[i] = temp[i];
    }
}

__device__ void u256_mod(uint64_t *res, const uint64_t *a, const uint64_t *mod) {
    uint64_t temp[4];
    for (int i = 3; i >= 0; i--) {
        if (a[i] > mod[i]) {
            u256_sub(temp, a, mod);
            for (int j = 0; j < 4; j++) res[j] = temp[j];
            return;
        }
    }
    for (int i = 0; i < 4; i++) res[i] = a[i];
}

// ECC point addition
__device__ void ec_point_add(uint64_t *rx, uint64_t *ry, const uint64_t *px, const uint64_t *py, const uint64_t *qx, const uint64_t *qy) {
    uint64_t lambda[4], temp1[4], temp2[4];

    // lambda = (qy - py) / (qx - px)
    u256_sub(temp1, qy, py);
    u256_sub(temp2, qx, px);
    u256_mod(temp2, temp2, SECP256K1_P);  // Modular inverse needed here for division
    u256_mul(lambda, temp1, temp2);
    u256_mod(lambda, lambda, SECP256K1_P);

    // rx = lambda^2 - px - qx
    u256_mul(temp1, lambda, lambda);
    u256_sub(temp1, temp1, px);
    u256_sub(rx, temp1, qx);
    u256_mod(rx, rx, SECP256K1_P);

    // ry = lambda * (px - rx) - py
    u256_sub(temp1, px, rx);
    u256_mul(temp1, lambda, temp1);
    u256_sub(ry, temp1, py);
    u256_mod(ry, ry, SECP256K1_P);
}

// ECC scalar multiplication
__device__ void ec_scalar_mul(uint64_t *rx, uint64_t *ry, const uint64_t *px, const uint64_t *py, const uint64_t *k) {
    uint64_t resx[4] = {0}, resy[4] = {0};
    uint64_t qx[4], qy[4];
    for (int i = 0; i < 4; i++) {
        qx[i] = px[i];
        qy[i] = py[i];
    }

    for (int i = 0; i < 256; i++) {
        if ((k[i / 64] >> (i % 64)) & 1) {
            if (resx[0] == 0 && resy[0] == 0 && resx[1] == 0 && resy[1] == 0 && resx[2] == 0 && resy[2] == 0 && resx[3] == 0 && resy[3] == 0) {
                for (int j = 0; j < 4; j++) {
                    resx[j] = qx[j];
                    resy[j] = qy[j];
                }
            } else {
                ec_point_add(resx, resy, resx, resy, qx, qy);
            }
        }
        ec_point_add(qx, qy, qx, qy, qx, qy);
    }
    for (int i = 0; i < 4; i++) {
        rx[i] = resx[i];
        ry[i] = resy[i];
    }
}

__global__ void kernel_ecc(uint64_t *d_rx, uint64_t *d_ry, uint64_t *d_k) {
    uint64_t px[4], py[4];
    for (int i = 0; i < 4; i++) {
        px[i] = SECP256K1_GX[i];
        py[i] = SECP256K1_GY[i];
    }
    ec_scalar_mul(d_rx, d_ry, px, py, d_k);
}

int main() {
    uint64_t h_k[4] = {0x12345678, 0x9abcdef0, 0x12345678, 0x9abcdef0};
    uint64_t h_rx[4], h_ry[4];
    uint64_t *d_k, *d_rx, *d_ry;

    cudaMalloc((void**)&d_k, 4 * sizeof(uint64_t));
    cudaMalloc((void**)&d_rx, 4 * sizeof(uint64_t));
    cudaMalloc((void**)&d_ry, 4 * sizeof(uint64_t));

    cudaMemcpy(d_k, h_k, 4 * sizeof(uint64_t), cudaMemcpyHostToDevice);

    kernel_ecc<<<1, 1>>>(d_rx, d_ry, d_k);

    cudaMemcpy(h_rx, d_rx, 4 * sizeof(uint64_t), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_ry, d_ry, 4 * sizeof(uint64_t), cudaMemcpyDeviceToHost);

    printf("Resulting Point:\n");
    for (int i = 0; i < 4; i++) {
        printf("rx[%d] = %016llx\n", i, h_rx[i]);
        printf("ry[%d] = %016llx\n", i, h_ry[i]);
    }

    cudaFree(d_k);
    cudaFree(d_rx);
    cudaFree(d_ry);

    return 0;
}
