# Stage 1: Build the Rust application
FROM rust:1.76 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin prices
WORKDIR /prices

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Cache dependencies
# This step ensures that your dependencies are cached, so they don't need to be rebuilt every time you make changes to your source code.
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/prices*
RUN cargo build --release

# Stage 2: Create the runtime image
FROM debian:buster-slim

# Install any runtime dependencies here
# For example, if your application depends on SSL certificates:
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /prices/target/release/prices /usr/local/bin/prices

# Set the CMD to your binary
CMD ["prices"]
