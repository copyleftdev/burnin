# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/burnin

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    libgcc \
    libstdc++ \
    ca-certificates

# Create non-root user
RUN addgroup -g 1000 burnin && \
    adduser -D -s /bin/sh -u 1000 -G burnin burnin

# Copy binary from builder
COPY --from=builder /usr/src/burnin/target/release/burnin /usr/local/bin/burnin

# Set ownership
RUN chown burnin:burnin /usr/local/bin/burnin

# Switch to non-root user
USER burnin

# Set entrypoint
ENTRYPOINT ["burnin"]

# Default command (show help)
CMD ["--help"]
