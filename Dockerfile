# Step 1: Compute a recipe file
FROM ekidd/rust-musl-builder as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Step 2: Cache project dependencies
FROM ekidd/rust-musl-builder as cacher
WORKDIR /app
USER root
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y \
  musl-tools libssl-dev cmake gcc make pkg-config \
  && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Step 3: Build the binary
FROM ekidd/rust-musl-builder as builder
WORKDIR /app
USER root
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --target x86_64-unknown-linux-musl

# Step 4: Create the final image with binary and deps
FROM alpine:latest

# Create appuser
ENV APP_USER=app
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${APP_USER}"

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/nmlbot .

RUN chown -R $APP_USER:$APP_USER nmlbot

USER $APP_USER

CMD ["./nmlbot"]