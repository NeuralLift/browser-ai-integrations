# =============================================================================
# Stage 1: Build Stage (using nightly for edition 2024)
# =============================================================================
FROM rustlang/rust:nightly-bookworm-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Enable edition 2024 feature
ENV RUSTFLAGS="--edition=2024"

# Copy manifests first for dependency caching
# Note: Dockerfile is in backend/, but build context is repo root
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Create a dummy main.rs to build dependencies (cache layer optimization)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY backend/src ./src

# Touch main.rs to invalidate the dummy build and rebuild with real code
RUN touch src/main.rs && \
    cargo build --release

# =============================================================================
# Stage 2: Runtime Stage
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (SSL certs for HTTPS requests to Gemini API)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd --create-home --shell /bin/bash appuser
USER appuser

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/backend-rig /app/backend-rig

# Expose the port (Koyeb will map this)
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Health check for Koyeb
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["./backend-rig"]
