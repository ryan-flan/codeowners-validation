# Build stage
FROM rust:slim AS builder

WORKDIR /code

COPY . .

RUN cargo build --release

# Final stage
FROM debian:bookworm-slim 

WORKDIR /code

COPY --from=builder /code/target/release/codeowners-validation .

RUN chmod +x codeowners-validation

ENTRYPOINT ["./codeowners-validation"]

