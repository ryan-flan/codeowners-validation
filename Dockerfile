# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/codeowners-validation
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/codeowners-validation/target/release/codeowners-validation /usr/local/bin/codeowners-validation
CMD ["codeowners-validation"]
