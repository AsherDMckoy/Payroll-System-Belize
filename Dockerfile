FROM rust:1.85-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY templates ./templates
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/payroll-management-system /usr/local/bin/payroll-management-system
COPY assets ./assets
COPY templates ./templates
COPY migrations ./migrations

ENV APP_HOST=0.0.0.0
ENV PORT=9000
ENV RUST_LOG=info

EXPOSE 9000

CMD ["payroll-management-system"]
