FROM rust:1-slim

RUN cargo install --locked codeowners-validation

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
