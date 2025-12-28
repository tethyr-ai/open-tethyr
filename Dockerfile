# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the application
RUN cargo build --release --bin open-tethyr

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy the binary
COPY --from=builder /app/target/release/open-tethyr /usr/local/bin/open-tethyr

# Set ownership and permissions
RUN chown appuser:appuser /usr/local/bin/open-tethyr

# Switch to non-root user
USER appuser

# Expose port (default for cache server)
EXPOSE 8080

# Set the binary as entrypoint
ENTRYPOINT ["/usr/local/bin/open-tethyr"]