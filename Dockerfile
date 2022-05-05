FROM lukemathwalker/cargo-chef:latest-rust-1.60.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# build dependencies (and add to docker's caching layer)
RUN cargo chef cook --release --recipe-path recipe.json
# build application
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin emailer

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/emailer /usr/local/bin
COPY configuration configuration
ENV APP_ENV production
ENTRYPOINT ["/usr/local/bin/emailer"]