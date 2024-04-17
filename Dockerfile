FROM rust:slim AS builder

WORKDIR /usr/src/codeowners-validation

# Copy the Rust project files to the container
COPY . .

# Build the Rust project
RUN cargo build --release

# Use a minimal Alpine Linux image as the final base
FROM alpine:latest

# Set working directory
WORKDIR /usr/src/codeowners-validation

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/codeowners-validation/target/release/codeowners-validation .

# Make the executable executable
RUN chmod +x codeowners-validation

# Set the entry point for the Docker container
ENTRYPOINT ["./codeowners-validation"]
