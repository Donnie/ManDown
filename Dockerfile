FROM rust:alpine as builder
RUN apk update && apk add --no-cache pkgconfig musl-dev libressl-dev sqlite-dev

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
COPY src/ src/
COPY migrations/ migrations/
COPY Cargo.* ./

# Build the application
RUN cargo build --release

############################
# STEP 2 build a small image
############################
FROM alpine

RUN apk update && apk add --no-cache libgcc libressl sqlite sqlite-libs

COPY --from=builder /build/target/release/man_down /mandown

# Import the user and group files from the builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY migrations/ migrations/

# Use the unprivileged user
USER appuser:appuser

# Command to run the executable
ENTRYPOINT [ "/mandown" ]
