# Stage 1: Build with musl for static linking
FROM rust:1.94-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release -p open-tethyr-cli

# Stage 2: Minimal runtime
FROM scratch
COPY --from=builder /app/target/release/open-tethyr /open-tethyr
ENTRYPOINT ["/open-tethyr"]
CMD ["serve"]
