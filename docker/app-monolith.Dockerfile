FROM rust:1.71 AS build-env
ENV SQLX_OFFLINE=true
WORKDIR /app
COPY ./Cargo.lock ./Cargo.toml /app/
COPY ./.sqlx /app/.sqlx
COPY ./crates /app/crates
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/app-monolith /app
EXPOSE 8000
CMD ["./app"]
