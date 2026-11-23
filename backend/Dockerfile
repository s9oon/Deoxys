# ---- base: Rust toolchain + cargo-chef ----
# Same Debian release (trixie) here and in runtime → no glibc mismatch.
# Swap `1` for your exact version (e.g. 1.xx) once rust-toolchain.toml exists.
FROM rust:1-trixie AS chef
RUN cargo install cargo-chef --locked --version ^0.1
WORKDIR /app

# ---- planner: work out the dependency list ----
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ---- builder: deps first (cached), then your code ----
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# This layer only rebuilds when Cargo.toml/Cargo.lock change.
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

# ---- runtime: just the binary ----
FROM debian:trixie-slim AS runtime
RUN useradd --system --no-create-home --shell /usr/sbin/nologin app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
USER app
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/backend"]