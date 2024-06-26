FROM rust:1.78 AS build-env
ENV SQLX_OFFLINE=true
WORKDIR /app
COPY ./Cargo.lock ./Cargo.toml /app/
COPY ./.sqlx /app/.sqlx
COPY ./worker /app/worker
RUN cargo build --release --package worker

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/worker /app
EXPOSE 8000
CMD ["./app"]
