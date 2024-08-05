FROM nvidia/cuda:12.5.1-devel-ubuntu24.04

RUN apt update && apt install -y curl mingw-w64

# Install Zig (https://ziglang.org/download)
RUN curl https://ziglang.org/builds/zig-linux-x86_64-0.14.0-dev.839+a931bfada.tar.xz -o /root/zig.tar.xz && \
    mkdir -p /root/.zig && \
    tar -xf /root/zig.tar.xz --strip-components 1 -C /root/.zig && \
    rm -rf /root/zig-*.tar.xz

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    --profile minimal \
    --default-toolchain 1.80.0 \
    --target \
      x86_64-unknown-linux-gnu \
      x86_64-pc-windows-gnu \
      x86_64-apple-darwin \
      aarch64-apple-darwin

ENV PATH="/root/.cargo/bin:/root/.zig:${PATH}"
ENV CUDA_COMPUTE_CAP=75

RUN cargo install --locked cargo-zigbuild

