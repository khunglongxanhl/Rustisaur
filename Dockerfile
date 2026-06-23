# Stage 1: Build
FROM rust:1.75-bookworm AS builder

# Install Lua dependencies
RUN apt-get update && apt-get install -y \
    lua5.4 \
    liblua5.4-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build release binary
RUN cargo build --release --bin rustisaur

# Stage 2: Runtime (nhỏ gọn)
FROM debian:bookworm-slim

# Install Lua runtime only
RUN apt-get update && apt-get install -y \
    lua5.4 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary từ build stage
COPY --from=builder /app/target/release/rustisaur /app/rustisaur

# Copy examples
COPY --from=builder /app/examples /app/examples

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
  CMD ./rustisaur --version || exit 1

# Default command
ENTRYPOINT ["./rustisaur"]
CMD ["--help"]