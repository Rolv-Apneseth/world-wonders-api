ARG RUST_VERSION=1.79.0
ARG APP_NAME=world-wonders-api
ARG PORT=8138
FROM rust:${RUST_VERSION} AS builder
WORKDIR /usr/src/${APP_NAME}
COPY . .
RUN cargo install --locked --path .

FROM debian:12-slim
ARG APP_NAME
ARG PORT
COPY --from=builder /usr/local/cargo/bin/$APP_NAME /usr/local/bin/server

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser
USER appuser

EXPOSE $PORT

ENV RUST_LOG=debug
CMD ["/usr/local/bin/server"]
