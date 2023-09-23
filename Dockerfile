FROM rust:1-slim-bullseye as chef
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin rvm

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/rvm rvm
ENTRYPOINT ["./rvm"]
