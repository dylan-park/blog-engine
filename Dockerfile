# Use the official Rust image
FROM rust:slim-bookworm AS builder

# Create app directory
WORKDIR /usr/src/app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy lib.rs to build dependencies
RUN mkdir src && echo "// dummy" > src/lib.rs

# Build dependencies for cache
RUN cargo build --release --lib

# Remove the dummy lib
RUN rm -rf src

# Copy the source code
COPY . .

# Build the project
RUN cargo build --release

# Use a minimal image for running
FROM debian:bookworm-slim

# Install required packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Copy the binary and static files from the builder
COPY --from=builder /usr/src/app/target/release/blog-engine /app/blog-engine
COPY --from=builder /usr/src/app/static /app/static
COPY --from=builder /usr/src/app/templates /app/templates

# Set working directory
WORKDIR /app

# Run the binary
CMD ["./blog-engine"]