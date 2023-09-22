# Builder stage 
FROM rust:1.72.0 As builder 

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim As runtime

WORKDIR /app
# Install OpenSSL - it is dynamically liked by some of our dependencies 
# Install ca-certificates - it is needed to verify TLS certificates 
# when establishing HTTP connections 
RUN apt update -y \ 
    && apt install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime! 
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]