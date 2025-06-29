FROM rust:alpine AS builder
RUN apk update && apk add --no-cache pkgconfig musl-dev openssl-dev

# Set necessary environmet variables needed for our image
ENV RUSTFLAGS='-C target-feature=-crt-static'

# Move to working directory /build
WORKDIR /build

# Create an unprivileged user
ENV USER=appuser
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

# Copy the code into the container
RUN rustup component add rustfmt clippy
COPY Cargo.* ./

# This is a workaround to avoid rebuilding the dependencies on every change.
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release

RUN cargo fmt --all -- --check
RUN cargo clippy --all-targets --all-features -- -D warnings
RUN cargo test
RUN rm -rf src target/release/man_down target/release/.fingerprint/man_down* target/debug/deps/man_down*

COPY src/ src/
COPY config.yaml ./

RUN cargo fmt --all -- --check

RUN cargo clippy --all-targets --all-features -- -D warnings

# Run tests
RUN cargo test

# Build the application
RUN cargo build --release

############################
# STEP 2 build a small image
############################
FROM alpine

RUN apk update && apk add --no-cache libgcc openssl

COPY --from=builder /build/target/release/man_down /mandown

# Import the user and group files from the builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY config.yaml ./

# Use the unprivileged user
USER appuser:appuser

# Command to run the executable
ENTRYPOINT [ "/mandown" ]
