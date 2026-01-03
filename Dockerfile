# Build stage
FROM rust:1.88-slim AS builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files and build dependencies
COPY Cargo.toml Cargo.lock ./

# Copy source code and build application
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/sunday /usr/local/bin/

WORKDIR /app

LABEL org.opencontainers.image.source=https://github.com/rezi-labs/sunday

CMD ["sunday"]
